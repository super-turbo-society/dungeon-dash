use super::*;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum Command {
    Any,
    CreatedNoLaterThan(u32),
}
impl Command {
    pub const NAME: &'static str = "delete_multiplayer_dungeon_lobby";
    pub fn new() -> Self {
        Self::Any
    }
    pub fn created_no_later_than(timestamp: u32) -> Self {
        Self::CreatedNoLaterThan(timestamp)
    }
}

#[export_name = "turbo/delete_multiplayer_dungeon_lobby"]
unsafe extern "C" fn exec() -> usize {
    // Get player id
    let user_id = os::server::get_user_id();

    // Get the command data
    let cmd = os::server::command!(Command);

    // lobby
    let lobby_list_filepath = paths::multiplayer_dungeon_list();
    let mut lobby_list = os::server::read_or!(
        BTreeMap<String, MultiplayerDungeonLobby>,
        &lobby_list_filepath,
        BTreeMap::new()
    );

    // Check if the lobby should be removed
    let should_remove = match cmd {
        Command::Any => true,
        Command::CreatedNoLaterThan(created_at) => {
            match lobby_list.get(&user_id) {
                // Only delete lobbies that were created no later than specified
                Some(lobby) => {
                    let is_old_enough = lobby.created_at <= created_at;
                    if !is_old_enough {
                        os::server::log!(
                            "Party won't be deleted since its creation ({}) is after the given deadline {}",
                            lobby.created_at,
                            created_at
                        );
                    }
                    is_old_enough
                }
                // early return if the lobby is already gone
                None => return os::server::COMMIT,
            }
        }
    };

    // Bail out if we shouldn't remove the lobby
    if !should_remove {
        return os::server::CANCEL;
    }

    // Remove lobby
    lobby_list.remove(&user_id);

    // Save updated lobby list
    if let Err(err) = os::server::write!(&lobby_list_filepath, &lobby_list) {
        os::server::log!("{err:?}");
        return os::server::CANCEL;
    };

    os::server::COMMIT
}
