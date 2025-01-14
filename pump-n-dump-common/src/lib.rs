pub mod rest;
pub mod ws;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum GameDirection {
    Pump,
    Dump,
}
