use candid::{Nat, Principal};
use serde::{Deserialize, Serialize};
use yral_identity::{msg_builder::Message, Signature};

/// Request for converting GDOLLR to DOLLR
#[derive(Serialize, Deserialize, Clone)]
pub struct ClaimReq {
    // user to send DOLLR to
    pub sender: Principal,
    // amount of DOLLR
    pub amount: Nat,
    // signature asserting the user's consent
    pub signature: Signature,
}

pub fn claim_msg(amount: Nat) -> Message {
    Message::default()
        .method_name("pump_or_dump_worker_claim".into())
        .args((amount,))
        .expect("Claim request should serialize")
}

impl ClaimReq {
    #[cfg(feature = "client")]
    pub fn new(sender: &impl ic_agent::Identity, amount: Nat) -> yral_identity::Result<Self> {
        use yral_identity::ic_agent::sign_message;
        let msg = claim_msg(amount.clone());
        let signature = sign_message(sender, msg)?;

        Ok(Self {
            sender: sender.sender().expect("signing was succesful"),
            amount,
            signature,
        })
    }
}

/// Response for user bets for a given game
#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct UserBetsResponse {
    pub pumps: u64,
    pub dumps: u64,
}
