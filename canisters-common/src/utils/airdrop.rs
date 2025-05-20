use hon_worker_common::{AirdropClaimError, ClaimRequest};

use crate::Canisters;

impl Canisters<true> {
    pub async fn claim_sats_airdrop(&self, request: ClaimRequest) -> Result<(), AirdropClaimError> {
        todo!("implement the actual call to worker")
    }
}
