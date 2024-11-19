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

pub mod channels {
    use super::*;
    use os::server::{ChannelError, ChannelMessage};

    #[export_name = "channel/multiplayer_dungeon"]
    unsafe extern "C" fn multiplayer_dungeon_channel() {
        loop {
            match os::server::channel_recv() {
                Ok(ChannelMessage::Data(user_id, data)) => {
                    let emote = Emote::try_from_slice(&data).unwrap();
                    let payload = (user_id, emote);
                    os::server::channel_broadcast(&payload.try_to_vec().unwrap());
                }
                Err(_err) => return,
                _ => {}
            }
        }
    }

    #[export_name = "channel/online_now"]
    unsafe extern "C" fn online_now_channel() {
        os::server::log!("CHANNEL OPENED");
        // 1. Handle channel creation parameters
        // let cmd = os::server::command!(...);

        // Process messages
        // 1. Process incoming subscriber messages
        // 2. Send outgoing subscriber messages
        // 3. Interact with files as-needed
        let mut connected = BTreeSet::new();
        let mut num_messages = 0;
        loop {
            match os::server::channel_recv() {
                // Handle a channel connection
                Ok(ChannelMessage::Connect(user_id, _data)) => {
                    connected.insert(user_id.clone());
                    os::server::log!("{user_id} CONNECTED");
                    let n = connected.len();
                    os::server::channel_broadcast(
                        format!("{user_id:.8} joined!\n{n} connected\n{num_messages} messages")
                            .as_bytes(),
                    );
                }
                // Handle a channel disconnection
                Ok(ChannelMessage::Disconnect(user_id, _data)) => {
                    connected.remove(&user_id);
                    os::server::log!("{user_id} DISCONNECTED");
                    let n = connected.len();
                    os::server::channel_broadcast(
                        format!(
                            "{user_id:.8} disconnected\n{n} connected\n{num_messages} messages"
                        )
                        .as_bytes(),
                    );
                }
                // Handle custom message data sent to
                Ok(ChannelMessage::Data(user_id, data)) => {
                    num_messages += 1;
                    os::server::log!("Got message: {user_id}");
                    if let Ok(data) = String::from_utf8(data) {
                        os::server::log!("Got message from {user_id}: {data}");
                        let n = connected.len();
                        os::server::channel_broadcast(
                            format!("{user_id:.8} says:\n'{data}'\n{n} connected\n{num_messages} messages").as_bytes(),
                        );
                    } else {
                        os::server::log!("Got message from {user_id}");
                        os::server::channel_send(&user_id, b"Got non-utf8 message");
                    }
                    // handle game-specific channel messages
                }
                // Handle a timeout error
                Err(ChannelError::Timeout) => {
                    continue;
                }
                // Handle a channel closure
                Err(err) => {
                    os::server::log!("ERROR: {err:?}");
                    return;
                }
            }
        }
    }
}
