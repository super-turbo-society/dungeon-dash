use super::*;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Dungeon {
    pub crawl_id: u32,
    pub theme: DungeonThemeKind,
    pub floor: u32,
    pub turn: u32,
    pub width: u32,
    pub height: u32,
    pub player: Player,
    pub monsters: Vec<Monster>,
    pub treasures: Vec<Treasure>,
    pub obstacles: Vec<Obstacle>,
    pub exit_key: Option<(i32, i32)>,
    pub exit: Option<(i32, i32)>,
    pub stats: DungeonStats,
    pub total_stats: DungeonStats,
    // TODO: move achievements to own struct/file
    pub unlocked: PlayerAchievements,
    pub all_unlocked: PlayerAchievements,
}
impl Dungeon {
    pub fn move_player(&mut self, direction: Direction, log: fn(&str)) -> bool {
        if self.player.health == 0 {
            log("P1 is dead.");
            return false;
        }

        let Player { x, y, .. } = self.player;
        let (new_x, new_y) = match direction {
            Direction::Up => (x, y - 1),
            Direction::Down => (x, y + 1),
            Direction::Left => (x - 1, y),
            Direction::Right => (x + 1, y),
        };

        if self.is_out_of_bounds(new_x, new_y) {
            log("P1 cannot move out-of-bounds");
            return false;
        }

        if self.is_obstacle(new_x, new_y) {
            log("P1 cannot move through obstacle");
            return false;
        }

        // do an attack if cooldown
        if self.is_monster(new_x, new_y) {
            let self_ptr = self as *mut Self;
            if let Some(monster) = self
                .monsters
                .iter_mut()
                .find(|m| m.x == new_x && m.y == new_y && m.health > 0)
            {
                // Swap positions with the stunned monsters
                if monster.stun_dur > 0 {
                    std::mem::swap(&mut self.player.x, &mut monster.x);
                    std::mem::swap(&mut self.player.y, &mut monster.y);
                    return true;
                }

                let monster_name = monster.kind.abbrev();
                let msg = format!("P1 attacks {}!", monster_name);
                log(&msg);
                let amount = self.player.strength;
                let msg = format!("P1 did {amount} damage.");
                log(&msg);
                monster.stun_dur = 2;
                let prev_monster_health = monster.health;
                monster.health = monster.health.saturating_sub(amount);
                let damage = prev_monster_health.abs_diff(monster.health);

                // Use unsafe to bypass the borrow checker and safely call other mut methods
                unsafe {
                    (*self_ptr).increment_stats(DungeonStatKind::DamageDealt, damage);

                    if monster.health == 0 {
                        let msg = format!("{} defeated!", monster_name);
                        log(&msg);
                        (*self_ptr).increment_stats(DungeonStatKind::Defeated(monster.kind), 1);
                    }
                }
            }

            // If all monsters are defeated, spawn a treasure
            if self.monsters.iter().all(|m| m.health == 0) {
                match os::server::random_number::<u8>() % 4 {
                    3 | 2 => {
                        self.treasures.push(Treasure {
                            x: new_x,
                            y: new_y,
                            value: 2,
                            kind: TreasureKind::HealthUp,
                        });
                    }
                    1 => {
                        self.treasures.push(Treasure {
                            x: new_x,
                            y: new_y,
                            value: 50,
                            kind: TreasureKind::Gold,
                        });
                    }
                    _ => {
                        self.treasures.push(Treasure {
                            x: new_x,
                            y: new_y,
                            value: 1,
                            kind: TreasureKind::Heal,
                        });
                    }
                }
            }
            return true; // Player doesn't move into the monster's position
        }

        // Player moved
        let msg = format!("P1 moved {direction:?}.");
        log(&msg);

        self.player.x = new_x;
        self.player.y = new_y;
        self.player.direction = direction;
        self.increment_stats(DungeonStatKind::StepsMoved, 1);

        // Player collected treasure
        if self.is_treasure(new_x, new_y) {
            if let Some(treasure) = self.treasures.iter().find(|m| m.x == new_x && m.y == new_y) {
                let amount = treasure.value;
                match treasure.kind {
                    TreasureKind::Gold => {
                        self.player.gold += amount;
                        self.increment_stats(DungeonStatKind::GoldCollected, amount);
                        let msg = format!("Got treasure! +${amount}");
                        log(&msg);
                    }
                    TreasureKind::Heal => {
                        let prev_player_health = self.player.health;
                        self.player.health =
                            (self.player.health + amount).min(self.player.max_health);
                        let recovered_health = prev_player_health.abs_diff(self.player.health);
                        self.increment_stats(DungeonStatKind::HealthRecovered, recovered_health);
                        let msg = format!("Recovered {} HP!", recovered_health);
                        log(&msg);
                    }
                    TreasureKind::HealthUp => {
                        self.player.max_health += 1;
                        let prev_player_health = self.player.health;
                        self.player.health =
                            (self.player.health + amount).min(self.player.max_health);
                        let recovered_health = prev_player_health.abs_diff(self.player.health);
                        self.increment_stats(DungeonStatKind::HealthRecovered, recovered_health);
                        let msg = format!("Health Up! Recovered {} HP!", recovered_health);
                        log(&msg);
                    }
                }
            }
            self.treasures.retain_mut(|t| t.x != new_x || t.y != new_y);
        }

        if self.is_exit_key(new_x, new_y) {
            let msg = "Found exit key.".to_string();
            log(&msg);
            self.exit_key = None;
            let (max_x, max_y) = self.bounds();
            // Initialize exit position at least 8 tiles away from player
            let min_distance = (self.width.min(self.height) / 2) as i32;
            loop {
                let x = os::server::random_number::<i32>().abs() % max_x;
                let y = os::server::random_number::<i32>().abs() % max_y;
                let dx = (x - new_x).abs();
                let dy = (y - new_y).abs();
                if dx + dy >= min_distance && !self.is_position_occupied(x, y) {
                    self.exit = Some((x, y));
                    break;
                }
            }
            let msg = "Hidden stairs appeared!".to_string();
            log(&msg);
        }

        return true;
    }
    pub fn move_monsters(&mut self, log: fn(&str)) {
        let mut player = self.player.clone();
        let mut monsters = self.monsters.clone();
        let mut n = 0;

        monsters.retain_mut(|monster| {
            n += 1;
            let i = n - 1;
            let (mx, my) = (monster.x, monster.y);

            // Skip dead monsters (but leave them in the dungeon)
            if monster.health == 0 {
                return true;
            }
            // Killed mid-loop during another monster action
            if self.monsters[i].health == 0 {
                return true;
            }

            // Skip stunned monsters
            if monster.stun_dur > 0 {
                monster.stun_dur = monster.stun_dur.saturating_sub(1);
                self.monsters[i] = monster.clone();
                return true;
            }

            // If the monster is adjacent to the player, it attacks
            if (mx - player.x).abs() + (my - player.y).abs() == 1 {
                let monster_name = monster.kind.abbrev();
                let msg = format!("{} attacks!", monster_name);
                log(&msg);
                if self.is_player(mx, my - 1) {
                    monster.direction = Direction::Up;
                }
                if self.is_player(mx, my + 1) {
                    monster.direction = Direction::Down;
                }
                if self.is_player(mx - 1, my) {
                    monster.direction = Direction::Left;
                }
                if self.is_player(mx + 1, my) {
                    monster.direction = Direction::Right;
                }
                let prev_player_health = player.health;
                player.health = player.health.saturating_sub(match monster.kind {
                    MonsterKind::IceYeti => {
                        // ice yeti has 50% chance to crit
                        if os::server::random_number::<usize>() % 2 == 0 {
                            monster.strength
                        } else {
                            monster.strength * 2
                        }
                    }
                    _ => monster.strength,
                });
                let damage = prev_player_health.abs_diff(player.health);
                self.increment_stats(DungeonStatKind::DamageTaken, damage);

                let msg = format!("{} did {} damage.", monster_name, damage);
                log(&msg);
                if player.health == 0 {
                    let msg = "P1 died.".to_string();
                    log(&msg);
                    self.increment_stats(DungeonStatKind::DefeatedBy(monster.kind), 1);
                }
                return true;
            }

            // Movement based on monster kind
            let k = if monster.kind == MonsterKind::EvilTurbi {
                MonsterKind::by_index(os::server::random_number())
            } else {
                monster.kind
            };
            let (dir, mx, my) = match k {
                MonsterKind::BlueBlob | MonsterKind::YellowBlob | MonsterKind::RedBlob => {
                    let dx = player.x - mx;
                    let dy = player.y - my;

                    // When player is 2 or fewer spaces away, chase them
                    if dx.abs() <= 2 && dy.abs() <= 2 {
                        let (dir, mx, my) = match (dx.abs() > dy.abs(), dx > 0, dy > 0) {
                            (false, _, false) => (Direction::Up, mx, my - 1),
                            (false, _, true) => (Direction::Down, mx, my + 1),
                            (true, false, _) => (Direction::Left, mx - 1, my),
                            (true, true, _) => (Direction::Right, mx + 1, my),
                        };
                        if self.is_position_occupied(mx, my) {
                            return true;
                        }
                        (dir, mx, my)
                    }
                    // Otherwise, move in a random direction
                    else {
                        let (dir, mx, my) = match os::server::random_number::<usize>() % 4 {
                            0 => (Direction::Up, mx, my - 1),
                            1 => (Direction::Down, mx, my + 1),
                            2 => (Direction::Left, mx - 1, my),
                            _ => (Direction::Right, mx + 1, my),
                        };
                        if self.is_position_occupied(mx, my) {
                            return true;
                        }
                        (dir, mx, my)
                    }
                }
                MonsterKind::Spider => {
                    // Moves up to 3 spaces in one direction towards the player every 3 turns
                    if self.turn % 3 != 0 {
                        return true;
                    }

                    let dx = player.x - mx;
                    let dy = player.y - my;

                    // Attempt to move up to 3 spaces towards player
                    let steps = 3.min(dx.abs().max(dy.abs()));

                    let mut new_mx = mx;
                    let mut new_my = my;
                    let mut dir = monster.direction;

                    for s in (1..=steps).rev() {
                        let (dir_next, mx_next, my_next) =
                            match (dx.abs() > dy.abs(), dx > 0, dy > 0) {
                                (false, _, false) => (Direction::Up, mx, my - s),
                                (false, _, true) => (Direction::Down, mx, my + s),
                                (true, false, _) => (Direction::Left, mx - s, my),
                                (true, true, _) => (Direction::Right, mx + s, my),
                            };

                        if !self.is_position_occupied(mx_next, my_next) {
                            new_mx = mx_next;
                            new_my = my_next;
                            dir = dir_next;
                            break;
                        }
                    }

                    (dir, new_mx, new_my)
                }
                // Moves towards the player every other turn
                // Can phase through obstacles
                MonsterKind::Shade => {
                    if self.turn % 2 != 0 {
                        return true;
                    }

                    let dx = player.x - mx;
                    let dy = player.y - my;

                    let steps = 1;

                    let (dir, mx, my) = match (dx.abs() > dy.abs(), dx > 0, dy > 0) {
                        (false, _, false) => (Direction::Up, mx, my - steps),
                        (false, _, true) => (Direction::Down, mx, my + steps),
                        (true, false, _) => (Direction::Left, mx - steps, my),
                        (true, true, _) => (Direction::Right, mx + steps, my),
                    };

                    if self.is_monster(mx, my) {
                        return true;
                    }

                    (dir, mx, my)
                }
                // Ghosts moves away from the player
                // Spectral Ghosts move towards the player if they are nearby
                // Both can phase through obstacles
                // Move towards any adjacent ghosts, then prioritize moving in relation to the player
                MonsterKind::Ghost | MonsterKind::SpectralGhost => {
                    let mut dir = Direction::Down;
                    let mut mx = monster.x;
                    let mut my = monster.y;

                    // First, check for nearby ghosts to absorb
                    let mut did_find_nearby_ghost = false;
                    for dx in -3..=3_i32 {
                        for dy in -3..=3_i32 {
                            if dx == 0 && dy == 0 {
                                continue;
                            }
                            let dir_ = match (dx.abs() > dy.abs(), dx > 0, dy > 0) {
                                (false, _, false) => Direction::Up,
                                (false, _, true) => Direction::Down,
                                (true, false, _) => Direction::Left,
                                (true, true, _) => Direction::Right,
                            };
                            let mx_ = monster.x + dx;
                            let my_ = monster.y + dy;
                            if self.monsters.iter().any(|m| {
                                m.health > 0
                                    && m.kind == MonsterKind::Ghost
                                    && m.x == mx_
                                    && m.y == my_
                            }) {
                                did_find_nearby_ghost = true;
                                // Move one space towards nearby ghost
                                dir = dir_;
                                let dx = dx.checked_div(dx.abs()).unwrap_or(0);
                                if dx != 0 {
                                    mx = monster.x + dx;
                                } else {
                                    let dy = dy.checked_div(dy.abs()).unwrap_or(0);
                                    my = monster.y + dy;
                                }
                                break;
                            }
                        }
                    }

                    // Either run from or move towards the player
                    // let mut did_react_to_player = false;
                    if !did_find_nearby_ghost {
                        let (steps, dist, range) = if monster.kind == MonsterKind::Ghost {
                            if self.turn % 4 == 0 {
                                return true;
                            }
                            (1, 1, 1)
                        } else {
                            if self.turn % 4 == 0 {
                                return true;
                            }
                            (1, 1, 4)
                        };
                        for _ in 0..steps {
                            let dx = player.x - mx;
                            let dy = player.y - my;
                            let dx_abs = dx.abs();
                            let dy_abs = dy.abs();
                            if dx_abs > range && dy_abs > range {
                                continue;
                            }
                            let x_or_y = if dx_abs == dy_abs {
                                os::server::random_number::<i32>().abs() % 2 == 0
                            } else {
                                dx_abs > dy_abs
                            };
                            let (dir_, mx_, my_) = match (x_or_y, dx > 0, dy > 0) {
                                (false, _, false) => (Direction::Up, mx, my - dist),
                                (false, _, true) => (Direction::Down, mx, my + dist),
                                (true, false, _) => (Direction::Left, mx - dist, my),
                                (true, true, _) => (Direction::Right, mx + dist, my),
                            };

                            dir = dir_;
                            mx = mx_;
                            my = my_;
                        }
                    }

                    // Didn't move
                    if monster.x == mx && monster.y == my {
                        return true;
                    }

                    if self.is_player(mx, my) || self.is_exit(mx, my) {
                        return true;
                    }

                    // "Absorb" any non-dead ghost in the same position
                    if let Some(idx) = self.monsters.iter().position(|m| {
                        m.health > 0 && m.x == mx && m.y == my && m.kind == MonsterKind::Ghost
                    }) {
                        // Remove ghost
                        self.monsters[idx].health = 0;
                        // Increase stats and upgrade to Spectral Ghost
                        if monster.kind != MonsterKind::EvilTurbi {
                            monster.kind = MonsterKind::SpectralGhost;
                        }
                        monster.strength *= 2;
                        monster.max_health *= 2;
                        monster.health = monster.max_health;
                    }

                    // Returns false for dead monsters
                    if self.is_monster(mx, my) {
                        return true;
                    }

                    (dir, mx, my)
                }
                // Chase players within range, otherwise, moves randomly every other turn
                MonsterKind::Zombie => {
                    let range = 4;
                    // Moves towards the player each turn
                    let dx = player.x - mx;
                    let dy = player.y - my;
                    let dx_abs = dx.abs();
                    let dy_abs = dy.abs();
                    if dx_abs + dy_abs > range {
                        if self.turn % 2 == 0 {
                            return true;
                        }
                        let (dir, mx, my) = match os::server::random_number::<usize>() % 4 {
                            0 => (Direction::Up, mx, my - 1),
                            1 => (Direction::Down, mx, my + 1),
                            2 => (Direction::Left, mx - 1, my),
                            _ => (Direction::Right, mx + 1, my),
                        };
                        if self.is_position_occupied(mx, my) {
                            return true;
                        }
                        (dir, mx, my)
                    } else {
                        let move_y = || {
                            if dy < 0 {
                                (Direction::Up, mx, my - 1)
                            } else {
                                (Direction::Down, mx, my + 1)
                            }
                        };
                        let move_x = || {
                            if dx < 0 {
                                (Direction::Left, mx - 1, my)
                            } else {
                                (Direction::Right, mx + 1, my)
                            }
                        };
                        let all = if dx.abs() > dy.abs() {
                            [move_x(), move_y()]
                        } else {
                            [move_y(), move_x()]
                        };
                        if let Some(a) = all.iter().find(|a| !self.is_position_occupied(a.1, a.2)) {
                            *a
                        } else {
                            return true;
                        }
                    }
                }
                MonsterKind::IceYeti => {
                    // Move every other turn
                    if self.turn % 2 != 0 {
                        return true;
                    }

                    let dx = player.x - mx;
                    let dy = player.y - my;

                    // Attempt to move up to 2 spaces towards player
                    let steps = 2.min(dx.abs().max(dy.abs()));

                    let mut new_mx = mx;
                    let mut new_my = my;
                    let mut dir = monster.direction;

                    for s in (1..=steps).rev() {
                        let (try_dir, try_x, try_y) = match (dx.abs() > dy.abs(), dx > 0, dy > 0) {
                            (false, _, false) => (Direction::Up, mx, my - s),
                            (false, _, true) => (Direction::Down, mx, my + s),
                            (true, false, _) => (Direction::Left, mx - s, my),
                            (true, true, _) => (Direction::Right, mx + s, my),
                        };

                        if !self.is_position_occupied(try_x, try_y) {
                            new_mx = try_x;
                            new_my = try_y;
                            dir = try_dir;
                            break;
                        }
                    }

                    // If no change occurred, fallback to previous single-step logic
                    if new_mx == mx && new_my == my {
                        let x_move = if dx < 0 {
                            (Direction::Left, mx - 1, my)
                        } else {
                            (Direction::Right, mx + 1, my)
                        };
                        let y_move = if dy < 0 {
                            (Direction::Up, mx, my - 1)
                        } else {
                            (Direction::Down, mx, my + 1)
                        };

                        let moves = if dx.abs() > dy.abs() {
                            [x_move, y_move]
                        } else {
                            [y_move, x_move]
                        };

                        if let Some(next) = moves
                            .iter()
                            .find(|(_, x, y)| !self.is_position_occupied(*x, *y))
                            .copied()
                        {
                            next
                        } else {
                            return true;
                        }
                    } else {
                        (dir, new_mx, new_my)
                    }
                }
                MonsterKind::Snowman => {
                    // Move every other turn
                    if self.turn % 2 != 0 {
                        return true;
                    }

                    // Moves towards the exit, the stairs, or the player
                    let mut next = (monster.direction, mx, my);
                    for (tx, ty) in [
                        self.exit.unwrap_or((player.x, player.y)),
                        self.exit_key.unwrap_or((player.x, player.y)),
                        (player.x, player.y),
                    ] {
                        let dx = tx - mx;
                        let dy = ty - my;

                        let x = if dx < 0 {
                            (Direction::Left, mx - 1, my)
                        } else {
                            (Direction::Right, mx + 1, my)
                        };
                        let y = if dy < 0 {
                            (Direction::Up, mx, my - 1)
                        } else {
                            (Direction::Down, mx, my + 1)
                        };

                        let moves = if dx.abs() > dy.abs() { [x, y] } else { [y, x] };

                        let Some((d, x, y)) = moves
                            .iter()
                            .find(|(_, x, y)| !self.is_position_occupied(*x, *y))
                            .copied()
                        else {
                            continue;
                        };
                        next = (d, x, y);
                    }
                    next
                }
                _ => {
                    // Moves towards the player each turn
                    let dx = player.x - mx;
                    let dy = player.y - my;

                    let x = if dx < 0 {
                        (Direction::Left, mx - 1, my)
                    } else {
                        (Direction::Right, mx + 1, my)
                    };
                    let y = if dy < 0 {
                        (Direction::Up, mx, my - 1)
                    } else {
                        (Direction::Down, mx, my + 1)
                    };

                    let moves = if dx.abs() > dy.abs() { [x, y] } else { [y, x] };

                    let Some(next) = moves
                        .iter()
                        .find(|(_, x, y)| !self.is_position_occupied(*x, *y))
                        .copied()
                    else {
                        return true;
                    };
                    next
                }
            };

            if self.is_out_of_bounds(mx, my) {
                return true;
            }

            monster.x = mx;
            monster.y = my;
            monster.direction = dir;
            self.monsters[i] = monster.clone();

            return true;
        });

        self.player = player;
    }
    pub fn is_player(&self, x: i32, y: i32) -> bool {
        self.player.x == x && self.player.y == y
    }
    pub fn bounds(&self) -> (i32, i32) {
        let max_x = self.width as i32 - 1;
        let max_y = self.height as i32 - 1;
        (max_x, max_y)
    }
    pub fn is_out_of_bounds(&self, x: i32, y: i32) -> bool {
        let (min_x, min_y) = (0, 0);
        let (max_x, max_y) = self.bounds();
        x < min_x || y < min_y || x > max_x || y > max_y
    }
    pub fn is_obstacle(&self, x: i32, y: i32) -> bool {
        self.obstacles.iter().any(|obs| obs.x == x && obs.y == y)
    }
    pub fn is_monster(&self, x: i32, y: i32) -> bool {
        self.monsters
            .iter()
            .any(|mon| mon.x == x && mon.y == y && mon.health > 0)
    }
    pub fn is_treasure(&self, x: i32, y: i32) -> bool {
        self.treasures.iter().any(|t| t.x == x && t.y == y)
    }
    pub fn is_exit_key(&self, x: i32, y: i32) -> bool {
        self.exit_key.map_or(false, |a| a.0 == x && a.1 == y)
    }
    pub fn is_exit(&self, x: i32, y: i32) -> bool {
        self.exit.map_or(false, |a| a.0 == x && a.1 == y)
    }
    pub fn is_position_blocked(&self, x: i32, y: i32) -> bool {
        self.is_obstacle(x, y) || self.is_monster(x, y) || self.is_player(x, y)
    }
    pub fn is_position_occupied(&self, x: i32, y: i32) -> bool {
        self.is_position_blocked(x, y)
            || self.is_treasure(x, y)
            || self.is_exit_key(x, y)
            || self.is_exit(x, y)
    }
    pub fn increment_stats(&mut self, kind: DungeonStatKind, amount: u32) {
        if amount > 0 {
            self.stats.increment(kind, amount);
            self.total_stats.increment(kind, amount);
        }
    }
}
