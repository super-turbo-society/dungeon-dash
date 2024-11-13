use super::*;

pub const COMMAND: &'static str = "move_monsters";

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Command {
    crawl_id: u32,
}
impl Command {
    pub fn for_crawl_id(crawl_id: u32) -> Self {
        Self { crawl_id }
    }
}

#[export_name = "turbo/move_monsters"]
unsafe extern "C" fn exec() -> usize {
    // Get player id
    let user_id = os::server::get_user_id();

    // Get command data
    let cmd = os::server::command!(Command);

    // Load player dungeon
    os::server::log!("Loading the dungeon for player {}...", user_id);
    let dungeon_filepath = paths::player_dungeon(&user_id);
    let mut dungeon = os::server::read!(Dungeon, &dungeon_filepath);

    os::server::log!(
        "cmd.crawl_id = {} / dungeon.crawl_id = {}",
        cmd.crawl_id,
        dungeon.crawl_id
    );
    if cmd.crawl_id != dungeon.crawl_id {
        return os::server::CANCEL;
    }
    // Cancel command if player has already won or lost
    os::server::log!("Checking game over conditions...");
    if dungeon.player.health == 0 {
        os::server::log!("P1 has died. Game over.");
        return os::server::CANCEL;
    }

    // Move monsters
    os::server::log!("Moving monsters...");
    dungeon.move_monsters(os::server::log);

    // Increment turn
    os::server::log!("Incrementing turn number...");
    dungeon.turn += 1;

    // If player died...
    if dungeon.player.health == 0 {
        // Increment dungeon stats (crawls completed)
        dungeon.increment_stats(DungeonStatKind::CrawlsCompleted, 1);

        // Update the global leaderboard
        os::server::log!("Reading global leaderboard...");
        let leaderboard_filepath = paths::global_leaderboard();
        let mut leaderboard =
            os::server::read_or!(Leaderboard, &leaderboard_filepath, Leaderboard::new());
        os::server::log!("Updating global leaderboard...");
        if let Some(entry) = leaderboard.update(
            dungeon.crawl_id,
            LeaderboardKind::LeastSteps,
            &user_id,
            dungeon.stats.get(DungeonStatKind::StepsMoved),
        ) {
            os::server::alert!(
                "Player {:.8} died after only {:?} steps! R.I.P. son",
                user_id,
                entry.score
            );
        }
        if let Some(entry) = leaderboard.update(
            dungeon.crawl_id,
            LeaderboardKind::MostKills,
            &user_id,
            dungeon.stats.total_monsters_defeated(),
        ) {
            os::server::alert!("Player {:.8} slayed {:?} monsters!", user_id, entry.score);
        }
        if let Some(entry) = leaderboard.update(
            dungeon.crawl_id,
            LeaderboardKind::MostGold,
            &user_id,
            dungeon.stats.get(DungeonStatKind::GoldCollected),
        ) {
            os::server::alert!("Player {:.8} amassed {:?} gold!", user_id, entry.score);
        }
        if let Some(entry) = leaderboard.update(
            dungeon.crawl_id,
            LeaderboardKind::HighestFloor,
            &user_id,
            dungeon.stats.get(DungeonStatKind::FloorsCleared) + 1,
        ) {
            os::server::alert!("Player {:.8} reached floor {:?}!", user_id, entry.score);
        }
        os::server::log!("Saving global leaderboard...");
        os::server::write!(&leaderboard_filepath, &leaderboard)
            .expect("Could not write leaderboard");

        // Update player stats
        os::server::log!("Saving player stats...");
        let player_stats_filepath = paths::player_dungeon_stats(&user_id);
        os::server::write!(&player_stats_filepath, &dungeon.total_stats)
            .expect("Could not write player stats");

        // Unlock achievements
        let next_achievements =
            dungeon
                .unlocked
                .apply_dungeon_stats(&dungeon.stats, &dungeon.total_stats, true);
        dungeon.unlocked = next_achievements.difference(&dungeon.all_unlocked);
        os::server::log!(
            "Achievements (crawl): {:?}",
            dungeon.unlocked.achievement_kinds()
        );
        dungeon.all_unlocked = dungeon.all_unlocked.union(&dungeon.unlocked);
        os::server::log!(
            "Achievements (all): {:?}",
            dungeon.all_unlocked.achievement_kinds()
        );
        os::server::log!("Saving player achievements...");
        let player_achievements_filepath = paths::player_achievements(&user_id);
        os::server::write!(&player_achievements_filepath, &dungeon.all_unlocked)
            .expect("Could not write player achievements");

        // // Enqueue move monster command
        // if dungeon.turn == 0 {
        //     let res = os::server::enequeue_command(
        //         &PROGRAM_ID,
        //         COMMAND,
        //         &Command::for_turn(dungeon.turn).try_to_vec().unwrap(),
        //         os::server::random_number(),
        //         Some(500), // 5 sec
        //     );
        //     match res {
        //         Ok(hash) => os::server::log!("Enqueued transaction with hash {hash}"),
        //         Err(_err) => os::server::log!("Failed to enqueue transaction"),
        //     }
        // }
    }

    // Save the dungeon
    os::server::log!("Saving the dungeon...");
    os::server::write!(&dungeon_filepath, &dungeon).expect("Could not write player dungeon");

    // Commit the command result
    os::server::COMMIT
}