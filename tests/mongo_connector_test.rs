#![cfg(feature = "mongodb")]

use std::env;

use fullstack_entity::mongodb::MongoDBStore;
use test_utils::storage_test::*;

async fn get_store() -> MongoDBStore {
    let connection_string = env::var("FSE_MONGODB_TEST_URL").expect("No MongoDB URL given.");
    let db = env::var("FSE_MONGODB_TEST_DB").expect("No MongoDB database name given.");
    MongoDBStore::new(connection_string, db, Some("fse_test".to_owned()))
        .await
        .expect("Failed to initialize MongoDB storage.")
}

#[tokio::test]
#[ignore]
async fn test_mongodb_connector() {
    let storage = get_store().await;
    test_storage_functions(storage).await;
}

#[tokio::test]
#[ignore]
async fn test_mongodb_connector_singletons() {
    let storage = get_store().await;
    test_storage_singleton_functions(storage).await;
}