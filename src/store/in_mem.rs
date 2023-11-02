use std::{collections::HashMap, error::Error, sync::Arc};

use async_trait::async_trait;
use tokio::sync::{broadcast::Sender, Mutex};
use typemap_rev::{TypeMap, TypeMapKey, Entry};

use crate::{Entity, Event, NotFoundError, Store, Singleton, SingletonEntity, SingletonEntityUpdate};

#[derive(Clone)]
pub struct InMemStore {
    retain: usize,
    stores: Arc<Mutex<TypeMap>>,
    singleton_stores: Arc<Mutex<TypeMap>>
}

impl InMemStore {
    pub fn new(retain: usize) -> Self {
        Self {
            retain,
            stores: Arc::new(Mutex::new(TypeMap::new())),
            singleton_stores: Arc::new(Mutex::new(TypeMap::new()))
        }
    }
}

#[derive(Clone)]
struct EntityWrapper<E: Entity>(E);
impl<E: Entity> TypeMapKey for EntityWrapper<E> {
    type Value = (Sender<Event<E>>, HashMap<E::ID, Self>);
}

struct SingletonWrapper<S: Singleton>(S);
impl<S: Singleton> TypeMapKey for SingletonWrapper<S> {
    type Value = (Sender<Event<SingletonEntity<S>>>, Option<Self>);
}

#[async_trait]
impl Store for InMemStore {
    async fn create<E: Entity>(&self, entity: &E) -> Result<(), Box<dyn Error>> {
        let mut stores = self.stores.lock().await;
        let (channel, map) = stores
            .entry::<EntityWrapper<E>>()
            .or_insert((Sender::new(self.retain), HashMap::default()));
        map.insert(entity.get_id().clone(), EntityWrapper(entity.clone()));
        if channel.receiver_count() > 0 {
            channel.send(Event::Create(entity.clone()))?;
        }
        Ok(())
    }

    async fn create_singleton<S: Singleton>(&self, entity: &S) -> Result<(), Box<dyn Error>> {
        let mut sings = self.singleton_stores.lock().await;
        let e = sings.entry::<SingletonWrapper<S>>();
        let channel = match e {
            Entry::Occupied(mut e) => {
                let (channel, s) = e.get_mut();
                s.replace(SingletonWrapper(entity.clone()));
                channel.clone()
            }
            Entry::Vacant(v) => {
                let channel = Sender::new(self.retain);
                v.insert((channel.clone(), Some(SingletonWrapper(entity.clone()))));
                channel
            }
        };
        if channel.receiver_count() > 0 {
            channel.send(Event::Create(SingletonEntity(entity.clone())))?;
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
            .get_mut::<EntityWrapper<E>>()
            .ok_or(NotFoundError(id.clone()))?;
        let current = map.get_mut(id).ok_or(NotFoundError(id.clone()))?;
        current.0.update(update);
        if channel.receiver_count() > 0 {
            channel.send(Event::Update {
                id: id.clone(),
                update: update.clone(),
            })?;
        }
        Ok(())
    }

    async fn update_singleton<S: Singleton>(&self, update: &S::Update) -> Result<(), Box<dyn Error>> {
        let mut sings = self.singleton_stores.lock().await;
        let (channel, current_opt) = sings.get_mut::<SingletonWrapper<S>>().ok_or(NotFoundError(S::ENTITY_ID))?;
        let current = current_opt.as_mut().ok_or(NotFoundError(S::ENTITY_ID))?;
        current.0.update(update);
        if channel.receiver_count() > 0 {
            channel.send(Event::Update { id: S::ENTITY_ID.clone(), update: SingletonEntityUpdate(update.clone()) })?;
        }
        Ok(())
    }

    async fn delete_all<E: Entity>(&self) -> Result<(), Box<dyn Error>> {
        let mut stores = self.stores.lock().await;
        let entry = stores.remove::<EntityWrapper<E>>();
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
            .get_mut::<EntityWrapper<E>>()
            .ok_or(NotFoundError(id.clone()))?;
        map.remove(id);
        if channel.receiver_count() > 0 {
            channel.send(Event::Delete(id.clone()))?;
        }
        Ok(())
    }

    async fn delete_singleton<S: Singleton>(&self) -> Result<(), Box<dyn Error>> {
        let mut sings = self.singleton_stores.lock().await;
        if let Entry::Occupied(mut e) = sings.entry::<SingletonWrapper<S>>() {
            let (channel, _) = e.get_mut();
            if channel.receiver_count() > 0 {
                channel.send(Event::Delete(S::ENTITY_ID.clone()))?;
            }
            e.remove();
        }
        Ok(())
    }

    async fn get_all<E: Entity>(&self) -> Result<Vec<E>, Box<dyn Error>> {
        let stores = self.stores.lock().await;
        match stores.get::<EntityWrapper<E>>() {
            Some((_, map)) => Ok(map.values().cloned().map(|w| w.0).collect()),
            None => Ok(Vec::default()),
        }
    }

    async fn get_by_id<E: Entity>(&self, id: &E::ID) -> Result<E, Box<dyn Error>> {
        let stores = self.stores.lock().await;
        let (_, map) = stores
            .get::<EntityWrapper<E>>()
            .ok_or(NotFoundError(id.clone()))?;
        map.get(id)
            .ok_or(NotFoundError(id.clone()))
            .cloned()
            .map(|w| w.0)
            .map_err(|e| e.into())
    }

    async fn get_singleton<S: Singleton>(&self) -> Result<SingletonEntity<S>, Box<dyn Error>> {
        let sings = self.singleton_stores.lock().await;
        let (_, opt_s) = sings.get::<SingletonWrapper<S>>().ok_or(NotFoundError(S::ENTITY_ID))?;
        let s = opt_s.as_ref().ok_or(NotFoundError(S::ENTITY_ID))?;
        Ok(SingletonEntity(s.0.clone()))
    }

    async fn watch<E: Entity>(&self, channel: Sender<Event<E>>) -> Result<(), Box<dyn Error>> {
        let mut ch = {
            let mut stores = self.stores.lock().await;
            let (channel, _) = stores
                .entry::<EntityWrapper<E>>()
                .or_insert((Sender::new(self.retain), HashMap::default()));
            channel.subscribe()
        };
        while let Ok(e) = ch.recv().await {
            channel.send(e)?;
        }
        Ok(())
    }

    async fn watch_singleton<S: Singleton>(&self, channel: Sender<Event<SingletonEntity<S>>>) -> Result<(), Box<dyn Error>> {
        let mut ch = {
            let mut sings = self.singleton_stores.lock().await;
            let (channel, _) = sings.entry::<SingletonWrapper<S>>().or_insert((Sender::new(self.retain), None));
            channel.subscribe()
        };
        while let Ok(e) = ch.recv().await {
            channel.send(e)?;
        }
        Ok(())
    }
}
