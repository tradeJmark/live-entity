#![cfg(feature = "in-mem")]

use fullstack_entity::in_mem::InMemStore;
use test_utils::storage_test::test_storage_functions;

#[tokio::test]
async fn test_in_mem_store() {
    let storage = InMemStore::new(1);
    test_storage_functions(storage).await;
}
