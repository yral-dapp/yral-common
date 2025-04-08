use candid::Principal;
use canisters_client::individual_user_template::{
    BetDirection, BetOutcomeForBetMaker, BettingStatus, PlaceBetArg, PlacedBetDetail, Result3,
};
use serde::{Deserialize, Serialize};
use web_time::Duration;

use crate::{Canisters, Error, Result};

use super::time::current_epoch;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum VoteOutcome {
    Won(u64),
    Draw(u64),
    Lost,
    AwaitingResult,
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum VoteKind {
    Hot,
    Not,
}

impl From<VoteKind> for BetDirection {
    fn from(kind: VoteKind) -> Self {
        match kind {
            VoteKind::Hot => BetDirection::Hot,
            VoteKind::Not => BetDirection::Not,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VoteDetails {
    pub outcome: VoteOutcome,
    pub post_id: u64,
    pub canister_id: Principal,
    pub vote_kind: VoteKind,
    pub vote_amount: u64,
    placed_at: Duration,
    slot_id: u8,
}

impl VoteDetails {
    pub fn reward(&self) -> Option<u64> {
        match self.outcome {
            VoteOutcome::Won(w) => Some(w),
            VoteOutcome::Draw(w) => Some(w),
            VoteOutcome::Lost => None,
            VoteOutcome::AwaitingResult => None,
        }
    }

    pub fn vote_duration(&self) -> Duration {
        // Vote duration + 5 minute overhead
        Duration::from_secs(((self.slot_id as u64) * 60 * 60) + 5 * 60)
    }

    pub fn end_time(&self, post_creation_time: Duration) -> Duration {
        post_creation_time + self.vote_duration()
    }

    pub fn time_remaining(&self, post_creation_time: Duration) -> Duration {
        let end_time = self.end_time(post_creation_time);
        end_time.saturating_sub(current_epoch())
    }
}

impl From<PlacedBetDetail> for VoteDetails {
    fn from(bet: PlacedBetDetail) -> Self {
        let outcome = match bet.outcome_received {
            BetOutcomeForBetMaker::Lost => VoteOutcome::Lost,
            BetOutcomeForBetMaker::Draw(w) => VoteOutcome::Draw(w),
            BetOutcomeForBetMaker::Won(w) => VoteOutcome::Won(w),
            BetOutcomeForBetMaker::AwaitingResult => VoteOutcome::AwaitingResult,
        };
        let vote_kind = match bet.bet_direction {
            BetDirection::Hot => VoteKind::Hot,
            BetDirection::Not => VoteKind::Not,
        };
        Self {
            outcome,
            post_id: bet.post_id,
            canister_id: bet.canister_id,
            vote_kind,
            vote_amount: bet.amount_bet,
            placed_at: Duration::new(
                bet.bet_placed_at.secs_since_epoch,
                bet.bet_placed_at.nanos_since_epoch,
            ),
            slot_id: bet.slot_id,
        }
    }
}

impl Canisters<true> {
    pub async fn vote_on_post(
        &self,
        vote_amount: u64,
        vote_direction: VoteKind,
        post_id: u64,
        post_canister_id: Principal,
    ) -> Result<BettingStatus> {
        let user = self.authenticated_user().await;

        let place_bet_arg = PlaceBetArg {
            bet_amount: vote_amount,
            post_id,
            bet_direction: vote_direction.into(),
            post_canister_id,
        };

        let res = user.bet_on_currently_viewing_post(place_bet_arg).await?;

        let betting_status = match res {
            Result3::Ok(p) => p,
            Result3::Err(e) => {
                // todo send event that betting failed
                return Err(Error::YralCanister(format!(
                    "bet_on_currently_viewing_post error {e:?}"
                )));
            }
        };

        Ok(betting_status)
    }
}
