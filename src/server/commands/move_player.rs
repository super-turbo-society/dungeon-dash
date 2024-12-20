use super::*;

pub const COMMAND: &'static str = "move_player";

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct MovePlayerCommand {
    pub direction: Direction,
}

pub fn new(direction: Direction) -> MovePlayerCommand {
    MovePlayerCommand { direction }
}

#[export_name = "turbo/move_player"]
unsafe extern "C" fn exec() -> usize {
    // Get player id
    let user_id = os::server::get_user_id();

    // Get command data
    let cmd = os::server::command!(MovePlayerCommand);

    // Load player dungeon
    os::server::log!("Loading the dungeon for player {}...", user_id);
    let dungeon_filepath = paths::player_dungeon(&user_id);
    let mut dungeon = os::server::read_else!(Dungeon, &dungeon_filepath, {
        // Reset dungeon file
        os::server::write_file(&dungeon_filepath, &[]).expect("Could not save dungeon file.");
        return os::server::COMMIT;
    });

    // Cancel command if player has already won or lost
    os::server::log!("Checking game over conditions...");
    if dungeon.player.health == 0 {
        os::server::log!("P1 has died. Game over.");
        return os::server::CANCEL;
    }

    // Move player
    os::server::log!("Moving player...");
    if !dungeon.move_player(cmd.direction, os::server::log) {
        return os::server::CANCEL;
    }

    // Move monsters if player has not reached the exit
    if !dungeon.is_exit(dungeon.player.x, dungeon.player.y) {
        os::server::log!("Moving monsters...");
        dungeon.move_monsters(os::server::log);

        // Alternatively, you could invoke an external command to perform actions
        // The flow to invoke the move_monsters command from within the move_players command:
        // 1. Write our dungeon updates to the player's dungeon file
        // 2. Execute move_monsters via invoke_command. This will update the dungeon file.
        // 3. Read the dungeon file and update our dungeon var
        // os::server::write!(&dungeon_filepath, &dungeon).expect("Could not write player dungeon");
        // let res = os::server::invoke_command(
        //     &PROGRAM_ID,
        //     &move_monsters::COMMAND,
        //     &move_monsters::Command::for_crawl_id(dungeon.crawl_id)
        //         .try_to_vec()
        //         .unwrap(),
        // )
        // .expect("Could not invoke move_monsters command");
        // dungeon = os::server::read!(Dungeon, &dungeon_filepath);
    } else {
        os::server::log!("P1 reached exit.");
    }

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

        let floor_rankings_filepath = "floor_rankings";
        let mut floor_rankings =
            os::server::read_or!(BTreeMap<String, u32>, floor_rankings_filepath, BTreeMap::new());
        // for (user_id, floor) in &mut floor_rankings {
        //     match user_id.as_ref() {
        //         "11b5dff9-fb1f-45f1-966b-4b8daf04f23c" => *floor = dungeon.floor.max(10),
        //         "30170022-a3e4-4902-b1a9-c6013acc91de" => *floor = dungeon.floor.max(29),
        //         "1aee449d-6e01-4d14-a049-611e25651762" => *floor = dungeon.floor.max(5),
        //         "80c5b359-ebec-4d4a-8555-29ae17e7fdbc" => *floor = dungeon.floor.max(12),
        //         "8aec2917-35e8-4f7a-b55b-c2a36d8ddc6b" => *floor = dungeon.floor.max(21),
        //         "8b3b2547-a74d-4068-988a-4378278b6416" => *floor = dungeon.floor.max(5),

        //         "b28da31c-442c-460e-89cb-50d08fc44b1b" => *floor = dungeon.floor.max(51),
        //         "c0f4704f-d16b-44b2-a6e9-07640b0a75b8" => *floor = dungeon.floor.max(33),
        //         "c43bfa55-afa0-4dbb-a2a9-730878345e6d" => *floor = dungeon.floor.max(1),
        //         "ce244460-d3ad-420e-95f1-a097385ab7a6" => *floor = dungeon.floor.max(32),
        //         "e8a81f83-7f2c-488b-9ed0-82edf68f6f3b" => *floor = dungeon.floor.max(3),
        //         _ => {
        //             if *floor > 51 {
        //                 *floor = 0;
        //             }
        //         }
        //     }
        // }
        floor_rankings
            .entry(user_id.clone())
            .and_modify(|floor| {
                if *floor < dungeon.floor {
                    *floor = dungeon.floor;
                }
            })
            .or_insert(dungeon.floor);
        if floor_rankings.len() > 100 {
            floor_rankings.pop_first();
        }
        os::server::write!(&floor_rankings_filepath, &floor_rankings)
            .expect("Could not write floor_rankings");

        let yeti_rankings_filepath = "yeti_rankings";
        let mut yeti_rankings =
            os::server::read_or!(BTreeMap<String, u32>, yeti_rankings_filepath, BTreeMap::new());
        yeti_rankings.insert(
            user_id.clone(),
            dungeon.total_stats.monster_kills(MonsterKind::IceYeti),
        );
        if yeti_rankings.len() > 100 {
            yeti_rankings.pop_first();
        }
        os::server::write!(&yeti_rankings_filepath, &yeti_rankings)
            .expect("Could not write yeti_rankings");
    }

    // Save the dungeon
    os::server::log!("Saving the dungeon...");
    os::server::write!(&dungeon_filepath, &dungeon).expect("Could not write player dungeon");

    // Commit the command result
    os::server::COMMIT
}
