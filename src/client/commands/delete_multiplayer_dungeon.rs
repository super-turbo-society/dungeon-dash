use super::*;

pub fn exec(crawl_id: u32) -> String {
    let command = server::commands::delete_multiplayer_dungeon::Command::NAME;
    let cmd = server::commands::delete_multiplayer_dungeon::Command::new(crawl_id);
    let data = &cmd.try_to_vec().unwrap();
    os::client::exec(server::PROGRAM_ID, command, data)
}
