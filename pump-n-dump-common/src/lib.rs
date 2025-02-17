pub mod rest;
pub mod ws;

use candid::Nat;
use canisters_client::individual_user_template::GameDirection as CanisterGameDirection;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameDirection {
    Pump,
    Dump,
}

impl From<GameDirection> for CanisterGameDirection {
    fn from(value: GameDirection) -> Self {
        match value {
            GameDirection::Pump => CanisterGameDirection::Pump,
            GameDirection::Dump => CanisterGameDirection::Dump,
        }
    }
}

impl From<CanisterGameDirection> for GameDirection {
    fn from(value: CanisterGameDirection) -> Self {
        match value {
            CanisterGameDirection::Pump => GameDirection::Pump,
            CanisterGameDirection::Dump => GameDirection::Dump,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum WithdrawalState {
    Value(Nat),
    NeedMoreEarnings(Nat),
}
