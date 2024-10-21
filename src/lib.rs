use borsh::{BorshDeserialize, BorshSerialize};
use std::collections::{BTreeMap, BTreeSet};

turbo::cfg! {r#"
    name = "Dungeon Dash"
    [settings]
    resolution = [132, 224]
    # resolution = [144, 256]
    [turbo-os]
    # api-url = "https://os.turbo.computer"
    api-url = "http://localhost:8000"
"#}

turbo::init! {
    struct LocalState {
        floor: Tween<u32>,
        turn: Tween<u32>,
        last_exec_at: usize,
        last_exec_turn: Option<u32>,
        player: struct Entity {
            x: Tween<i32>,
            y: Tween<i32>,
            offset_x: Tween<i32>,
            offset_y: Tween<i32>,
        },
        monsters: Vec<Entity>,
        leaderboard_kind: LeaderboardKind,
        particles: Vec<struct Particle {
            x: f32,
            y: f32,
            vx: f32,
            vy: f32,
            lifetime: u32,
            color: u32,
        }>,
        clouds: Vec<struct Cloud {
            x: f32,
            y: f32,
            radius: f32,
            speed: f32,
            color: u32,
        }>,
        raindrops: Vec<struct Raindrop {
            x: f32,
            y: f32,
            vel: f32,
            length: f32,
        }>,
        achievements_modal: Option<struct AchievementsModal {
            y: Tween<i32>,
            current: usize,
            kinds: Vec<AchievementKind>,
            confetti: Vec<struct Confetti {
                x: f32,
                y: f32,
                radius: f32,
                color: u32,
                vy: f32,
            }>,
        }>,
        last_crawl_achievements_modal: u32,
    } = {
        Self {
            floor: Tween::new(0).duration(FLOOR_DUR),
            turn: Tween::new(0).duration(TURN_DUR),
            last_exec_at: 0,
            last_exec_turn: None,
            player: Entity {
                x: Tween::new(0).duration(MOVE_DUR).ease(Easing::EaseInOutQuad),
                y: Tween::new(0).duration(MOVE_DUR).ease(Easing::EaseInOutQuad),
                offset_x: Tween::new(0).duration(MOVE_DUR / 2).ease(Easing::EaseInOutQuad),
                offset_y: Tween::new(0).duration(MOVE_DUR / 2).ease(Easing::EaseInOutQuad),
            },
            monsters: vec![],
            leaderboard_kind: LeaderboardKind::HighestFloor,
            particles: vec![],
            clouds: vec![],
            raindrops: vec![],
            achievements_modal: None,
            last_crawl_achievements_modal: 0,
        }
    }
}

impl Entity {
    fn is_idle(&mut self) -> bool {
        let is_x_done = self.x.done();
        let is_y_done = self.y.done();
        let is_offset_x_done = self.offset_x.done();
        let is_offset_y_done = self.offset_y.done();
        is_x_done && is_y_done && is_offset_x_done && is_offset_y_done
    }
}

const TILE_SIZE: i32 = 16;
const TURN_DUR: usize = 20;
const FLOOR_DUR: usize = 32;
const MOVE_DUR: usize = 10;
const MODAL_TRANS_DUR: usize = 48;
const MOVE_Y_OFFSET: i32 = 6;
const MOVE_X_OFFSET: i32 = 6;
const EXEC_TIMEOUT_DUR: usize = 32;
const SHADOW_COLOR: u32 = 0x000000dd;

turbo::go!({
    // Load the game state
    let mut state = LocalState::load();

    // Clear the screen
    clear(0x0e071bff);
    // clear(0x1b1126ff);

    let [w, h] = canvas_size!();
    sprite!(
        // "dotted_tile_border",
        "skull_pattern",
        w = w,
        h = h,
        tx = (tick() / 4) % w as usize,
        ty = (tick() / 4) % h as usize,
        repeat = true,
        absolute = true,
    );

    // Load dungeon
    let user_id = os::user_id();
    // log!("USER ID {:?}", user_id.clone());

    let dungeon = user_id
        .ok_or_else(|| "Not logged in".to_string())
        .and_then(|user_id| {
            DungeonDashProgram::fetch_player_dungeon(&user_id).map_err(|err| err.to_string())
        });
    // log!("DUNGEON {:?}", dungeon);

    if let Ok(dungeon) = &dungeon {
        // Size constants
        let [w, h] = canvas_size!();
        let menubar_h = 40;
        let dungeon_w = dungeon.width * TILE_SIZE as u32;
        let dungeon_h = dungeon.height * TILE_SIZE as u32;

        // Get the theme info
        let dungeon_theme = dungeon.theme.theme();

        // Draw undulating clouds on the edges
        let edge_count = 32;
        let circle_radius = 16.0;
        let angle_offset = (tick() as f32) * 0.05;

        // Draw top edge
        for i in 0..edge_count {
            let progress = i as f32 / edge_count as f32;
            let x = progress * (dungeon_w as f32);
            let y = (angle_offset + progress * PI * 2.0).sin() * 5.0;
            circ!(
                x = x - circle_radius,
                y = y - circle_radius,
                d = circle_radius * 2.0,
                color = dungeon_theme.mist_color,
                // absolute = true
            );
        }
        // Draw bottom edge
        for i in 0..edge_count {
            let progress = i as f32 / edge_count as f32;
            let x = progress * (dungeon_w as f32);
            let y = (dungeon_h as f32) - ((angle_offset + progress * PI * 2.0).sin() * 5.0);
            circ!(
                x = x - circle_radius,
                y = y - circle_radius * 0.5,
                d = circle_radius * 2.0,
                color = dungeon_theme.mist_color,
                // absolute = true
            );
        }
        // Draw left edge
        for i in 0..edge_count {
            let progress = i as f32 / edge_count as f32;
            let y = progress * (dungeon_h as f32);
            let x = (angle_offset + progress * PI * 2.0).cos() * 5.0;
            circ!(
                x = x - circle_radius * 1.5,
                y = y - circle_radius * 0.5,
                d = circle_radius * 2.0,
                color = dungeon_theme.mist_color,
                // absolute = true
            );
        }
        // Draw right edge
        for i in 0..edge_count {
            let progress = i as f32 / edge_count as f32;
            let y = progress * (dungeon_h as f32);
            let x = (dungeon_w as f32) - ((angle_offset + progress * PI * 2.0).cos() * 5.0);
            circ!(
                x = x - circle_radius * 0.5,
                y = y - circle_radius * 0.5,
                d = circle_radius * 2.0,
                color = dungeon_theme.mist_color,
                // absolute = true
            );
        }

        // Update achievements modal
        if dungeon.player.health == 0
            && state.last_crawl_achievements_modal != dungeon.crawl_id
            && state.achievements_modal.is_none()
            && !dungeon.unlocked.is_empty()
        {
            let modal = AchievementsModal::new(&dungeon.unlocked.achievement_kinds());
            state.achievements_modal = Some(modal);
            state.last_crawl_achievements_modal = dungeon.crawl_id;
        }

        // Update turn
        state.turn.set(dungeon.turn);

        // Update player tweens
        state.player.x.set(dungeon.player.x * TILE_SIZE);
        state.player.y.set(dungeon.player.y * TILE_SIZE);

        // Player "nudge" animation
        if (!state.player.y.done() || !state.player.x.done()) && state.player.offset_y.done() {
            state.player.offset_y.set(-MOVE_Y_OFFSET);
        }
        if state.player.offset_x.done() && state.player.offset_x.get() != 0 {
            state.player.offset_x.set(0);
        }
        if state.player.offset_y.done() && state.player.offset_y.get() != 0 {
            state.player.offset_y.set(0);
        }

        // Update monster tweens
        if state.floor.get() != dungeon.floor || state.monsters.len() != dungeon.monsters.len() {
            state.floor.set(dungeon.floor);
            state.monsters.clear();
            for monster in &dungeon.monsters {
                state.monsters.push(Entity {
                    x: Tween::new(monster.x * TILE_SIZE)
                        .duration(MOVE_DUR)
                        .ease(Easing::EaseOutSine),
                    y: Tween::new(monster.y * TILE_SIZE)
                        .duration(MOVE_DUR)
                        .ease(Easing::EaseOutSine),
                    offset_x: Tween::new(0)
                        .duration(MOVE_DUR / 2)
                        .ease(Easing::EaseInOutQuad),
                    offset_y: Tween::new(0)
                        .duration(MOVE_DUR / 2)
                        .ease(Easing::EaseInOutQuad),
                })
            }
        }
        if state.player.is_idle() {
            for (monster, entity) in dungeon
                .monsters
                .iter()
                .zip(state.monsters.iter_mut())
                .collect::<Vec<(_, _)>>()
            {
                entity.x.set(monster.x * TILE_SIZE);
                entity.y.set(monster.y * TILE_SIZE);

                // Monster "nudge" animation
                if !state.turn.done() && entity.x.done() && entity.y.done() {
                    let is_player_on_exit =
                        dungeon.exit.unwrap_or((-1, -1)) == (dungeon.player.x, dungeon.player.y);
                    let is_monster_stunned = monster.stun_dur > 0;
                    if !is_player_on_exit && !is_monster_stunned {
                        match monster.direction {
                            Direction::Up => {
                                if dungeon.is_player(monster.x, monster.y - 1) {
                                    entity.offset_y.set(-MOVE_Y_OFFSET);
                                }
                            }
                            Direction::Down => {
                                if dungeon.is_player(monster.x, monster.y + 1) {
                                    entity.offset_y.set(MOVE_Y_OFFSET);
                                }
                            }
                            Direction::Left => {
                                if dungeon.is_player(monster.x - 1, monster.y) {
                                    entity.offset_x.set(-MOVE_X_OFFSET);
                                }
                            }
                            Direction::Right => {
                                if dungeon.is_player(monster.x + 1, monster.y) {
                                    entity.offset_x.set(MOVE_X_OFFSET);
                                }
                            }
                        }
                    }
                }
                if (!entity.y.done() || !entity.x.done()) && entity.offset_y.done() {
                    entity.offset_y.set(-MOVE_Y_OFFSET);
                }
                if entity.offset_x.done() && entity.offset_x.get() != 0 {
                    entity.offset_x.set(0);
                }
                if entity.offset_y.done() && entity.offset_y.get() != 0 {
                    entity.offset_y.set(0);
                }
            }
        }

        let did_turn_transition_end = state.turn.done();
        let was_last_exec_on_diff_turn = state.last_exec_turn.map_or(true, |t| t != dungeon.turn);
        let did_exec_timeout = (tick() - state.last_exec_at) >= EXEC_TIMEOUT_DUR;
        let is_ready_to_exec =
            did_turn_transition_end && (was_last_exec_on_diff_turn || did_exec_timeout);
        let is_alive = dungeon.player.health > 0;

        // Handle player input
        let gp = gamepad(0);

        // Hard reset game
        if gp.start.just_pressed() && gp.select.pressed() {
            DungeonDashProgram::create_new_dungeon(CreateNewDungeonCommandInput { reset: true });
            state.last_exec_at = tick();
            state.last_exec_turn = Some(dungeon.turn);
        }
        // Dungeon controls
        else if is_ready_to_exec {
            // Next floor or restart
            if gp.start.just_pressed() && state.achievements_modal.is_none() {
                DungeonDashProgram::create_new_dungeon(CreateNewDungeonCommandInput {
                    reset: dungeon.player.health == 0,
                });
                state.last_exec_at = tick();
                state.last_exec_turn = Some(dungeon.turn);
            }
            // Move
            else if gp.up.pressed() && is_alive {
                DungeonDashProgram::move_player(MovePlayerCommandInput {
                    direction: Direction::Up,
                });
                state.last_exec_at = tick();
                state.last_exec_turn = Some(dungeon.turn);
                if dungeon.is_position_blocked(dungeon.player.x, dungeon.player.y - 1) {
                    state.player.offset_y.set(-MOVE_Y_OFFSET);
                }
            } else if gp.down.pressed() && is_alive {
                DungeonDashProgram::move_player(MovePlayerCommandInput {
                    direction: Direction::Down,
                });
                state.last_exec_at = tick();
                state.last_exec_turn = Some(dungeon.turn);
                if dungeon.is_position_blocked(dungeon.player.x, dungeon.player.y + 1) {
                    state.player.offset_y.set(MOVE_Y_OFFSET);
                }
            } else if gp.left.pressed() && is_alive {
                DungeonDashProgram::move_player(MovePlayerCommandInput {
                    direction: Direction::Left,
                });
                state.last_exec_at = tick();
                state.last_exec_turn = Some(dungeon.turn);
                if dungeon.is_position_blocked(dungeon.player.x - 1, dungeon.player.y) {
                    state.player.offset_x.set(-MOVE_X_OFFSET);
                }
            } else if gp.right.pressed() && is_alive {
                DungeonDashProgram::move_player(MovePlayerCommandInput {
                    direction: Direction::Right,
                });
                state.last_exec_at = tick();
                state.last_exec_turn = Some(dungeon.turn);
                if dungeon.is_position_blocked(dungeon.player.x + 1, dungeon.player.y) {
                    state.player.offset_x.set(MOVE_X_OFFSET);
                }
            }
        }

        // Center camera on player
        set_cam!(
            x = state.player.x.get() + (TILE_SIZE / 2),
            y = {
                let n = state.player.y.get() + (TILE_SIZE / 2) + menubar_h;
                let n = n as f32 + ((tick() as f32 / 16.).cos() * 2.) - 2.;
                n.round()
            },
        );

        // Draw dungeon floor and border
        sprite!(
            dungeon_theme.floor_sprite,
            w = dungeon_w,
            h = dungeon_h,
            repeat = true,
        );
        nine_slice!(
            dungeon_theme.dungeon_border,
            w = dungeon_w + (TILE_SIZE as u32 * 2),
            h = dungeon_h + (TILE_SIZE as u32 / 2 + TILE_SIZE as u32),
            x = -TILE_SIZE,
            y = -TILE_SIZE / 2,
            slice_size = 16
        );

        // Draw exit
        if let Some(exit) = &dungeon.exit {
            sprite!("stairs_up", x = exit.0 * TILE_SIZE, y = exit.1 * TILE_SIZE)
        }

        // Draw obstacles
        for obstacle in &dungeon.obstacles {
            match obstacle.kind {
                ObstacleKind::WallA => {
                    sprite!(
                        dungeon_theme.block_a_sprite,
                        x = obstacle.x * TILE_SIZE,
                        y = obstacle.y * TILE_SIZE,
                    );
                }
                ObstacleKind::WallB => {
                    sprite!(
                        dungeon_theme.block_b_sprite,
                        x = obstacle.x * TILE_SIZE,
                        y = obstacle.y * TILE_SIZE,
                        fps = fps::MEDIUM
                    );
                }
            }
        }

        // Draw player
        if dungeon.player.health > 0 {
            let x = state.player.x.get();
            let y = state.player.y.get();
            sprite!(
                "dotted_tile_border",
                x = x,
                y = y,
                opacity = 0.25,
                fps = fps::FAST,
            );
            let x = x + state.player.offset_x.get();
            // let y = y - 9;
            ellipse!(
                x = x + 2,
                y = y + 3,
                w = TILE_SIZE - 4,
                h = TILE_SIZE - 4,
                color = SHADOW_COLOR,
            );
            let y = y + state.player.offset_y.get() - 4;
            sprite!("hero", x = x, y = y, fps = fps::FAST,);
        } else {
            sprite!(
                "tombstone",
                x = state.player.x.get(),
                y = state.player.y.get() - 5,
            );
        }

        // Draw monsters
        for (monster, entity) in dungeon
            .monsters
            .iter()
            .zip(state.monsters.iter_mut())
            .collect::<Vec<(_, _)>>()
        {
            if monster.health == 0 {
                continue;
            }
            if monster.stun_dur > 0 && tick() % 16 < 8 {
                continue;
            }
            let opacity = if monster.stun_dur > 0 { 0.5 } else { 1.0 };
            let x = entity.x.get() + entity.offset_x.get();
            let y = entity.y.get() + entity.offset_y.get() - 6;
            match monster.kind {
                MonsterKind::BlueBlob => {
                    ellipse!(
                        x = x + 1,
                        y = y + 9,
                        w = TILE_SIZE - 2,
                        h = TILE_SIZE - 6,
                        color = SHADOW_COLOR,
                    );
                    sprite!(
                        "blue_blob",
                        x = x,
                        y = y + 3,
                        fps = fps::FAST,
                        opacity = opacity
                    );
                }
                MonsterKind::YellowBlob => {
                    ellipse!(
                        x = x + 1,
                        y = y + 9,
                        w = TILE_SIZE - 2,
                        h = TILE_SIZE - 6,
                        color = SHADOW_COLOR,
                    );
                    sprite!(
                        "yellow_blob",
                        x = x,
                        y = y + 3,
                        fps = fps::FAST,
                        opacity = opacity
                    );
                }
                MonsterKind::RedBlob => {
                    ellipse!(
                        x = x + 1,
                        y = y + 9,
                        w = TILE_SIZE - 2,
                        h = TILE_SIZE - 6,
                        color = SHADOW_COLOR,
                    );
                    sprite!(
                        "red_blob",
                        x = x,
                        y = y + 3,
                        fps = fps::FAST,
                        opacity = opacity
                    );
                }
                MonsterKind::OrangeGoblin => {
                    ellipse!(
                        x = x + 2,
                        y = y + 8,
                        w = TILE_SIZE - 4,
                        h = TILE_SIZE - 4,
                        color = SHADOW_COLOR,
                    );
                    sprite!(
                        "orange_goblin",
                        x = x,
                        y = y,
                        fps = fps::FAST,
                        opacity = opacity
                    );
                }
                MonsterKind::GreenGoblin => {
                    ellipse!(
                        x = x + 2,
                        y = y + 8,
                        w = TILE_SIZE - 4,
                        h = TILE_SIZE - 4,
                        color = SHADOW_COLOR,
                    );
                    sprite!("goblin", x = x, y = y, fps = fps::FAST, opacity = opacity);
                }
                MonsterKind::Shade => {
                    let n = (tick() as f32 / 32.0).sin();
                    ellipse!(
                        x = x + 1,
                        y = y + 10 + (n * 2.) as i32,
                        w = TILE_SIZE - 2,
                        h = TILE_SIZE - 6,
                        color = SHADOW_COLOR,
                    );
                    sprite!(
                        "shade",
                        x = x,
                        y = y + (n * 2.) as i32,
                        fps = fps::FAST,
                        opacity = if dungeon.is_obstacle(monster.x, monster.y) {
                            0.5
                        } else {
                            opacity
                        }
                    );
                }
                MonsterKind::Spider => {
                    ellipse!(
                        x = x + 2,
                        y = y + 8,
                        w = TILE_SIZE - 4,
                        h = TILE_SIZE - 4,
                        color = SHADOW_COLOR,
                    );
                    sprite!(
                        "spider",
                        x = x,
                        y = y + 2,
                        fps = fps::MEDIUM,
                        opacity = opacity
                    );
                }
                MonsterKind::Ghost => {
                    let n = (tick() as f32 / 16.0).sin();
                    ellipse!(
                        x = x + 2,
                        y = y + 13 + (n * 2.) as i32,
                        w = TILE_SIZE - 4,
                        h = TILE_SIZE - 8,
                        color = SHADOW_COLOR,
                    );
                    sprite!(
                        "ghost",
                        x = x,
                        y = y + (n * 4.) as i32,
                        fps = fps::MEDIUM + 1,
                        opacity = if dungeon.is_obstacle(monster.x, monster.y) {
                            0.5
                        } else {
                            opacity
                        }
                    );
                }
                MonsterKind::SpectralGhost => {
                    let n = (tick() as f32 / 16.0).sin();
                    ellipse!(
                        x = x + 2,
                        y = y + 13 + (n * 2.) as i32,
                        w = TILE_SIZE - 4,
                        h = TILE_SIZE - 8,
                        color = SHADOW_COLOR,
                    );
                    sprite!(
                        "spectral_ghost",
                        x = x,
                        y = y + (n * 4.) as i32,
                        fps = fps::MEDIUM + 1,
                        opacity = if dungeon.is_obstacle(monster.x, monster.y) {
                            0.5
                        } else {
                            opacity
                        }
                    );
                }
                MonsterKind::Zombie => {
                    ellipse!(
                        x = x + 2,
                        y = y + 8,
                        w = TILE_SIZE - 4,
                        h = TILE_SIZE - 4,
                        color = SHADOW_COLOR,
                    );
                    sprite!(
                        "zombie",
                        x = x,
                        y = y,
                        fps = fps::MEDIUM + 2,
                        opacity = opacity
                    );
                }
                _ => {
                    ellipse!(
                        x = x + 2,
                        y = y + 8,
                        w = TILE_SIZE - 4,
                        h = TILE_SIZE - 4,
                        color = SHADOW_COLOR,
                    );
                    sprite!(
                        "generic_monster",
                        x = x,
                        y = y,
                        fps = fps::FAST,
                        opacity = opacity
                    );
                }
            }
        }

        let t = tick() as f32;

        // Draw exit key
        if let Some(exit_key) = &dungeon.exit_key {
            let y = exit_key.1 * TILE_SIZE - 4;
            let y_offset = ((t / 10.).cos() * 2.) - 1.;
            ellipse!(
                x = (exit_key.0 * TILE_SIZE + 5) as f32 + (y_offset / 4.),
                y = (y + 12) as f32 + (y_offset / 4.),
                w = (TILE_SIZE - 9) as f32 - (y_offset / 2.),
                h = (TILE_SIZE - 12) as f32 - (y_offset / 2.),
                color = SHADOW_COLOR,
            );
            sprite!(
                "boss_key",
                x = exit_key.0 * TILE_SIZE,
                y = y as f32 + y_offset - 4.
            )
        }

        // Draw treasures
        for treasure in &dungeon.treasures {
            let y = treasure.y * TILE_SIZE - 5;
            let y_offset = ((t / 10.).cos() * 2.) - 1.;
            match treasure.kind {
                TreasureKind::Gold => {
                    ellipse!(
                        x = (treasure.x * TILE_SIZE + 5) as f32 + (y_offset / 4.),
                        y = (y + 12) as f32 + (y_offset / 4.),
                        w = (TILE_SIZE - 9) as f32 - (y_offset / 2.),
                        h = (TILE_SIZE - 12) as f32 - (y_offset / 2.),
                        color = SHADOW_COLOR,
                    );
                    match treasure.value {
                        50 => sprite!(
                            "yellow_gem",
                            x = treasure.x * TILE_SIZE,
                            y = y as f32 + y_offset - 4.,
                        ),
                        10 => sprite!(
                            "purple_gem",
                            x = treasure.x * TILE_SIZE,
                            y = y as f32 + y_offset - 4.,
                        ),
                        _ => sprite!("coin", x = treasure.x * TILE_SIZE, y = y as f32 + y_offset,),
                    }
                }
                TreasureKind::Heal => {
                    ellipse!(
                        x = (treasure.x * TILE_SIZE + 5) as f32 + (y_offset / 4.),
                        y = (y + 12) as f32 + (y_offset / 4.),
                        w = (TILE_SIZE - 9) as f32 - (y_offset / 2.),
                        h = (TILE_SIZE - 12) as f32 - (y_offset / 2.),
                        color = SHADOW_COLOR,
                    );
                    sprite!(
                        "full_heart",
                        x = treasure.x * TILE_SIZE,
                        y = y as f32 + y_offset,
                    );
                }
                TreasureKind::HealthUp => {
                    ellipse!(
                        x = (treasure.x * TILE_SIZE + 5) as f32 + (y_offset / 4.),
                        y = (y + 14) as f32 + (y_offset / 4.),
                        w = (TILE_SIZE - 9) as f32 - (y_offset / 2.),
                        h = (TILE_SIZE - 12) as f32 - (y_offset / 2.),
                        color = SHADOW_COLOR,
                    );
                    sprite!(
                        "super_heart",
                        x = treasure.x * TILE_SIZE,
                        y = y as f32 + y_offset,
                        fps = fps::MEDIUM + 3,
                    );
                }
            }
        }

        // Draw obstacles effects
        for obstacle in &dungeon.obstacles {
            match (dungeon.theme, obstacle.kind) {
                (DungeonThemeKind::Castle, ObstacleKind::WallB) => {
                    let color = 0xff333300;
                    let t = tick() as f32 % 4. / 4.;
                    let intensity = ((t * 2.0 * PI).sin() + 1.0) / 2.0;
                    let intensity = (intensity * 63.0) as u32;
                    circ!(
                        d = TILE_SIZE,
                        x = obstacle.x * TILE_SIZE,
                        y = obstacle.y * TILE_SIZE,
                        color = color | (intensity * 2),
                    );
                    circ!(
                        d = TILE_SIZE * 2,
                        x = obstacle.x * TILE_SIZE - (TILE_SIZE / 2),
                        y = obstacle.y * TILE_SIZE - (TILE_SIZE / 2),
                        color = color | intensity,
                    );
                }
                _ => {}
            }
        }

        // Draw monsters health bars
        for (monster, entity) in dungeon
            .monsters
            .iter()
            .zip(state.monsters.iter_mut())
            .collect::<Vec<(_, _)>>()
        {
            if monster.health == 0 {
                continue;
            }
            let x = entity.x.get();
            let y = entity.y.get() - 8;

            // Draw health bar background (black)
            rect!(
                x = x,
                y = y - 5,
                w = TILE_SIZE,
                h = 5,
                color = 0x000000fa,
                border_radius = 2
            );

            // Constants
            let spacing = 1;
            let segment_width = 2;
            let min_segment_width = 1;
            let max_segments = ((TILE_SIZE + spacing) / (segment_width + spacing)) as i32;

            // Draw health segments
            let mut remaining_health = monster.health as i32;
            let mut layer = 0;

            while remaining_health > 0 {
                let segments_this_layer = remaining_health.min(max_segments);
                for i in 0..segments_this_layer {
                    let color = if layer == 0 {
                        0x5ac54fff // Green for first layer
                    } else {
                        0x0000ffaa // Blue for additional layers
                    };
                    rect!(
                        x = x + 1 + (i * (segment_width + spacing)),
                        y = y - 4,
                        w = segment_width.max(min_segment_width),
                        h = 1,
                        color = color,
                    );
                }
                remaining_health -= segments_this_layer;
                layer += 1;
            }

            // Draw strength segments overlapping the health bar
            let mut remaining_strength = monster.strength as i32;
            let mut layer = 0;

            while remaining_strength > 0 {
                let segments_this_layer = remaining_strength.min(max_segments);
                for i in 0..segments_this_layer {
                    let color = if layer == 0 {
                        0xea323cff // Red for first layer
                    } else {
                        0x0000ff66 // Blue for additional layers
                    };
                    rect!(
                        x = x + 1 + (i * (segment_width + spacing)),
                        y = y - 2,
                        w = segment_width.max(min_segment_width),
                        h = 1,
                        color = color,
                    );
                }
                remaining_strength -= segments_this_layer;
                layer += 1;
            }
        }

        // Rain weather effect
        if dungeon.theme == DungeonThemeKind::Pirate {
            let t = tick();
            let lightning_timer = 512;
            if t % lightning_timer <= 10 {
                if t % lightning_timer <= 5 {
                    if (rand() % 4) != 0 {
                        // Bright flash effect for a short duration
                        rect!(absolute = true, w = w, h = h, color = 0xffffff66);
                    }
                } else if t % lightning_timer <= 10 {
                    // Dim after-flash effect
                    rect!(absolute = true, w = w, h = h, color = 0xaaaaaa33);
                }
            }
            // Generate new raindrops at random intervals
            if rand() % 2 == 0 {
                // Create a new raindrop with random attributes
                let raindrop = Raindrop {
                    x: (rand() % w) as f32,
                    y: 0.0,
                    vel: (rand() % 5 + 3) as f32,
                    length: (rand() % 10 + 5) as f32,
                };
                state.raindrops.push(raindrop);
            }

            // Update raindrop positions
            state.raindrops.retain_mut(|raindrop| {
                raindrop.y += raindrop.vel;
                raindrop.y < (h as f32) + raindrop.length // Keep the raindrop in the game if it's within the screen
            });

            // Draw the falling raindrops
            for raindrop in &state.raindrops {
                path!(
                    start = (raindrop.x, raindrop.y),
                    end = (raindrop.x, raindrop.y + raindrop.length),
                    color = 0xffffff60,
                    absolute = true,
                ); // Render the raindrop
            }
        }

        // Draw Leaderboards
        if dungeon.player.health == 0 && state.turn.done() {
            if let Some(user_id) = &os::user_id() {
                if gp.right.just_pressed() {
                    state.leaderboard_kind = state.leaderboard_kind.prev();
                }
                if gp.left.just_pressed() {
                    state.leaderboard_kind = state.leaderboard_kind.next();
                }
                if let Ok(leaderboard) = DungeonDashProgram::fetch_global_leaderboard() {
                    rect!(absolute = true, w = w, h = h, color = 0x000000fa);

                    let slide_dot_y = h as i32 - (menubar_h + 34);
                    let dot_spacing = 8;
                    let start_x =
                        (w as i32 / 2) - (LeaderboardKind::ALL.len() as i32 * dot_spacing / 2);

                    for (i, kind) in LeaderboardKind::ALL.iter().enumerate() {
                        circ!(
                            absolute = true,
                            d = 6,
                            x = start_x + (i as i32 * dot_spacing),
                            y = slide_dot_y,
                            color = 0xacaabdff,
                            border_width = if state.leaderboard_kind == *kind {
                                0
                            } else {
                                1
                            },
                            border_color = 0,
                        );
                    }

                    let leaderboard_x = 0;
                    let leaderboard_y = 0;
                    text!(
                        "LEADERBOARD",
                        absolute = true,
                        x = leaderboard_x + 8,
                        y = 7,
                        font = Font::L
                    );

                    let mut i = 2; // Initial y position index for leaderboard text
                    match state.leaderboard_kind {
                        #[rustfmt::skip]
                        LeaderboardKind::HighestFloor => {
                            text!("Highest Floor", absolute = true, x = leaderboard_x + 8, y = i * 10);
                            i += 1;
                            let leaderboard_y = leaderboard_y + 4;
                            text!("#  PLAYER {:>7} FLOOR", ""; absolute = true, x = leaderboard_x + 8, y = leaderboard_y + i * 10);
                            i += 1;
                            leaderboard.render_entries(dungeon.crawl_id, i, state.leaderboard_kind, user_id, leaderboard_x, leaderboard_y);
                        }
                        #[rustfmt::skip]
                        LeaderboardKind::MostGold => {
                            text!("Most Gold", absolute = true, x = leaderboard_x + 8, y = i * 10);
                            i += 1;
                            let leaderboard_y = leaderboard_y + 4;
                            text!("#  PLAYER {:>8} GOLD", ""; absolute = true, x = leaderboard_x + 8, y = leaderboard_y + i * 10);
                            i += 1;
                            leaderboard.render_entries(dungeon.crawl_id, i, state.leaderboard_kind, user_id, leaderboard_x, leaderboard_y);
                        }
                        #[rustfmt::skip]
                        LeaderboardKind::MostKills => {
                            text!("Most Kills", absolute = true, x = leaderboard_x + 8, y = i * 10);
                            i += 1;
                            let leaderboard_y = leaderboard_y + 4;
                            text!("#  PLAYER {:>7} KILLS", ""; absolute = true, x = leaderboard_x + 8, y = leaderboard_y + i * 10);
                            i += 1;
                            leaderboard.render_entries(dungeon.crawl_id, i, state.leaderboard_kind, user_id, leaderboard_x, leaderboard_y);
                        }
                        #[rustfmt::skip]
                        LeaderboardKind::LeastSteps => {
                            text!("Least Steps", absolute = true, x = leaderboard_x + 8, y = i * 10);
                            i += 1;
                            let leaderboard_y = leaderboard_y + 4;
                            text!("#  PLAYER {:>7} STEPS", ""; absolute = true, x = leaderboard_x + 8, y = leaderboard_y + i * 10);
                            i += 1;
                            leaderboard.render_entries(dungeon.crawl_id, i, state.leaderboard_kind, user_id, leaderboard_x, leaderboard_y);
                        }
                    }
                }
            }
        }

        // Generate new particles at random intervals
        let max_particle_size = 16.;
        let amount_percent = dungeon.player.health as f32 / dungeon.player.max_health as f32;
        let should_add_particle = rand() % (1 + (8. * amount_percent) as u32) == 0;
        if should_add_particle && dungeon.player.health > 0 {
            let is_black = rand() % 3 == 0;
            let color = if is_black {
                0x000000ff
            } else {
                dungeon_theme.particle_color
            };
            // Create a new particle with random attributes
            let particle = Particle {
                x: (rand() % w) as f32,
                y: h as f32 - (menubar_h as f32) + max_particle_size,
                vx: (rand() % 3) as f32 - 1.0 / 8.0,
                vy: -((rand() % 3 + 1) as f32) / 4.0,
                lifetime: 60 + (rand() % 60) as u32,
                color: color,
            };
            state.particles.push(particle);
        }

        // Update particles positions and reduce their lifetimes
        state.particles.retain_mut(|particle| {
            particle.x += particle.vx;
            particle.y += particle.vy;
            particle.lifetime -= 1;
            particle.lifetime > 0
                && particle.y > 0.0
                && particle.x >= (0.0 - max_particle_size)
                && particle.x < (w as f32)
        });

        // Draw particles
        for particle in &state.particles {
            if particle.color == 0x000000ff {
                continue;
            }
            let intensity = particle.lifetime as f32 / 60.0;
            let opacity = (intensity * 255.0) as u32;
            circ!(
                x = particle.x,
                y = particle.y,
                d = max_particle_size * intensity,
                color = (particle.color & 0xffffff00) | opacity,
                absolute = true,
            );
        }

        // Button menubar
        let menubar_y = (h - menubar_h as u32) as i32;
        let y = menubar_y;

        // Draw undulating clouds on the edges
        let edge_count = 16;
        let circle_radius = 32.0;
        let angle_offset = (tick() as f32 + 3.) * 0.05;

        // Draw bottom edge
        for i in 0..edge_count {
            let progress = (i) as f32 / edge_count as f32;
            let x = progress * (w as f32);
            let y = (h as f32) - ((angle_offset + progress * PI * 2.0).sin() * 5.0 + circle_radius);
            circ!(
                x = x - (circle_radius),
                y = y - 40. + 24.,       // + (circle_radius * 0.5),
                d = circle_radius * 2.0, // * (rand() as f32 % 100. * 0.01),
                color = dungeon_theme.particle_color & 0xffffff88,
                absolute = true
            );
        }
        for particle in &state.particles {
            if particle.color != 0x000000ff {
                continue;
            }
            let intensity = particle.lifetime as f32 / 60.0;
            let opacity = (intensity * 255.0) as u32;
            circ!(
                x = particle.x,
                y = particle.y,
                d = max_particle_size * intensity,
                color = particle.color | opacity,
                absolute = true,
            );
        }
        let edge_count = 8;
        let circle_radius = 16.;
        let angle_offset = (tick() as f32) * 0.05;
        for i in 0..edge_count {
            let progress = (i) as f32 / edge_count as f32;
            let x = progress * (w as f32);
            let y = (h as f32) - ((angle_offset + progress * PI * 2.0).sin() * 5.0 + circle_radius);
            circ!(
                x = x - (circle_radius * 0.5),
                y = y - 40. + (circle_radius * 0.5),
                d = circle_radius * 2.0, // * (rand() as f32 % 100. * 0.01),
                color = 0x000000ff,
                absolute = true
            );
        }

        // Menubar background
        rect!(
            absolute = true,
            w = w,
            h = menubar_h,
            y = y,
            color = 0x000000ff
        );
        let y = y + 2;

        // HP
        sprite!("full_heart", absolute = true, y = y);
        let y = y + 4;
        let hp_color = match dungeon.player.health as f32 / dungeon.player.max_health as f32 {
            0.75..=1.0 => 0x71f341ff,
            0.25..=0.75 => 0xffa200ff,
            _ => 0xb41c39ff,
        };
        text!("  {:0>2}/  ", dungeon.player.health; absolute = true, x = 0, y = y, font = Font::L, color = hp_color);
        text!("    /{:0>2}", dungeon.player.max_health; absolute = true, x = 0, y = y, font = Font::L);
        let y = y + 8;

        // Gold
        sprite!("coin", absolute = true, y = y);
        text!("  ${:0>4}", dungeon.player.gold; absolute = true, x = 0, y = y + 5, font = Font::L);

        if dungeon.player.health == 0 {
            let t = tick() as f32;
            let cos_16 = ((t / 16.).cos()) + 1.;
            let action_btn_x = w / 2;
            let action_btn_y = (menubar_y + 3) - (cos_16 as i32);
            let action_btn_w = (w / 2) - 4;
            let action_btn_h = 24;
            rect!(
                absolute = true,
                w = action_btn_w,
                h = action_btn_h,
                x = action_btn_x,
                y = action_btn_y + 1 + (cos_16 as i32),
                color = 0x81090aaa,
                border_radius = 4,
            );
            rect!(
                absolute = true,
                w = action_btn_w,
                h = action_btn_h,
                x = action_btn_x,
                y = action_btn_y,
                color = 0x81090aff,
                border_radius = 4,
                border_width = cos_16,
                border_color = 0xb41c39ff,
            );
            let action_btn_text = "GAME OVER";
            let action_btn_text_len = action_btn_text.len() as u32;
            let action_btn_text_w = action_btn_text_len * 5;
            let action_btn_text_x = 1 + action_btn_x + (action_btn_w / 2) - (action_btn_text_w / 2);
            let action_btn_text_y = action_btn_y + 5;
            text!(
                action_btn_text,
                absolute = true,
                // color = 0x000000ff,
                x = action_btn_text_x,
                y = action_btn_text_y,
                font = Font::M,
            );
            let action_btn_text = "Try again?";
            let action_btn_text_len = action_btn_text.len() as u32;
            let action_btn_text_w = action_btn_text_len * 5;
            let action_btn_text_x = 1 + action_btn_x + (action_btn_w / 2) - (action_btn_text_w / 2);
            let action_btn_text_y = action_btn_y + 13;
            text!(
                action_btn_text,
                absolute = true,
                x = action_btn_text_x,
                y = action_btn_text_y,
                font = Font::M,
            );

            // Handle next floor click / tap
            let m = mouse(0);
            let [mx, my] = m.position;
            let mx = (mx - (cam!().0)) + (w / 2) as i32;
            let my = (my - (cam!().1)) + (h / 2) as i32;
            let hit_x0 = action_btn_x as i32;
            let hit_x1 = (action_btn_x + action_btn_w) as i32;
            let hit_y0 = action_btn_y as i32;
            let hit_y1 = (action_btn_y + action_btn_h) as i32;
            let is_in_btn = mx >= hit_x0 && mx < hit_x1 && my >= hit_y0 && my < hit_y1;
            let is_modal_closed = state.achievements_modal.is_none();
            if m.left.just_pressed() && is_in_btn && is_modal_closed {
                DungeonDashProgram::create_new_dungeon(CreateNewDungeonCommandInput {
                    reset: true,
                });
            }
        }
        // Next floor button
        else if dungeon.is_exit(dungeon.player.x, dungeon.player.y) {
            let t = tick() as f32;
            let cos_16 = ((t / 16.).cos()) + 1.;
            let action_btn_x = w / 2;
            let action_btn_y = (menubar_y + 3) - (cos_16 as i32);
            let action_btn_w = (w / 2) - 4;
            let action_btn_h = 24;
            rect!(
                absolute = true,
                w = action_btn_w,
                h = action_btn_h,
                x = action_btn_x,
                y = action_btn_y + 1 + (cos_16 as i32),
                color = 0x7b34bdaa,
                border_radius = 4,
            );
            rect!(
                absolute = true,
                w = action_btn_w,
                h = action_btn_h,
                x = action_btn_x,
                y = action_btn_y,
                color = 0x7b34bdff,
                border_radius = 4,
                border_width = cos_16,
                border_color = 0xbd59deff,
            );
            let action_btn_text = "ENTER";
            let action_btn_text_len = action_btn_text.len() as u32;
            let action_btn_text_w = action_btn_text_len * 8;
            let action_btn_text_x = 1 + action_btn_x + (action_btn_w / 2) - (action_btn_text_w / 2);
            let action_btn_text_y = action_btn_y + 5;
            text!(
                action_btn_text,
                absolute = true,
                // color = 0x000000ff,
                x = action_btn_text_x,
                y = action_btn_text_y,
                font = Font::L,
            );
            let action_btn_text = "NEXT FLOOR";
            let action_btn_text_len = action_btn_text.len() as u32;
            let action_btn_text_w = action_btn_text_len * 5;
            let action_btn_text_x = 1 + action_btn_x + (action_btn_w / 2) - (action_btn_text_w / 2);
            let action_btn_text_y = action_btn_y + 13;
            text!(
                action_btn_text,
                absolute = true,
                x = action_btn_text_x,
                y = action_btn_text_y,
                font = Font::M,
            );

            // Handle next floor click / tap
            let m = mouse(0);
            let [mx, my] = m.position;
            let mx = (mx - (cam!().0)) + (w / 2) as i32;
            let my = (my - (cam!().1)) + (h / 2) as i32;
            let hit_x0 = action_btn_x as i32;
            let hit_x1 = (action_btn_x + action_btn_w) as i32;
            let hit_y0 = action_btn_y as i32;
            let hit_y1 = (action_btn_y + action_btn_h) as i32;
            let is_in_btn = mx >= hit_x0 && mx < hit_x1 && my >= hit_y0 && my < hit_y1;
            if m.left.just_pressed() && is_in_btn {
                DungeonDashProgram::create_new_dungeon(CreateNewDungeonCommandInput {
                    reset: false,
                });
            }
        }
        // CTA: Find exit
        else if dungeon.exit.is_some() {
            let cta_x = w / 2;
            let cta_y = menubar_y + 4;
            let cta_w = (w / 2) - 4;
            let cta_text = "~TASK~";
            let cta_text_len = cta_text.len() as u32;
            let cta_text_w = cta_text_len * 8;
            let cta_text_x = 1 + cta_x + (cta_w / 2) - (cta_text_w / 2);
            let cta_text_y = cta_y + 5;
            text!(
                cta_text,
                absolute = true,
                color = 0x524c52ff,
                x = cta_text_x,
                y = cta_text_y,
                font = Font::L,
            );
            let cta_text = "Find exit";
            let cta_text_len = cta_text.len() as u32;
            let cta_text_w = cta_text_len * 5;
            let cta_text_x = 1 + cta_x + (cta_w / 2) - (cta_text_w / 2);
            let cta_text_y = cta_y + 13;
            text!(
                cta_text,
                absolute = true,
                color = 0x524c52ff,
                x = cta_text_x,
                y = cta_text_y,
                font = Font::M,
            );
        }
        // CTA: Get the key
        else {
            let cta_x = w / 2;
            let cta_y = menubar_y + 4;
            let cta_w = (w / 2) - 4;
            let cta_text = "~TASK~";
            let cta_text_len = cta_text.len() as u32;
            let cta_text_w = cta_text_len * 8;
            let cta_text_x = 1 + cta_x + (cta_w / 2) - (cta_text_w / 2);
            let cta_text_y = cta_y + 5;
            text!(
                cta_text,
                absolute = true,
                color = 0x524c52ff,
                x = cta_text_x,
                y = cta_text_y,
                font = Font::L,
            );
            let cta_text = "Get the key";
            let cta_text_len = cta_text.len() as u32;
            let cta_text_w = cta_text_len * 5;
            let cta_text_x = 1 + cta_x + (cta_w / 2) - (cta_text_w / 2);
            let cta_text_y = cta_y + 13;
            text!(
                cta_text,
                absolute = true,
                color = 0x524c52ff,
                x = cta_text_x,
                y = cta_text_y,
                font = Font::M,
            );
        }

        // Bottom info bar
        let info_bar_y = menubar_y + 32;
        if let Some(user_id) = &os::user_id() {
            rect!(
                absolute = true,
                w = w,
                h = 8,
                y = info_bar_y,
                color = 0x293c8bff,
            );
            rect!(
                absolute = true,
                w = w / 2,
                h = 8,
                y = info_bar_y,
                color = 0x524c52ff,
            );
            let id_text = format!("ID:{:.8}", user_id);
            text!(
                &id_text,
                absolute = true,
                x = 4,
                y = info_bar_y + 2,
                font = Font::S,
                color = 0xacaabdff
            );
            let floor_text = format!("FLOOR:{:0>2}", dungeon.floor + 1);
            let floor_text_len = floor_text.len() as u32;
            let floor_text_w = floor_text_len * 5;
            text!(
                &floor_text,
                absolute = true,
                x = w - floor_text_w - 4,
                y = info_bar_y + 2,
                font = Font::S,
                color = 0x4181c5ff
            );
        }

        // Achievements Modal
        if let Some(mut modal) = state.achievements_modal.take() {
            // current tick
            let t = tick();

            // Background overlay
            rect!(w = w, h = h, color = 0x000000fe, absolute = true);

            // Draw confetti
            for particle in &modal.confetti {
                circ!(
                    x = particle.x,
                    y = particle.y,
                    d = particle.radius * 2.,
                    color = particle.color,
                    absolute = true,
                );
            }
            // Update confetti positions
            for particle in &mut modal.confetti {
                particle.y += particle.vy;

                // Reset position if it goes off the screen
                if particle.y > (h as f32) + particle.radius {
                    particle.y = 0.0;
                    particle.x = (rand() % w) as f32;
                    particle.vy = (rand() % 2 + 1) as f32;
                }
            }

            rect!(w = w, h = 16, color = 0x411883ff, absolute = true);
            let text = "NEW ACHIEVEMENT!";
            let color = if t % 128 < 64 { 0xbd59deff } else { 0x7b34bdff };
            let text_w = (text.len() * 8) as u32;
            let text_x = (w / 2) - (text_w / 2);
            text!(
                text,
                x = text_x,
                y = 2,
                color = color,
                font = Font::L,
                absolute = true
            );
            let percent =
                dungeon.all_unlocked.completed.len() as f32 / AchievementKind::INFO.len() as f32;
            let percent = percent * 100.;
            let text = &format!("COMPLETION:{:.0}%", percent);
            let text_w = (text.len() * 5) as u32;
            let text_x = (w / 2) - (text_w / 2);
            text!(
                text,
                x = text_x,
                y = 10,
                color = 0x7b34bdff,
                font = Font::S,
                absolute = true
            );

            // Render each achievement modal
            let mut did_dismiss = false;
            for (achivement_idx, achievement_kind) in modal.kinds.iter().enumerate() {
                // Render the modal
                let modal_w = w - 8;
                let modal_h = 100; //h - 64;
                let modal_x = ((w / 2) - (modal_w / 2)) as i32;
                let modal_y = 56 - modal.y.get() + (h as i32 * achivement_idx as i32);
                let modal_bg_color = 0x1a1932ff;

                // modal_y_offset = modal_y_offset.saturating_sub(8).max(0);
                rect!(
                    w = modal_w,
                    h = modal_h,
                    x = modal_x,
                    y = modal_y,
                    color = modal_bg_color,
                    border_radius = 8,
                    absolute = true
                );

                // Render the badge sprite placeholder
                ellipse!(
                    w = 64,
                    h = 64,
                    x = modal_x + ((modal_w / 2) - (64 / 2)) as i32,
                    y = modal_y + -32,
                    color = 0x2a2f4eff,
                    border_width = 4,
                    border_color = modal_bg_color,
                    absolute = true
                );
                sprite!(
                    "achievement_unlocked_icon",
                    x = modal_x + ((modal_w / 2) - (64 / 2)) as i32,
                    y = modal_y + -32,
                    absolute = true
                );

                // Find achievement info
                let (name, description) = achievement_kind.info();

                // Render the achievement name
                let mut text_y = modal_y + 40;
                let pad = 8;
                let font = Font::L;
                for line in wrap_text(&name.to_ascii_uppercase(), modal_w - (pad * 2), font) {
                    let text_w = (line.len() * 8) as u32;
                    text!(
                        &line,
                        x = (modal_x as i32) + ((modal_w / 2) - (text_w / 2)) as i32,
                        y = text_y,
                        color = 0xffffffff,
                        font = font,
                        absolute = true
                    );
                    text_y += 10; // Adjust line spacing as needed
                }
                text_y += 8;

                // Render the achievement description
                let font = Font::M;
                for line in wrap_text(description, modal_w, font) {
                    let text_w = (line.len() * 5) as u32;
                    text!(
                        &line,
                        x = modal_x + ((modal_w / 2) - (text_w / 2)) as i32,
                        y = text_y,
                        color = 0xe1e5d8ff,
                        font = font,
                        absolute = true
                    );
                    text_y += 10; // Adjust line spacing as needed
                }

                let text = "OKAY";
                let text_w = (text.len() * 8) as u32;
                let text_x = (w / 2) - (text_w / 2);
                let action_btn_x = text_x - 8;
                let action_btn_y = modal_y + modal_h + 16 - 8;
                let action_btn_w = text_w + 16;
                let action_btn_h = 24;
                rect!(
                    w = action_btn_w,
                    h = action_btn_h,
                    x = action_btn_x,
                    y = action_btn_y,
                    color = 0x411883ff,
                    border_radius = 4,
                    absolute = true
                );
                text!(
                    text,
                    x = text_x,
                    y = modal_y + modal_h + 16,
                    font = Font::L,
                    absolute = true
                );

                // Handle OKAY click / tap
                if achivement_idx == modal.current {
                    let m = mouse(0);
                    let [mx, my] = m.position;
                    let mx = (mx - (cam!().0)) + (w / 2) as i32;
                    let my = (my - (cam!().1)) + (h / 2) as i32;
                    let hit_x0 = action_btn_x as i32;
                    let hit_x1 = (action_btn_x + action_btn_w) as i32;
                    let hit_y0 = action_btn_y as i32;
                    let hit_y1 = (action_btn_y + action_btn_h) as i32;
                    let is_in_btn = mx >= hit_x0 && mx < hit_x1 && my >= hit_y0 && my < hit_y1;
                    let did_click_btn = m.left.just_pressed() && is_in_btn;
                    if (did_click_btn || gamepad(0).start.just_pressed()) && modal.y.done() {
                        did_dismiss = true;
                    }
                }
            }

            // Handle modal dismiss
            if did_dismiss {
                if modal.current < modal.kinds.len() {
                    modal.current += 1;
                }
            }

            // Update modal transition tween
            modal.y.set(modal.current as i32 * (h as i32));

            // Put the modal back if modal player hasn't seen all achievements
            if modal.current < modal.kinds.len() || !modal.y.done() {
                state.achievements_modal = Some(modal);
            }
        }

        // Swipe transition
        let p = state.floor.elapsed as f64 / FLOOR_DUR as f64;
        {
            let xo = p * w as f64;
            rect!(absolute = true, x = xo, w = w, h = h, color = 0x000000ff);
            rect!(absolute = true, x = -xo, w = w, h = h, color = 0x000000ff);
        }

        // text!("{:#?}", dungeon.stats.entries; absolute = true, x = -24, y = 64, font = Font::M, color = 0xffffffaa);
    }

    // If no existing dungeon, allow player to create one
    if let Err(err) = &dungeon {
        // Handle user input
        let gp = gamepad(0);
        if gp.start.just_pressed() {
            DungeonDashProgram::create_new_dungeon(CreateNewDungeonCommandInput { reset: true });
        }

        // Reset camera position
        reset_cam!();

        // Current tick and timers
        let t = tick() as f32;
        let cos_32 = ((t / 32.).cos()) * 2. + 1.;
        let cos_24 = (t / 24.).cos();
        let cos_16 = (t / 16.).cos();
        let cos_10 = (t / 10.).cos();
        let cos_08 = (t / 08.).cos();

        // Calculate y offset and base y position
        let v_offset = if h < 256 { h } else { 256 };
        let y = (h - v_offset) as f32;

        // Draw background sky and clouds
        sprite!("night_sky", y = y, w = w, sw = w, tx = t, repeat = true);
        if t % 2. == 0. {
            sprite!(
                "clouds_3",
                y = y + (cos_16 * 2.) + 1.,
                w = w,
                sw = w,
                tx = t / 2.,
                repeat = true,
                opacity = 0.5
            );
        }
        sprite!(
            "clouds_0",
            y = y + (cos_10 * 2.) + 1.,
            w = w,
            sw = w,
            tx = t / 8.,
            repeat = true
        );

        // Draw background castle
        let castle_scale = 0.5;
        let castle_h = 256. * castle_scale;
        let castle_w = 256. * castle_scale;
        let castle_x = (w as f32 / 2.) - (castle_w / 2.);
        let castle_y = h as f32 - castle_h - cos_32;
        sprite!("title_b", scale = castle_scale, x = castle_x, y = castle_y);

        // Draw foreground clouds
        sprite!(
            "clouds_1",
            y = y + (cos_24 * 2.) + 1.,
            w = w,
            sw = w,
            tx = t / 4.,
            repeat = true
        );
        sprite!(
            "clouds_2",
            y = y + (cos_08 * 2.) + 1.,
            w = w,
            sw = w,
            tx = t / 2.,
            repeat = true
        );

        // Draw title text
        let title_scale = 0.75;
        let title_h = 93. * title_scale;
        let title_w = 146. * title_scale;
        let title_x = (w as f32 / 2.) - (title_w / 2.);
        let title_y = h as f32 - (title_h * 3.);
        sprite!(
            "title_text",
            scale = title_scale,
            y = title_y + 2.,
            x = title_x,
            color = 0x000000ff,
            opacity = 0.75
        );
        sprite!("title_text", scale = title_scale, y = title_y, x = title_x,);

        if os::user_id().is_some() {
            if mouse(0).left.just_pressed() {
                DungeonDashProgram::create_new_dungeon(CreateNewDungeonCommandInput {
                    reset: true,
                });
            }
            rect!(
                absolute = true,
                y = h - 32,
                w = w,
                h = 32,
                color = 0x222034ff
            );
            if t / 2. % 32. < 16. {
                let text = "TAP TO START";
                let text_len = text.len() as u32;
                let text_w = text_len * 8;
                text!(
                    text,
                    x = (w / 2) - (text_w / 2),
                    y = h - 20,
                    color = 0xffffffff,
                    font = Font::L
                );
            }
        }

        // text!("PRESS START {:?}", os::user_id(););
        // let msg = format!("{}", err)
        //     .replace("ParsingError(", "")
        //     .replace(")", "");
        // text!("{}", msg; y = 0, font = Font::S, absolute = true);
        // text!("PRESS START");
    }

    state.save();
});

impl AchievementsModal {
    pub fn new(kinds: &[AchievementKind]) -> Self {
        let [w, h] = canvas_size!();
        Self {
            y: Tween::new(-(h as i32))
                .duration(MODAL_TRANS_DUR)
                .ease(Easing::EaseInOutQuad),
            current: 0,
            kinds: kinds.to_vec(),
            confetti: {
                let mut confetti = vec![];
                for _ in 0..50 {
                    confetti.push(Confetti {
                        x: (rand() % w) as f32,
                        y: (rand() % h) as f32,
                        radius: (rand() % 5 + 2) as f32,
                        color: rand() % 0xFFFFFF11 | 0xFF00ff88,
                        vy: (rand() % 2 + 1) as f32,
                    });
                }
                confetti
            },
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
struct PlayerAchievements {
    completed: BTreeSet<u32>, // stores index of completed achievements
}
impl PlayerAchievements {
    pub fn empty() -> Self {
        Self {
            completed: BTreeSet::new(),
        }
    }
    pub fn from_dungeon_stats(
        stats: &DungeonStats,
        total_stats: &DungeonStats,
        did_crawl_end: bool,
    ) -> Self {
        let mut completed = BTreeSet::new();
        for (kind, _, _) in AchievementKind::INFO {
            if kind.test(stats, total_stats, did_crawl_end) {
                completed.insert(*kind as u32);
            }
        }
        Self { completed }
    }
    pub fn from_achievement_kinds(kinds: &[AchievementKind]) -> Self {
        let mut completed = BTreeSet::new();
        for kind in kinds {
            completed.insert(*kind as u32);
        }
        Self { completed }
    }
    pub fn achievement_kinds(&self) -> Vec<AchievementKind> {
        self.completed
            .iter()
            .cloned()
            .map(|id| id.try_into().unwrap())
            .collect()
    }
    pub fn is_empty(&self) -> bool {
        self.completed.is_empty()
    }
    pub fn apply_dungeon_stats(
        &self,
        stats: &DungeonStats,
        total_stats: &DungeonStats,
        did_crawl_end: bool,
    ) -> Self {
        let mut completed = BTreeSet::new();
        for (kind, _, _) in AchievementKind::INFO {
            if kind.test(stats, total_stats, did_crawl_end) {
                completed.insert(*kind as u32);
            }
        }
        let next = Self { completed };
        self.union(&next)
    }
    pub fn difference(&self, other: &Self) -> Self {
        let completed = self
            .completed
            .difference(&other.completed)
            .cloned()
            .collect::<BTreeSet<_>>();
        Self { completed }
    }
    pub fn union(&self, other: &Self) -> Self {
        let completed = self
            .completed
            .union(&other.completed)
            .cloned()
            .collect::<BTreeSet<_>>();
        Self { completed }
    }
}

// TODO: in-a-row achievements
// ("Exterminator",       "Kill all enemies for 10 floors in a row"),
// ("Untouchable",        "Don't get hit for 5 floors in a row"),
// ("Dungeon Marathon",   "Play for 3 days in a row"),

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum AchievementKind {
    WelcomeToTheDungeon = 1,
    GoldGatherer = 2,
    SpaDay = 3,
    MonsterMenace = 4,
    DungeonDiver = 5,
    DungeonHobbyist = 6,
    DungeonExplorer = 7,
    DungeonConqueror = 8,
    Unbothered = 9,
    Survivalist = 10,
    GritAndGlory = 11,
    Unbreakable = 12,
    TreasureSeeker = 13,
    WealthAccumulator = 14,
    RichAdventurer = 15,
    MonsterSlayer = 16,
    MonsterVanquisher = 17,
    MonsterExterminator = 18,
    Pedestrian = 19,
    Wanderer = 20,
    Traveler = 21,
    GoblinSlayer = 22,
    OrangeMenace = 23,
    BlobBuster = 24,
    ShadeHunter = 25,
    SpiderSquasher = 26,
    Ghostbuster = 27,
    ZombieSlayer = 28,
    GoblinFodder = 29,
    Haunted = 30,
    Arachnophobia = 31,
    Blobbed = 32,
    NoviceExplorer = 33,
    SeasonedExplorer = 34,
    VeteranExplorer = 35,
    ReturningAdventurer = 36,
    DedicatedDelver = 37,
    // LearningTheRopes = 38,
    // PersistentSpirit = 39,
    SelfCare = 40,
    HealerSupreme = 41,
    // GreenGoblinHunter = 42,
    BlueBlobBuster = 43,
    SpectralBanisher = 44,
    ZombieHunter = 45,
    // Unlucky = 46,
    // Determined = 47,
    MasterOfTheDeep = 48,
    GoldHoarder = 49,
    GoldCollector = 50,
    DeadBroke = 51,
}

impl TryFrom<u32> for AchievementKind {
    type Error = ();
    fn try_from(v: u32) -> Result<Self, Self::Error> {
        match v {
            x if x == Self::WelcomeToTheDungeon as u32 => Ok(Self::WelcomeToTheDungeon),
            x if x == Self::GoldGatherer as u32 => Ok(Self::GoldGatherer),
            x if x == Self::SpaDay as u32 => Ok(Self::SpaDay),
            x if x == Self::MonsterMenace as u32 => Ok(Self::MonsterMenace),
            x if x == Self::DungeonDiver as u32 => Ok(Self::DungeonDiver),
            x if x == Self::DungeonHobbyist as u32 => Ok(Self::DungeonHobbyist),
            x if x == Self::DungeonExplorer as u32 => Ok(Self::DungeonExplorer),
            x if x == Self::DungeonConqueror as u32 => Ok(Self::DungeonConqueror),
            x if x == Self::Unbothered as u32 => Ok(Self::Unbothered),
            x if x == Self::Survivalist as u32 => Ok(Self::Survivalist),
            x if x == Self::GritAndGlory as u32 => Ok(Self::GritAndGlory),
            x if x == Self::Unbreakable as u32 => Ok(Self::Unbreakable),
            x if x == Self::TreasureSeeker as u32 => Ok(Self::TreasureSeeker),
            x if x == Self::WealthAccumulator as u32 => Ok(Self::WealthAccumulator),
            x if x == Self::RichAdventurer as u32 => Ok(Self::RichAdventurer),
            x if x == Self::MonsterSlayer as u32 => Ok(Self::MonsterSlayer),
            x if x == Self::MonsterVanquisher as u32 => Ok(Self::MonsterVanquisher),
            x if x == Self::MonsterExterminator as u32 => Ok(Self::MonsterExterminator),
            x if x == Self::Pedestrian as u32 => Ok(Self::Pedestrian),
            x if x == Self::Wanderer as u32 => Ok(Self::Wanderer),
            x if x == Self::Traveler as u32 => Ok(Self::Traveler),
            x if x == Self::GoblinSlayer as u32 => Ok(Self::GoblinSlayer),
            x if x == Self::OrangeMenace as u32 => Ok(Self::OrangeMenace),
            x if x == Self::BlobBuster as u32 => Ok(Self::BlobBuster),
            x if x == Self::ShadeHunter as u32 => Ok(Self::ShadeHunter),
            x if x == Self::SpiderSquasher as u32 => Ok(Self::SpiderSquasher),
            x if x == Self::Ghostbuster as u32 => Ok(Self::Ghostbuster),
            x if x == Self::ZombieSlayer as u32 => Ok(Self::ZombieSlayer),
            x if x == Self::GoblinFodder as u32 => Ok(Self::GoblinFodder),
            x if x == Self::Haunted as u32 => Ok(Self::Haunted),
            x if x == Self::Arachnophobia as u32 => Ok(Self::Arachnophobia),
            x if x == Self::Blobbed as u32 => Ok(Self::Blobbed),
            x if x == Self::NoviceExplorer as u32 => Ok(Self::NoviceExplorer),
            x if x == Self::SeasonedExplorer as u32 => Ok(Self::SeasonedExplorer),
            x if x == Self::VeteranExplorer as u32 => Ok(Self::VeteranExplorer),
            x if x == Self::ReturningAdventurer as u32 => Ok(Self::ReturningAdventurer),
            x if x == Self::DedicatedDelver as u32 => Ok(Self::DedicatedDelver),
            // x if x == Self::LearningTheRopes as u32 => Ok(Self::LearningTheRopes),
            // x if x == Self::PersistentSpirit as u32 => Ok(Self::PersistentSpirit),
            x if x == Self::SelfCare as u32 => Ok(Self::SelfCare),
            x if x == Self::HealerSupreme as u32 => Ok(Self::HealerSupreme),
            // x if x == Self::GreenGoblinHunter as u32 => Ok(Self::GreenGoblinHunter),
            x if x == Self::BlueBlobBuster as u32 => Ok(Self::BlueBlobBuster),
            x if x == Self::SpectralBanisher as u32 => Ok(Self::SpectralBanisher),
            x if x == Self::ZombieHunter as u32 => Ok(Self::ZombieHunter),
            // x if x == Self::Unlucky as u32 => Ok(Self::Unlucky),
            // x if x == Self::Determined as u32 => Ok(Self::Determined),
            x if x == Self::MasterOfTheDeep as u32 => Ok(Self::MasterOfTheDeep),
            x if x == Self::GoldHoarder as u32 => Ok(Self::GoldHoarder),
            x if x == Self::GoldCollector as u32 => Ok(Self::GoldCollector),
            x if x == Self::DeadBroke as u32 => Ok(Self::DeadBroke),
            _ => {
                log!("Invalid AchievementKind u32 - {v}");
                Err(())
            }
        }
    }
}

impl AchievementKind {
    #[rustfmt::skip]
    pub const INFO: &'static [(Self, &'static str, &'static str)] = &[
        // Crawl completion (all-time)
        (Self::WelcomeToTheDungeon, "Welcome to the Dungeon", "Complete your first crawl!"),
        (Self::ReturningAdventurer, "Returning Adventurer",   "Complete 2 crawls"),
        (Self::DedicatedDelver,     "Dedicated Delver",       "Complete 5 crawls"),

        // Total floors cleared (all-time)
        (Self::NoviceExplorer,      "Novice Explorer",        "Clear 20 floors total"),
        (Self::SeasonedExplorer,    "Seasoned Explorer",      "Clear 50 floors total"),
        (Self::VeteranExplorer,     "Veteran Explorer",       "Clear 100 floors total"),
        (Self::MasterOfTheDeep,     "Master of the Deep",     "Clear 500 floors total"),

        // Floor cleared (per-crawl)
        (Self::DungeonDiver,        "Dungeon Diver",          "Reach floor 5"),
        (Self::DungeonHobbyist,     "Dungeon Hobbyist",       "Reach floor 10"),
        (Self::DungeonExplorer,     "Dungeon Explorer",       "Reach floor 15"),
        (Self::DungeonConqueror,    "Dungeon Conqueror",      "Reach floor 20"),

        // Steps moved (all-time)
        (Self::Pedestrian,          "Pedestrian",             "Take 100 steps"),
        (Self::Wanderer,            "Wanderer",               "Take 500 steps"),
        (Self::Traveler,            "Traveler",               "Take 1,000 steps"),

        // Gold collected (all-time)
        (Self::GoldGatherer,        "Gold Gatherer",          "Collect 125 gold"),
        (Self::TreasureSeeker,      "Treasure Seeker",        "Collect 250 gold"),
        (Self::WealthAccumulator,   "Wealth Accumulator",     "Collect 375 gold"),
        (Self::RichAdventurer,      "Rich Adventurer",        "Collect 500 gold"),

        // Gold collected (per-crawl)
        (Self::GoldCollector,       "Gold Collector",         "Collect 50 gold in one crawl"),
        (Self::GoldHoarder,         "Gold Hoarder",           "Collect 100 gold in one crawl"),
        (Self::DeadBroke,           "Dead Broke",             "Collect 0 gold in one crawl"),

        // Health recovery (all-time)
        (Self::SelfCare,            "Self Care",              "Recover 20 health"),
        (Self::SpaDay,              "Spa Day",                "Recover 100 health"),
        (Self::HealerSupreme,       "Healer Supreme",         "Recover 250 health"),

        // Health recovery (per-crawl)
        (Self::Unbothered,          "Unbothered",             "Reach floor 5 without healing"),
        (Self::Survivalist,         "Survivalist",            "Reach floor 10 without healing"),
        (Self::GritAndGlory,        "Grit & Glory",           "Reach floor 15 without healing"),
        (Self::Unbreakable,         "Unbreakable",            "Reach floor 20 without healing"),

        // Total monsters defeated (all-time)
        (Self::MonsterMenace,       "Monster Menace",         "Defeat 10 monsters"),
        (Self::MonsterSlayer,       "Monster Slayer",         "Defeat 25 monsters"),
        (Self::MonsterVanquisher,   "Monster Vanquisher",     "Defeat 50 monsters"),
        (Self::MonsterExterminator, "Monster Exterminator",   "Defeat 100 monsters"),

        // Specific monsters defeated (all-time)
        (Self::GoblinSlayer,        "Goblin Slayer",          "Defeat 10 Green Goblins"),
        (Self::OrangeMenace,        "Orange Menace",          "Defeat 10 Orange Goblins"),
        (Self::BlobBuster,          "Blob Buster",            "Defeat 10 Yellow Blobs"),
        (Self::ShadeHunter,         "Shade Hunter",           "Defeat 10 Shades"),
        (Self::SpiderSquasher,      "Spider Squasher",        "Defeat 10 Spiders"),
        (Self::Ghostbuster,         "Ghostbuster",            "Defeat 10 Ghosts"),
        (Self::ZombieSlayer,        "Zombie Slayer",          "Defeat 10 Zombies"),
        (Self::BlueBlobBuster,      "Blue Blob Buster",       "Defeat 10 Blue Blobs"),
        (Self::ZombieHunter,        "Zombie Hunter",          "Defeat 10 Zombies"),
        (Self::SpectralBanisher,    "Spectral Banisher",      "Defeat 1 Spectral Ghost"),

        // Defeats by specific monsters (all-time)
        (Self::GoblinFodder,        "Goblin Fodder",          "Defeated by Green Goblin 5 times"),
        (Self::Haunted,             "Haunted",                "Defeated by Ghost 5 times"),
        (Self::Arachnophobia,       "Arachnophobia",          "Defeated by Spider 5 times"),
        (Self::Blobbed,             "Blobbed",                "Defeated by Red Blob 5 times"),
    ];

    pub fn info(self) -> (&'static str, &'static str) {
        let (_, name, description) = Self::INFO.iter().find(|a| a.0 == self).unwrap();
        (name, description)
    }

    #[rustfmt::skip]
    pub fn test(&self, crawl_stats: &DungeonStats, total_stats: &DungeonStats, did_crawl_end: bool) -> bool {
        match self {
            // Crawl completion (all-time)
            Self::WelcomeToTheDungeon => total_stats.get(DungeonStatKind::CrawlsCompleted) >= 1,
            Self::ReturningAdventurer => total_stats.get(DungeonStatKind::CrawlsCompleted) >= 2,
            Self::DedicatedDelver => total_stats.get(DungeonStatKind::CrawlsCompleted) >= 5,

            // Total floors cleared (all-time)
            Self::NoviceExplorer => total_stats.get(DungeonStatKind::FloorsCleared) >= 20,
            Self::SeasonedExplorer => total_stats.get(DungeonStatKind::FloorsCleared) >= 50,
            Self::VeteranExplorer => total_stats.get(DungeonStatKind::FloorsCleared) >= 100,
            Self::MasterOfTheDeep => total_stats.get(DungeonStatKind::FloorsCleared) >= 500,

            // Floors cleared (per-crawl)
            Self::DungeonDiver => crawl_stats.get(DungeonStatKind::FloorsCleared) >= 5,
            Self::DungeonHobbyist => crawl_stats.get(DungeonStatKind::FloorsCleared) >= 10,
            Self::DungeonExplorer => crawl_stats.get(DungeonStatKind::FloorsCleared) >= 15,
            Self::DungeonConqueror => crawl_stats.get(DungeonStatKind::FloorsCleared) >= 20,

            // Steps moved (all-time)
            Self::Pedestrian => total_stats.get(DungeonStatKind::StepsMoved) >= 100,
            Self::Wanderer => total_stats.get(DungeonStatKind::StepsMoved) >= 500,
            Self::Traveler => total_stats.get(DungeonStatKind::StepsMoved) >= 1000,

            // Gold collected (all-time)
            Self::GoldGatherer => total_stats.get(DungeonStatKind::GoldCollected) >= 125,
            Self::TreasureSeeker => total_stats.get(DungeonStatKind::GoldCollected) >= 250,
            Self::WealthAccumulator => total_stats.get(DungeonStatKind::GoldCollected) >= 375,
            Self::RichAdventurer => total_stats.get(DungeonStatKind::GoldCollected) >= 500,

            // Gold collected (per-crawl)
            Self::GoldCollector => crawl_stats.get(DungeonStatKind::GoldCollected) >= 50,
            Self::GoldHoarder => crawl_stats.get(DungeonStatKind::GoldCollected) >= 100,
            Self::DeadBroke => crawl_stats.get(DungeonStatKind::GoldCollected) == 0 && did_crawl_end,

            // Health recovery (all-time)
            Self::SelfCare => total_stats.get(DungeonStatKind::HealthRecovered) >= 20,
            Self::SpaDay => total_stats.get(DungeonStatKind::HealthRecovered) >= 100,
            Self::HealerSupreme => total_stats.get(DungeonStatKind::HealthRecovered) >= 250,

            // Health recovery (per-crawl)
            Self::Unbothered => crawl_stats.get(DungeonStatKind::FloorsCleared) >= 5 && crawl_stats.get(DungeonStatKind::HealthRecovered) == 0,
            Self::Survivalist => crawl_stats.get(DungeonStatKind::FloorsCleared) >= 10 && crawl_stats.get(DungeonStatKind::HealthRecovered) == 0,
            Self::GritAndGlory => crawl_stats.get(DungeonStatKind::FloorsCleared) >= 15 && crawl_stats.get(DungeonStatKind::HealthRecovered) == 0,
            Self::Unbreakable => crawl_stats.get(DungeonStatKind::FloorsCleared) >= 20 && crawl_stats.get(DungeonStatKind::HealthRecovered) == 0,

            // Total monsters defeated (all-time)
            Self::MonsterMenace => total_stats.total_monsters_defeated() >= 10,
            Self::MonsterSlayer => total_stats.total_monsters_defeated() >= 25,
            Self::MonsterVanquisher => total_stats.total_monsters_defeated() >= 50,
            Self::MonsterExterminator => total_stats.total_monsters_defeated() >= 100,

            // Specific monsters defeated (all-time)
            Self::GoblinSlayer => total_stats.monster_kills(MonsterKind::GreenGoblin) >= 10,
            Self::OrangeMenace => total_stats.monster_kills(MonsterKind::OrangeGoblin) >= 10,
            Self::BlobBuster => total_stats.monster_kills(MonsterKind::YellowBlob) >= 10,
            Self::ShadeHunter => total_stats.monster_kills(MonsterKind::Shade) >= 10,
            Self::SpiderSquasher => total_stats.monster_kills(MonsterKind::Spider) >= 10,
            Self::Ghostbuster => total_stats.monster_kills(MonsterKind::Ghost) >= 10,
            Self::ZombieSlayer => total_stats.monster_kills(MonsterKind::Zombie) >= 10,
            Self::BlueBlobBuster => total_stats.monster_kills(MonsterKind::BlueBlob) >= 10,
            Self::ZombieHunter => total_stats.monster_kills(MonsterKind::Zombie) >= 10,
            Self::SpectralBanisher => total_stats.monster_kills(MonsterKind::SpectralGhost) >= 1,

            // Defeats by specific monsters (all-time)
            Self::GoblinFodder => total_stats.deaths_by_monster(MonsterKind::GreenGoblin) >= 5,
            Self::Haunted => total_stats.deaths_by_monster(MonsterKind::Ghost) >= 5,
            Self::Arachnophobia => total_stats.deaths_by_monster(MonsterKind::Spider) >= 5,
            Self::Blobbed => total_stats.deaths_by_monster(MonsterKind::RedBlob) >= 5,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
struct Player {
    x: i32,
    y: i32,
    health: u32,
    max_health: u32,
    strength: u32,
    gold: u32,
    direction: Direction,
}

#[derive(
    BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
enum MonsterKind {
    GreenGoblin,
    OrangeGoblin,
    YellowBlob,
    BlueBlob,
    RedBlob,
    Shade,
    Spider,
    Ghost,
    SpectralGhost,
    Zombie,
}
impl MonsterKind {
    pub fn abbrev<'a>(&self) -> &'a str {
        match self {
            Self::BlueBlob => "B. Blob",
            Self::RedBlob => "R. Blob",
            Self::YellowBlob => "Y. Blob",
            Self::GreenGoblin => "G. Goblin",
            Self::OrangeGoblin => "O. Goblin",
            Self::Shade => "Shade",
            Self::Spider => "Spider",
            Self::Ghost => "Ghost",
            Self::SpectralGhost => "S. Ghost",
            Self::Zombie => "Zombie",
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
struct Monster {
    x: i32,
    y: i32,
    health: u32,
    max_health: u32,
    strength: u32,
    direction: Direction,
    kind: MonsterKind,
    stun_dur: u32,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
enum TreasureKind {
    Gold,
    Heal,
    HealthUp,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
struct Treasure {
    x: i32,
    y: i32,
    value: u32,
    kind: TreasureKind,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy)]
enum ObstacleKind {
    WallA,
    WallB,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
struct Obstacle {
    x: i32,
    y: i32,
    kind: ObstacleKind,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum DungeonThemeKind {
    Castle,
    Crypt,
    Pirate,
    Forest,
}
impl DungeonThemeKind {
    pub const KINDS: &'static [Self] = &[Self::Castle, Self::Crypt, Self::Pirate, Self::Forest];
    pub const THEMES: &'static [DungeonTheme] = &[
        DungeonTheme {
            particle_color: 0xb41c39ff,
            // particle_color: 0x000000ff,
            mist_color: 0xffffff09,
            dungeon_border: "castle_dungeon_nine_slice",
            floor_sprite: "floor",
            block_a_sprite: "wall",
            block_b_sprite: "firepit",
        },
        DungeonTheme {
            particle_color: 0x7b34bdff,
            mist_color: 0xffffff09,
            dungeon_border: "crypt_dungeon_nine_slice",
            floor_sprite: "dark_floor",
            // block_a_sprite: "metal_block",
            block_b_sprite: "metal_crate",
            // block_a_sprite: "dark_stone",
            // block_a_sprite: "necro_block",
            block_a_sprite: "tombstone2",
            // block_b_sprite: "crumbled_pillar",
        },
        DungeonTheme {
            particle_color: 0xff9e21ff,
            mist_color: 0xffffff09,
            dungeon_border: "pirate_dungeon_nine_slice",
            floor_sprite: "wood_floor",
            block_a_sprite: "crate",
            block_b_sprite: "barrel",
        },
        DungeonTheme {
            particle_color: 0x6ab2c5ff,
            mist_color: 0xffffff09,
            dungeon_border: "forest_dungeon_nine_slice",
            floor_sprite: "floor_forest",
            block_a_sprite: "shrub",
            block_b_sprite: "stump",
        },
    ];
    pub fn theme(&self) -> DungeonTheme {
        match self {
            Self::Castle => Self::THEMES[0],
            Self::Crypt => Self::THEMES[1],
            Self::Pirate => Self::THEMES[2],
            Self::Forest => Self::THEMES[3],
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct DungeonTheme {
    particle_color: u32,
    mist_color: u32,
    dungeon_border: &'static str,
    floor_sprite: &'static str,
    block_a_sprite: &'static str,
    block_b_sprite: &'static str,
}

#[derive(
    BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
enum DungeonStatKind {
    CrawlsCompleted,
    FloorsCleared,
    HealthRecovered,
    GoldCollected,
    DamageDealt,
    StepsMoved,
    DamageTaken,
    Defeated(MonsterKind),
    DefeatedBy(MonsterKind),
}
impl DungeonStatKind {
    pub const ALL: &'static [Self] = &[
        Self::CrawlsCompleted,
        Self::FloorsCleared,
        Self::HealthRecovered,
        Self::GoldCollected,
        Self::DamageDealt,
        Self::DamageTaken,
        Self::StepsMoved,
        Self::Defeated(MonsterKind::BlueBlob),
        Self::Defeated(MonsterKind::Ghost),
        Self::Defeated(MonsterKind::GreenGoblin),
        Self::Defeated(MonsterKind::OrangeGoblin),
        Self::Defeated(MonsterKind::RedBlob),
        Self::Defeated(MonsterKind::Shade),
        Self::Defeated(MonsterKind::SpectralGhost),
        Self::Defeated(MonsterKind::Spider),
        Self::Defeated(MonsterKind::YellowBlob),
        Self::Defeated(MonsterKind::Zombie),
        Self::DefeatedBy(MonsterKind::BlueBlob),
        Self::DefeatedBy(MonsterKind::Ghost),
        Self::DefeatedBy(MonsterKind::GreenGoblin),
        Self::DefeatedBy(MonsterKind::OrangeGoblin),
        Self::DefeatedBy(MonsterKind::RedBlob),
        Self::DefeatedBy(MonsterKind::Shade),
        Self::DefeatedBy(MonsterKind::SpectralGhost),
        Self::DefeatedBy(MonsterKind::Spider),
        Self::DefeatedBy(MonsterKind::YellowBlob),
        Self::DefeatedBy(MonsterKind::Zombie),
    ];
    pub const DEFEATED: &'static [Self] = &[
        Self::Defeated(MonsterKind::BlueBlob),
        Self::Defeated(MonsterKind::Ghost),
        Self::Defeated(MonsterKind::GreenGoblin),
        Self::Defeated(MonsterKind::OrangeGoblin),
        Self::Defeated(MonsterKind::RedBlob),
        Self::Defeated(MonsterKind::Shade),
        Self::Defeated(MonsterKind::SpectralGhost),
        Self::Defeated(MonsterKind::Spider),
        Self::Defeated(MonsterKind::YellowBlob),
        Self::Defeated(MonsterKind::Zombie),
    ];
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
struct DungeonStats {
    entries: BTreeMap<String, u32>,
}
impl DungeonStats {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }
    pub fn get(&self, kind: DungeonStatKind) -> u32 {
        let key = format!("{kind:?}");
        *self.entries.get(&key).unwrap_or(&0)
    }
    pub fn total_monsters_defeated(&self) -> u32 {
        let mut total = 0;
        for kind in DungeonStatKind::DEFEATED {
            total += self.get(*kind);
        }
        total
    }
    pub fn monster_kills(&self, monster_kind: MonsterKind) -> u32 {
        self.get(DungeonStatKind::Defeated(monster_kind))
    }
    pub fn deaths_by_monster(&self, monster_kind: MonsterKind) -> u32 {
        self.get(DungeonStatKind::DefeatedBy(monster_kind))
    }
    pub fn increment(&mut self, kind: DungeonStatKind, amount: u32) {
        let key = format!("{kind:?}");
        self.entries
            .entry(key)
            .and_modify(|n| *n = *n + amount)
            .or_insert(amount);
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
struct Dungeon {
    crawl_id: u32,
    theme: DungeonThemeKind,
    floor: u32,
    turn: u32,
    width: u32,
    height: u32,
    player: Player,
    monsters: Vec<Monster>,
    treasures: Vec<Treasure>,
    obstacles: Vec<Obstacle>,
    logs: Vec<String>,
    exit_key: Option<(i32, i32)>,
    exit: Option<(i32, i32)>,
    stats: DungeonStats,
    total_stats: DungeonStats,
    // TODO: move achievements to own struct/file
    unlocked: PlayerAchievements,
    all_unlocked: PlayerAchievements,
}
impl Dungeon {
    fn move_player(&mut self, direction: Direction, log: fn(&str)) -> bool {
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
                self.logs.push(msg);
                let amount = self.player.strength;
                let msg = format!("P1 did {amount} damage.");
                log(&msg);
                self.logs.push(msg);
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
                        self.logs.push(msg);
                        (*self_ptr).increment_stats(DungeonStatKind::Defeated(monster.kind), 1);
                    }
                }
            }

            // If all monsters are defeated, spawn a treasure
            if self.monsters.iter().all(|m| m.health == 0) {
                match program::random_number::<u8>() % 3 {
                    2 => {
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
        // self.logs.push(msg);
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
                        self.logs.push(msg);
                    }
                    TreasureKind::Heal => {
                        let prev_player_health = self.player.health;
                        self.player.health =
                            (self.player.health + amount).min(self.player.max_health);
                        let recovered_health = prev_player_health.abs_diff(self.player.health);
                        self.increment_stats(DungeonStatKind::HealthRecovered, recovered_health);
                        let msg = format!("Recovered {} HP!", recovered_health);
                        log(&msg);
                        self.logs.push(msg);
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
                        self.logs.push(msg);
                    }
                }
            }
            self.treasures.retain_mut(|t| t.x != new_x || t.y != new_y);
        }

        if self.is_exit_key(new_x, new_y) {
            let msg = "Found exit key.".to_string();
            log(&msg);
            self.logs.push(msg);
            self.exit_key = None;
            let (max_x, max_y) = self.bounds();
            // Initialize exit position at least 8 tiles away from player
            let min_distance = (self.width.min(self.height) / 2) as i32;
            loop {
                let x = program::random_number::<i32>().abs() % max_x;
                let y = program::random_number::<i32>().abs() % max_y;
                let dx = (x - new_x).abs();
                let dy = (y - new_y).abs();
                if dx + dy >= min_distance && !self.is_position_occupied(x, y) {
                    self.exit = Some((x, y));
                    break;
                }
            }
            let msg = "Hidden stairs appeared!".to_string();
            log(&msg);
            self.logs.push(msg);
        }

        return true;
    }
    fn move_monsters(&mut self, log: fn(&str)) {
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
                self.logs.push(msg);
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
                player.health = player.health.saturating_sub(monster.strength);
                let damage = prev_player_health.abs_diff(player.health);
                self.increment_stats(DungeonStatKind::DamageTaken, damage);

                let msg = format!("{} did {} damage.", monster_name, damage);
                log(&msg);
                self.logs.push(msg);
                if player.health == 0 {
                    let msg = "P1 died.".to_string();
                    log(&msg);
                    self.logs.push(msg);
                    self.increment_stats(DungeonStatKind::DefeatedBy(monster.kind), 1);
                }
                return true;
            }

            // Movement based on monster kind
            let (dir, mx, my) = match monster.kind {
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
                        let (dir, mx, my) = match program::random_number::<usize>() % 4 {
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
                    // Moves up to 3 spaces in one direction towards the player every 4 turns
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
                                program::random_number::<i32>().abs() % 2 == 0
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
                        monster.kind = MonsterKind::SpectralGhost;
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
                        let (dir, mx, my) = match program::random_number::<usize>() % 4 {
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
                _ => {
                    // Moves towards the player each turn
                    let dx = player.x - mx;
                    let dy = player.y - my;

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
    fn is_player(&self, x: i32, y: i32) -> bool {
        self.player.x == x && self.player.y == y
    }
    fn bounds(&self) -> (i32, i32) {
        let max_x = self.width as i32 - 1;
        let max_y = self.height as i32 - 1;
        (max_x, max_y)
    }
    fn is_out_of_bounds(&self, x: i32, y: i32) -> bool {
        let (min_x, min_y) = (0, 0);
        let (max_x, max_y) = self.bounds();
        x < min_x || y < min_y || x > max_x || y > max_y
    }
    fn is_obstacle(&self, x: i32, y: i32) -> bool {
        self.obstacles.iter().any(|obs| obs.x == x && obs.y == y)
    }
    fn is_monster(&self, x: i32, y: i32) -> bool {
        self.monsters
            .iter()
            .any(|mon| mon.x == x && mon.y == y && mon.health > 0)
    }
    fn is_treasure(&self, x: i32, y: i32) -> bool {
        self.treasures.iter().any(|t| t.x == x && t.y == y)
    }
    fn is_exit_key(&self, x: i32, y: i32) -> bool {
        self.exit_key.map_or(false, |a| a.0 == x && a.1 == y)
    }
    fn is_exit(&self, x: i32, y: i32) -> bool {
        self.exit.map_or(false, |a| a.0 == x && a.1 == y)
    }
    fn is_position_blocked(&self, x: i32, y: i32) -> bool {
        self.is_obstacle(x, y) || self.is_monster(x, y) || self.is_player(x, y)
    }
    fn is_position_occupied(&self, x: i32, y: i32) -> bool {
        self.is_position_blocked(x, y)
            || self.is_treasure(x, y)
            || self.is_exit_key(x, y)
            || self.is_exit(x, y)
    }
    fn increment_stats(&mut self, kind: DungeonStatKind, amount: u32) {
        if amount > 0 {
            self.stats.increment(kind, amount);
            self.total_stats.increment(kind, amount);
        }
    }
}

#[derive(
    BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
enum LeaderboardKind {
    HighestFloor,
    MostGold,
    MostKills,
    LeastSteps,
}
impl LeaderboardKind {
    pub const ALL: &'static [Self] = &[
        Self::HighestFloor,
        Self::MostGold,
        Self::MostKills,
        Self::LeastSteps,
    ];
    pub fn is_most(&self) -> bool {
        match self {
            Self::LeastSteps => false,
            _ => true,
        }
    }
    pub fn next(&self) -> Self {
        let i = Self::ALL.binary_search(&self).unwrap() + 1;
        let len = Self::ALL.len();
        Self::ALL[i % len]
    }
    pub fn prev(&self) -> Self {
        let i = Self::ALL.binary_search(&self).unwrap();
        let len = Self::ALL.len();
        Self::ALL[(i + len - 1) % len]
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
struct LeaderboardEntry {
    name: String,
    score: u32,
    crawl_id: u32,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
struct Leaderboard {
    entries: BTreeMap<String, Vec<LeaderboardEntry>>,
}
impl Leaderboard {
    const LEADERBOARD_SIZE: usize = 10;
    fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }
    fn update(&mut self, crawl_id: u32, kind: LeaderboardKind, name: &str, score: u32) {
        let key = format!("{kind:?}");
        let entry = LeaderboardEntry {
            name: name.to_string(),
            score: score,
            crawl_id,
        };
        self.entries
            .entry(key)
            .and_modify(|leaderboard| {
                leaderboard.push(entry.clone());
                leaderboard.sort_by(|a, b| {
                    if kind.is_most() {
                        b.score.cmp(&a.score)
                    } else {
                        a.score.cmp(&b.score)
                    }
                });
                if leaderboard.len() > Self::LEADERBOARD_SIZE {
                    leaderboard.truncate(Self::LEADERBOARD_SIZE);
                }
            })
            .or_insert(vec![entry.clone()]);
    }
    fn render_entries(
        &self,
        crawl_id: u32,
        mut i: i32,
        kind: LeaderboardKind,
        name: &str,
        x: i32,
        y: i32, // 9
    ) {
        let key = format!("{kind:?}");
        let mut rank = 0;
        let mut prev_value = if kind.is_most() { u32::MAX } else { 0 };
        for entry in self.entries.get(&key).unwrap_or(&vec![]) {
            // Only increment rank if the value changes
            if kind.is_most() && entry.score < prev_value {
                rank += 1;
                prev_value = entry.score;
            } else if !kind.is_most() && entry.score > prev_value {
                rank += 1;
                prev_value = entry.score;
            } else if rank == 0 {
                rank += 1;
                prev_value = entry.score;
            }
            let color: u32 = if crawl_id == entry.crawl_id && tick() % 16 < 8 {
                0x1e6f50ff
            } else if entry.name == name {
                0x6ecb62ff
            } else {
                0xacaabdff
            };
            text!(
                "{} {}{:.8} {:>11} ",
                rank,
                if rank > 9 { "" } else { " " },
                entry.name,
                &format!("{}", entry.score);
                absolute = true,
                x = x + 8,
                y = y + i * 10,
                color = color
            );

            i += 1;
            if i - 3 > 10 {
                break;
            }
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
struct CreateNewDungeonCommandInput {
    reset: bool,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
struct MovePlayerCommandInput {
    direction: Direction,
}

struct DungeonDashProgram;
impl DungeonDashProgram {
    pub const PROGRAM_ID: &'static str = "dungeon_dash";
    pub const VERSION: usize = 1;
    // Client methods
    pub fn fetch_player_dungeon(user_id: &str) -> Result<Dungeon, os::ReadFileError> {
        let filepath = Self::get_dungeon_filepath(&user_id);
        let file = os::read_file(Self::PROGRAM_ID, &filepath)?;
        Dungeon::try_from_slice(&file.contents)
            .map_err(|err| os::ReadFileError::ParsingError(err.to_string()))
    }
    pub fn fetch_global_leaderboard() -> Result<Leaderboard, os::ReadFileError> {
        let filepath = "leaderboard";
        let file = os::read_file(Self::PROGRAM_ID, &filepath)?;
        Leaderboard::try_from_slice(&file.contents)
            .map_err(|err| os::ReadFileError::ParsingError(err.to_string()))
    }
    pub fn create_new_dungeon(input: CreateNewDungeonCommandInput) -> String {
        os::exec(
            Self::PROGRAM_ID,
            "create_new_dungeon",
            &input.try_to_vec().unwrap(),
        )
    }
    pub fn move_player(input: MovePlayerCommandInput) -> String {
        os::exec(
            Self::PROGRAM_ID,
            "move_player",
            &input.try_to_vec().unwrap(),
        )
    }
    // Dungeon
    fn get_dungeon_filepath(user_id: &str) -> String {
        format!("v{}/{}.dungeon", Self::VERSION, user_id)
    }
    fn load_player_dungeon(user_id: &str) -> Result<Dungeon, &'static str> {
        let filepath = Self::get_dungeon_filepath(user_id);
        let data = program::read_file(&filepath)?;
        match Dungeon::try_from_slice(&data) {
            Ok(dungeon) => Ok(dungeon),
            Err(err) => {
                program::log(&err.to_string());
                Err("Failed to deserialize dungeon")
            }
        }
    }
    fn save_player_dungeon(user_id: &str, dungeon: &Dungeon) {
        let filepath = Self::get_dungeon_filepath(user_id);
        let data = dungeon.try_to_vec().unwrap();
        program::write_file(&filepath, &data).unwrap()
    }
    // Leaderboard
    fn get_global_leaderboard_filepath() -> String {
        "leaderboard".to_string()
    }
    fn load_global_leaderboard() -> Result<Leaderboard, &'static str> {
        let filepath = Self::get_global_leaderboard_filepath();
        let data = program::read_file(&filepath)?;
        match Leaderboard::try_from_slice(&data) {
            Ok(leaderboard) => Ok(leaderboard),
            Err(err) => {
                program::log(&err.to_string());
                Err("Failed to deserialize global leaderboard")
            }
        }
    }
    fn save_global_leaderboard(leaderboard: &Leaderboard) {
        let filepath = Self::get_global_leaderboard_filepath();
        let data = leaderboard.try_to_vec().unwrap();
        program::write_file(&filepath, &data).unwrap()
    }
    // Stats
    fn get_player_stats_filepath(user_id: &str) -> String {
        format!("/users/{}/v{}/stats", user_id, Self::VERSION)
    }
    fn load_player_stats(user_id: &str) -> Result<DungeonStats, &'static str> {
        let filepath = Self::get_player_stats_filepath(user_id);
        let data = program::read_file(&filepath)?;
        match DungeonStats::try_from_slice(&data) {
            Ok(stats) => Ok(stats),
            Err(err) => {
                program::log(&err.to_string());
                Err("Failed to deserialize dungeon stats")
            }
        }
    }
    fn save_player_stats(user_id: &str, stats: &DungeonStats) {
        let filepath = Self::get_player_stats_filepath(user_id);
        let data = stats.try_to_vec().unwrap();
        program::write_file(&filepath, &data).unwrap()
    }
    // Achievements
    fn get_player_achievements_filepath(user_id: &str) -> String {
        format!("/users/{}/v{}/achievements", user_id, Self::VERSION)
    }
    fn load_player_achievements(user_id: &str) -> Result<PlayerAchievements, &'static str> {
        let filepath = Self::get_player_achievements_filepath(user_id);
        let data = program::read_file(&filepath)?;
        match PlayerAchievements::try_from_slice(&data) {
            Ok(achievements) => Ok(achievements),
            Err(err) => {
                program::log(&err.to_string());
                Err("Failed to deserialize player achievements")
            }
        }
    }
    fn save_player_achievements(user_id: &str, achievements: &PlayerAchievements) {
        let filepath = Self::get_player_achievements_filepath(user_id);
        let data = achievements.try_to_vec().unwrap();
        program::write_file(&filepath, &data).unwrap()
    }
    #[export_name = "turbo/create_new_dungeon"]
    unsafe extern "C" fn on_create_new_dungeon() -> usize {
        // Get player id
        let user_id = program::get_user_id();

        // Parse command input
        program::log("Parsing command input...");
        let input_bytes = program::get_input_data();
        let input = match CreateNewDungeonCommandInput::try_from_slice(&input_bytes) {
            Ok(input) => input,
            Err(err) => {
                program::log(&err.to_string());
                return program::CANCEL;
            }
        };

        // Create a default dungeon
        let mut dungeon = if input.reset {
            // program::write_file(&Self::get_player_achievements_filepath(&user_id), &[]).unwrap();
            // program::write_file(&Self::get_player_stats_filepath(&user_id), &[]).unwrap();
            let w = 5;
            let h = 5;
            Dungeon {
                crawl_id: program::random_number::<u32>(),
                theme: DungeonThemeKind::Castle,
                floor: 0,
                turn: 0,
                width: w,
                height: h,
                player: Player {
                    x: program::random_number::<i32>().abs() % w as i32,
                    y: program::random_number::<i32>().abs() % h as i32,
                    health: 8,
                    max_health: 8,
                    strength: 1,
                    gold: 0,
                    direction: Direction::Down,
                },
                monsters: vec![],
                treasures: vec![],
                obstacles: vec![],
                logs: vec![],
                exit: None,
                exit_key: None,
                stats: DungeonStats::new(),
                total_stats: Self::load_player_stats(&user_id).unwrap_or(DungeonStats::new()),
                unlocked: PlayerAchievements::empty(),
                all_unlocked: Self::load_player_achievements(&user_id)
                    .unwrap_or(PlayerAchievements::empty()),
            }
        } else {
            // Load player dungeon
            program::log("Loading the dungeon...");
            let mut dungeon = match Self::load_player_dungeon(&user_id) {
                Ok(dungeon) => dungeon,
                Err(err) => {
                    program::log(err);
                    return program::CANCEL;
                }
            };

            if !dungeon.is_exit(dungeon.player.x, dungeon.player.y) {
                program::log("P1 has not reached the exit.");
                return program::CANCEL;
            }
            // Remove exit
            dungeon.exit = None;
            // Clear monsters, treasures, and obstacles
            dungeon.monsters.clear();
            dungeon.treasures.clear();
            dungeon.obstacles.clear();
            // Clear logs
            dungeon.logs.clear();
            // Increase floor
            dungeon.floor += 1;
            dungeon.increment_stats(DungeonStatKind::FloorsCleared, 1);
            // Update dungeon theme
            let i = program::random_number::<usize>();
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
            // program::log(&format!("Crawl stats: {:?}", dungeon.stats));
            // program::log(&format!("Total stats: {:?}", dungeon.total_stats));
            let next_achievements =
                dungeon
                    .unlocked
                    .apply_dungeon_stats(&dungeon.stats, &dungeon.total_stats, false);
            let floor_achievements =
                next_achievements.difference(&dungeon.unlocked.union(&dungeon.all_unlocked));
            dungeon.unlocked = next_achievements.difference(&dungeon.all_unlocked);
            program::log(&format!(
                "Achievements (floor): {:?}",
                floor_achievements.achievement_kinds()
            ));
            program::log(&format!(
                "Achievements (crawl): {:?}",
                dungeon.unlocked.achievement_kinds()
            ));
            program::log(&format!(
                "Achievements (all): {:?}",
                dungeon.all_unlocked.achievement_kinds()
            ));
            // dungeon.all_unlocked = dungeon.all_unlocked.union(&dungeon.unlocked);
            // program::log(&format!(
            //     "Achievements all-time: {:?}",
            //     dungeon.all_unlocked.achievement_kinds()
            // ));

            dungeon
        };

        // Get the dungeon bounds
        let (max_x, max_y) = dungeon.bounds();

        let magic_ratio = ((max_x * max_y) / 32) as usize;

        // After first floor, add monsters and treasures
        if dungeon.floor > 0 {
            // Randomize monsters
            program::log("Randomizing monsters...");
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
            };
            let total_weight: u32 = monster_weights.iter().map(|(weight, _)| *weight).sum();

            while dungeon.monsters.len() < num_monsters {
                let x = program::random_number::<i32>().abs() % max_x;
                let y = program::random_number::<i32>().abs() % max_y;
                if !dungeon.is_position_occupied(x, y) {
                    // Generate a random number within the total weight
                    let rng = program::random_number::<u32>() % total_weight;
                    let mut selected_monster = MonsterKind::GreenGoblin;
                    // Select the monster based on weighted probability
                    let mut cumulative_weight = 0;
                    for (weight, monster_kind) in monster_weights {
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
            program::log("Randomizing treasures...");
            let num_treasures = magic_ratio + (dungeon.floor as usize / 2);
            while dungeon.treasures.len() < num_treasures {
                let x = program::random_number::<i32>().abs() % max_x;
                let y = program::random_number::<i32>().abs() % max_y;
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
                        let n = program::random_number::<u32>() % 10;
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
        program::log("Initializing exit key position...");
        let min_distance = (dungeon.width.min(dungeon.height) / 2) as i32;
        loop {
            let x = program::random_number::<i32>().abs() % max_x;
            let y = program::random_number::<i32>().abs() % max_y;
            let dx = (x - dungeon.player.x).abs();
            let dy = (y - dungeon.player.y).abs();
            if dx + dy >= min_distance && !dungeon.is_position_occupied(x, y) {
                dungeon.exit_key = Some((x, y));
                break;
            }
        }

        // Randomize obstacles
        program::log("Randomizing obstacles...");
        for (x, y) in generate_maze(max_x as usize, max_y as usize) {
            // 1/3 chance to skip a obstacle placement
            if program::random_number::<u8>() % 3 == 0 {
                continue;
            }
            // Make sure spot is empty
            if dungeon.is_position_occupied(x, y) {
                continue;
            }
            dungeon.obstacles.push(Obstacle {
                x,
                y,
                kind: if program::random_number::<usize>() % 10 == 9 {
                    // 10% chance for firepit
                    ObstacleKind::WallB
                } else {
                    // 90% chance for stone block
                    ObstacleKind::WallA
                },
            });
        }

        // Save the dungeon
        program::log("Saving dungeon...");
        let user_id = program::get_user_id();
        Self::save_player_dungeon(&user_id, &dungeon);

        // Commit the command result
        return program::COMMIT;
    }

    #[export_name = "turbo/move_player"]
    unsafe extern "C" fn on_move_player() -> usize {
        // Parse command input
        program::log("Parsing command input...");
        let input_bytes = program::get_input_data();
        let input = match MovePlayerCommandInput::try_from_slice(&input_bytes) {
            Ok(input) => input,
            Err(err) => {
                program::log(&err.to_string());
                return program::CANCEL;
            }
        };

        // Load player dungeon
        program::log("Loading the dungeon...");
        let user_id = program::get_user_id();
        let mut dungeon = match Self::load_player_dungeon(&user_id) {
            Ok(dungeon) => dungeon,
            Err(err) => {
                program::log(err);
                return program::CANCEL;
            }
        };

        // Cancel command if player has already won or lost
        program::log("Checking game over conditions...");
        if dungeon.player.health == 0 {
            program::log("P1 has died. Game over.");
            return program::CANCEL;
        }

        // Move player
        program::log("Moving player...");
        if !dungeon.move_player(input.direction, program::log) {
            return program::CANCEL;
        }

        // Move monsters if player has not reached the exit
        if !dungeon.is_exit(dungeon.player.x, dungeon.player.y) {
            program::log("Moving monsters...");
            dungeon.move_monsters(program::log);
        } else {
            let msg = "P1 reached exit.".to_string();
            program::log(&msg);
            dungeon.logs.push(msg);
        }

        // Truncate dungeon logs
        program::log("Truncating logs...");
        let num_logs = dungeon.logs.len();
        let log_limit = 23;
        if num_logs > log_limit {
            dungeon.logs = dungeon.logs.split_off(num_logs - log_limit);
        }

        // Increment turn
        program::log("Incrementing turn number...");
        dungeon.turn += 1;

        // If player died...
        if dungeon.player.health == 0 {
            // Increment dungeon stats (crawls completed)
            dungeon.increment_stats(DungeonStatKind::CrawlsCompleted, 1);

            // Update the global leaderboard
            program::log("Reading global leaderboard...");
            let mut leaderboard = Self::load_global_leaderboard().unwrap_or(Leaderboard::new());
            program::log("Updating global leaderboard...");
            leaderboard.update(
                dungeon.crawl_id,
                LeaderboardKind::HighestFloor,
                &user_id,
                dungeon.stats.get(DungeonStatKind::FloorsCleared) + 1,
            );
            leaderboard.update(
                dungeon.crawl_id,
                LeaderboardKind::MostGold,
                &user_id,
                dungeon.stats.get(DungeonStatKind::GoldCollected),
            );
            leaderboard.update(
                dungeon.crawl_id,
                LeaderboardKind::MostKills,
                &user_id,
                dungeon.stats.total_monsters_defeated(),
            );
            leaderboard.update(
                dungeon.crawl_id,
                LeaderboardKind::LeastSteps,
                &user_id,
                dungeon.stats.get(DungeonStatKind::StepsMoved),
            );
            program::log("Saving global leaderboard...");
            Self::save_global_leaderboard(&leaderboard);

            // Update player stats
            program::log("Saving player stats...");
            Self::save_player_stats(&user_id, &dungeon.total_stats);

            // Unlock achievements
            let next_achievements =
                dungeon
                    .unlocked
                    .apply_dungeon_stats(&dungeon.stats, &dungeon.total_stats, true);
            dungeon.unlocked = next_achievements.difference(&dungeon.all_unlocked);
            program::log(&format!(
                "Achievements (crawl): {:?}",
                dungeon.unlocked.achievement_kinds()
            ));
            dungeon.all_unlocked = dungeon.all_unlocked.union(&dungeon.unlocked);
            program::log(&format!(
                "Achievements (all): {:?}",
                dungeon.all_unlocked.achievement_kinds()
            ));
            program::log("Saving player achievements...");
            Self::save_player_achievements(&user_id, &dungeon.all_unlocked);
        }

        // Save the dungeon
        program::log("Saving the dungeon...");
        Self::save_player_dungeon(&user_id, &dungeon);

        // Commit the command result
        return program::COMMIT;
    }
}

fn generate_maze(width: usize, height: usize) -> Vec<(i32, i32)> {
    let mut grid = vec![vec![false; width]; height];
    let mut walls = vec![];

    fn divide(
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

        let horizontal = program::random_number::<u8>() % 2 == 0;

        if horizontal {
            let max_wall_y = y + height - 2;
            let min_wall_y = y + 1;
            if max_wall_y < min_wall_y {
                return;
            }
            let wall_y = min_wall_y
                + (program::random_number::<usize>() % ((max_wall_y - min_wall_y) / 2 + 1)) * 2;

            for i in x..x + width {
                grid[wall_y][i] = true;
                walls.push((i as i32, wall_y as i32));
            }

            let passage_x = x + program::random_number::<usize>() % width;
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
                + (program::random_number::<usize>() % ((max_wall_x - min_wall_x) / 2 + 1)) * 2;

            for i in y..y + height {
                grid[i][wall_x] = true;
                walls.push((wall_x as i32, i as i32));
            }

            let passage_y = y + program::random_number::<usize>() % height;
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

// Function to wrap text into lines that fit within max_width
fn wrap_text(text: &str, max_width: u32, font: Font) -> Vec<String> {
    // Determine the character width based on the font
    let char_width = match font {
        Font::S => 5,
        Font::M => 5,
        Font::L => 8,
        Font::XL => 16,
    };

    // Calculate the maximum number of characters per line
    let max_chars_per_line = (max_width / char_width) as usize;

    let mut lines = Vec::new();
    let mut current_line = String::new();

    let mut i = 0;
    for word in text.split_whitespace() {
        // Add a space if the line is not empty
        if !current_line.is_empty() {
            current_line.push(' ');
        }

        // Check if adding the next word exceeds the maximum line length
        if current_line.len() + word.len() > max_chars_per_line {
            // If current line is not empty, push it to lines
            if !current_line.is_empty() {
                lines.push(current_line.trim().to_string());
                current_line.clear();
            }
        }

        current_line.push_str(word);
        i += 1;
    }

    // Add any remaining text to lines
    if !current_line.is_empty() {
        lines.push(current_line.trim().to_string());
    }

    lines
}
