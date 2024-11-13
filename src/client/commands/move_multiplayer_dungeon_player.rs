use super::*;

pub fn exec(crawl_id: u32, direction: Direction) -> String {
    let command = server::commands::move_player_multiplayer_dungeon::Command::NAME;
    let cmd = server::commands::move_player_multiplayer_dungeon::Command::new(crawl_id, direction);
    let data = &cmd.try_to_vec().unwrap();
    os::client::exec(server::PROGRAM_ID, command, data)
}
