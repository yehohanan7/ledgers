use etcd_rs::*;

pub async fn register(name: &str, endpoint: &str, etcd_endpoints: Vec<String>) {
    println!("Registering with {:?}", etcd_endpoints);
    let config = ClientConfig {
        endpoints: etcd_endpoints,
        auth: None,
    };
    let client = Client::connect(config).await.unwrap();
    let request = PutRequest::new(name, endpoint);
    client.kv().put(request).await.unwrap();
}
