mod error;

pub use error::*;

use candid::{CandidType, Principal};
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use yral_identity::{Signature, msg_builder::Message};

// TODO switch back to prod before merge
pub const WORKER_URL: &str = "https://yral-hot-or-not-preview.go-bazzinga.workers.dev/";
// pub const WORKER_URL: &str = "https://yral-hot-or-not.go-bazzinga.workers.dev/";
pub type WorkerResponse<T> = Result<T, WorkerError>;

#[derive(Serialize, Deserialize, Clone, Debug, CandidType)]
pub struct ClaimRequest {
    /// User's principal id
    pub user_principal: Principal,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VerifiableClaimRequest {
    pub sender: Principal,
    pub request: ClaimRequest,
    /// The amount of airdrop to be claimed in sats (e0s)
    pub amount: u64,
    pub signature: Signature,
}

pub fn verifiable_claim_request_message(args: ClaimRequest) -> Message {
    Message::default()
        .method_name("claim_sats_airdrop_request".into())
        .args((args,))
        .expect("Request must serialize")
}

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

#[cfg(feature = "client")]
pub fn sign_claim_request(
    sender: &impl ic_agent::Identity,
    request: ClaimRequest,
) -> yral_identity::Result<Signature> {
    use yral_identity::ic_agent::sign_message;
    let msg = verifiable_claim_request_message(request.clone());
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
