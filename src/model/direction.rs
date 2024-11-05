use super::*;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
