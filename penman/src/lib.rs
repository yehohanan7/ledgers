pub mod store;
mod types;
use api::{ledger_api_client::LedgerApiClient, CreateLedgerRequest};
use store::*;
use tonic::transport::channel::Channel;
use types::*;

pub struct Penman {
    store: Store,
    clients: Vec<Client>,
}

struct Client {
    endpoint: String,
    api: LedgerApiClient<Channel>,
}

impl Penman {
    pub async fn new(etcd: Vec<String>) -> Result<Penman> {
        let mut clients = vec![];
        let store = Store::new(etcd).await?;
        for endpoint in store.get_prefix("/ledgers/nodes/").await? {
            clients.push(Client {
                endpoint: endpoint.clone(),
                api: LedgerApiClient::connect(endpoint).await?,
            });
        }
        Ok(Penman { store, clients })
    }

    pub async fn create_ledger(&mut self) -> Result<String> {
        let client = self.clients.get_mut(0).unwrap();
        let request = tonic::Request::new(CreateLedgerRequest {});
        let response = client.api.create(request).await?;
        let ledger_id = response.into_inner().ledger_id;
        let initial_segment = format!("/ledgers/{}/partitions/0/segments/0", ledger_id);
        self.store.put(&initial_segment, &client.endpoint).await?;
        Ok(ledger_id)
    }
}

pub async fn new(etcd: Vec<String>) -> Result<Penman> {
    Penman::new(etcd).await
}
