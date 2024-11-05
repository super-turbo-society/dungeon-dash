use super::*;

pub const PROGRAM_ID: &'static str = "dungeon_dash";
pub const PROGRAM_VERSION: usize = 1;

pub mod paths {
    use super::*;
    pub fn player_dungeon(user_id: &str) -> String {
        format!("/users/{}/v{}/dungeon", user_id, PROGRAM_VERSION)
    }
    pub fn player_dungeon_stats(user_id: &str) -> String {
        format!("/users/{}/v{}/stats", user_id, PROGRAM_VERSION)
    }
    pub fn player_achievements(user_id: &str) -> String {
        format!("/users/{}/v{}/achievements", user_id, PROGRAM_VERSION)
    }
    pub fn player_leaderboard(user_id: &str) -> String {
        format!("/users/{}/v{}/leaderboard", user_id, PROGRAM_VERSION)
    }
    pub fn global_leaderboard() -> String {
        "leaderboard".to_string()
    }
}

pub mod commands {
    use super::*;
    pub mod create_new_dungeon;
    pub mod move_player;
}
