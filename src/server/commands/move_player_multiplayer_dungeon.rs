use super::*;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Command {
    crawl_id: u32,
    direction: Direction,
}
impl Command {
    pub const NAME: &'static str = "move_player_multiplayer_dungeon";
    pub fn new(crawl_id: u32, direction: Direction) -> Self {
        Self {
            crawl_id,
            direction,
        }
    }
}

#[export_name = "turbo/move_player_multiplayer_dungeon"]
unsafe extern "C" fn exec() -> usize {
    // Get player id
    let user_id = os::server::get_user_id();

    // Get command data
    let cmd = os::server::command!(Command);

    // Load multiplayer dungeon
    os::server::log!("Loading the multiplayer dungeon ({})...", cmd.crawl_id);
    let dungeon_filepath = paths::multiplayer_dungeon(cmd.crawl_id);
    let mut dungeon = os::server::read!(MultiplayerDungeon, &dungeon_filepath);

    // Move player
    os::server::log!("Moving player...");
    if dungeon.move_player(&user_id, cmd.direction).is_err() {
        return os::server::CANCEL;
    }
    dungeon.turn += 1;

    // If no more players can move this round, move the monsters
    if dungeon.did_all_players_move() {
        os::server::log!("Moving monsters...");
        if let Err(err) = dungeon.move_all_monsters() {
            os::server::log!("Error moving monsters: {err}");
            return os::server::CANCEL;
        };
        // Increment turn
        os::server::log!("Incrementing dungeon round...");
        dungeon.round += 1;
    }

    // If all players died, update stats and achievements
    if dungeon.did_all_players_die() {
        // // Increment dungeon stats (crawls completed)
        // dungeon.increment_stats(DungeonStatKind::CrawlsCompleted, 1);

        // // Update the global leaderboard
        // os::server::log!("Reading global leaderboard...");
        // let leaderboard_filepath = paths::player_dungeon(&user_id);
        // let mut leaderboard =
        //     os::server::read_or!(Leaderboard, &leaderboard_filepath, Leaderboard::new());
        // os::server::log!("Updating global leaderboard...");
        // if let Some(entry) = leaderboard.update(
        //     dungeon.crawl_id,
        //     LeaderboardKind::LeastSteps,
        //     &user_id,
        //     dungeon.stats.get(DungeonStatKind::StepsMoved),
        // ) {
        //     os::server::alert!(
        //         "Player {:.8} died after only {:?} steps! R.I.P. son",
        //         user_id,
        //         entry.score
        //     );
        // }
        // if let Some(entry) = leaderboard.update(
        //     dungeon.crawl_id,
        //     LeaderboardKind::MostKills,
        //     &user_id,
        //     dungeon.stats.total_monsters_defeated(),
        // ) {
        //     os::server::alert!("Player {:.8} slayed {:?} monsters!", user_id, entry.score);
        // }
        // if let Some(entry) = leaderboard.update(
        //     dungeon.crawl_id,
        //     LeaderboardKind::MostGold,
        //     &user_id,
        //     dungeon.stats.get(DungeonStatKind::GoldCollected),
        // ) {
        //     os::server::alert!("Player {:.8} amassed {:?} gold!", user_id, entry.score);
        // }
        // if let Some(entry) = leaderboard.update(
        //     dungeon.crawl_id,
        //     LeaderboardKind::HighestFloor,
        //     &user_id,
        //     dungeon.stats.get(DungeonStatKind::FloorsCleared) + 1,
        // ) {
        //     os::server::alert!("Player {:.8} reached floor {:?}!", user_id, entry.score);
        // }
        // os::server::log!("Saving global leaderboard...");
        // os::server::write!(&leaderboard_filepath, &leaderboard)
        //     .expect("Could not write leaderboard");

        // // Update player stats
        // os::server::log!("Saving player stats...");
        // let player_stats_filepath = paths::player_dungeon_stats(&user_id);
        // os::server::write!(&player_stats_filepath, &dungeon.total_stats)
        //     .expect("Could not write player stats");

        // // Unlock achievements
        // let next_achievements =
        //     dungeon
        //         .unlocked
        //         .apply_dungeon_stats(&dungeon.stats, &dungeon.total_stats, true);
        // dungeon.unlocked = next_achievements.difference(&dungeon.all_unlocked);
        // os::server::log!(
        //     "Achievements (crawl): {:?}",
        //     dungeon.unlocked.achievement_kinds()
        // );
        // dungeon.all_unlocked = dungeon.all_unlocked.union(&dungeon.unlocked);
        // os::server::log!(
        //     "Achievements (all): {:?}",
        //     dungeon.all_unlocked.achievement_kinds()
        // );
        // os::server::log!("Saving player achievements...");
        // let player_achievements_filepath = paths::player_achievements(&user_id);
        // os::server::write!(&player_achievements_filepath, &dungeon.all_unlocked)
        //     .expect("Could not write player achievements");
    }

    // Save the dungeon
    os::server::log!("Saving the dungeon...");
    os::server::write!(&dungeon_filepath, &dungeon).expect("Could not write player dungeon");

    // Commit the command result
    os::server::COMMIT
}
