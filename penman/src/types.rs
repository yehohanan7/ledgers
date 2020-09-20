pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    TranportError(tonic::transport::Error),
    RpcError(tonic::Code),
    EtcdError(Box<dyn std::error::Error + Send + Sync + 'static>),
    KeyNotUnique,
    KeyNotFound,
}

impl std::convert::From<tonic::transport::Error> for Error {
    fn from(e: tonic::transport::Error) -> Self {
        Error::TranportError(e)
    }
}

impl std::convert::From<tonic::Status> for Error {
    fn from(status: tonic::Status) -> Self {
        Error::RpcError(status.code())
    }
}
