use super::*;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Treasure {
    pub x: i32,
    pub y: i32,
    pub value: u32,
    pub kind: TreasureKind,
}
