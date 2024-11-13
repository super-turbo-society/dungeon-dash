use super::*;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Command;
impl Command {
    pub const NAME: &'static str = "delete_multiplayer_dungeon_lobby";
    pub fn new() -> Self {
        Self
    }
}

#[export_name = "turbo/delete_multiplayer_dungeon_lobby"]
unsafe extern "C" fn exec() -> usize {
    // Get player id
    let user_id = os::server::get_user_id();

    // lobby
    let lobby_list_filepath = paths::multiplayer_dungeon_list();
    let mut lobby_list = os::server::read_or!(
        BTreeMap<String, MultiplayerDungeonLobby>,
        &lobby_list_filepath,
        BTreeMap::new()
    );

    // Remove lobby
    lobby_list.remove(&user_id);

    // Save updated lobby list
    if let Err(err) = os::server::write!(&lobby_list_filepath, &lobby_list) {
        os::server::log!("{err:?}");
        return os::server::CANCEL;
    };

    os::server::COMMIT
}
