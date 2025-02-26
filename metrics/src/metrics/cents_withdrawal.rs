use candid::{Nat, Principal};
use serde::Serialize;

use super::sealed_metric::SealedMetric;

#[derive(Serialize, Clone, Debug)]
pub struct CentsWithdrawal {
    pub user_canister: Principal,
    pub amount: Nat,
}

impl SealedMetric for CentsWithdrawal {
    fn tag(&self) -> String {
        "cents_withdrawal".into()
    }
}
