use super::*;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct LeaderboardEntry {
    pub name: String,
    pub score: u32,
    pub crawl_id: u32,
}
