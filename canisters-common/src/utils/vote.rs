use candid::{CandidType, Principal};
use canisters_client::individual_user_template::{
    BetDirection, BetOutcomeForBetMaker, BettingStatus, PlaceBetArg, PlacedBetDetail, Result3,
};
use serde::{Deserialize, Serialize};
use web_time::Duration;
use yral_identity::{ic_agent::sign_message, msg_builder::Message, Signature};

use crate::{consts::CENTS_IN_E6S, Canisters, Error, Result};

use super::time::current_epoch;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum VoteOutcome {
    Won(u64),
    Draw(u64),
    Lost,
    AwaitingResult,
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, CandidType)]
pub enum VoteKind {
    Hot,
    Not,
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, CandidType)]
pub struct HonBetArg {
    pub bet_amount: u64,
    pub post_id: u64,
    pub bet_direction: VoteKind,
    pub post_canister_id: Principal,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VerifiableHonBetReq {
    pub sender: Principal,
    pub signature: Signature,
    pub args: HonBetArg,
}

pub fn verifiable_hon_bet_message(args: HonBetArg) -> Message {
    Message::default()
        .method_name("place_hon_bet_worker_req".into())
        .args((args,))
        .expect("Place bet request should serialize")
}

impl VerifiableHonBetReq {
    pub fn new(sender: &impl ic_agent::Identity, args: HonBetArg) -> yral_identity::Result<Self> {
        let msg = verifiable_hon_bet_message(args);
        let signature = sign_message(sender, msg)?;

        Ok(Self {
            sender: sender.sender().expect("signing was succesful"),
            args,
            signature,
        })
    }
}

impl From<HonBetArg> for PlaceBetArg {
    fn from(
        HonBetArg {
            bet_amount,
            post_id,
            bet_direction,
            post_canister_id,
        }: HonBetArg,
    ) -> Self {
        Self {
            bet_direction: bet_direction.into(),
            bet_amount,
            post_id,
            post_canister_id,
        }
    }
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

    pub async fn vote_with_cents_on_post(
        &self,
        vote_amount: u64,
        vote_direction: VoteKind,
        post_id: u64,
        post_canister_id: Principal,
    ) -> Result<BettingStatus> {
        let user = self.authenticated_user().await;

        let place_bet_arg = PlaceBetArg {
            bet_amount: vote_amount * CENTS_IN_E6S,
            post_id,
            bet_direction: vote_direction.into(),
            post_canister_id,
        };

        let res = user
            .bet_on_currently_viewing_post_v_1(place_bet_arg)
            .await?;

        let betting_status = match res {
            Result3::Ok(p) => p,
            Result3::Err(e) => {
                // todo send event that betting failed
                return Err(Error::YralCanister(format!(
                    "bet_on_currently_viewing_post_v_1 error {e:?}"
                )));
            }
        };

        Ok(betting_status)
    }

    /// Places a vote on a post via cloudflare. The vote amount must be in cents e0s
    pub async fn vote_with_cents_on_post_via_cloudflare(
        &self,
        cloudflare_url: reqwest::Url,
        vote_amount: u64,
        bet_direction: VoteKind,
        post_id: u64,
        post_canister_id: Principal,
    ) -> Result<BettingStatus> {
        let req = VerifiableHonBetReq::new(
            self.identity(),
            HonBetArg {
                bet_amount: vote_amount * CENTS_IN_E6S,
                post_id,
                bet_direction,
                post_canister_id,
            },
        )?;

        let url = cloudflare_url.join("/place_hot_or_not_bet")?;

        let client = reqwest::Client::new();
        let betting_status: BettingStatus =
            client.post(url).json(&req).send().await?.json().await?;

        Ok(betting_status)
    }
}
