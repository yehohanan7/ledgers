mod utils;

#[tokio::test]
async fn create_new_ledger() {
    let (tx, port) = utils::start_server().await;

    let ledger_id = penman::create_ledger(port).await.unwrap();

    assert_eq!(ledger_id.len(), 36);
    tx.send(()).unwrap();
}
