use etcd_rs::*;

pub async fn register(endpoints: Vec<String>) {
    println!("Registering with {:?}", endpoints);
    let config = ClientConfig {
        endpoints: endpoints,
        auth: None,
    };
    let client = Client::connect(config).await.unwrap();
    let request = PutRequest::new("ledgers.servera", "servera");
    client.kv().put(request).await.unwrap();
}
