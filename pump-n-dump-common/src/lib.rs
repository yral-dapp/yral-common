pub mod rest;
pub mod ws;

use candid::Nat;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum GameDirection {
    Pump,
    Dump,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum WithdrawalState {
    Value(Nat),
    NeedMoreEarnings(Nat),
}
