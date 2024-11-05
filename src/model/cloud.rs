use super::*;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct Cloud {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub speed: f32,
    pub color: u32,
}
