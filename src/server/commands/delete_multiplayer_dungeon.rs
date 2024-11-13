use super::*;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Command {
    crawl_id: u32,
}
impl Command {
    pub const NAME: &'static str = "delete_new_multiplayer_dungeon";
    pub fn new(crawl_id: u32) -> Self {
        Self { crawl_id }
    }
}

#[export_name = "turbo/delete_new_multiplayer_dungeon"]
unsafe extern "C" fn exec() -> usize {
    // Get player id
    let user_id = os::server::get_user_id();

    // Get command data
    let cmd = os::server::command!(Command);

    // Load player dungeon
    os::server::log!("Loading the multiplayer dungeon ({})...", cmd.crawl_id);
    let dungeon_filepath = paths::multiplayer_dungeon(cmd.crawl_id);
    let mut dungeon = os::server::read!(MultiplayerDungeon, &dungeon_filepath);

    // If user is owner or 2nd-to-last player, delete the whole dungeon
    if dungeon.owner == user_id
        || dungeon.player.players.contains_key(&user_id) && dungeon.player.players.len() == 2
    {
        // Clear each player's multiplayer dungeon manifest
        for user_id in dungeon.player.players.keys() {
            let filepath = paths::player_multiplayer_dungeon_manifest(user_id);
            if let Err(err) = os::server::write_file(&filepath, &[]) {
                os::server::log!("{err:?}");
                return os::server::CANCEL;
            };
        }

        // Delete the dungeon
        os::server::log!("Deleting dungeon...");
        let dungeon_filepath = paths::multiplayer_dungeon(cmd.crawl_id);
        // TODO: clear/delete macro
        os::server::write_file(&dungeon_filepath, &[]).expect("Could not save dungeon file.");

        return os::server::COMMIT;
    }

    // If user is player, make them leave the dungeon
    if dungeon.player.players.contains_key(&user_id) {
        os::server::emit(
            &format!("multiplayer_dungeon_{}", cmd.crawl_id),
            format!("{:.8} left the party", user_id).as_bytes(),
        );

        // Remove player from dungeon
        os::server::log!("Removing {user_id} from dungeon...");
        dungeon.player.players.remove(&user_id);

        // Clear this player's multiplayer dungeon manifest
        os::server::log!("Clearing {user_id} manifest...");
        let filepath = paths::player_multiplayer_dungeon_manifest(&user_id);
        if let Err(err) = os::server::write_file(&filepath, &[]) {
            os::server::log!("{err:?}");
            return os::server::CANCEL;
        };

        os::server::log!("Saving dungeon...");
        let dungeon_filepath = paths::multiplayer_dungeon(cmd.crawl_id);
        os::server::write!(&dungeon_filepath, &dungeon).expect("Could not save dungeon file.");

        return os::server::COMMIT;
    }

    return os::server::CANCEL;
}
