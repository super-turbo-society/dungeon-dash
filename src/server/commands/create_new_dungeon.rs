use super::*;

pub const COMMAND: &'static str = "create_new_dungeon";

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct CreateDungeonCommand {
    pub reset: bool,
}

pub fn new(reset: bool) -> CreateDungeonCommand {
    CreateDungeonCommand { reset }
}

#[export_name = "turbo/create_new_dungeon"]
unsafe extern "C" fn exec() -> usize {
    // Get player id
    let user_id = os::server::get_user_id();

    // Get command data
    let cmd = os::server::command!(CreateDungeonCommand);

    let dungeon_filepath = paths::player_dungeon(&user_id);
    let player_stats_filepath = paths::player_dungeon_stats(&user_id);

    // Get the dungeon
    let mut dungeon = if cmd.reset {
        // Trigger an alert for new players!
        let total_stats =
            os::server::read_or!(DungeonStats, &player_stats_filepath, DungeonStats::new());
        if total_stats.get(DungeonStatKind::CrawlsCompleted) == 0 {
            os::server::alert!("Player {:.8} has entered the dungeon!", user_id);
        }

        let w = 5;
        let h = 5;
        Dungeon {
            crawl_id: os::server::random_number::<u32>(),
            theme: DungeonThemeKind::Castle,
            floor: 0,
            turn: 0,
            width: w,
            height: h,
            player: Player {
                x: os::server::random_number::<i32>().abs() % w as i32,
                y: os::server::random_number::<i32>().abs() % h as i32,
                health: 8,
                max_health: 8,
                strength: 1,
                gold: 0,
                direction: Direction::Down,
            },
            monsters: vec![],
            treasures: vec![],
            obstacles: vec![],
            exit: None,
            exit_key: None,
            stats: DungeonStats::new(),
            total_stats,
            unlocked: PlayerAchievements::empty(),
            all_unlocked: os::server::read_or!(
                PlayerAchievements,
                &paths::player_achievements(&user_id),
                PlayerAchievements::empty()
            ),
        }
    } else {
        // Load player dungeon
        os::server::log!("Loading the dungeon for player {}...", user_id);
        let mut dungeon = os::server::read!(Dungeon, &dungeon_filepath);

        // Check if player can move to next floor
        if !dungeon.is_exit(dungeon.player.x, dungeon.player.y) {
            os::server::log!("P1 has not reached the exit.");
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
        dungeon.increment_stats(DungeonStatKind::FloorsCleared, 1);

        // Update dungeon theme
        let i = os::server::random_number::<usize>();
        let theme = DungeonThemeKind::KINDS[i % DungeonThemeKind::KINDS.len()];
        dungeon.theme = theme;

        // Embiggen every 3 floors
        if dungeon.floor % 3 == 0 {
            dungeon.width += 2;
            dungeon.height += 2;
        }

        // Reset turn
        dungeon.turn = 0;

        // Update achievements every floor
        let next_achievements =
            dungeon
                .unlocked
                .apply_dungeon_stats(&dungeon.stats, &dungeon.total_stats, false);
        let floor_achievements =
            next_achievements.difference(&dungeon.unlocked.union(&dungeon.all_unlocked));
        dungeon.unlocked = next_achievements.difference(&dungeon.all_unlocked);
        os::server::log!(
            "Achievements (floor): {:?}",
            floor_achievements.achievement_kinds()
        );
        os::server::log!(
            "Achievements (crawl): {:?}",
            dungeon.unlocked.achievement_kinds()
        );
        os::server::log!(
            "Achievements (all): {:?}",
            dungeon.all_unlocked.achievement_kinds()
        );

        dungeon
    };

    // Get the dungeon bounds
    let (max_x, max_y) = dungeon.bounds();

    let magic_ratio = ((max_x * max_y) / 32) as usize;

    // After first floor, add monsters and treasures
    if dungeon.floor > 0 {
        // Randomize treasures
        os::server::log!("Randomizing monsters...");
        let num_monsters = 2 + (magic_ratio / 2);
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
        };
        let mut monster_weights = monster_weights.to_vec();

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
                let (health, strength) = match selected_monster {
                    MonsterKind::OrangeGoblin => (5, 1),
                    MonsterKind::GreenGoblin => (2, 1),
                    MonsterKind::RedBlob => (3, 2),
                    MonsterKind::YellowBlob => (2, 1),
                    MonsterKind::Shade => (3, 2),
                    MonsterKind::Spider => (4, 2),
                    MonsterKind::Ghost => (2, 2),
                    MonsterKind::Zombie => (3, 3),
                    MonsterKind::EvilTurbi => (3, 3),
                    _ => (1, 1),
                };
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
        let num_treasures = magic_ratio + (dungeon.floor as usize / 2);
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

    // Initialize exit_key position at least 8 tiles away from player
    os::server::log!("Initializing exit key position...");
    let min_distance = (dungeon.width.min(dungeon.height) / 2) as i32;
    loop {
        let x = os::server::random_number::<i32>().abs() % max_x;
        let y = os::server::random_number::<i32>().abs() % max_y;
        let dx = (x - dungeon.player.x).abs();
        let dy = (y - dungeon.player.y).abs();
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
