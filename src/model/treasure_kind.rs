use super::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum TreasureKind {
    Gold,
    Heal,
    HealthUp,
}
