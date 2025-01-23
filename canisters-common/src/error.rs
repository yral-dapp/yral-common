use std::{io, str::FromStr};

use candid::Nat;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PndError {
    #[error("worker didn't return a number: {0}")]
    Parse(<Nat as FromStr>::Err),
    #[error("network error when accessing worker: {0}")]
    Network(#[from] reqwest::Error),
}

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
    #[error("{0}")]
    PndError(#[from] PndError),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
