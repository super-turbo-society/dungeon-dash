use super::*;

pub fn exec(crawl_id: u32) -> String {
    let command = server::commands::create_new_multiplayer_dungeon::Command::NAME;
    let cmd = server::commands::create_new_multiplayer_dungeon::Command::next_floor(crawl_id);
    let data = &cmd.try_to_vec().unwrap();
    os::client::exec(server::PROGRAM_ID, command, data)
}
