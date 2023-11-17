#![cfg(feature = "in-mem")]

use live_entity::in_mem::InMemStore;
use test_utils::storage_test::{test_storage_functions, test_storage_singleton_functions};

#[tokio::test]
async fn test_in_mem_store() {
    let storage = InMemStore::new(1);
    test_storage_functions(storage).await;
}

#[tokio::test]
async fn test_in_mem_store_singletons() {
    let storage = InMemStore::new(1);
    test_storage_singleton_functions(storage).await;
}
