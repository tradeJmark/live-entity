use live_entity::{derive::{Entity, Updatable}, Event, SingletonEvent, Store, Singleton};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::channel;

#[derive(Entity, Clone, Serialize, Deserialize, Debug)]
#[entity_name = "employees"]
struct Employee {
    #[entity_id]
    #[serde(rename = "_id")]
    name: String,
    age: u8,
    children: u8,
}

#[derive(Entity, Clone, Serialize, Deserialize, Debug)]
#[entity_name = "stock_items"]
struct StockItem {
    #[entity_id]
    #[serde(rename = "_id")]
    item_name: String,
    price: f32,
}

pub async fn test_storage_functions<T: Store + 'static>(storage: T) {
    storage
        .delete_all::<Employee>()
        .await
        .expect("Failed to clear employees table");
    storage
        .delete_all::<StockItem>()
        .await
        .expect("Failed to clear employees table");

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
    let propane_accessory_id = "Spatula".to_owned();
    let propane_accessory = StockItem {
        item_name: propane_accessory_id.clone(),
        price: 12.34,
    };

    let (e_tx, mut e_rx) = channel(1);
    let (s_tx, mut s_rx) = channel(1);
    let e_store = storage.clone();
    let s_store = storage.clone();
    tokio::spawn(async move {
        e_store
            .watch::<Employee>(e_tx)
            .await
            .expect("Failed to initiate Employee watch.");
    });
    tokio::task::yield_now().await;
    tokio::spawn(async move {
        s_store
            .watch::<StockItem>(s_tx)
            .await
            .expect("Failed to initiate StockItem watch.");
    });
    tokio::task::yield_now().await;

    storage
        .create(&hank)
        .await
        .expect("Failed to create employee.");

    storage
        .create(&propane)
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
        .update::<Employee>(&hank_id, &UpdatedEmployee::default().age(new_age))
        .await
        .expect("Error updating employee.");
    storage
        .update::<StockItem>(&propane_id, &UpdatedStockItem::default().price(new_price))
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
        _ => panic!("Received wrong type of event on stock item update."),
    }

    storage
        .create(&propane_accessory)
        .await
        .expect("Failed to create second stock item.");
    s_rx.recv().await.unwrap();

    let stock_items = storage
        .get_all::<StockItem>()
        .await
        .expect("Failed to get stock items.");
    assert!(stock_items.iter().any(|si| si.item_name == propane_id));
    assert!(stock_items
        .iter()
        .any(|si| si.item_name == propane_accessory_id));
    let retrieved_propane_accessory = storage
        .get_by_id(&propane_accessory_id)
        .await
        .expect("failed retrieving stock item by ID.");
    assert_eq!(propane_accessory, retrieved_propane_accessory);

    storage
        .delete_by_id::<Employee>(&hank_id)
        .await
        .expect("Failed to delete employee.");
    storage
        .delete_by_id::<StockItem>(&propane_id)
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
    storage.delete_all::<StockItem>().await.unwrap();
}

#[derive(Serialize, Deserialize, Clone, Debug, Updatable, Eq, PartialEq)]
struct HomePage {
    header: String,
    body: String
}

impl Singleton for HomePage {
    type Update = UpdatedHomePage;
    const TYPE_NAME: &'static str = "pages";
    const ENTITY_ID: &'static str = "home";
}

pub async fn test_storage_singleton_functions<T: Store + 'static>(storage: T) {
    let hp = HomePage { header: "Welcome!".to_owned(), body: "Please stay long enough to see some ads".to_owned() };
    storage.create_singleton(&hp).await.expect("Failed to create singleton.");

    let retrieved = storage.get_singleton::<HomePage>().await.expect("Failed to retrieve stored singleton.");
    assert_eq!(hp, retrieved);

    let (tx, mut rx) = channel(1);
    let clone_store = storage.clone();
    tokio::spawn(async move {
        clone_store
            .watch_singleton::<HomePage>(tx, 1)
            .await
            .expect("Failed to initiate singleton watch.");
    });
    tokio::task::yield_now().await;

    let updated_body = "Subscribe to our Patreon for ad-free content!".to_owned();
    let update = UpdatedHomePage::default().body(updated_body.clone());
    storage.update_singleton::<HomePage>(&update).await.expect("Failed to update singleton.");

    let event = rx.recv().await.expect("Error receiving singleton event.");
    match event {
        SingletonEvent::Update(update) => {
            assert_eq!(None, update.header);
            assert_eq!(Some(updated_body), update.body);
        }
        _ => panic!("Did not recieve an update event for singleton.")
    }

    storage.delete_singleton::<HomePage>().await.expect("Failed to delete singleton.");
    if storage.get_singleton::<HomePage>().await.is_ok() {
        panic!("Singleton was not deleted.")
    }
}