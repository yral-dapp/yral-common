mod error;

pub use error::*;

use candid::{CandidType, Principal};
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use yral_identity::Signature;

pub const WORKER_URL: &str = "http://localhost:8080";
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
    pub signature: Signature,
}

pub fn hon_game_vote_msg(request: VoteRequest) -> yral_identity::msg_builder::Message {
    yral_identity::msg_builder::Message::default()
        .method_name("hon_worker_game_vote".into())
        .args((request,))
        .expect("Vote request should serialize")
}

impl HoNGameVoteReq {
    #[cfg(feature = "client")]
    pub fn new(
        sender: &impl ic_agent::Identity,
        request: VoteRequest,
    ) -> yral_identity::Result<Self> {
        use yral_identity::ic_agent::sign_message;
        let msg = hon_game_vote_msg(request.clone());
        let signature = sign_message(sender, msg)?;

        Ok(Self { request, signature })
    }
}
