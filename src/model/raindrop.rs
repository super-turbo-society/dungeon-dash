use super::*;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct Raindrop {
    pub x: f32,
    pub y: f32,
    pub vel: f32,
    pub length: f32,
}