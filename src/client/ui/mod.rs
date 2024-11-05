use super::*;

pub const TILE_SIZE: i32 = 16;
pub const TURN_DUR: usize = 16;
pub const FLOOR_DUR: usize = 32;
pub const MOVE_DUR: usize = 8;
pub const MOVE_Y_OFFSET: i32 = 6;
pub const MOVE_X_OFFSET: i32 = 6;
pub const EXEC_TIMEOUT_DUR: usize = 32;
pub const SHADOW_COLOR: u32 = 0x000000dd;

pub fn initialize() -> LocalState {
    LocalState {
        floor: Tween::new(0).duration(FLOOR_DUR),
        turn: Tween::new(0).duration(TURN_DUR),
        last_exec_at: 0,
        last_exec_turn: None,
        player: Entity {
            x: Tween::new(0).duration(MOVE_DUR).ease(Easing::EaseInOutQuad),
            y: Tween::new(0).duration(MOVE_DUR).ease(Easing::EaseInOutQuad),
            offset_x: Tween::new(0)
                .duration(MOVE_DUR / 2)
                .ease(Easing::EaseInOutQuad),
            offset_y: Tween::new(0)
                .duration(MOVE_DUR / 2)
                .ease(Easing::EaseInOutQuad),
        },
        monsters: vec![],
        leaderboard_kind: LeaderboardKind::HighestFloor,
        particles: vec![],
        clouds: vec![],
        raindrops: vec![],
        achievements_modal: None,
        last_crawl_achievements_modal: 0,
        show_stats_modal: false,
    }
}

pub fn render() {
    // Load the game state
    let mut state = LocalState::load();

    // Clear the screen
    clear(0x0e071bff);

    // Draw background
    let [w, h] = canvas_size!();
    sprite!(
        "skull_pattern",
        w = w,
        h = h,
        tx = (tick() / 4) % w as usize,
        ty = (tick() / 4) % h as usize,
        repeat = true,
        absolute = true,
    );

    // Get user ID
    let Some(user_id) = os::client::user_id() else {
        // Draw title screen
        return screens::title::render(&mut state, false);
    };

    // Fetch user dungeon
    let Ok(dungeon) = &client::queries::player_dungeon::fetch(&user_id) else {
        // Draw title screen
        return screens::title::render(&mut state, true);
    };

    // Draw dungeon screen
    screens::dungeon::render(&mut state, &user_id, dungeon);

    // Save local state
    state.save();
}

mod screens {
    use super::*;
    pub mod dungeon;
    pub mod title;
}
