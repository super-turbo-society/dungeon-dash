use super::*;

pub const TILE_SIZE: i32 = 16;
pub const TURN_DUR: usize = 16;
pub const FLOOR_DUR: usize = 32;
pub const HIT_DUR: usize = 64;
pub const MOVE_DUR: usize = 8;
pub const MOVE_Y_OFFSET: i32 = 6;
pub const MOVE_X_OFFSET: i32 = 6;
// pub const EXEC_TIMEOUT_DUR: usize = 32;
pub const EXEC_TIMEOUT_DUR: usize = 16;
pub const SHADOW_COLOR: u32 = 0x000000bb;
pub const BTN_PRIMARY_COLOR: u32 = 0x1e7061ff;
pub const BTN_SECONDARY_COLOR: u32 = 0x524c52ff; //0x293c8bff;
pub const BTN_NEGATIVE_COLOR: u32 = 0x81090aff;

fn new_player_entity() -> Entity {
    Entity {
        hp: Tween::new(0).duration(HIT_DUR).ease(Easing::EaseInBack),
        x: Tween::new(0).duration(MOVE_DUR).ease(Easing::EaseInOutQuad),
        y: Tween::new(0).duration(MOVE_DUR).ease(Easing::EaseInOutQuad),
        offset_x: Tween::new(0)
            .duration(MOVE_DUR / 2)
            .ease(Easing::EaseInOutQuad),
        offset_y: Tween::new(0)
            .duration(MOVE_DUR / 2)
            .ease(Easing::EaseInOutQuad),
        emote: None,
    }
}

pub fn initialize() -> LocalState {
    LocalState {
        screen: Screen::Title,
        floor: Tween::new(0).duration(FLOOR_DUR),
        turn: Tween::new(0).duration(TURN_DUR),
        last_exec_at: 0,
        last_exec_turn: None,
        players: vec![new_player_entity()],
        monsters: vec![],
        leaderboard_kind: LeaderboardKind::HighestFloor,
        particles: vec![],
        clouds: vec![],
        raindrops: vec![],
        snowflakes: vec![],
        achievements_modal: None,
        last_crawl_achievements_modal: 0,
        show_stats_modal: false,
        last_channel_message: "".to_string(),
    }
}

pub fn render() {
    // Load the game state
    let mut state = LocalState::load();

    // Draw background
    let [w, h] = canvas_size!();
    let is_winter = true; // TODO: timestamp-based
    if is_winter {
        clear(0x000044ff);
        sprite!(
            "snowflake_pattern",
            w = w,
            h = h,
            tx = (tick() / 4) as u32,
            ty = (tick() / 4) as u32,
            repeat = true,
            absolute = true,
        );
    } else {
        clear(0x0e071bff);
        sprite!(
            "skull_pattern",
            w = w,
            h = h,
            tx = (tick() / 4) as u32,
            ty = (tick() / 4) as u32,
            repeat = true,
            absolute = true,
        );
    }

    // Get user ID
    let Some(user_id) = os::client::user_id() else {
        // Draw title screen
        return screens::title::render(&mut state, false);
    };

    // Render based on chosen screen
    match state.screen.clone() {
        Screen::Title => {
            screens::title::render(&mut state, true);
            if let Ok(crawl_id) =
                client::queries::current_multiplayer_dungeon_crawl_id::fetch(&user_id)
            {
                if gamepad(0).b.just_pressed() {
                    client::commands::delete_multiplayer_dungeon::exec(crawl_id);
                }
            }
        }
        Screen::SelectMode => {
            screens::select_mode::render(&mut state, &user_id);
        }
        Screen::Dungeon => {
            // Fetch user dungeon
            if let Ok(dungeon) = &client::queries::player_dungeon::fetch(&user_id) {
                if state.players.len() != 1 {
                    state.players = vec![new_player_entity(); 1];
                }
                screens::dungeon::render(&mut state, &user_id, dungeon);
            } else {
                screens::title::render(&mut state, true);
            }
        }
        Screen::MultiplayerDungeonLobbies(mut ctx) => {
            screens::multiplayer_dungeon_lobbies::render(&mut state, &user_id, &mut ctx);
            if let Screen::MultiplayerDungeonLobbies(_) = state.screen {
                state.screen = Screen::MultiplayerDungeonLobbies(ctx);
            }
        }
        Screen::MultiplayerDungeon(crawl_id) => {
            if let Ok(dungeon) = client::queries::multiplayer_dungeon::fetch(crawl_id) {
                if state.players.len() != dungeon.player.players.len() {
                    state.players = vec![new_player_entity(); dungeon.player.players.len()];
                }
                screens::multiplayer_dungeon::render(&mut state, &user_id, &dungeon);
            } else {
                reset_cam!();
                text!("Loading multiplayer dungeon...");
                if gamepad(0).start.just_pressed() {
                    client::commands::delete_multiplayer_dungeon::exec(crawl_id);
                }
                // Check if the player is already in a multiplayer dungeon
                if client::queries::current_multiplayer_dungeon_crawl_id::fetch(&user_id).is_err() {
                    state.screen =
                        Screen::MultiplayerDungeonLobbies(MultiplayerDungeonLobbiesContext {
                            cursor: 0,
                            selected: false,
                        });
                }
            }
        }
    }

    // Save local state
    state.save();
}

mod screens {
    use super::*;
    pub mod dungeon;
    pub mod multiplayer_dungeon;
    pub mod multiplayer_dungeon_lobbies;
    pub mod select_mode;
    pub mod title;
}

#[allow(arithmetic_overflow)]
pub fn button(text: &str, color: u32, x: i32, y: i32, w: u32) -> bool {
    let h = 12;
    let color = if hovered(x, y, w, h) {
        let opacity: f32 = 0.8;
        // Apply gamma correction
        let gamma = 2.2;
        let linear_opacity = opacity.powf(1.0 / gamma);
        // Calculate the alpha value
        let alpha = (255.0 * linear_opacity) as u32;
        alpha << 32 | (color & 0xffffff00)
    } else {
        color
    };
    #[rustfmt::skip]
    rect!(absolute = true, x = x, y = y, w = w, h = h, color = color, border_radius = 3);
    text!(text, absolute = true, x = x + 4, y = y + 3);
    clickable(x, y, w, h)
}

pub fn primary_button(text: &str, x: i32, y: i32, w: u32) -> bool {
    button(text, BTN_PRIMARY_COLOR, x, y, w)
}

pub fn secondary_button(text: &str, x: i32, y: i32, w: u32) -> bool {
    button(text, BTN_SECONDARY_COLOR, x, y, w)
}

pub fn negative_button(text: &str, x: i32, y: i32, w: u32) -> bool {
    button(text, BTN_NEGATIVE_COLOR, x, y, w)
}

pub fn clickable(x: i32, y: i32, w: u32, h: u32) -> bool {
    let m = mouse(0);
    m.intersects_abs(x, y, w, h) && m.left.just_pressed()
}

pub fn hovered(x: i32, y: i32, w: u32, h: u32) -> bool {
    let m = mouse(0);
    m.intersects_abs(x, y, w, h)
}
