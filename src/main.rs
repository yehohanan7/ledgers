mod files;
mod handle;
mod index;
mod ledger;
mod log;
mod segment;
mod test_util;
mod types;
use api::ledger_api_server::LedgerApiServer;
use std::path::PathBuf;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("starting ledgers server...");
    let addr = "[::1]:50051".parse()?;
    let location = PathBuf::from("./target/default_ledgers");
    let service = LedgerApiServer::new(service::new(location, 1000));
    Server::builder().add_service(service).serve(addr).await?;
    Ok(())
}
