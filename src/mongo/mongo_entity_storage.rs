use crate::{Entity, Event, StoreOf};
use async_trait::async_trait;
use futures_util::StreamExt;
use mongodb::bson::{doc, from_bson, from_document, to_bson, to_document};
use mongodb::change_stream::event::{ChangeStreamEvent, OperationType};
use mongodb::options::ClientOptions;
use mongodb::{Client, Database};
use std::error::Error;
use std::fmt::Formatter;
use tokio::sync::broadcast::Sender;

#[derive(Clone)]
pub struct MongoEntityStorage {
    db: Database,
}

impl MongoEntityStorage {
    pub async fn new(
        connection_string: String,
        database_name: String,
        app_name: Option<String>,
    ) -> Result<Self, mongodb::error::Error> {
        let mut options = ClientOptions::parse(connection_string).await?;
        options.app_name = app_name;
        let client = Client::with_options(options);
        client
            .map(|c| c.database(&database_name))
            .map(|db| Self { db })
    }
}

impl Into<MongoEntityStorage> for Database {
    fn into(self) -> MongoEntityStorage {
        MongoEntityStorage { db: self }
    }
}

#[derive(Debug)]
pub struct MongoContractViolationError(String);
impl std::fmt::Display for MongoContractViolationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl Error for MongoContractViolationError {}

#[async_trait]
impl<E: MongoEntity> StoreOf<E> for MongoEntityStorage {
    async fn create(&self, entity: &E) -> Result<(), Box<dyn Error>> {
        let collection = self.db.collection::<E>(E::COLLECTION_NAME);
        collection.insert_one(entity, None).await?;
        Ok(())
    }

    async fn update(&self, id: &E::ID, update: &E::Update) -> Result<(), Box<dyn Error>> {
        let collection = self.db.collection::<E>(E::COLLECTION_NAME);
        let query = doc! { "_id": to_bson(id)? };
        let update = vec![doc! {
            "$set": to_document(&update)?
        }];
        collection.update_one(query, update, None).await?;
        Ok(())
    }

    async fn delete(&self, id: &E::ID) -> Result<(), Box<dyn Error>> {
        let collection = self.db.collection::<E>(E::COLLECTION_NAME);
        let query = doc! { "_id": to_bson(id)? };
        collection.delete_one(query, None).await?;
        Ok(())
    }

    async fn watch(&self, channel: Sender<Event<E>>) -> Result<(), Box<dyn Error>> {
        let collection = self.db.collection::<E>(E::COLLECTION_NAME);
        let pipeline = [doc! { "$match": {
            "operationType": {
                "$in": to_bson(&[OperationType::Update, OperationType::Insert, OperationType::Delete, OperationType::Replace])?
            }
        } }];
        let mut watch = collection.watch(pipeline, None).await?;
        while let Some(evt) = watch.next().await.transpose()? {
            match evt.operation_type {
                OperationType::Insert => {
                    let doc = evt.full_document.ok_or(MongoContractViolationError(
                        "Mongo did not provide full document on insert event".to_owned(),
                    ))?;
                    channel.send(Event::Create(doc))?;
                }
                OperationType::Update => {
                    let id: E::ID = get_id_from_change_event(&evt)?;
                    let doc = evt
                        .update_description
                        .ok_or(MongoContractViolationError(
                            "Mongo did not provide update description on update event".to_owned(),
                        ))?
                        .updated_fields;
                    let update: E::Update = from_document(doc)?;
                    channel.send(Event::Update { id, update })?;
                }
                OperationType::Delete => {
                    let id: E::ID = get_id_from_change_event(&evt)?;
                    channel.send(Event::Delete(id))?;
                }
                OperationType::Replace => {
                    let doc = evt.full_document.ok_or(MongoContractViolationError(
                        "Mongo did not provide full document on insert event".to_owned(),
                    ))?;
                    channel.send(Event::Update {
                        id: doc.get_id().clone(),
                        update: doc.into(),
                    })?;
                }
                _ => {
                    return Err(MongoContractViolationError(format!(
                        "Mongo returned an event type that was filtered out: {:?}.",
                        evt.operation_type
                    ))
                    .into())
                }
            }
        }
        Ok(())
    }
}

pub trait MongoEntity: Entity {
    const COLLECTION_NAME: &'static str;
}

fn get_id_from_change_event<E: Entity>(
    event: &ChangeStreamEvent<E>,
) -> Result<E::ID, Box<dyn Error>> {
    let id = from_bson(
        event
            .document_key
            .as_ref()
            .ok_or(MongoContractViolationError(
                "Mongo did not provide document key on change event".to_owned(),
            ))?
            .get("_id")
            .cloned()
            .ok_or(MongoContractViolationError(
                "Mongo provided no _id on document key".to_owned(),
            ))?,
    )?;
    Ok(id)
}
