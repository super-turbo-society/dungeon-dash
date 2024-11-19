use super::*;
use std::f32::consts::PI;

pub fn render(state: &mut LocalState, user_id: &str, dungeon: &Dungeon) {
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

    // Update turn
    state.turn.set(dungeon.turn);

    // Update achievements modal
    if dungeon.player.health == 0
        && state.turn.done()
        && state.last_crawl_achievements_modal != dungeon.crawl_id
        && state.achievements_modal.is_none()
        && !dungeon.unlocked.is_empty()
    {
        let modal = AchievementsModal::new(&dungeon.unlocked.achievement_kinds());
        state.achievements_modal = Some(modal);
        state.last_crawl_achievements_modal = dungeon.crawl_id;
    }

    // Update player tweens
    state.players[0].x.set(dungeon.player.x * TILE_SIZE);
    state.players[0].y.set(dungeon.player.y * TILE_SIZE);

    // Player "nudge" animation
    if (!state.players[0].y.done() || !state.players[0].x.done())
        && state.players[0].offset_y.done()
    {
        state.players[0].offset_y.set(-MOVE_Y_OFFSET);
    }
    if state.players[0].offset_x.done() && state.players[0].offset_x.get() != 0 {
        state.players[0].offset_x.set(0);
    }
    if state.players[0].offset_y.done() && state.players[0].offset_y.get() != 0 {
        state.players[0].offset_y.set(0);
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
                emote: None,
            })
        }
    }
    if state.players[0].is_idle() {
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

    // let did_turn_transition_end = state.turn.done();
    // let was_last_exec_on_diff_turn = state.last_exec_turn.map_or(true, |t| t != dungeon.turn);
    let did_exec_timeout = (tick() - state.last_exec_at) >= EXEC_TIMEOUT_DUR;
    // let is_ready_to_exec =
    //     did_turn_transition_end && (was_last_exec_on_diff_turn || did_exec_timeout);
    let is_ready_to_exec = did_exec_timeout;
    let is_alive = dungeon.player.health > 0;

    // Handle player input
    let gp = gamepad(0);

    // Hard reset game
    if gp.start.just_pressed() && gp.select.pressed() {
        client::commands::create_new_dungeon::exec(true);
        state.last_exec_at = tick();
        state.last_exec_turn = Some(dungeon.turn);
    }
    // Dungeon controls
    else if is_ready_to_exec {
        // Next floor or restart
        if gp.start.just_pressed() && state.achievements_modal.is_none() {
            client::commands::create_new_dungeon::exec(dungeon.player.health == 0);
            state.last_exec_at = tick();
            state.last_exec_turn = Some(dungeon.turn);
        }
        // Move
        else if gp.up.pressed() && is_alive {
            client::commands::move_player::exec(Direction::Up);
            state.last_exec_at = tick();
            state.last_exec_turn = Some(dungeon.turn);
            if dungeon.is_position_blocked(dungeon.player.x, dungeon.player.y - 1) {
                state.players[0].offset_y.set(-MOVE_Y_OFFSET);
            }
        } else if gp.down.pressed() && is_alive {
            client::commands::move_player::exec(Direction::Down);
            state.last_exec_at = tick();
            state.last_exec_turn = Some(dungeon.turn);
            if dungeon.is_position_blocked(dungeon.player.x, dungeon.player.y + 1) {
                state.players[0].offset_y.set(MOVE_Y_OFFSET);
            }
        } else if gp.left.pressed() && is_alive {
            client::commands::move_player::exec(Direction::Left);
            state.last_exec_at = tick();
            state.last_exec_turn = Some(dungeon.turn);
            if dungeon.is_position_blocked(dungeon.player.x - 1, dungeon.player.y) {
                state.players[0].offset_x.set(-MOVE_X_OFFSET);
            }
        } else if gp.right.pressed() && is_alive {
            client::commands::move_player::exec(Direction::Right);
            state.last_exec_at = tick();
            state.last_exec_turn = Some(dungeon.turn);
            if dungeon.is_position_blocked(dungeon.player.x + 1, dungeon.player.y) {
                state.players[0].offset_x.set(MOVE_X_OFFSET);
            }
        }
    }

    // Center camera on player
    set_cam!(
        x = state.players[0].x.get() + (TILE_SIZE / 2),
        y = {
            let n = state.players[0].y.get() + (TILE_SIZE / 2) + menubar_h;
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
        let x = state.players[0].x.get();
        let y = state.players[0].y.get();
        sprite!(
            "dotted_tile_border",
            x = x,
            y = y,
            opacity = 0.25,
            fps = fps::FAST,
        );
        let x = x + state.players[0].offset_x.get();
        // let y = y - 9;
        ellipse!(
            x = x + 2,
            y = y + 3,
            w = TILE_SIZE - 4,
            h = TILE_SIZE - 4,
            color = SHADOW_COLOR,
        );
        let y = y + state.players[0].offset_y.get() - 4;
        sprite!("hero", x = x, y = y, fps = fps::FAST);
        // if user_id == "00000000-0000-0000-0000-000000000000" {
        if user_id == "79d09d42-6f28-4a3c-a99d-1a8544da9572" {
            sprite!("crown", x = x, y = y, fps = fps::FAST);
        }
    } else {
        sprite!(
            "tombstone",
            x = state.players[0].x.get(),
            y = state.players[0].y.get() - 5,
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
            MonsterKind::EvilTurbi => {
                ellipse!(
                    x = x + 2,
                    y = y + 8,
                    w = TILE_SIZE - 4,
                    h = TILE_SIZE - 4,
                    color = SHADOW_COLOR,
                );
                sprite!(
                    "evil_turbi",
                    x = x,
                    y = y,
                    fps = fps::FAST,
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
                let color: u32 = if layer == 0 {
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
        if let Some(user_id) = &os::client::user_id() {
            if gp.right.just_pressed() {
                state.leaderboard_kind = state.leaderboard_kind.prev();
            }
            if gp.left.just_pressed() {
                state.leaderboard_kind = state.leaderboard_kind.next();
            }
            if let Ok(leaderboard) = client::queries::global_leaderboard::fetch() {
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
    let hp_color: u32 = match dungeon.player.health as f32 / dungeon.player.max_health as f32 {
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
            client::commands::create_new_dungeon::exec(true);
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
            client::commands::create_new_dungeon::exec(false);
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
    if let Some(user_id) = &os::client::user_id() {
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
        let color: u32 = if t % 128 < 64 { 0xbd59deff } else { 0x7b34bdff };
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
        let percent = dungeon.all_unlocked.len() as f32 / AchievementKind::INFO.len() as f32;
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

    // Pause button
    let gp = gamepad(0);
    let m = mouse(0);
    sprite!("pause_icon", absolute = true, x = w - 18, y = 2);
    if !state.show_stats_modal {
        if gp.select.just_pressed()
            || (m.intersects_abs(w as i32 - 18, 2, 16, 16) && m.left.just_pressed())
        {
            state.show_stats_modal = true;
        }
    } else {
        // Background overlay
        rect!(w = w, h = h, color = 0x000000fe, absolute = true);
        // Render the modal
        let modal_w = w - 8;
        let modal_h = 180; //h - 64;
        let modal_x = ((w / 2) - (modal_w / 2)) as i32;
        let modal_y = 4;
        let modal_bg_color = 0x1a1932ff;

        if gp.select.just_pressed()
            || (!m.intersects_abs(modal_x, modal_y, modal_w, modal_h) && m.left.just_pressed())
        {
            state.show_stats_modal = false;
        }
        rect!(
            w = modal_w,
            h = modal_h,
            x = modal_x,
            y = modal_y,
            color = modal_bg_color,
            border_radius = 8,
            absolute = true
        );
        #[rustfmt::skip]
        circ!(absolute = true, d = 16, x = w - 16, y = 0, color = modal_bg_color);
        #[rustfmt::skip]
        sprite!("close_icon", absolute = true, x = w - 16, y = 0, color = 0xffffffaa);
        if ui::clickable(w as i32 - 16, 0, 16, 16) {
            state.show_stats_modal = false;
        }
        if let Ok(stats) = client::queries::player_dungeon_stats::fetch(&user_id) {
            let x = 9;
            let mut y = modal_y + 4;

            #[rustfmt::skip]
            text!("DUNGEON RECORD", absolute = true, font = Font::L, x = x, y = y);
            y += 12;

            #[rustfmt::skip]
            let entries = &[
                ("Crawls Completed", stats.get(DungeonStatKind::CrawlsCompleted)),
                ("Floors Cleared",   stats.get(DungeonStatKind::FloorsCleared)),
                ("Health Recovered", stats.get(DungeonStatKind::HealthRecovered)),
                ("Gold Collected",   stats.get(DungeonStatKind::GoldCollected)),
                ("Damage Dealt",     stats.get(DungeonStatKind::DamageDealt)),
                ("Damage Taken",     stats.get(DungeonStatKind::DamageTaken)),
                ("Steps Travelled",  stats.get(DungeonStatKind::StepsMoved)),
            ];
            for (label, val) in entries {
                text!("{:.<16}> {:0>5}", label, val; absolute = true, font = Font::M, x = x, y = y, color = 0x3a445aff);
                text!("{:<16}> {:>5}", label, val; absolute = true, font = Font::M, x = x, y = y, color = 0xe1e5d8ff);
                y += 8;
            }
            y += 6;

            #[rustfmt::skip]
            text!("MONSTER KILLS", absolute = true, font = Font::L, x = x, y = y);
            y += 12;
            for kind in MonsterKind::ALL {
                let val = stats.get(DungeonStatKind::Defeated(*kind));
                text!("{:.<16}> {:0>5}", format!("{:?}",kind), val; absolute = true, font = Font::M, x = x, y = y, color = 0x3a445aff);
                text!("{:<16}> {:>5}", format!("{:?}",kind), val; absolute = true, font = Font::M, x = x, y = y, color = 0xe1e5d8ff);
                y += 8;
            }
        } else {
            let x = 9;
            let mut y = modal_y + 4;
            #[rustfmt::skip]
            text!("DUNGEON RECORD", absolute = true, font = Font::L, x = x, y = y);
            y += 12;
            #[rustfmt::skip]
            text!("No stats yet!", absolute = true, font = Font::M, x = x, y = y, color = 0xe1e5d8ff);
        }

        let mut y = h as i32 - 36;

        // Back to Select Mode screen
        if secondary_button("< BACK TO SELECT MODE", modal_x, y, w - 8) {
            state.screen = Screen::SelectMode;
            state.show_stats_modal = false;
        }
        y += 16;

        // End crawl button
        if negative_button("END CRAWL", modal_x, y, w - 8) {
            client::commands::delete_dungeon::exec();
            state.screen = Screen::SelectMode;
        }
        y += 14;
        text!(
            "(PROGRESS WILL BE LOST!)",
            absolute = true,
            x = modal_x + 3,
            y = y,
            font = Font::S,
            color = 0xffffffaa
        );
    }

    // Swipe transition
    let p = state.floor.elapsed as f64 / FLOOR_DUR as f64;
    {
        let xo = p * w as f64;
        rect!(absolute = true, x = xo, w = w, h = h, color = 0x000000ff);
        rect!(absolute = true, x = -xo, w = w, h = h, color = 0x000000ff);
    }

    // Alerts
    if let Some(event) = os::client::watch_events(server::PROGRAM_ID, Some("alert")).data {
        // Display an alert banner for notifications that are < 10s old
        let duration = 10_000;
        let millis_since_event = time::now() as u32 - event.created_at * 1000;
        if millis_since_event < duration {
            if let Ok(msg) = std::str::from_utf8(&event.data) {
                let line_height = 6;
                let mut y = 0;
                let bg_color: u32 = 0x33984bff;
                let alert_kind = "DUNGEON ALERT:";
                #[rustfmt::skip]
                rect!(w = w, h = line_height, y = y, color = bg_color, absolute = true);
                #[rustfmt::skip]
                text!(alert_kind, absolute = true, x = 1, y = y + 1, font = Font::S, color = 0xffffffff);
                y += line_height;
                for line in wrap_text(msg, w, Font::S) {
                    #[rustfmt::skip]
                    rect!(w = w, h = line_height, y = y, color = bg_color, absolute = true);
                    #[rustfmt::skip]
                    text!(&line, absolute = true, x = 1, y = y + 1, font = Font::S, color = 0xffffffff);
                    y += line_height;
                }
                #[rustfmt::skip]
                rect!(w = w, h = 1, y = y, color = bg_color, absolute = true);
                // If the player has no current party, allow them to view the parties list by clicking the alert banner
                if client::queries::current_multiplayer_dungeon_crawl_id::fetch(&user_id).is_err() {
                    if clickable(0, 0, w, y) {
                        state.screen =
                            Screen::MultiplayerDungeonLobbies(MultiplayerDungeonLobbiesContext {
                                cursor: 0,
                                selected: false,
                            })
                    }
                }
            }
        }
    } // text!("{:#?}", dungeon.try_to_vec().unwrap().len(); absolute = true, x = 0, y = 0, font = Font::L, color = 0xffffffaa);
}

// Function to wrap text into lines that fit within max_width
pub fn wrap_text(text: &str, max_width: u32, font: Font) -> Vec<String> {
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
