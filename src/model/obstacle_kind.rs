use super::*;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Debug, Clone, Copy)]
pub enum ObstacleKind {
    WallA,
    WallB,
}
