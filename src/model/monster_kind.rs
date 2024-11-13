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
        }
    }
}
