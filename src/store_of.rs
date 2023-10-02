use crate::{Entity, Event};
use async_trait::async_trait;
use std::{error::Error, fmt::Debug};
use tokio::sync::broadcast::Sender;

#[async_trait]
pub trait StoreOf<E: Entity>: Send + Sync {
    type Filter;
    async fn create(&self, entity: &E) -> Result<(), Box<dyn Error>>;
    async fn update(&self, id: &E::ID, update: &E::Update) -> Result<(), Box<dyn Error>>;
    async fn delete(&self, filter: Option<&Self::Filter>) -> Result<(), Box<dyn Error>>;
    async fn delete_by_id(&self, id: &E::ID) -> Result<(), Box<dyn Error>>;
    async fn get(&self, filter: Option<&Self::Filter>) -> Result<Vec<E>, Box<dyn Error>>;
    async fn get_by_id(&self, id: &E::ID) -> Result<E, Box<dyn Error>>;
    async fn watch(
        &self,
        channel: Sender<Event<E>>,
        filter: Option<&Self::Filter>,
    ) -> Result<(), Box<dyn Error>>;
}

#[derive(Debug)]
pub struct NotFoundError<T: Debug>(pub T);
impl<T: Debug> std::fmt::Display for NotFoundError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Not found: {:?}", self.0))
    }
}
impl<T: Debug> Error for NotFoundError<T> {}
