#[cfg(feature = "local")]
mod local;
#[cfg(not(feature = "local"))]
mod remote;

#[cfg(feature = "local")]
pub use local::*;
#[cfg(not(feature = "local"))]
pub use remote::*;

pub mod canister_ids;

pub const GOBGOB_TOTAL_COUNT: u32 = 18557;
pub const GOBGOB_PROPIC_URL: &str = "https://imagedelivery.net/abXI9nS4DYYtyR1yFFtziA/gob.";

pub const CDAO_SWAP_PRE_READY_TIME_SECS: u64 = 150;
pub const CDAO_SWAP_TIME_SECS: u64 = CDAO_SWAP_PRE_READY_TIME_SECS + 150;
