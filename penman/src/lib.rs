mod store;
use api::{ledger_api_client::LedgerApiClient, CreateLedgerRequest, LedgerCreatedResponse};
use store::*;

pub struct Penman {
    store: Store,
}

impl Penman {
    pub async fn new(etcd: Vec<String>) -> api::Result<Penman> {
        let store = Store::new(etcd).await;
        Ok(Penman { store })
    }

    pub async fn create_ledger(&self, port: u16) -> api::Result<String> {
        let mut client = LedgerApiClient::connect(format!("http://[::1]:{}", port)).await?;
        let request = tonic::Request::new(CreateLedgerRequest {});
        let response = client.create(request).await?;
        Ok(response.into_inner().ledger_id)
    }
}

pub async fn new(etcd: Vec<String>) -> api::Result<Penman> {
    Penman::new(etcd).await
}
