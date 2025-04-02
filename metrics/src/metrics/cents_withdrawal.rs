use candid::{Nat, Principal};
use serde::{Deserialize, Serialize};

use super::sealed_metric::SealedMetric;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CentsWithdrawal {
    pub user_canister: Principal,
    pub amount: Nat,
}

impl SealedMetric for CentsWithdrawal {
    fn tag(&self) -> String {
        "cents_withdrawal".into()
    }

    fn user_id(&self) -> Option<String> {
        Some(self.user_canister.to_text())
    }

    fn user_canister(&self) -> Option<Principal> {
        Some(self.user_canister)
    }
}
