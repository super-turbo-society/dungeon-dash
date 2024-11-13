use super::*;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Obstacle {
    pub x: i32,
    pub y: i32,
    pub kind: ObstacleKind,
}
