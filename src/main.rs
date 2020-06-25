mod files;
mod handle;
mod index;
mod ledger;
mod log;
mod meta;
mod segment;
mod test_util;
mod types;
use api::ledger_api_server::LedgerApiServer;
use std::env;
use std::path::PathBuf;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting ledgers...");
    let args: Vec<String> = env::args().collect();
    if let Some(ips) = args.get(2) {
        meta::register(ips.split(",").map(|r| r.to_owned()).collect()).await;
    }
    let addr = "[::1]:50051".parse()?;
    let location = PathBuf::from("./target/default_ledgers");
    let service = LedgerApiServer::new(service::new(location, 1000));
    Server::builder().add_service(service).serve(addr).await?;
    Ok(())
}
