mod files;
mod handle;
mod index;
mod ledger;
mod log;
mod segment;
mod test_util;
mod types;

use api::ledger_api_server::LedgerApi;
use api::{CreateLedgerRequest, LedgerCreatedResponse};
use std::path::PathBuf;
use tonic::{Request, Response, Status};

pub struct LedgerService {
    path: PathBuf,
    segment_size: u64,
    repository: ledger::LedgerRepository,
}

#[tonic::async_trait]
impl LedgerApi for LedgerService {
    async fn create(
        &self,
        _request: Request<CreateLedgerRequest>,
    ) -> Result<Response<LedgerCreatedResponse>, Status> {
        println!("creating new ledger...");
        let repo = &self.repository;
        let response = match repo.create(&self.path, self.segment_size).await {
            Ok(id) => api::LedgerCreatedResponse { ledger_id: id },
            Err(_) => panic!("error creating ledger"), //TODO: handle error gracefully
        };
        Ok(Response::new(response))
    }
}

pub fn new(path: PathBuf, segment_size: u64) -> LedgerService {
    LedgerService {
        path: path,
        segment_size: segment_size,
        repository: ledger::new_repository(),
    }
}
