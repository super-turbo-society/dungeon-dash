use super::*;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Monster {
    pub x: i32,
    pub y: i32,
    pub health: u32,
    pub max_health: u32,
    pub strength: u32,
    pub direction: Direction,
    pub kind: MonsterKind,
    pub stun_dur: u32,
}
