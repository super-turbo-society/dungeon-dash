use super::*;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Command;
impl Command {
    pub const NAME: &'static str = "delete_dungeon";
    pub fn new() -> Self {
        Self
    }
}

#[export_name = "turbo/delete_dungeon"]
unsafe extern "C" fn exec() -> usize {
    // Get player id
    let user_id = os::server::get_user_id();

    // Save updated lobby list
    let dungeon_filepath = paths::player_dungeon(&user_id);
    if let Err(err) = os::server::write_file(&dungeon_filepath, &[]) {
        os::server::log!("Could not delete player dungeon: {err:?}");
        return os::server::CANCEL;
    };

    os::server::COMMIT
}
