mod store;
use api::{ledger_api_client::LedgerApiClient, CreateLedgerRequest, LedgerCreatedResponse};

pub async fn create_ledger(port: u16) -> api::Result<String> {
    let mut client = LedgerApiClient::connect(format!("http://[::1]:{}", port)).await?;
    let request = tonic::Request::new(CreateLedgerRequest {});
    let response = client.create(request).await?;
    Ok(response.into_inner().ledger_id)
}
