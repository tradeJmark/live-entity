#![cfg(feature = "mongodb")]

use std::env;

use fullstack_entity::mongodb::MongoDBStore;
use test_utils::storage_test::test_storage_functions;

#[tokio::test]
#[ignore]
async fn test_mongodb_connector() {
    let connection_string = env::var("FSE_MONGODB_TEST_URL").expect("No MongoDB URL given.");
    let db = env::var("FSE_MONGODB_TEST_DB").expect("No MongoDB database name given.");
    let storage = MongoDBStore::new(connection_string, db, Some("fse_test".to_owned()))
        .await
        .expect("Failed to initialize MongoDB storage.");
    test_storage_functions(storage).await;
}
