use thiserror::Error;

#[derive(Error, Debug)]
pub enum MdqError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("entity not found: {0}")]
    EntityNotFound(String),

    #[error("invalid XML metadata: {0}")]
    InvalidXml(String),

    #[error("signature verification failed: {0}")]
    SignatureError(String),

    #[error("invalid entity ID: {0}")]
    InvalidEntityId(String),
}

pub type Result<T> = std::result::Result<T, MdqError>;
