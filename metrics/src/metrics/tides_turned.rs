use candid::Principal;
use serde::Serialize;

use super::sealed_metric::SealedMetric;

#[derive(Serialize, Clone, Debug)]
pub struct TidesTurned {
    pub user_canister: Principal,
    pub staked_amount: u64,
    pub round_num: u64,
    pub user_pumps: u64,
    pub user_dumps: u64,
    pub round_pumps: u64,
    pub round_dumps: u64,
    pub cumulative_pumps: u64,
    pub cumulative_dumps: u64,
    pub token_root: Principal,
}

impl SealedMetric for TidesTurned {
    fn tag(&self) -> String {
        "tides_turned".into()
    }

    fn user_id(&self) -> Option<String> {
        Some(self.user_canister.to_text())
    }
}
