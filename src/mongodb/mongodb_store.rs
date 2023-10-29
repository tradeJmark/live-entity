use crate::{Entity, Event, NotFoundError, Store};
use async_trait::async_trait;
use futures_util::{StreamExt, TryStreamExt};
use mongodb::bson::{doc, from_bson, from_document, to_bson, to_document, Document};
use mongodb::change_stream::event::{ChangeStreamEvent, OperationType};
use mongodb::options::{ChangeStreamOptions, ClientOptions, FullDocumentType};
use mongodb::{Client, Database};
use std::error::Error;
use std::fmt::Formatter;
use tokio::sync::broadcast::Sender;

#[derive(Clone)]
pub struct MongoDBStore {
    db: Database,
}

impl MongoDBStore {
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

    async fn delete_filtered<E: Entity>(
        &self,
        filter: Option<Document>,
    ) -> Result<(), Box<dyn Error>> {
        let collection = self.db.collection::<E>(E::NAME);
        collection
            .delete_many(filter.unwrap_or(doc! {}), None)
            .await?;
        Ok(())
    }

    async fn get_filtered<E: Entity>(
        &self,
        filter: Option<Document>,
    ) -> Result<Vec<E>, Box<dyn Error>> {
        let collection = self.db.collection::<E>(E::NAME);
        let res = collection.find(filter, None).await?;
        Ok(res.try_collect().await?)
    }

    async fn watch_filtered<E: Entity>(
        &self,
        channel: Sender<Event<E>>,
        filter: Option<Document>,
    ) -> Result<(), Box<dyn Error>> {
        let collection = self.db.collection::<E>(E::NAME);
        let mut mtch = doc! { "$match": {
            "operationType": {
                "$in": to_bson(&[OperationType::Update, OperationType::Insert, OperationType::Delete, OperationType::Replace])?
            }
        } };
        if let Some(f) = filter {
            for (k, v) in f {
                mtch.insert(&format!("fullDocument.{}", k), v);
            }
        }
        let options = ChangeStreamOptions::builder()
            .full_document(Some(FullDocumentType::UpdateLookup))
            .build();
        let mut watch: mongodb::change_stream::ChangeStream<ChangeStreamEvent<E>> =
            collection.watch([mtch], options).await?;
        while let Some(evt) = watch.next().await.transpose()? {
            match evt.operation_type {
                OperationType::Insert => {
                    let doc = evt.full_document.ok_or(MongoDBContractViolationError(
                        "MongoDB did not provide full document on insert event".to_owned(),
                    ))?;
                    channel.send(Event::Create(doc))?;
                }
                OperationType::Update => {
                    let id: E::ID = get_id_from_change_event(&evt)?;
                    let doc = evt
                        .update_description
                        .ok_or(MongoDBContractViolationError(
                            "MongoDB did not provide update description on update event".to_owned(),
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
                    let doc = evt.full_document.ok_or(MongoDBContractViolationError(
                        "MongoDB did not provide full document on insert event".to_owned(),
                    ))?;
                    channel.send(Event::Update {
                        id: doc.get_id().clone(),
                        update: doc.into(),
                    })?;
                }
                _ => {
                    return Err(MongoDBContractViolationError(format!(
                        "MongoDB returned an event type that was filtered out: {:?}.",
                        evt.operation_type
                    ))
                    .into())
                }
            }
        }
        Ok(())
    }
}

impl Into<MongoDBStore> for Database {
    fn into(self) -> MongoDBStore {
        MongoDBStore { db: self }
    }
}

#[derive(Debug)]
pub struct MongoDBContractViolationError(String);
impl std::fmt::Display for MongoDBContractViolationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl Error for MongoDBContractViolationError {}

#[async_trait]
impl Store for MongoDBStore {
    async fn create<E: Entity>(&self, entity: &E) -> Result<(), Box<dyn Error>> {
        let collection = self.db.collection::<E>(E::NAME);
        collection.insert_one(entity, None).await?;
        Ok(())
    }

    async fn update<E: Entity>(
        &self,
        id: &E::ID,
        update: &E::Update,
    ) -> Result<(), Box<dyn Error>> {
        let collection = self.db.collection::<E>(E::NAME);
        let query = doc! { "_id": to_bson(id)? };
        let update = vec![doc! {
            "$set": to_document(&update)?
        }];
        collection.update_one(query, update, None).await?;
        Ok(())
    }

    async fn delete_all<E: Entity>(&self) -> Result<(), Box<dyn Error>> {
        self.delete_filtered::<E>(None).await
    }

    async fn delete_by_id<E: Entity>(&self, id: &E::ID) -> Result<(), Box<dyn Error>> {
        let collection = self.db.collection::<E>(E::NAME);
        let query = doc! { "_id": to_bson(id)? };
        collection.delete_one(query, None).await?;
        Ok(())
    }

    async fn get_all<E: Entity>(&self) -> Result<Vec<E>, Box<dyn Error>> {
        self.get_filtered(None).await
    }

    async fn get_by_id<E: Entity>(&self, id: &E::ID) -> Result<E, Box<dyn Error>> {
        let collection = self.db.collection::<E>(E::NAME);
        let query = doc! { "_id": to_bson(id)? };
        collection
            .find_one(query, None)
            .await?
            .ok_or(NotFoundError(id.clone()).into())
    }

    async fn watch<E: Entity>(&self, channel: Sender<Event<E>>) -> Result<(), Box<dyn Error>> {
        self.watch_filtered(channel, None).await
    }
}

fn get_id_from_change_event<E: Entity>(
    event: &ChangeStreamEvent<E>,
) -> Result<E::ID, Box<dyn Error>> {
    let id = from_bson(
        event
            .document_key
            .as_ref()
            .ok_or(MongoDBContractViolationError(
                "MongoDB did not provide document key on change event".to_owned(),
            ))?
            .get("_id")
            .cloned()
            .ok_or(MongoDBContractViolationError(
                "MongoDB provided no _id on document key".to_owned(),
            ))?,
    )?;
    Ok(id)
}
