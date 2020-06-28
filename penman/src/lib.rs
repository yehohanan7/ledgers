mod store;
use api::{ledger_api_client::LedgerApiClient, CreateLedgerRequest};
use store::*;
use tonic::transport::channel::Channel;

pub struct Penman {
    store: Store,
    ledgers: Vec<LedgerApiClient<Channel>>,
}

impl Penman {
    pub async fn new(etcd: Vec<String>) -> api::Result<Penman> {
        let mut ledgers = vec![];
        let store = Store::new(etcd).await;
        for endpoint in store.get_prefix("ledgers.").await {
            println!("url: {}", endpoint);
            ledgers.push(LedgerApiClient::connect(endpoint).await?);
        }
        Ok(Penman { store, ledgers })
    }

    pub async fn create_ledger(&mut self) -> api::Result<String> {
        let client = self.ledgers.get_mut(0).unwrap();
        let request = tonic::Request::new(CreateLedgerRequest {});
        let response = client.create(request).await?;
        Ok(response.into_inner().ledger_id)
    }
}

pub async fn new(etcd: Vec<String>) -> api::Result<Penman> {
    Penman::new(etcd).await
}
