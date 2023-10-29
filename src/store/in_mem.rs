use std::{collections::HashMap, error::Error, sync::Arc};

use async_trait::async_trait;
use tokio::sync::{broadcast::Sender, Mutex};
use typemap_rev::{TypeMap, TypeMapKey};

use crate::{Entity, Event, NotFoundError, Store};

#[derive(Clone)]
pub struct InMemStore {
    retain: usize,
    stores: Arc<Mutex<TypeMap>>,
}

impl InMemStore {
    pub fn new(retain: usize) -> Self {
        Self {
            retain,
            stores: Arc::new(Mutex::new(TypeMap::new())),
        }
    }
}

struct EntityKey<E: Entity>(E);

impl<E: Entity> TypeMapKey for EntityKey<E> {
    type Value = (Sender<Event<E>>, HashMap<E::ID, E>);
}

#[async_trait]
impl Store for InMemStore {
    async fn create<E: Entity>(&self, entity: &E) -> Result<(), Box<dyn Error>> {
        let mut stores = self.stores.lock().await;
        let (channel, map) = stores
            .entry::<EntityKey<E>>()
            .or_insert((Sender::new(self.retain), HashMap::default()));
        map.insert(entity.get_id().clone(), entity.clone());
        if channel.receiver_count() > 0 {
            channel.send(Event::Create(entity.clone()))?;
        }
        Ok(())
    }

    async fn update<E: Entity>(
        &self,
        id: &E::ID,
        update: &E::Update,
    ) -> Result<(), Box<dyn Error>> {
        let mut stores = self.stores.lock().await;
        let (channel, map) = stores
            .get_mut::<EntityKey<E>>()
            .ok_or(NotFoundError(id.clone()))?;
        let current = map.get_mut(id).ok_or(NotFoundError(id.clone()))?;
        current.update(update);
        if channel.receiver_count() > 0 {
            channel.send(Event::Update {
                id: id.clone(),
                update: update.clone(),
            })?;
        }
        Ok(())
    }

    async fn delete_all<E: Entity>(&self) -> Result<(), Box<dyn Error>> {
        let mut stores = self.stores.lock().await;
        let entry = stores.remove::<EntityKey<E>>();
        if let Some((channel, map)) = entry {
            if channel.receiver_count() > 0 {
                for id in map.keys() {
                    channel.send(Event::Delete(id.clone()))?;
                }
            }
        }
        Ok(())
    }

    async fn delete_by_id<E: Entity>(&self, id: &E::ID) -> Result<(), Box<dyn Error>> {
        let mut stores = self.stores.lock().await;
        let (channel, map) = stores
            .get_mut::<EntityKey<E>>()
            .ok_or(NotFoundError(id.clone()))?;
        map.remove(id);
        if channel.receiver_count() > 0 {
            channel.send(Event::Delete(id.clone()))?;
        }
        Ok(())
    }

    async fn get_all<E: Entity>(&self) -> Result<Vec<E>, Box<dyn Error>> {
        let stores = self.stores.lock().await;
        match stores.get::<EntityKey<E>>() {
            Some((_, map)) => Ok(map.values().cloned().collect()),
            None => Ok(Vec::default()),
        }
    }

    async fn get_by_id<E: Entity>(&self, id: &E::ID) -> Result<E, Box<dyn Error>> {
        let stores = self.stores.lock().await;
        let (_, map) = stores
            .get::<EntityKey<E>>()
            .ok_or(NotFoundError(id.clone()))?;
        map.get(id)
            .ok_or(NotFoundError(id.clone()))
            .cloned()
            .map_err(|e| e.into())
    }

    async fn watch<E: Entity>(&self, channel: Sender<Event<E>>) -> Result<(), Box<dyn Error>> {
        let mut ch = {
            let mut stores = self.stores.lock().await;
            let (channel, _) = stores
                .entry::<EntityKey<E>>()
                .or_insert((Sender::new(self.retain), HashMap::default()));
            channel.subscribe()
        };
        while let Ok(e) = ch.recv().await {
            channel.send(e)?;
        }
        Ok(())
    }
}
