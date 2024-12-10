use super::*;

use serde::{Deserialize, Serialize};

#[derive(
    Serialize,
    Deserialize,
    BorshSerialize,
    BorshDeserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
pub enum MonsterKind {
    GreenGoblin,
    OrangeGoblin,
    YellowBlob,
    BlueBlob,
    RedBlob,
    Shade,
    Spider,
    Ghost,
    SpectralGhost,
    Zombie,
    EvilTurbi,
    IceYeti,
    Snowman,
}
impl MonsterKind {
    pub const ALL: &'static [Self] = &[
        Self::GreenGoblin,
        Self::OrangeGoblin,
        Self::YellowBlob,
        Self::BlueBlob,
        Self::RedBlob,
        Self::Shade,
        Self::Spider,
        Self::Ghost,
        Self::SpectralGhost,
        Self::Zombie,
        Self::EvilTurbi,
        Self::IceYeti,
        Self::Snowman,
    ];
    pub fn by_index(n: usize) -> Self {
        match n % 10 {
            0 => Self::GreenGoblin,
            1 => Self::OrangeGoblin,
            2 => Self::YellowBlob,
            3 => Self::BlueBlob,
            4 => Self::RedBlob,
            5 => Self::Shade,
            6 => Self::Spider,
            7 => Self::Ghost,
            8 => Self::SpectralGhost,
            9 => Self::Zombie,
            10 => Self::EvilTurbi,
            11 => Self::IceYeti,
            12 => Self::Snowman,
            _ => unreachable!(),
        }
    }
    pub fn abbrev<'a>(&self) -> &'a str {
        match self {
            Self::BlueBlob => "B. Blob",
            Self::RedBlob => "R. Blob",
            Self::YellowBlob => "Y. Blob",
            Self::GreenGoblin => "G. Goblin",
            Self::OrangeGoblin => "O. Goblin",
            Self::Shade => "Shade",
            Self::Spider => "Spider",
            Self::Ghost => "Ghost",
            Self::SpectralGhost => "S. Ghost",
            Self::Zombie => "Zombie",
            Self::EvilTurbi => "E. Turbi",
            Self::IceYeti => "Ice Yeti",
            Self::Snowman => "Snowman",
        }
    }
    pub fn stats(&self) -> (u32, u32) {
        // (hp, strength)
        match self {
            Self::OrangeGoblin => (5, 1),
            Self::GreenGoblin => (2, 1),
            Self::RedBlob => (3, 2),
            Self::YellowBlob => (2, 1),
            Self::Shade => (3, 2),
            Self::Spider => (4, 2),
            Self::Ghost => (2, 2),
            Self::Zombie => (3, 3),
            Self::EvilTurbi => (3, 3),
            Self::IceYeti => (6, 2),
            Self::Snowman => (3, 3),
            _ => (1, 1),
        }
    }
}
