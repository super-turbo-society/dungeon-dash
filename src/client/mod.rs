use super::*;

pub mod ui;

pub mod commands {
    use super::*;
    pub mod create_new_dungeon;
    pub mod move_player;
}

pub mod queries {
    use super::*;
    pub mod global_leaderboard;
    pub mod player_achievements;
    pub mod player_dungeon;
    pub mod player_dungeon_stats;
}
