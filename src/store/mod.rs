use crate::{Entity, Event, SingletonEntity, Singleton, SingletonEntityUpdate, SingletonEvent};
use async_trait::async_trait;
use std::{error::Error, fmt::Debug};
use tokio::sync::broadcast::{Receiver, Sender};

#[cfg(feature = "in-mem")]
pub mod in_mem;

#[async_trait]
pub trait Store: Send + Sync + Clone + 'static {
    async fn create<E: Entity>(&self, entity: &E) -> Result<(), Box<dyn Error>>;
    async fn create_singleton<S: Singleton>(&self, entity: &S) -> Result<(), Box<dyn Error>> {
        self.create(&SingletonEntity::new(entity.clone())).await
    }
    async fn update<E: Entity>(&self, id: &E::ID, update: &E::Update)
        -> Result<(), Box<dyn Error>>;
    async fn update_singleton<S: Singleton>(&self, update: &S::Update) -> Result<(), Box<dyn Error>> {
        self.update::<SingletonEntity<S>>(&S::ENTITY_ID.to_owned(), &SingletonEntityUpdate(update.clone())).await
    }
    async fn delete_all<E: Entity>(&self) -> Result<(), Box<dyn Error>>;
    async fn delete_by_id<E: Entity>(&self, id: &E::ID) -> Result<(), Box<dyn Error>>;
    async fn delete_singleton<S: Singleton>(&self) -> Result<(), Box<dyn Error>> {
        self.delete_by_id::<SingletonEntity<S>>(&S::ENTITY_ID.to_owned()).await
    }
    async fn get_all<E: Entity>(&self) -> Result<Vec<E>, Box<dyn Error>>;
    async fn get_by_id<E: Entity>(&self, id: &E::ID) -> Result<E, Box<dyn Error>>;
    async fn get_singleton<S: Singleton>(&self) -> Result<S, Box<dyn Error>> {
        self.get_by_id::<SingletonEntity<S>>(&S::ENTITY_ID.to_owned()).await.map(|se| se.0)
    }
    async fn watch<E: Entity>(&self, channel: Sender<Event<E>>) -> Result<(), Box<dyn Error>>;
    async fn watch_singleton<S: Singleton>(&self, channel: Sender<SingletonEvent<S>>, capacity: usize) -> Result<(), Box<dyn Error>> {
        let (tx, mut rx) = tokio::sync::broadcast::channel(capacity);
        let clone = self.clone();
        let job = tokio::spawn(async move { clone.watch::<SingletonEntity<S>>(tx).await.unwrap(); });
        while let Ok(evt) = rx.recv().await {
            channel.send(evt.into())?;
        }
        job.abort();
        job.await?;
        Ok(())
    }

    async fn sync<E: Entity>(&self, mut channel: Receiver<Event<E>>) -> Result<(), Box<dyn Error>> {
        while let Ok(event) = channel.recv().await {
            match event {
                Event::Create(e) => self.create(&e).await?,
                Event::Update { id, update } => self.update::<E>(&id, &update).await?,
                Event::Delete(id) => self.delete_by_id::<E>(&id).await?,
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct NotFoundError<T: Debug>(pub T);
impl<T: Debug> std::fmt::Display for NotFoundError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Not found: {:?}", self.0))
    }
}
impl<T: Debug> Error for NotFoundError<T> {}
