use super::*;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct MultiplayerDungeonLobby {
    pub id: u32,
    pub created_at: u32,
    pub players: BTreeSet<String>,
}
impl MultiplayerDungeonLobby {
    pub fn new(id: u32, created_at: u32, owner: &str) -> Self {
        Self {
            id,
            created_at,
            players: vec![owner.to_string()].into_iter().collect(),
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct PlayerContext {
    pub player: Player,
    pub stats: DungeonStats,
    pub total_stats: DungeonStats,
    pub unlocked: PlayerAchievements,
    pub all_unlocked: PlayerAchievements,
    pub next_round: u32,
}
impl PlayerContext {
    pub fn increment_stats(&mut self, kind: DungeonStatKind, amount: u32) {
        if amount > 0 {
            self.stats.increment(kind, amount);
            self.total_stats.increment(kind, amount);
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct PlayerState {
    pub players: BTreeMap<String, PlayerContext>,
}
impl PlayerState {
    pub fn new() -> Self {
        Self {
            players: BTreeMap::new(),
        }
    }
    pub fn get(&self, player_id: &str) -> Option<PlayerContext> {
        self.players.get(player_id).cloned()
    }
    pub fn get_index(&self, player_id: &str) -> Option<usize> {
        self.players.keys().position(|user_id| user_id == player_id)
    }
    // Modify a player’s context without holding a mutable borrow on the whole map
    pub fn modify_player<F>(&mut self, player_id: &str, modify_fn: F) -> Result<(), &str>
    where
        F: FnOnce(&mut PlayerContext),
    {
        // Temporarily take the player context out of the map
        if let Some(mut player_ctx) = self.players.remove(player_id) {
            modify_fn(&mut player_ctx); // Modify the context
            self.players.insert(player_id.to_string(), player_ctx); // Put it back
            Ok(())
        } else {
            Err("Player not found")
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct MultiplayerDungeon {
    pub owner: String,
    pub crawl_id: u32,
    pub theme: DungeonThemeKind,
    pub floor: u32,
    pub round: u32,
    pub turn: u32,
    pub width: u32,
    pub height: u32,
    pub monsters: Vec<Monster>,
    pub treasures: Vec<Treasure>,
    pub obstacles: Vec<Obstacle>,
    pub exit_key: Option<(i32, i32)>,
    pub exit: Option<(i32, i32)>,
    pub player: PlayerState,
}
impl MultiplayerDungeon {
    pub fn move_player(&mut self, user_id: &str, direction: Direction) -> Result<(), &str> {
        unsafe {
            // get mut ptr
            let self_ptr = self as *mut Self;

            // Get player
            let Some(ctx) = self.player.players.get(user_id) else {
                os::server::log!("Player is not in this dungeon");
                return Err("Player is not in this dungeon");
            };

            // Ensure player is still alive
            if ctx.player.health == 0 {
                os::server::log!("Player is dead");
                return Err("Player is dead");
            }

            // Ensure player can move this round
            if ctx.next_round != self.round {
                os::server::log!("Player cannot move until round {}", ctx.next_round);
                return Err("Player cannot move this turn");
            }

            // Get next position based on movement direction
            let Player { x, y, .. } = ctx.player;
            let (new_x, new_y) = match direction {
                Direction::Up => (x, y - 1),
                Direction::Down => (x, y + 1),
                Direction::Left => (x - 1, y),
                Direction::Right => (x + 1, y),
            };

            // Ensure player is in bounds
            if self.is_out_of_bounds(new_x, new_y) {
                os::server::log!("Player cannot move out-of-bounds");
                return Err("Player cannot move out-of-bounds");
            }

            // Ensure player is not blocked by an obstacle
            if self.is_obstacle(new_x, new_y) {
                os::server::log!("Player is blocked by an obstacle");
                return Err("Player is blocked by an obstacle");
            }

            // Ensure player is not blocked by another player
            if self.is_player(new_x, new_y) {
                os::server::log!("Player is blocked by another player");
                return Err("Player is blocked by another player");
            }

            // If player moves towards an adjacent monster...
            if self.is_monster(new_x, new_y) {
                // Find the monster
                if let Some(monster) = self
                    .monsters
                    .iter_mut()
                    .find(|m| m.x == new_x && m.y == new_y && m.health > 0)
                {
                    // Swap positions with the stunned monsters
                    if monster.stun_dur > 0 {
                        os::server::log!("Player swapped positions with {:?}", monster.kind);
                        return self.player.modify_player(user_id, |ctx| {
                            std::mem::swap(&mut ctx.player.x, &mut monster.x);
                            std::mem::swap(&mut ctx.player.y, &mut monster.y);
                            ctx.next_round += 1;
                        });
                    }

                    // Calculate and apply damage to monster
                    os::server::log!("Player attacks {:?}!", monster.kind);
                    let amount = ctx.player.strength;
                    let prev_monster_health = monster.health;
                    monster.health = monster.health.saturating_sub(amount);
                    let damage = prev_monster_health.abs_diff(monster.health);
                    os::server::log!("Player did {damage} damage");
                    self.player.modify_player(user_id, |ctx| {
                        ctx.increment_stats(DungeonStatKind::DamageDealt, damage);
                    })?;

                    // Apply stun to monster
                    monster.stun_dur = 1;

                    // If monster was defeated...
                    if monster.health == 0 {
                        os::server::log!("Player defeated {:?}", monster.kind);
                        (*self_ptr).player.modify_player(user_id, |ctx| {
                            ctx.increment_stats(DungeonStatKind::Defeated(monster.kind), 1);
                        })?;
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
                return (*self_ptr).player.modify_player(user_id, |ctx| {
                    ctx.next_round += 1;
                });
            }

            // Player moved
            os::server::log!("Player moved {direction:?}.");
            (*self_ptr).player.modify_player(user_id, |ctx| {
                ctx.player.x = new_x;
                ctx.player.y = new_y;
                ctx.player.direction = direction;
                ctx.increment_stats(DungeonStatKind::StepsMoved, 1);
            })?;

            // Player collected treasure
            if self.is_treasure(new_x, new_y) {
                if let Some(treasure) = self.treasures.iter().find(|m| m.x == new_x && m.y == new_y)
                {
                    let amount = treasure.value;
                    match treasure.kind {
                        TreasureKind::Gold => {
                            os::server::log!("Got treasure! +${amount}");
                            (*self_ptr).player.modify_player(user_id, |ctx| {
                                ctx.player.gold += amount;
                                ctx.increment_stats(DungeonStatKind::GoldCollected, amount);
                            })?;
                        }
                        TreasureKind::Heal => {
                            (*self_ptr).player.modify_player(user_id, |ctx| {
                                let prev_player_health = ctx.player.health;
                                ctx.player.health =
                                    (ctx.player.health + amount).min(ctx.player.max_health);
                                let recovered_health =
                                    prev_player_health.abs_diff(ctx.player.health);
                                os::server::log!("Recovered {} HP!", recovered_health);
                                ctx.increment_stats(
                                    DungeonStatKind::HealthRecovered,
                                    recovered_health,
                                );
                            })?;
                        }
                        TreasureKind::HealthUp => {
                            (*self_ptr).player.modify_player(user_id, |ctx| {
                                ctx.player.max_health += 1;
                                let prev_player_health = ctx.player.health;
                                ctx.player.health =
                                    (ctx.player.health + amount).min(ctx.player.max_health);
                                let recovered_health =
                                    prev_player_health.abs_diff(ctx.player.health);
                                ctx.increment_stats(
                                    DungeonStatKind::HealthRecovered,
                                    recovered_health,
                                );
                                os::server::log!("Health Up! Recovered {} HP!", recovered_health);
                            })?;
                        }
                    }
                }
                self.treasures.retain_mut(|t| t.x != new_x || t.y != new_y);
            }

            // Check if player on an exit
            if self.is_exit_key(new_x, new_y) {
                os::server::log!("Found exit key.");
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
                os::server::log!("Hidden stairs appeared!");
            }

            os::server::log!("Player moved {direction:?}.");
            return (*self_ptr).player.modify_player(user_id, |ctx| {
                ctx.next_round += 1;
            });
        }
    }
    pub fn move_all_monsters(&mut self) -> Result<(), &str> {
        unsafe {
            // get mut ptr
            let self_ptr = self as *mut Self;
            let player_ctx = self.player.clone();
            let mut monsters = self.monsters.clone();

            // Move each monster
            for (i, monster) in monsters.iter_mut().enumerate() {
                // Get monster position
                let (mx, my) = (monster.x, monster.y);

                // Skip dead monsters (but leave them in the dungeon)
                if monster.health == 0 {
                    continue;
                }

                // Killed mid-loop during another monster action
                if self.monsters[i].health == 0 {
                    continue;
                }

                // Skip stunned monsters
                if monster.stun_dur > 0 {
                    os::server::log!("{:?} is stunned", monster.kind);
                    monster.stun_dur = monster.stun_dur.saturating_sub(1);
                    self.monsters[i] = monster.clone();
                    continue;
                }

                // Check for adjacent players
                os::server::log!("{:?} is checking for players to attack...", monster.kind);
                let mut did_attack_player = false;
                for (user_id, ctx) in player_ctx.players.iter() {
                    if ctx.player.health == 0 {
                        continue;
                    }
                    // If the monster is adjacent to the player, it attacks
                    if (mx - ctx.player.x).abs() + (my - ctx.player.y).abs() == 1 {
                        os::server::log!("{:?} attacks!", monster.kind);
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
                        // Apply damage
                        (*self_ptr).player.modify_player(&user_id, |ctx| {
                            let prev_player_health = ctx.player.health;
                            ctx.player.health = ctx.player.health.saturating_sub(monster.strength);
                            let damage = prev_player_health.abs_diff(ctx.player.health);
                            ctx.increment_stats(DungeonStatKind::DamageTaken, damage);
                            os::server::log!("{:?} did {} damage.", monster.kind, damage);

                            if ctx.player.health == 0 {
                                os::server::log!("{user_id} died");
                                ctx.increment_stats(DungeonStatKind::DefeatedBy(monster.kind), 1);
                            }
                        })?;
                        did_attack_player = true;
                        break;
                    }
                }
                if did_attack_player {
                    continue;
                }

                // Find the closest player
                os::server::log!("{:?} is trying to move", monster.kind);
                let Some((user_id, ctx)) = player_ctx.players.iter().min_by_key(|(_, ctx)| {
                    if ctx.player.health == 0 {
                        return i32::MAX;
                    }
                    let dx = ctx.player.x - mx;
                    let dy = ctx.player.y - my;
                    dx * dx + dy * dy // Squared distance for comparison
                }) else {
                    os::server::log!("Could not find closest player");
                    continue;
                };
                os::server::log!("The closest player to {:?} is {}", monster.kind, user_id);

                // Movement based on monster kind
                let k: MonsterKind = match monster.kind {
                    MonsterKind::EvilTurbi => {
                        let kind = MonsterKind::by_index(os::server::random_number());
                        os::server::log!("{:?} is feeling like a {:?}", monster.kind, kind);
                        kind
                    }
                    _ => monster.kind,
                };
                let (dir, mx, my) = match k {
                    MonsterKind::BlueBlob | MonsterKind::YellowBlob | MonsterKind::RedBlob => {
                        // Get distance from player
                        let dx = ctx.player.x - mx;
                        let dy = ctx.player.y - my;

                        // When player is 2 or fewer spaces away, chase them
                        if dx.abs() <= 2 && dy.abs() <= 2 {
                            let (dir, mx, my) = match (dx.abs() > dy.abs(), dx > 0, dy > 0) {
                                (false, _, false) => (Direction::Up, mx, my - 1),
                                (false, _, true) => (Direction::Down, mx, my + 1),
                                (true, false, _) => (Direction::Left, mx - 1, my),
                                (true, true, _) => (Direction::Right, mx + 1, my),
                            };
                            if self.is_position_occupied(mx, my) {
                                os::server::log!("{:?} is blocked", monster.kind);
                                continue;
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
                                os::server::log!("{:?} is blocked", monster.kind);
                                continue;
                            }
                            (dir, mx, my)
                        }
                    }
                    MonsterKind::Spider => {
                        // Move every 3 rounds
                        if self.round % 3 != 0 {
                            os::server::log!("{:?} is resting", monster.kind);
                            continue;
                        }

                        // Get distance from player
                        let dx = ctx.player.x - mx;
                        let dy = ctx.player.y - my;

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
                    MonsterKind::Shade => {
                        // Move every other round
                        if self.round % 2 != 0 {
                            os::server::log!("{:?} is resting", monster.kind);
                            continue;
                        }

                        // Get distance from player
                        let dx = ctx.player.x - mx;
                        let dy = ctx.player.y - my;

                        // Move one space
                        let steps = 1;

                        // Calculate next position
                        let (dir, mx, my) = match (dx.abs() > dy.abs(), dx > 0, dy > 0) {
                            (false, _, false) => (Direction::Up, mx, my - steps),
                            (false, _, true) => (Direction::Down, mx, my + steps),
                            (true, false, _) => (Direction::Left, mx - steps, my),
                            (true, true, _) => (Direction::Right, mx + steps, my),
                        };

                        // Check for blocks
                        if self.is_monster(mx, my) {
                            os::server::log!("{:?} is blocked by another monster", monster.kind);
                            continue;
                        }

                        (dir, mx, my)
                    }
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
                        if !did_find_nearby_ghost {
                            let (steps, dist, range) = if monster.kind == MonsterKind::Ghost {
                                // Move every 4 rounds
                                if self.round % 4 == 0 {
                                    os::server::log!("{:?} is resting", monster.kind);
                                    continue;
                                }
                                (1, 1, 1)
                            } else {
                                // Move every 4 rounds
                                if self.round % 4 == 0 {
                                    os::server::log!("{:?} is resting", monster.kind);
                                    continue;
                                }
                                (1, 1, 4)
                            };
                            for _ in 0..steps {
                                let dx = ctx.player.x - mx;
                                let dy = ctx.player.y - my;
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
                            os::server::log!("{:?} has nowhere to move", monster.kind);
                            continue;
                        }

                        // Check if the monster is blocked by a player
                        if self.is_player(mx, my) {
                            os::server::log!("{:?} is blocked by a player", monster.kind);
                            continue;
                        }

                        // Check if the monster is blocked by an exit
                        if self.is_exit(mx, my) {
                            os::server::log!("{:?} is blocked by an exit", monster.kind);
                            continue;
                        }

                        // "Absorb" any non-dead ghost in the same position
                        if let Some(idx) = self.monsters.iter().position(|m| {
                            m.health > 0 && m.x == mx && m.y == my && m.kind == MonsterKind::Ghost
                        }) {
                            os::server::log!("{:?} is absorbing a ghost!", monster.kind);
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
                            os::server::log!("{:?} is blocked by a monster", monster.kind);
                            continue;
                        }

                        (dir, mx, my)
                    }
                    MonsterKind::Zombie => {
                        let range = 4;
                        // Moves towards the player each turn
                        let dx = ctx.player.x - mx;
                        let dy = ctx.player.y - my;
                        let dx_abs = dx.abs();
                        let dy_abs = dy.abs();
                        if dx_abs + dy_abs > range {
                            os::server::log!("{:?} is moving randomly", monster.kind);
                            if self.round % 2 == 0 {
                                os::server::log!("{:?} is resting", monster.kind);
                                continue;
                            }
                            let (dir, mx, my) = match os::server::random_number::<usize>() % 4 {
                                0 => (Direction::Up, mx, my - 1),
                                1 => (Direction::Down, mx, my + 1),
                                2 => (Direction::Left, mx - 1, my),
                                _ => (Direction::Right, mx + 1, my),
                            };
                            if self.is_position_occupied(mx, my) {
                                os::server::log!("{:?} is blocked", monster.kind);
                                continue;
                            }
                            (dir, mx, my)
                        } else {
                            os::server::log!("{:?} is chasing a player", monster.kind);
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
                            // Determine whether to try moving L/R or U/D first
                            let all = if dx.abs() > dy.abs() {
                                [move_x(), move_y()]
                            } else {
                                [move_y(), move_x()]
                            };
                            // Find the first unoccupied position
                            let Some((dir, mx, my)) = all
                                .into_iter()
                                .find(|a| !self.is_position_occupied(a.1, a.2))
                            else {
                                os::server::log!("{:?} is blocked", monster.kind);
                                continue;
                            };

                            (dir, mx, my)
                        }
                    }
                    MonsterKind::IceYeti => {
                        // Move every other turn
                        if self.turn % 2 != 0 {
                            continue;
                        }

                        let dx = ctx.player.x - mx;
                        let dy = ctx.player.y - my;

                        // Attempt to move up to 2 spaces towards player
                        let steps = 2.min(dx.abs().max(dy.abs()));

                        let mut new_mx = mx;
                        let mut new_my = my;
                        let mut dir = monster.direction;

                        for s in (1..=steps).rev() {
                            let (try_dir, try_x, try_y) =
                                match (dx.abs() > dy.abs(), dx > 0, dy > 0) {
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
                                continue;
                            }
                        } else {
                            (dir, new_mx, new_my)
                        }
                    }
                    MonsterKind::Snowman => {
                        // Move every other turn
                        if self.turn % 2 != 0 {
                            continue;
                        }

                        // Moves towards the exit, the stairs, or the player
                        let mut next = (monster.direction, mx, my);
                        for (tx, ty) in [
                            self.exit.unwrap_or((ctx.player.x, ctx.player.y)),
                            self.exit_key.unwrap_or((ctx.player.x, ctx.player.y)),
                            (ctx.player.x, ctx.player.y),
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
                        let dx = ctx.player.x - mx;
                        let dy = ctx.player.y - my;

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
                            continue;
                        };
                        next
                    }
                };

                if self.is_out_of_bounds(mx, my) {
                    os::server::log!("{:?} cannot move out-of-bounds", monster.kind);
                    continue;
                }

                monster.x = mx;
                monster.y = my;
                monster.direction = dir;
                self.monsters[i] = monster.clone();
            }
            Ok(())
        }
    }
    pub fn did_all_players_move(&self) -> bool {
        self.player
            .players
            .values()
            .all(|ctx| ctx.player.health == 0 || (ctx.next_round > self.round))
    }
    pub fn did_all_players_die(&self) -> bool {
        self.player
            .players
            .values()
            .all(|ctx| ctx.player.health == 0)
    }
    pub fn is_player(&self, x: i32, y: i32) -> bool {
        self.player
            .players
            .values()
            .any(|ctx| ctx.player.x == x && ctx.player.y == y)
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
}
