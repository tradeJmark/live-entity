#![cfg(feature = "mongo")]
use fullstack_entity::backend::mongo::{MongoEntity, MongoEntityStorage};
use fullstack_entity::derive::{storage_wrapper, Entity};
use fullstack_entity::Event;
use fullstack_entity_derive::MongoStorage;
use serde::{Deserialize, Serialize};
use std::env;
use tokio::sync::broadcast::channel;
#[derive(Entity, Clone, Serialize, Deserialize, Debug)]
struct Employee {
    #[entity_id]
    #[serde(rename = "_id")]
    name: String,
    age: u8,
    children: u8,
}

#[derive(Entity, Clone, Serialize, Deserialize, Debug)]
struct StockItem {
    #[entity_id]
    #[serde(rename = "_id")]
    item_name: String,
    price: f32,
}

#[storage_wrapper{employee: Employee, stock_item: StockItem}]
#[derive(Clone, MongoStorage)]
#[mongo_collections{"employees": Employee, "stock_items": StockItem}]
struct Storage;

#[cfg(feature = "mongo")]
#[tokio::test]
#[ignore]
async fn test_mongo_connector() {
    let connection_string = env::var("FSE_MONGO_TEST_URL").expect("No Mongo URL given.");
    let db = env::var("FSE_MONGO_TEST_DB").expect("No Mongo DB name given.");
    let storage = Storage::of_mongo(connection_string, db, None)
        .await
        .expect("Error creating DB connection.");

    let hank_id = "Hank Hill".to_owned();
    let hank = Employee {
        name: hank_id.clone(),
        age: 49,
        children: 1,
    };

    let propane_id = "Propane".to_owned();
    let propane = StockItem {
        item_name: propane_id.clone(),
        price: 100.200,
    };

    let (e_tx, mut e_rx) = channel(1);
    let (s_tx, mut s_rx) = channel(1);
    let e_store = storage.clone();
    let s_store = storage.clone();
    tokio::spawn(async move {
        e_store
            .watch_employee(e_tx)
            .await
            .expect("Failed to initiate Employee watch.");
    });
    tokio::spawn(async move {
        s_store
            .watch_stock_item(s_tx)
            .await
            .expect("Failed to initiate StockItem watch.");
    });

    storage
        .create_employee(&hank)
        .await
        .expect("Failed to create employee.");
    storage
        .create_stock_item(&propane)
        .await
        .expect("Failed to create stock item.");
    let e_event = e_rx.recv().await.expect("Error receiving employee event.");
    let s_event = s_rx
        .recv()
        .await
        .expect("Error receiving stock item event.");
    match e_event {
        Event::Create(e) => assert_eq!(hank_id, e.name),
        _ => panic!("Received wrong type of event for employee creation."),
    }
    match s_event {
        Event::Create(si) => assert_eq!(propane_id, si.item_name),
        _ => panic!("Received wrong type of event for stock item creation."),
    }

    let new_age = 34;
    let new_price = 123.45;
    storage
        .update_employee(&hank_id, &UpdatedEmployee::default().age(new_age))
        .await
        .expect("Error updating employee.");
    storage
        .update_stock_item(&propane_id, &UpdatedStockItem::default().price(new_price))
        .await
        .expect("Error updating stock item.");
    let e_event = e_rx
        .recv()
        .await
        .expect("Failed to receive employee update message.");
    let s_event = s_rx
        .recv()
        .await
        .expect("Failed to receive stock item update message.");
    match e_event {
        Event::Update { id, update } => {
            assert_eq!(hank_id, id);
            assert_eq!(Some(new_age), update.age);
        }
        _ => panic!("Received wrong type of event on employee update."),
    }
    match s_event {
        Event::Update { id, update } => {
            assert_eq!(propane_id, id);
            assert_eq!(Some(new_price), update.price);
        }
        _ => panic!("Received wrong type of event on employee update."),
    }

    storage
        .delete_employee(&hank_id)
        .await
        .expect("Failed to delete employee.");
    storage
        .delete_stock_item(&propane_id)
        .await
        .expect("Failed to delete stock item.");
    let e_event = e_rx
        .recv()
        .await
        .expect("Failed receiving employee delete message.");
    let s_event = s_rx
        .recv()
        .await
        .expect("Failed receiving stock item delete message.");
    match e_event {
        Event::Delete(id) => assert_eq!(hank_id, id),
        _ => panic!("Received wrong event type on employee delete."),
    }
    match s_event {
        Event::Delete(id) => assert_eq!(propane_id, id),
        _ => panic!("Received wrong event type on stock item delete."),
    }
}
