use super::*;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Obstacle {
    pub x: i32,
    pub y: i32,
    pub kind: ObstacleKind,
}
