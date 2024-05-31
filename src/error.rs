use thiserror::Error;
#[derive(Error, Debug)]
pub enum Error {
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Cache error: {0}")]
    CacheError(#[from] CacheError),
    #[error("Exec error: {0}")]
    ExecError(#[from] ExecError),
    #[error("Invalid target")]
    InvalidTarget,
    #[error("Route operation error: {0}")]
    RouteOpError(#[from] RouteOpError),
}

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("")]
    IOError(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum ExecError {
    #[error("")]
    ExecError(#[from] std::io::Error),
}

/// Error type for route table operation.
#[derive(Error, Debug)]
pub enum RouteOpError {
    #[error("Any IO Error")]
    OpError(#[from] std::io::Error),
    #[error("cannot find system default gateway")]
    NoGatewayError,
    #[error("cannot create handle")]
    HandleInitError,
    #[error("futures join error: {0}")]
    FutureError(#[from] tokio::task::JoinError),
    #[error("Get default interface error: {0}")]
    GetInterfaceError(String),
}

pub type Result<T> = std::result::Result<T, Error>;
