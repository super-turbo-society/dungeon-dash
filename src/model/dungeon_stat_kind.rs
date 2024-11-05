use super::*;

#[derive(
    BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub enum DungeonStatKind {
    CrawlsCompleted,
    FloorsCleared,
    HealthRecovered,
    GoldCollected,
    DamageDealt,
    StepsMoved,
    DamageTaken,
    Defeated(MonsterKind),
    DefeatedBy(MonsterKind),
}
impl DungeonStatKind {
    pub const ALL: &'static [Self] = &[
        Self::CrawlsCompleted,
        Self::FloorsCleared,
        Self::HealthRecovered,
        Self::GoldCollected,
        Self::DamageDealt,
        Self::DamageTaken,
        Self::StepsMoved,
        Self::Defeated(MonsterKind::BlueBlob),
        Self::Defeated(MonsterKind::Ghost),
        Self::Defeated(MonsterKind::GreenGoblin),
        Self::Defeated(MonsterKind::OrangeGoblin),
        Self::Defeated(MonsterKind::RedBlob),
        Self::Defeated(MonsterKind::Shade),
        Self::Defeated(MonsterKind::SpectralGhost),
        Self::Defeated(MonsterKind::Spider),
        Self::Defeated(MonsterKind::YellowBlob),
        Self::Defeated(MonsterKind::Zombie),
        Self::DefeatedBy(MonsterKind::BlueBlob),
        Self::DefeatedBy(MonsterKind::Ghost),
        Self::DefeatedBy(MonsterKind::GreenGoblin),
        Self::DefeatedBy(MonsterKind::OrangeGoblin),
        Self::DefeatedBy(MonsterKind::RedBlob),
        Self::DefeatedBy(MonsterKind::Shade),
        Self::DefeatedBy(MonsterKind::SpectralGhost),
        Self::DefeatedBy(MonsterKind::Spider),
        Self::DefeatedBy(MonsterKind::YellowBlob),
        Self::DefeatedBy(MonsterKind::Zombie),
    ];
    pub const DEFEATED: &'static [Self] = &[
        Self::Defeated(MonsterKind::BlueBlob),
        Self::Defeated(MonsterKind::Ghost),
        Self::Defeated(MonsterKind::GreenGoblin),
        Self::Defeated(MonsterKind::OrangeGoblin),
        Self::Defeated(MonsterKind::RedBlob),
        Self::Defeated(MonsterKind::Shade),
        Self::Defeated(MonsterKind::SpectralGhost),
        Self::Defeated(MonsterKind::Spider),
        Self::Defeated(MonsterKind::YellowBlob),
        Self::Defeated(MonsterKind::Zombie),
    ];
}
