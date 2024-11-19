use super::*;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Command;
impl Command {
    pub const NAME: &'static str = "create_multiplayer_dungeon_lobby";
    pub fn new() -> Self {
        Self
    }
}

#[export_name = "turbo/create_multiplayer_dungeon_lobby"]
unsafe extern "C" fn exec() -> usize {
    // Get player id
    let user_id = os::server::get_user_id();

    // Emit a lobby creation alert
    os::server::alert!("{:.8} created a party!", user_id);

    // lobby
    let lobby_list_filepath = paths::multiplayer_dungeon_list();
    let mut lobby_list = os::server::read_or!(
        BTreeMap<String, MultiplayerDungeonLobby>,
        &lobby_list_filepath,
        BTreeMap::new()
    );

    // Add new lobby
    let now = os::server::secs_since_unix_epoch();
    os::server::log!("NOW = {now}");
    let _lobby = lobby_list.insert(
        user_id.clone(),
        MultiplayerDungeonLobby::new(os::server::random_number(), now, &user_id),
    );

    // Remove old lobbies
    let ttl = 60 * 10; // 10 min
    lobby_list.retain(|_owner, lobby| {
        os::server::log!("Comparing... {now} - {}", lobby.created_at);
        now - lobby.created_at < ttl
    });

    // Save updated lobby list
    if let Err(err) = os::server::write!(&lobby_list_filepath, &lobby_list) {
        os::server::log!("{err:?}");
        return os::server::CANCEL;
    };

    // Attempt to delete your party lobby if it still exists after some duration
    os::server::enqueue_command(
        &PROGRAM_ID,
        delete_multiplayer_dungeon_lobby::Command::NAME,
        &delete_multiplayer_dungeon_lobby::Command::created_no_later_than(now)
            .try_to_vec()
            .unwrap(),
        os::server::random_number(),
        Some(60_000 * 10), // 10 min
    )
    .unwrap();

    os::server::COMMIT
}
