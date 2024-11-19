use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Agent(#[from] ic_agent::AgentError),
    #[error("{0}")]
    Candid(#[from] candid::Error),
    #[error("{0}")]
    Metadata(#[from] yral_metadata_client::Error),
    #[error("error from yral canister: {0}")]
    YralCanister(String),
    #[error("invalid identity: {0}")]
    Identity(#[from] k256::elliptic_curve::Error),
    #[error("failed to get transactions: {0}")]
    GetTransactions(String),
    #[error("failed to parse transaction")]
    ParseTransaction,
    #[error("invalid tip certificate in ledger")]
    TipCertificate,
    #[error("{0}")]
    CborDe(#[from] ciborium::de::Error<io::Error>),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
