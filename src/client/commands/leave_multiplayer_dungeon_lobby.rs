use super::*;

pub fn exec(owner: &str) -> String {
    let command = server::commands::leave_multiplayer_dungeon_lobby::Command::NAME;
    let cmd = server::commands::leave_multiplayer_dungeon_lobby::Command::new(owner);
    let data = &cmd.try_to_vec().unwrap();
    os::client::exec(server::PROGRAM_ID, command, data)
}
