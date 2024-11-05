use super::*;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct Confetti {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub color: u32,
    pub vy: f32,
}