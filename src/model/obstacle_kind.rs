use super::*;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy)]
pub enum ObstacleKind {
    WallA,
    WallB,
}