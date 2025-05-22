mod error;

pub use error::*;

use candid::{CandidType, Principal};
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use yral_identity::Signature;

pub const WORKER_URL: &str = "https://yral-hot-or-not.go-bazzinga.workers.dev/";
pub type WorkerResponse<T> = Result<T, WorkerError>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SatsBalanceInfo {
    pub balance: BigUint,
    pub airdropped: BigUint,
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Debug, CandidType)]
pub enum HotOrNot {
    Hot,
    Not,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum GameResult {
    Win { win_amt: BigUint },
    Loss { lose_amt: BigUint },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum GameInfo {
    CreatorReward(BigUint),
    Vote {
        vote_amount: BigUint,
        game_result: GameResult,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VoteRes {
    pub game_result: GameResult,
}

#[derive(Serialize, Deserialize, Clone, Debug, CandidType)]
pub struct VoteRequest {
    pub post_canister: Principal,
    pub post_id: u64,
    pub vote_amount: u128,
    pub direction: HotOrNot,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameRes {
    pub post_canister: Principal,
    pub post_id: u64,
    pub game_info: GameInfo,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PaginatedGamesReq {
    pub page_size: usize,
    pub cursor: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PaginatedGamesRes {
    pub games: Vec<GameRes>,
    pub next: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct GameInfoReq {
    pub post_canister: Principal,
    pub post_id: u64,
}

impl From<(Principal, u64)> for GameInfoReq {
    fn from((post_canister, post_id): (Principal, u64)) -> Self {
        Self {
            post_canister,
            post_id,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HoNGameVoteReq {
    pub request: VoteRequest,
    /// Sentiment from alloydb
    pub fetched_sentiment: HotOrNot,
    pub post_creator: Option<Principal>,
    pub signature: Signature,
}

pub fn hon_game_vote_msg(request: VoteRequest) -> yral_identity::msg_builder::Message {
    yral_identity::msg_builder::Message::default()
        .method_name("hon_worker_game_vote".into())
        .args((request,))
        .expect("Vote request should serialize")
}

#[cfg(feature = "client")]
pub fn sign_vote_request(
    sender: &impl ic_agent::Identity,
    request: VoteRequest,
) -> yral_identity::Result<Signature> {
    use yral_identity::ic_agent::sign_message;
    let msg = hon_game_vote_msg(request.clone());
    sign_message(sender, msg)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WithdrawRequest {
    pub receiver: Principal,
    pub amount: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HoNGameWithdrawReq {
    pub request: WithdrawRequest,
    pub signature: Signature,
}

pub fn hon_game_withdraw_msg(request: &WithdrawRequest) -> yral_identity::msg_builder::Message {
    yral_identity::msg_builder::Message::default()
        .method_name("hon_worker_game_withdraw".into())
        .args((request.amount,))
        .expect("Withdraw request should serialize")
}

#[cfg(feature = "client")]
pub fn sign_withdraw_request(
    sender: &impl ic_agent::Identity,
    request: WithdrawRequest,
) -> yral_identity::Result<Signature> {
    use yral_identity::ic_agent::sign_message;
    let msg = hon_game_withdraw_msg(&request);
    sign_message(sender, msg)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReferralItem {
    pub referrer: Principal,
    pub referee: Principal,
    pub amount: u64,
    pub created_at: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReferralReqWithSignature {
    pub request: ReferralReq,
    pub signature: Signature,
}

#[derive(Serialize, Deserialize, Clone, Debug, CandidType)]
pub struct ReferralReq {
    pub referrer: Principal,
    pub referee: Principal,
}

pub fn hon_referral_msg(request: ReferralReq) -> yral_identity::msg_builder::Message {
    yral_identity::msg_builder::Message::default()
        .method_name("hon_worker_referral".into())
        .args((request,))
        .expect("Referral request should serialize")
}

#[cfg(feature = "client")]
pub fn sign_referral_request(
    sender: &impl ic_agent::Identity,
    request: ReferralReq,
) -> yral_identity::Result<Signature> {
    use yral_identity::ic_agent::sign_message;
    let msg = hon_referral_msg(request);
    sign_message(sender, msg)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PaginatedReferralsReq {
    pub cursor: Option<u64>,
    pub limit: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PaginatedReferralsRes {
    pub items: Vec<ReferralItem>,
    pub cursor: Option<u64>,
}
