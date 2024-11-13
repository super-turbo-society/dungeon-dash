use super::*;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct PlayerAchievements {
    completed: BTreeSet<u32>, // stores index of completed achievements
}
impl PlayerAchievements {
    pub fn empty() -> Self {
        Self {
            completed: BTreeSet::new(),
        }
    }
    pub fn from_dungeon_stats(
        stats: &DungeonStats,
        total_stats: &DungeonStats,
        did_crawl_end: bool,
    ) -> Self {
        let mut completed = BTreeSet::new();
        for (kind, _, _) in AchievementKind::INFO {
            if kind.test(stats, total_stats, did_crawl_end) {
                completed.insert(*kind as u32);
            }
        }
        Self { completed }
    }
    pub fn from_achievement_kinds(kinds: &[AchievementKind]) -> Self {
        let mut completed = BTreeSet::new();
        for kind in kinds {
            completed.insert(*kind as u32);
        }
        Self { completed }
    }
    pub fn achievement_kinds(&self) -> Vec<AchievementKind> {
        self.completed
            .iter()
            .cloned()
            .map(|id| id.try_into().unwrap())
            .collect()
    }
    pub fn is_empty(&self) -> bool {
        self.completed.is_empty()
    }
    pub fn len(&self) -> usize {
        self.completed.len()
    }
    pub fn apply_dungeon_stats(
        &self,
        stats: &DungeonStats,
        total_stats: &DungeonStats,
        did_crawl_end: bool,
    ) -> Self {
        let mut completed = BTreeSet::new();
        for (kind, _, _) in AchievementKind::INFO {
            if kind.test(stats, total_stats, did_crawl_end) {
                completed.insert(*kind as u32);
            }
        }
        let next = Self { completed };
        self.union(&next)
    }
    pub fn difference(&self, other: &Self) -> Self {
        let completed = self
            .completed
            .difference(&other.completed)
            .cloned()
            .collect::<BTreeSet<_>>();
        Self { completed }
    }
    pub fn union(&self, other: &Self) -> Self {
        let completed = self
            .completed
            .union(&other.completed)
            .cloned()
            .collect::<BTreeSet<_>>();
        Self { completed }
    }
}
