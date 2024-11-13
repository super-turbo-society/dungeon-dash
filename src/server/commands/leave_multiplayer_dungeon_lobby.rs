use super::*;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Command {
    owner: String,
}
impl Command {
    pub const NAME: &'static str = "leave_multiplayer_dungeon_lobby";
    pub fn new(owner: &str) -> Self {
        Self {
            owner: owner.to_string(),
        }
    }
}

#[export_name = "turbo/leave_multiplayer_dungeon_lobby"]
unsafe extern "C" fn exec() -> usize {
    // Get player id
    let user_id = os::server::get_user_id();

    // Get command data
    let cmd = os::server::command!(Command);

    // Load the lobby list
    let lobby_list_filepath = paths::multiplayer_dungeon_list();
    let mut lobby_list = os::server::read_or!(
        BTreeMap<String, MultiplayerDungeonLobby>,
        &lobby_list_filepath,
        BTreeMap::new()
    );

    // Get the requested lobby
    let Some(lobby) = lobby_list.get_mut(&cmd.owner) else {
        os::server::log!("Lobby is not available");
        // Lobby is not available, so this is a success/noop
        return os::server::COMMIT;
    };

    // Remove this user from the lobby
    lobby.players.remove(&user_id);

    // Save updated lobby list
    if let Err(err) = os::server::write!(&lobby_list_filepath, &lobby_list) {
        os::server::log!("{err:?}");
        return os::server::CANCEL;
    };

    os::server::COMMIT
}
