use super::*;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum Command {
    Start,
    Reset(u32),
    NextFloor(u32),
}
impl Command {
    pub const NAME: &'static str = "create_new_multiplayer_dungeon";
    pub fn start() -> Self {
        Self::Start
    }
    pub fn reset(crawl_id: u32) -> Self {
        Self::Reset(crawl_id)
    }
    pub fn next_floor(crawl_id: u32) -> Self {
        Self::NextFloor(crawl_id)
    }
}

#[export_name = "turbo/create_new_multiplayer_dungeon"]
unsafe extern "C" fn exec() -> usize {
    // Get player id
    let user_id = os::server::get_user_id();

    // Get command data
    let cmd = os::server::command!(Command);

    // Get the dungeon
    let mut dungeon = match cmd {
        Command::Start => {
            // Load the lobby list
            let lobby_list_filepath = paths::multiplayer_dungeon_list();
            let mut lobby_list = os::server::read_or!(
                BTreeMap<String, MultiplayerDungeonLobby>,
                &lobby_list_filepath,
                BTreeMap::new()
            );

            // Get the requested lobby
            let Some(lobby) = lobby_list.get_mut(&user_id) else {
                os::server::log!("{user_id} does not have a lobby");
                return os::server::CANCEL;
            };

            // Make sure we have at least 2 players
            if lobby.players.len() < 2 {
                os::server::log!("At least 2 players must join before the crawl can begin!");
                return os::server::CANCEL;
            }

            // Get the crawl id from the lobby
            let crawl_id = lobby.id;

            // Create the dungeon
            let w = 5;
            let h = 5;
            let mut dungeon = MultiplayerDungeon {
                owner: user_id.clone(),
                crawl_id: crawl_id,
                theme: DungeonThemeKind::Castle,
                floor: 0,
                round: 0,
                turn: 0,
                width: w,
                height: h,
                monsters: vec![],
                treasures: vec![],
                obstacles: vec![],
                exit: None,
                exit_key: None,
                player: PlayerState::new(),
            };

            // Randomize player positions
            os::server::log!("Randomizing player positions...");
            let mut i = 0;
            while dungeon.player.players.len() != lobby.players.len() {
                let x = os::server::random_number::<i32>().abs() % dungeon.width as i32;
                let y = os::server::random_number::<i32>().abs() % dungeon.height as i32;
                if !dungeon.is_position_occupied(x, y) {
                    let user_id = lobby.players.iter().nth(i).cloned().unwrap();
                    os::server::log!("Inserting player data for {user_id}...");
                    dungeon.player.players.insert(
                        user_id.clone(),
                        PlayerContext {
                            player: Player {
                                x,
                                y,
                                health: 10,
                                max_health: 10,
                                strength: 1,
                                gold: 0,
                                direction: Direction::Down,
                            },
                            stats: DungeonStats::new(),
                            total_stats: os::server::read_or!(
                                DungeonStats,
                                &paths::player_dungeon_stats(&user_id),
                                DungeonStats::new()
                            ),
                            unlocked: PlayerAchievements::empty(),
                            all_unlocked: os::server::read_or!(
                                PlayerAchievements,
                                &paths::player_achievements(&user_id),
                                PlayerAchievements::empty()
                            ),
                            next_round: 0,
                        },
                    );
                    i += 1;
                }
            }

            // Remove the lobby
            lobby_list.remove(&user_id);
            if let Err(err) = os::server::write!(&lobby_list_filepath, &lobby_list) {
                os::server::log!("{err:?}");
                return os::server::CANCEL;
            };

            // Update each player's multiplayer dungeon manifest
            for user_id in dungeon.player.players.keys() {
                let filepath = paths::player_multiplayer_dungeon_manifest(user_id);
                if let Err(err) = os::server::write!(&filepath, &dungeon.crawl_id.to_le_bytes()) {
                    os::server::log!("{err:?}");
                    return os::server::CANCEL;
                };
            }

            dungeon
        }
        Command::Reset(crawl_id) => {
            // Load player dungeon
            os::server::log!("Loading the multiplayer dungeon ({})...", crawl_id);
            let dungeon_filepath = paths::multiplayer_dungeon(crawl_id);
            let mut dungeon = os::server::read!(MultiplayerDungeon, &dungeon_filepath);

            // Gather dungeon user IDs
            let user_ids: Vec<_> = dungeon.player.players.keys().cloned().collect();

            // Only dungeon members may reset the dungeon
            if !user_ids.iter().any(|id| *id == user_id) {
                os::server::log!("Only the dungeon owner can reset the dungeon.");
                return os::server::CANCEL;
            }

            // Create the dungeon
            let w = 5;
            let h = 5;
            let is_winter = true;
            dungeon = MultiplayerDungeon {
                owner: dungeon.owner,
                crawl_id: crawl_id,
                theme: if is_winter {
                    DungeonThemeKind::Arctic
                } else {
                    DungeonThemeKind::Castle
                },
                floor: 0,
                round: 0,
                turn: 0,
                width: w,
                height: h,
                monsters: vec![],
                treasures: vec![],
                obstacles: vec![],
                exit: None,
                exit_key: None,
                player: PlayerState::new(),
            };

            // Randomize player positions
            os::server::log!("Randomizing player positions...");
            let mut i = 0;
            while dungeon.player.players.len() != user_ids.len() {
                let x = os::server::random_number::<i32>().abs() % dungeon.width as i32;
                let y = os::server::random_number::<i32>().abs() % dungeon.height as i32;
                if !dungeon.is_position_occupied(x, y) {
                    let user_id = user_ids.iter().nth(i).cloned().unwrap();
                    os::server::log!("Inserting player data for {user_id}...");
                    dungeon.player.players.insert(
                        user_id.clone(),
                        PlayerContext {
                            player: Player {
                                x,
                                y,
                                health: 8,
                                max_health: 8,
                                strength: 1,
                                gold: 0,
                                direction: Direction::Down,
                            },
                            stats: DungeonStats::new(),
                            total_stats: os::server::read_or!(
                                DungeonStats,
                                &paths::player_dungeon_stats(&user_id),
                                DungeonStats::new()
                            ),
                            unlocked: PlayerAchievements::empty(),
                            all_unlocked: os::server::read_or!(
                                PlayerAchievements,
                                &paths::player_achievements(&user_id),
                                PlayerAchievements::empty()
                            ),
                            next_round: 0,
                        },
                    );
                    i += 1;
                }
            }

            dungeon
        }
        Command::NextFloor(crawl_id) => {
            // Load player dungeon
            os::server::log!("Loading the multiplayer dungeon ({})...", crawl_id);
            let dungeon_filepath = paths::multiplayer_dungeon(crawl_id);
            let mut dungeon = os::server::read!(MultiplayerDungeon, &dungeon_filepath);

            // Get the player context
            let Some(ctx) = dungeon.player.get(&user_id) else {
                os::server::log!("Not controlling any players.");
                return os::server::CANCEL;
            };

            // Check if player can move to next floor
            if !dungeon.is_exit(ctx.player.x, ctx.player.y) {
                os::server::log!("Player has not reached the exit.");
                return os::server::CANCEL;
            }

            // Remove exit
            dungeon.exit = None;

            // Clear monsters, treasures, and obstacles
            dungeon.monsters.clear();
            dungeon.treasures.clear();
            dungeon.obstacles.clear();

            // Increase floor
            dungeon.floor += 1;

            // Increment floor stats and reset next round
            for ctx in dungeon.player.players.values_mut() {
                ctx.increment_stats(DungeonStatKind::FloorsCleared, 1);
                ctx.next_round = 0;
                if ctx.player.health == 0 {
                    ctx.player.health = 1;
                }
            }

            // Update dungeon theme
            let is_winter = true;
            let i = os::server::random_number::<usize>();
            let theme = if is_winter {
                DungeonThemeKind::WINTER[i % DungeonThemeKind::WINTER.len()]
            } else {
                DungeonThemeKind::ALL[i % DungeonThemeKind::ALL.len()]
            };
            dungeon.theme = theme;

            // Embiggen every 3 floors
            if dungeon.floor % 3 == 0 {
                dungeon.width += 2;
                dungeon.height += 2;
            }

            // Reset turn
            dungeon.round = 0;

            // Randomize positions
            for user_id in dungeon.clone().player.players.keys().clone() {
                loop {
                    let x = os::server::random_number::<i32>().abs() % dungeon.width as i32;
                    let y = os::server::random_number::<i32>().abs() % dungeon.height as i32;
                    if !dungeon.is_position_occupied(x, y) {
                        if dungeon
                            .player
                            .modify_player(user_id, |ctx| {
                                ctx.player.x = x;
                                ctx.player.y = y;
                            })
                            .is_err()
                        {
                            os::server::log!("Player {user_id} not found");
                            return os::server::CANCEL;
                        };
                        break;
                    }
                }
            }

            // Update achievements every floor
            // for (player_index, user_id) in dungeon.player_controllers.iter().enumerate() {
            //     let next_achievements = dungeon.unlocked[player_index].apply_dungeon_stats(
            //         &dungeon.stats[player_index],
            //         &dungeon.total_stats[player_index],
            //         false,
            //     );
            //     let floor_achievements = next_achievements.difference(
            //         &dungeon.unlocked[player_index].union(&dungeon.all_unlocked[player_index]),
            //     );
            //     dungeon.unlocked[player_index] =
            //         next_achievements.difference(&dungeon.all_unlocked[player_index]);
            //     os::server::log!(
            //         "{user_id} - Achievements (floor): {:?}",
            //         floor_achievements.achievement_kinds()
            //     );
            //     os::server::log!(
            //         "{user_id} - Achievements (crawl): {:?}",
            //         dungeon.unlocked[player_index].achievement_kinds()
            //     );
            //     os::server::log!(
            //         "{user_id} - Achievements (all): {:?}",
            //         dungeon.all_unlocked[player_index].achievement_kinds()
            //     );
            // }

            dungeon
        }
    };

    // Get the dungeon bounds
    let (max_x, max_y) = dungeon.bounds();

    // Get the magic ratio
    let magic_ratio = ((max_x * max_y) / 32) as usize;

    // After first floor, add monsters and treasures
    if dungeon.floor > 0 {
        // Randomize treasures
        os::server::log!("Randomizing monsters...");
        let num_monsters = 2 + magic_ratio;
        // Define monsters and their weights
        let monster_weights: &[(u32, MonsterKind)] = match dungeon.theme {
            DungeonThemeKind::Castle => &[
                (2, MonsterKind::BlueBlob),
                (1, MonsterKind::GreenGoblin),
                (1, MonsterKind::OrangeGoblin),
            ],
            DungeonThemeKind::Crypt => &[
                (3, MonsterKind::Ghost),
                (2, MonsterKind::Shade),
                (1, MonsterKind::Zombie),
            ],
            DungeonThemeKind::Pirate => &[
                (1, MonsterKind::Shade),
                (2, MonsterKind::OrangeGoblin),
                (1, MonsterKind::Zombie),
            ],
            DungeonThemeKind::Forest => &[
                (1, MonsterKind::YellowBlob),
                (1, MonsterKind::RedBlob),
                (2, MonsterKind::Spider),
            ],
            DungeonThemeKind::IceCave => &[
                (3, MonsterKind::GreenGoblin),
                (2, MonsterKind::Ghost),
                (1, MonsterKind::BlueBlob),
            ],
            DungeonThemeKind::Arctic => &[
                (3, MonsterKind::GreenGoblin),
                (2, MonsterKind::Shade),
                (1, MonsterKind::Spider),
            ],
        };
        let mut monster_weights = monster_weights.to_vec();

        let is_winter = true;
        if is_winter {
            monster_weights.push((1, MonsterKind::Snowman));
        }

        // After level 20, Evil Turbi will probably show up
        if dungeon.floor + 1 >= 20 {
            monster_weights.push((3, MonsterKind::EvilTurbi));
        }
        let total_weight: u32 = monster_weights.iter().map(|(weight, _)| *weight).sum();

        while dungeon.monsters.len() < num_monsters {
            let x = os::server::random_number::<i32>().abs() % max_x;
            let y = os::server::random_number::<i32>().abs() % max_y;
            if !dungeon.is_position_occupied(x, y) {
                // Generate a random number within the total weight
                let rng = os::server::random_number::<u32>() % total_weight;
                let mut selected_monster = MonsterKind::GreenGoblin;
                // Select the monster based on weighted probability
                let mut cumulative_weight = 0;
                for (weight, monster_kind) in &monster_weights {
                    cumulative_weight += *weight;
                    if rng < cumulative_weight {
                        selected_monster = *monster_kind;
                        break;
                    }
                }
                // Define monster stats based on the selected kind
                let (health, strength) = selected_monster.stats();
                let monster = Monster {
                    x,
                    y,
                    health,
                    max_health: health,
                    strength,
                    direction: Direction::Down,
                    kind: selected_monster,
                    stun_dur: 0,
                };
                dungeon.monsters.push(monster);
            }
        }

        // Randomize treasures
        os::server::log("Randomizing treasures...");
        let num_treasures = magic_ratio + (dungeon.floor as f32 * 0.75) as usize;
        while dungeon.treasures.len() < num_treasures {
            let x = os::server::random_number::<i32>().abs() % max_x;
            let y = os::server::random_number::<i32>().abs() % max_y;
            if !dungeon.is_position_occupied(x, y) {
                // Last treasure is a healing item
                if dungeon.treasures.len() == num_treasures - 1 {
                    dungeon.treasures.push(Treasure {
                        x,
                        y,
                        value: 2,
                        kind: TreasureKind::Heal,
                    })
                }
                // Every other treasure gives the player gold
                else {
                    let n = os::server::random_number::<u32>() % 10;
                    dungeon.treasures.push(if n < 9 {
                        // 90% chance for $1 gold treasure
                        Treasure {
                            x,
                            y,
                            value: 1,
                            kind: TreasureKind::Gold,
                        }
                    } else {
                        // 10% chance for $10 gold treasure
                        Treasure {
                            x,
                            y,
                            value: 10,
                            kind: TreasureKind::Gold,
                        }
                    });
                }
            }
        }
    }

    // Initialize exit_key position at least 8 tiles away from player 0
    os::server::log!("Initializing exit key position...");
    let min_distance = (dungeon.width.min(dungeon.height) / 2) as i32;
    loop {
        let x = os::server::random_number::<i32>().abs() % max_x;
        let y = os::server::random_number::<i32>().abs() % max_y;
        let ctx = dungeon.player.players.values().nth(0).unwrap();
        let dx = (x - ctx.player.x).abs();
        let dy = (y - ctx.player.y).abs();
        if dx + dy >= min_distance && !dungeon.is_position_occupied(x, y) {
            dungeon.exit_key = Some((x, y));
            break;
        }
    }

    // Randomize obstacles
    os::server::log!("Randomizing obstacles...");
    for (x, y) in generate_maze(max_x as usize, max_y as usize) {
        // 1/3 chance to skip a obstacle placement
        if os::server::random_number::<u8>() % 3 == 0 {
            continue;
        }
        // Make sure spot is empty
        if dungeon.is_position_occupied(x, y) {
            continue;
        }
        dungeon.obstacles.push(Obstacle {
            x,
            y,
            kind: if os::server::random_number::<usize>() % 10 == 9 {
                // 10% chance for firepit
                ObstacleKind::WallB
            } else {
                // 90% chance for stone block
                ObstacleKind::WallA
            },
        });
    }

    // Save the dungeon
    os::server::log!("Saving dungeon...");
    let dungeon_filepath = paths::multiplayer_dungeon(dungeon.crawl_id);
    os::server::write!(&dungeon_filepath, &dungeon).expect("Could not save dungeon file.");

    os::server::COMMIT
}

fn generate_maze(width: usize, height: usize) -> Vec<(i32, i32)> {
    let mut grid = vec![vec![false; width]; height];
    let mut walls = vec![];

    pub fn divide(
        grid: &mut Vec<Vec<bool>>,
        walls: &mut Vec<(i32, i32)>,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) {
        if width <= 3 || height <= 3 {
            return;
        }

        let horizontal = os::server::random_number::<u8>() % 2 == 0;

        if horizontal {
            let max_wall_y = y + height - 2;
            let min_wall_y = y + 1;
            if max_wall_y < min_wall_y {
                return;
            }
            let wall_y = min_wall_y
                + (os::server::random_number::<usize>() % ((max_wall_y - min_wall_y) / 2 + 1)) * 2;

            for i in x..x + width {
                grid[wall_y][i] = true;
                walls.push((i as i32, wall_y as i32));
            }

            let passage_x = x + os::server::random_number::<usize>() % width;
            grid[wall_y][passage_x] = false;
            walls.retain(|&(wx, wy)| !(wx == passage_x as i32 && wy == wall_y as i32));

            // Ensure at least one passage in the adjacent walls
            if wall_y > 0 && wall_y + 1 < grid.len() {
                if !grid[wall_y - 1][passage_x] && !grid[wall_y + 1][passage_x] {
                    grid[wall_y][passage_x] = false;
                    walls.retain(|&(wx, wy)| !(wx == passage_x as i32 && wy == wall_y as i32));
                }
            }

            divide(grid, walls, x, y, width, wall_y - y);
            divide(grid, walls, x, wall_y + 1, width, y + height - wall_y - 1);
        } else {
            let max_wall_x = x + width - 2;
            let min_wall_x = x + 1;
            if max_wall_x < min_wall_x {
                return;
            }
            let wall_x = min_wall_x
                + (os::server::random_number::<usize>() % ((max_wall_x - min_wall_x) / 2 + 1)) * 2;

            for i in y..y + height {
                grid[i][wall_x] = true;
                walls.push((wall_x as i32, i as i32));
            }

            let passage_y = y + os::server::random_number::<usize>() % height;
            grid[passage_y][wall_x] = false;
            walls.retain(|&(wx, wy)| !(wx == wall_x as i32 && wy == passage_y as i32));

            // Ensure at least one passage in the adjacent walls
            if wall_x > 0 && wall_x + 1 < grid[0].len() {
                if !grid[passage_y][wall_x - 1] && !grid[passage_y][wall_x + 1] {
                    grid[passage_y][wall_x] = false;
                    walls.retain(|&(wx, wy)| !(wx == wall_x as i32 && wy == passage_y as i32));
                }
            }

            divide(grid, walls, x, y, wall_x - x, height);
            divide(grid, walls, wall_x + 1, y, x + width - wall_x - 1, height);
        }
    }

    divide(&mut grid, &mut walls, 0, 0, width, height);
    walls
}
