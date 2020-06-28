mod utils;

#[tokio::test]
async fn create_new_ledger() {
    let etcd = "http://localhost:2379".to_owned();
    let (tx, port) = utils::start_server(etcd.clone()).await;

    let penman = penman::new(vec![etcd]).await.unwrap();
    let id = penman.create_ledger(port).await.unwrap();

    assert_eq!(id.len(), 36);
    tx.send(()).unwrap();
}
