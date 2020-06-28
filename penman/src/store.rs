use crate::types;
use etcd_rs::*;

pub struct Store {
    client: Client,
}

impl Store {
    pub async fn new(endpoints: Vec<String>) -> types::Result<Store> {
        let config = ClientConfig {
            endpoints: endpoints,
            auth: None,
        };
        let client = Client::connect(config).await.unwrap();
        Ok(Store { client })
    }
}

impl Store {
    pub async fn put(&self, key: &str, value: &str) -> types::Result<()> {
        let request = PutRequest::new(key, value);
        match self.client.kv().put(request).await {
            Err(e) => Err(types::Error::EtcdError(e)),
            Ok(_) => Ok(()),
        }
    }

    pub async fn get_prefix(&self, prefix: &str) -> types::Result<Vec<String>> {
        let request = RangeRequest::new(KeyRange::prefix(prefix));
        let result = self.client.kv().range(request).await;
        let to_string = |kv: KeyValue| String::from_utf8(kv.value().to_vec()).unwrap();

        match result {
            Ok(mut response) => Ok(response.take_kvs().into_iter().map(to_string).collect()),
            Err(e) => Err(types::Error::EtcdError(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fetch_active_servers() {
        let etcd = vec!["http://localhost:2379".to_owned()];
        let store = Store::new(etcd).await.unwrap();
        store.put("testkey.1", "localhost:7777").await.unwrap();
        store.put("testkey.2", "localhost:8888").await.unwrap();

        let values = store.get_prefix("testkey.").await.unwrap();

        assert_eq!(values, vec!["localhost:7777", "localhost:8888"]);
    }
}
