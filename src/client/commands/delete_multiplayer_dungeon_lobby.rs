use super::*;

pub fn exec() -> String {
    let command = server::commands::delete_multiplayer_dungeon_lobby::Command::NAME;
    let cmd = server::commands::delete_multiplayer_dungeon_lobby::Command::new();
    let data = &cmd.try_to_vec().unwrap();
    os::client::exec(server::PROGRAM_ID, command, data)
}
