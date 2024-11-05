use super::*;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct DungeonStats {
    pub entries: BTreeMap<String, u32>,
}
impl DungeonStats {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }
    pub fn get(&self, kind: DungeonStatKind) -> u32 {
        let key = format!("{kind:?}");
        *self.entries.get(&key).unwrap_or(&0)
    }
    pub fn total_monsters_defeated(&self) -> u32 {
        let mut total = 0;
        for kind in DungeonStatKind::DEFEATED {
            total += self.get(*kind);
        }
        total
    }
    pub fn monster_kills(&self, monster_kind: MonsterKind) -> u32 {
        self.get(DungeonStatKind::Defeated(monster_kind))
    }
    pub fn deaths_by_monster(&self, monster_kind: MonsterKind) -> u32 {
        self.get(DungeonStatKind::DefeatedBy(monster_kind))
    }
    pub fn increment(&mut self, kind: DungeonStatKind, amount: u32) {
        let key = format!("{kind:?}");
        self.entries
            .entry(key)
            .and_modify(|n| *n = *n + amount)
            .or_insert(amount);
    }
}
