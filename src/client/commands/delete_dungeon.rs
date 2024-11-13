use super::*;

pub fn exec() -> String {
    use server::commands::delete_dungeon::Command;
    let data = Command::new().try_to_vec().unwrap();
    os::client::exec(server::PROGRAM_ID, Command::NAME, &data)
}
