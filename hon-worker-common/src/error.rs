use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug, Error)]
pub enum WorkerError {
    #[error("Invalid Signature")]
    InvalidSignature,
    #[error("internal error: {0}")]
    Internal(String),
    #[error("user has already voted on this post")]
    AlreadyVotedOnPost,
    #[error("post not found")]
    PostNotFound,
    #[error("user does not have sufficient balance")]
    InsufficientFunds,
}
