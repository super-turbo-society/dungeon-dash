use super::*;

pub fn render(state: &mut LocalState, user_id: &str) {
    reset_cam!();
    let [w, h] = canvas_size!();

    let mut x = 4;
    let mut y = 4;

    #[rustfmt::skip]
    text!("SELECT A MODE", absolute = true, x = 4, y = 4, font = Font::L);
    y += 16;

    if let Ok(_dungeon) = &client::queries::player_dungeon::fetch(&user_id) {
        if primary_button("RESUME 1P DUNGEON CRAWL", x, y, w - 8) {
            state.screen = Screen::Dungeon;
        }
    } else {
        if secondary_button("1P DUNGEON CRAWL", x, y, w - 8) {
            client::commands::create_new_dungeon::exec(true);
            state.screen = Screen::Dungeon;
        }
    }
    y += 16;

    if let Ok(crawl_id) = client::queries::current_multiplayer_dungeon_crawl_id::fetch(&user_id) {
        // Resume
        if primary_button("RESUME ONLINE CO-OP", x, y, w - 8) {
            state.screen = Screen::MultiplayerDungeon(crawl_id);
        }
    } else {
        if secondary_button("ONLINE CO-OP", x, y, w - 8) {
            state.screen = Screen::MultiplayerDungeonLobbies(MultiplayerDungeonLobbiesContext {
                cursor: 0,
                selected: false,
            });
        }
    }
}
