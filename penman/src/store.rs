use etcd_rs::*;

pub struct Store {
    client: Client,
}

impl Store {
    pub async fn new(endpoints: Vec<String>) -> Store {
        let config = ClientConfig {
            endpoints: endpoints,
            auth: None,
        };
        let client = Client::connect(config).await.unwrap();
        Store { client }
    }
}

impl Store {
    pub async fn put(&self, key: &str, value: &str) {
        self.client
            .kv()
            .put(PutRequest::new(key, value))
            .await
            .unwrap();
    }

    pub async fn get_ledger_servers(&self) -> Vec<String> {
        let mut resp = self
            .client
            .kv()
            .range(RangeRequest::new(KeyRange::prefix("ledgers.")))
            .await
            .unwrap();

        resp.take_kvs()
            .into_iter()
            .map(|kv| String::from_utf8(kv.value().to_vec()).unwrap())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fetch_active_servers() {
        let store = Store::new(vec!["http://localhost:2379".to_owned()]).await;
        store.put("ledgers.one", "localhost:7777").await;
        store.put("ledgers.two", "localhost:8888").await;

        let servers = store.get_ledger_servers().await;

        assert_eq!(servers, vec!["localhost:7777", "localhost:8888"]);
    }
}
