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
use clap::{App, Arg};
use std::path::PathBuf;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting ledgers..");
    let matches = App::new("Ledgers")
        .arg(Arg::with_name("etcd").long("etcd").takes_value(true))
        .arg(Arg::with_name("name").long("name").takes_value(true))
        .arg(Arg::with_name("port").long("port").takes_value(true))
        .get_matches();

    let etcd_address = matches.value_of("etcd").unwrap();
    let etcd_address = etcd_address.split(",").map(|r| r.to_owned()).collect();
    let port = matches.value_of("port").unwrap();
    let addr = format!("[::1]:{}", port);
    let name = format!("ledgers.{}", matches.value_of("name").unwrap());
    meta::register(&name, &addr, etcd_address).await;

    let location = PathBuf::from("./target/default_ledgers");
    let service = LedgerApiServer::new(service::new(location, 1000));
    Server::builder()
        .add_service(service)
        .serve(addr.parse()?)
        .await?;
    Ok(())
}
