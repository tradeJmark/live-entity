use crate::{Entity, Event};
use async_trait::async_trait;
use std::error::Error;
use tokio::sync::broadcast::Sender;

#[async_trait]
pub trait StoreOf<E: Entity>: Send + Sync {
    async fn create(&self, entity: &E) -> Result<(), Box<dyn Error>>;
    async fn update(&self, id: &E::ID, update: &E::Update) -> Result<(), Box<dyn Error>>;
    async fn delete(&self, id: &E::ID) -> Result<(), Box<dyn Error>>;
    async fn watch(&self, channel: Sender<Event<E>>) -> Result<(), Box<dyn Error>>;
}
