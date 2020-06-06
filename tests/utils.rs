use api::ledger_api_server::LedgerApiServer;
use futures::prelude::*;
use std::net::TcpListener;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::oneshot::{self, Sender};
use tonic::transport::Server;

pub async fn start_server() -> (Sender<()>, u16) {
    let (tx, rx) = oneshot::channel::<()>();
    let port = get_available_port().unwrap();
    tokio::spawn(async move {
        let addr = format!("[::1]:{}", port).parse().unwrap();
        let location = PathBuf::from("./target/default_ledgers");
        Server::builder()
            .add_service(LedgerApiServer::new(service::new(location, 1000)))
            .serve_with_shutdown(addr, rx.map(drop))
            .await
            .unwrap();
    });
    tokio::time::delay_for(Duration::from_millis(100)).await;
    (tx, port)
}

fn get_available_port() -> Option<u16> {
    (1025..65535).find(|port: &u16| match TcpListener::bind(("127.0.0.1", *port)) {
        Ok(_) => true,
        Err(_) => false,
    })
}
