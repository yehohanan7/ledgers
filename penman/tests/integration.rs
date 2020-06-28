mod utils;

#[tokio::test]
async fn create_new_ledger() {
    let tx = utils::start_server().await;
    let etcd = vec!["http://localhost:2379".to_owned()];
    let mut penman = penman::new(etcd).await.unwrap();

    let id = penman.create_ledger().await.unwrap();

    assert_eq!(id.len(), 36);
    tx.send(()).unwrap();
}
