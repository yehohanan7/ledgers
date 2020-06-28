mod store;
mod types;
use api::{ledger_api_client::LedgerApiClient, CreateLedgerRequest};
use store::*;
use tonic::transport::channel::Channel;
use types::*;

pub struct Penman {
    store: Store,
    clients: Vec<LedgerApiClient<Channel>>,
}

impl Penman {
    pub async fn new(etcd: Vec<String>) -> Result<Penman> {
        let mut clients = vec![];
        let store = Store::new(etcd).await?;
        for endpoint in store.get_prefix("ledgers.").await? {
            clients.push(LedgerApiClient::connect(endpoint).await?);
        }
        Ok(Penman { store, clients })
    }

    pub async fn create_ledger(&mut self) -> Result<String> {
        let client = self.clients.get_mut(0).unwrap();
        let request = tonic::Request::new(CreateLedgerRequest {});
        let response = client.create(request).await?;
        Ok(response.into_inner().ledger_id)
    }
}

pub async fn new(etcd: Vec<String>) -> Result<Penman> {
    Penman::new(etcd).await
}
