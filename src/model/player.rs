use super::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Player {
    pub x: i32,
    pub y: i32,
    pub health: u32,
    pub max_health: u32,
    pub strength: u32,
    pub gold: u32,
    pub direction: Direction,
}
