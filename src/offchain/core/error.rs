// error — unified error type for the off-chain layer
//
// Maps failures from the smplx-sdk provider, signer, program, and std I/O
// into a single enum so engine code can use a single `Result` alias.

use thiserror::Error;

/// All off-chain errors reduce to this enum.
#[derive(Debug, Error)]
pub enum Error {
    #[error("provider error: {0}")]
    Provider(String),

    #[error("signer error: {0}")]
    Signer(String),

    #[error("program error: {0}")]
    Program(String),

    #[error("asset error: {0}")]
    Asset(String),

    #[error("order book error: {0}")]
    OrderBook(String),

    #[error("settlement error: {0}")]
    Settlement(String),

    #[error("server error: {0}")]
    Server(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

impl From<simplex::provider::ProviderError> for Error {
    fn from(e: simplex::provider::ProviderError) -> Self {
        Error::Provider(e.to_string())
    }
}

impl From<simplex::signer::SignerError> for Error {
    fn from(e: simplex::signer::SignerError) -> Self {
        Error::Signer(e.to_string())
    }
}

impl From<simplex::program::ProgramError> for Error {
    fn from(e: simplex::program::ProgramError) -> Self {
        Error::Program(e.to_string())
    }
}

/// Convenience alias used across the off-chain layer.
pub type Result<T> = std::result::Result<T, Error>;
