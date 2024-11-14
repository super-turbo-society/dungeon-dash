use super::*;

pub const PROGRAM_ID: &'static str = "dungeon_dash";
pub const PROGRAM_VERSION: usize = 1;

pub mod paths {
    use super::*;
    pub fn global_leaderboard() -> String {
        "leaderboard".to_string()
    }
    pub fn multiplayer_dungeon_list() -> String {
        "multiplayer_dungeon_list".to_string()
    }
    pub fn multiplayer_dungeon(crawl_id: u32) -> String {
        format!("/multiplayer_dungeons/v{}/{}", PROGRAM_VERSION, crawl_id)
    }
    pub fn player_multiplayer_dungeon_manifest(user_id: &str) -> String {
        format!(
            "/users/{}/v{}/multiplayer_dungeon_manifest",
            user_id, PROGRAM_VERSION
        )
    }
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
}

pub mod commands {
    use super::*;
    pub mod create_multiplayer_dungeon_lobby;
    pub mod create_new_dungeon;
    pub mod create_new_multiplayer_dungeon;
    pub mod delete_dungeon;
    pub mod delete_multiplayer_dungeon;
    pub mod delete_multiplayer_dungeon_lobby;
    pub mod join_multiplayer_dungeon_lobby;
    pub mod leave_multiplayer_dungeon_lobby;
    pub mod move_monsters;
    pub mod move_player;
    pub mod move_player_multiplayer_dungeon;
}

pub mod deserializers {
    use super::*;
    use serde_json::json;
    use turbo::os;

    #[export_name = "deserializers/dungeon"]
    unsafe extern "C" fn deserialize_dungeon() {
        let bytes = os::server::get_command_data();
        if bytes.is_empty() {
            return os::server::log!("File is empty");
        }
        match Dungeon::try_from_slice(&bytes) {
            Ok(dungeon) => os::server::log!("{:#?}", dungeon),
            Err(err) => os::server::log!("{:#?}", err),
        };
    }

    #[export_name = "deserializers/dungeon_json"]
    unsafe extern "C" fn deserialize_dungeon_json() {
        let bytes = os::server::get_command_data();
        if bytes.is_empty() {
            return os::server::log!("{{}}");
        }
        let dungeon = match Dungeon::try_from_slice(&bytes) {
            Ok(dungeon) => dungeon,
            Err(err) => return os::server::log!("{:#?}", err),
        };
        let json = json!(dungeon);
        os::server::log!("{}", json)
    }
}
