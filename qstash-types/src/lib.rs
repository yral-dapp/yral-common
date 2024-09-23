use candid::Principal;
use serde::{Deserialize, Serialize};
use types::delegated_identity::DelegatedIdentityWire;

#[derive(Serialize, Deserialize)]
pub struct ClaimTokensRequest {
    pub identity: DelegatedIdentityWire,
    pub user_canister: Principal,
    pub token_root: Principal,
}
