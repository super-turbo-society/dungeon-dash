use super::*;

pub mod ui;

pub mod commands {
    use super::*;
    pub mod ascend_multiplayer_dungeon;
    pub mod create_multiplayer_dungeon_lobby;
    pub mod create_new_dungeon;
    pub mod delete_dungeon;
    pub mod delete_multiplayer_dungeon;
    pub mod delete_multiplayer_dungeon_lobby;
    pub mod join_multiplayer_dungeon_lobby;
    pub mod leave_multiplayer_dungeon_lobby;
    pub mod move_multiplayer_dungeon_player;
    pub mod move_player;
    pub mod reset_multiplayer_dungeon;
    pub mod start_new_multiplayer_dungeon;
}

pub mod queries {
    use super::*;
    pub mod current_multiplayer_dungeon_crawl_id;
    pub mod global_leaderboard;
    pub mod multiplayer_dungeon;
    pub mod multiplayer_dungeon_list;
    pub mod player_achievements;
    pub mod player_dungeon;
    pub mod player_dungeon_stats;
}
