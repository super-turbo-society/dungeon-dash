use super::*;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Leaderboard {
    entries: BTreeMap<String, Vec<LeaderboardEntry>>,
}
impl Leaderboard {
    const LEADERBOARD_SIZE: usize = 10;
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }
    pub fn find<F: Fn(&LeaderboardEntry) -> bool>(
        &self,
        kind: LeaderboardKind,
        callback: F,
    ) -> Option<LeaderboardEntry> {
        let key = format!("{kind:?}");
        self.entries
            .get(&key)
            .and_then(|leaderboard| leaderboard.iter().find(|&e| callback(e)).cloned())
    }
    pub fn update(
        &mut self,
        crawl_id: u32,
        kind: LeaderboardKind,
        name: &str,
        score: u32,
    ) -> Option<LeaderboardEntry> {
        let key = format!("{kind:?}");
        let entry = LeaderboardEntry {
            name: name.to_string(),
            score: score,
            crawl_id,
        };
        self.entries
            .entry(key)
            .and_modify(|leaderboard| {
                leaderboard.push(entry.clone());
                leaderboard.sort_by(|a, b| {
                    if kind.is_most() {
                        b.score.cmp(&a.score)
                    } else {
                        a.score.cmp(&b.score)
                    }
                });
                if leaderboard.len() > Self::LEADERBOARD_SIZE {
                    leaderboard.truncate(Self::LEADERBOARD_SIZE);
                }
            })
            .or_insert(vec![entry.clone()]);
        self.find(kind, |entry| entry.crawl_id == crawl_id)
    }
    pub fn render_entries(
        &self,
        crawl_id: u32,
        mut i: i32,
        kind: LeaderboardKind,
        name: &str,
        x: i32,
        y: i32, // 9
    ) {
        let key = format!("{kind:?}");
        let mut rank = 0;
        let mut prev_value = if kind.is_most() { u32::MAX } else { 0 };
        for entry in self.entries.get(&key).unwrap_or(&vec![]) {
            // Only increment rank if the value changes
            if kind.is_most() && entry.score < prev_value {
                rank += 1;
                prev_value = entry.score;
            } else if !kind.is_most() && entry.score > prev_value {
                rank += 1;
                prev_value = entry.score;
            } else if rank == 0 {
                rank += 1;
                prev_value = entry.score;
            }
            let color: u32 = if crawl_id == entry.crawl_id && tick() % 16 < 8 {
                0x1e6f50ff
            } else if entry.name == name {
                0x6ecb62ff
            } else {
                0xacaabdff
            };
            text!(
                "{} {}{:.8} {:>11} ",
                rank,
                if rank > 9 { "" } else { " " },
                entry.name,
                &format!("{}", entry.score);
                absolute = true,
                x = x + 8,
                y = y + i * 10,
                color = color
            );

            i += 1;
            if i - 3 > 10 {
                break;
            }
        }
    }
}
