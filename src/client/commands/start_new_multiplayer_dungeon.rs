use super::*;

pub fn exec() -> String {
    let command = server::commands::create_new_multiplayer_dungeon::Command::NAME;
    let cmd = server::commands::create_new_multiplayer_dungeon::Command::start();
    let data = &cmd.try_to_vec().unwrap();
    os::client::exec(server::PROGRAM_ID, command, data)
}
