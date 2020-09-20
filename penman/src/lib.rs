pub mod store;
mod types;
use api::{ledger_api_client::LedgerApiClient, CreateLedgerRequest};
use store::*;
use tonic::transport::channel::Channel;
use types::*;

pub struct Penman {
    store: Store,
    nodes: Vec<Node>,
}

struct Node {
    endpoint: String,
    client: LedgerApiClient<Channel>,
}

pub struct Ledger {
    pub id: String,
}

impl Ledger {
    pub async fn new(id: String, node: &str, store: &Store) -> Result<Ledger> {
        let segment_key = format!("/ledgers/{}/partitions/0/segments/0", id);
        store.put(&segment_key, &node).await?;
        Ok(Ledger { id })
    }
}

impl Penman {
    pub async fn new(etcd: Vec<String>) -> Result<Penman> {
        let mut nodes = vec![];
        let store = Store::new(etcd).await?;
        for endpoint in store.get_prefix("/ledgers/nodes/").await? {
            nodes.push(Node {
                endpoint: endpoint.clone(),
                client: LedgerApiClient::connect(endpoint).await?,
            });
        }
        Ok(Penman { store, nodes })
    }

    pub async fn create_ledger(&mut self) -> Result<Ledger> {
        let node = self.nodes.get_mut(0).unwrap();
        let request = tonic::Request::new(CreateLedgerRequest {});
        let response = node.client.create(request).await?;
        let ledger_id = response.into_inner().ledger_id;
        Ledger::new(ledger_id, &node.endpoint, &self.store).await
    }
}

pub async fn new(etcd: Vec<String>) -> Result<Penman> {
    Penman::new(etcd).await
}
