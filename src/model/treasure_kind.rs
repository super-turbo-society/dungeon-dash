use super::*;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum TreasureKind {
    Gold,
    Heal,
    HealthUp,
}
