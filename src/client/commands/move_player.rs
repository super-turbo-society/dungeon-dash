use super::*;

pub fn exec(direction: Direction) -> String {
    let command = server::commands::move_player::COMMAND;
    let cmd = server::commands::move_player::new(direction);
    let data = &cmd.try_to_vec().unwrap();
    os::client::exec(server::PROGRAM_ID, command, data)
}
