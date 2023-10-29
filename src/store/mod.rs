use crate::{Entity, Event};
use async_trait::async_trait;
use std::{error::Error, fmt::Debug};
use tokio::sync::broadcast::{Receiver, Sender};

#[cfg(feature = "in-mem")]
pub mod in_mem;

#[async_trait]
pub trait Store: Send + Sync + Clone {
    async fn create<E: Entity>(&self, entity: &E) -> Result<(), Box<dyn Error>>;
    async fn update<E: Entity>(&self, id: &E::ID, update: &E::Update)
        -> Result<(), Box<dyn Error>>;
    async fn delete_all<E: Entity>(&self) -> Result<(), Box<dyn Error>>;
    async fn delete_by_id<E: Entity>(&self, id: &E::ID) -> Result<(), Box<dyn Error>>;
    async fn get_all<E: Entity>(&self) -> Result<Vec<E>, Box<dyn Error>>;
    async fn get_by_id<E: Entity>(&self, id: &E::ID) -> Result<E, Box<dyn Error>>;
    async fn watch<E: Entity>(&self, channel: Sender<Event<E>>) -> Result<(), Box<dyn Error>>;

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
