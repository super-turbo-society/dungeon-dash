use super::*;

pub fn exec(reset: bool) -> String {
    let command = server::commands::create_new_dungeon::COMMAND;
    let cmd = server::commands::create_new_dungeon::new(reset);
    let data = &cmd.try_to_vec().unwrap();
    os::client::exec(server::PROGRAM_ID, command, data)
}
