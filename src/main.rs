mod files;
mod handle;
mod index;
mod ledger;
mod log;
mod segment;
mod test_util;
mod types;
use api::ledger_api_server::LedgerApiServer;
use clap::{App, Arg};
use std::path::PathBuf;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting ledgers..");
    let matches = App::new("Ledgers")
        .arg(Arg::with_name("port").long("port").takes_value(true))
        .arg(Arg::with_name("path").long("path").takes_value(true))
        .get_matches();

    let port = matches.value_of("port").unwrap_or("5678");
    let path = matches
        .value_of("path")
        .unwrap_or("./target/default_ledgers");
    let path = PathBuf::from(path);
    let addr = format!("[::1]:{}", port);
    let service = LedgerApiServer::new(service::new(path, 1000));
    Server::builder()
        .add_service(service)
        .serve(addr.parse()?)
        .await?;
    Ok(())
}
