use super::*;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct Snowflake {
    pub x: f32,
    pub y: f32,
    pub vy: f32,
    pub size: f32,
    pub offset: f32,
}