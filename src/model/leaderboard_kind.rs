use super::*;

#[derive(
    BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub enum LeaderboardKind {
    HighestFloor,
    MostGold,
    MostKills,
    LeastSteps,
}
impl LeaderboardKind {
    pub const ALL: &'static [Self] = &[
        Self::HighestFloor,
        Self::MostGold,
        Self::MostKills,
        Self::LeastSteps,
    ];
    pub fn is_most(&self) -> bool {
        match self {
            Self::LeastSteps => false,
            _ => true,
        }
    }
    pub fn next(&self) -> Self {
        let i = Self::ALL.binary_search(&self).unwrap() + 1;
        let len = Self::ALL.len();
        Self::ALL[i % len]
    }
    pub fn prev(&self) -> Self {
        let i = Self::ALL.binary_search(&self).unwrap();
        let len = Self::ALL.len();
        Self::ALL[(i + len - 1) % len]
    }
}
