mod utils;

#[tokio::test]
async fn create_new_ledger() {
    let tx = utils::start_server().await;
    let etcd = vec!["http://localhost:2379".to_owned()];
    let store = penman::store::Store::new(etcd.clone()).await.unwrap();
    let mut penman = penman::new(etcd).await.unwrap();

    let ledger = penman.create_ledger().await.unwrap();

    assert_eq!(ledger.id.len(), 36);
    assert!(store
        .get(&format!("/ledgers/{}/partitions/0/segments/0", ledger.id))
        .await
        .unwrap()
        .unwrap()
        .contains("http://127.0.0.1:"));
    tx.send(()).unwrap();
}
