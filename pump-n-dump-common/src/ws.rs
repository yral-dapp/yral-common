use candid::{Nat, Principal};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use yral_identity::msg_builder::Message;

use crate::GameDirection;

/// Types of request that worker can handle
#[derive(Serialize, Deserialize)]
pub enum WsMessage {
    Bet(GameDirection),
}

/// A complete request that the worker expects
/// for a given request_id, a corresponding [`WsResponse`]
/// is expected to be produced with the same request_id
#[derive(Serialize, Deserialize)]
pub struct WsRequest {
    pub request_id: Uuid,
    pub msg: WsMessage,
}

/// Result for a game
#[derive(Serialize, Deserialize, Clone)]
pub struct GameResult {
    pub direction: GameDirection,
    pub reward_pool: Nat,
}

/// Types of responses from the worker
#[derive(Serialize, Deserialize, Clone)]
pub enum WsResp {
    Ok,
    Error(String),
    GameResultEvent(GameResult),
}

/// A complete response from the worker
/// this can be an event or a response to some previous request
/// from the client
#[derive(Serialize, Deserialize)]
pub struct WsResponse {
    pub request_id: Uuid,
    pub response: WsResp,
}

impl WsResponse {
    pub fn is_event(&self) -> bool {
        self.request_id == Uuid::max()
    }
}

pub fn identify_message(game_canister: Principal, token_root: Principal) -> Message {
    Message::default()
        .method_name("pump_or_dump_worker".into())
        .args((game_canister, token_root))
        .expect("identify request should serialize")
}

/// Create a connection url for a new game
/// caller must ensure `base_url` ends with a slash (`/`)
/// e.g "https://example.com/"
#[cfg(feature = "client")]
pub fn websocket_connection_url(
    base_url: url::Url,
    sender: &impl ic_agent::Identity,
    token_creator_canister: candid::Principal,
    token_root: candid::Principal,
) -> Result<url::Url, String> {
    use yral_identity::ic_agent::sign_message;

    let path = format!("ws/{token_creator_canister}/{token_root}");
    let mut new_url = base_url.join(&path).map_err(|e| e.to_string())?;

    let msg = identify_message(token_creator_canister, token_root);
    let sig = sign_message(sender, msg).map_err(|e| e.to_string())?;
    let sig_json = serde_json::to_string(&sig).expect("signature should serialize");

    let sender_id = sender.sender().expect("sender should have principal");

    new_url
        .query_pairs_mut()
        .append_pair("sender", &sender_id.to_text())
        .append_pair("signature", &sig_json);

    Ok(new_url)
}
