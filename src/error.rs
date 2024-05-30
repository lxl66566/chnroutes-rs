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

pub type Result<T> = std::result::Result<T, Error>;
