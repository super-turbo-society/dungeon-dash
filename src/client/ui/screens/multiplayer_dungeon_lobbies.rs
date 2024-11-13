use super::*;

pub fn render(state: &mut LocalState, user_id: &str, ctx: &mut MultiplayerDungeonLobbiesContext) {
    reset_cam!();
    // Check if the player is already in a multiplayer dungeon
    if let Ok(crawl_id) = client::queries::current_multiplayer_dungeon_crawl_id::fetch(user_id) {
        log!("redirecting -- PARTY ID = {crawl_id}");
        state.screen = Screen::MultiplayerDungeon(crawl_id);
        return;
    }

    // Fetch user dungeon
    let Ok(lobbies) = &client::queries::multiplayer_dungeon_list::fetch() else {
        text!("Loading parties...");
        text!("Press B to create a party", y = 8, font = Font::S);
        if gamepad(0).b.just_pressed() {
            client::commands::create_multiplayer_dungeon_lobby::exec();
        }
        return;
    };

    // Check if you are in a lobby

    let gp = gamepad(0);

    let [w, h] = canvas_size!();

    // Player made a lobby
    if let Some(lobby) = lobbies.get(user_id) {
        #[rustfmt::skip]
        text!("YOUR PARTY", absolute = true, x = 4, y = 4, font = Font::L);
        let mut y = 16;
        text!("Members", absolute = true, x = 4, y = y, font = Font::M);
        y += 10;
        for player in &lobby.players {
            let (btn_x, btn_y, btn_w, btn_h) = (4, y, w - 8, 12u32);
            #[rustfmt::skip]
            rect!(absolute = true, x = btn_x, y = btn_y, w = btn_w, h = btn_h, color = 0, border_color = 0xffffffaa, border_radius = 3, border_width = 1);
            if player == user_id {
                text!("{:.8} (you/owner)", player; x = btn_x + 4, y = btn_y + 3);
            } else {
                text!("{:.8}", player; x = btn_x + 4, y = btn_y + 3);
            }
            y += 16;
        }

        // Delete Party
        let (btn_x, btn_y, btn_w, btn_h) = (4, h as i32 - 16, w / 2 - 4, 12u32);
        #[rustfmt::skip]
        rect!(absolute = true, x = btn_x, y = btn_y, w = btn_w, h = btn_h, color = 0x81090aff, border_radius = 3);
        text!("Cancel", x = btn_x + 4, y = btn_y + 3);
        let m = mouse(0);
        if m.intersects_abs(btn_x, btn_y, btn_w, btn_h) && m.left.just_pressed() {
            client::commands::delete_multiplayer_dungeon_lobby::exec();
        }

        if lobby.players.len() > 1 {
            // Create a lobby button
            let (btn_x, btn_y, btn_w, btn_h) =
                ((w / 2) as i32 + 2, h as i32 - 16, w / 2 - 4, 12u32);
            #[rustfmt::skip]
            rect!(absolute = true, x = btn_x, y = btn_y, w = btn_w, h = btn_h, color = 0x1e7061ff, border_radius = 3);
            text!("START GAME!", x = btn_x + 4, y = btn_y + 3);
            let m = mouse(0);
            if m.intersects_abs(btn_x, btn_y, btn_w, btn_h) && m.left.just_pressed() {
                // Start the game
                client::commands::start_new_multiplayer_dungeon::exec();
            }
        } else {
            // Disabled create a lobby button
            let (btn_x, btn_y, btn_w, btn_h) =
                ((w / 2) as i32 + 2, h as i32 - 16, w / 2 - 4, 12u32);
            #[rustfmt::skip]
            rect!(absolute = true, x = btn_x, y = btn_y, w = btn_w, h = btn_h, color = 0x83758bff, border_radius = 3);
            text!("Waiting...", x = btn_x + 4, y = btn_y + 3);
        }
    } else {
        let joined_or_selected_lobby = lobbies
            .iter()
            // Find lobby player has joined
            .find(|(_, lobby)| lobby.players.contains(user_id))
            // Find lobby player has selected
            .or_else(|| {
                if ctx.selected {
                    lobbies.iter().nth(ctx.cursor)
                } else {
                    None
                }
            });
        // Player is viewing a lobby
        if let Some((owner, lobby)) = joined_or_selected_lobby {
            #[rustfmt::skip]
            text!("{:.8}'s PARTY", owner; absolute = true, x = 4, y = 4, font = Font::L);
            let mut y = 16;
            text!("Members", absolute = true, x = 4, y = y, font = Font::M);
            y += 10;
            for player in &lobby.players {
                let (btn_x, btn_y, btn_w, btn_h) = (4, y, w - 8, 12u32);
                #[rustfmt::skip]
                rect!(absolute = true, x = btn_x, y = btn_y, w = btn_w, h = btn_h, color = 0, border_color = 0xffffffaa, border_radius = 3, border_width = 1);
                if player == owner {
                    text!("{:.8} (owner)", player; x = btn_x + 4, y = btn_y + 3);
                } else if player == user_id {
                    text!("{:.8} (you)", player; x = btn_x + 4, y = btn_y + 3);
                } else {
                    text!("{:.8}", player; x = btn_x + 4, y = btn_y + 3);
                }
                y += 16;
            }
            if !lobby.players.contains(user_id) {
                // Go back
                let (btn_x, btn_y, btn_w, btn_h) = (4, h as i32 - 16, w / 2 - 4, 12u32);
                #[rustfmt::skip]
                rect!(absolute = true, x = btn_x, y = btn_y, w = btn_w, h = btn_h, color = 0x293c8bff, border_radius = 3);
                text!("Back", x = btn_x + 4, y = btn_y + 3);
                let m = mouse(0);
                if m.intersects_abs(btn_x, btn_y, btn_w, btn_h) && m.left.just_pressed() {
                    ctx.cursor = 0;
                    ctx.selected = false;
                }
                // Join the lobby
                let (btn_x, btn_y, btn_w, btn_h) =
                    ((w / 2) as i32 + 2, h as i32 - 16, w / 2 - 4, 12u32);
                #[rustfmt::skip]
                rect!(absolute = true, x = btn_x, y = btn_y, w = btn_w, h = btn_h, color = 0x1e7061ff, border_radius = 3);
                text!("Join", x = btn_x + 4, y = btn_y + 3);
                let m = mouse(0);
                if m.intersects_abs(btn_x, btn_y, btn_w, btn_h) && m.left.just_pressed() {
                    client::commands::join_multiplayer_dungeon_lobby::exec(owner);
                }
            } else {
                if gp.b.just_pressed() {
                    client::commands::leave_multiplayer_dungeon_lobby::exec(owner);
                }
                // Leave the lobby
                let (btn_x, btn_y, btn_w, btn_h) = (4, h as i32 - 16, w - 8, 12u32);
                #[rustfmt::skip]
                rect!(absolute = true, x = btn_x, y = btn_y, w = btn_w, h = btn_h, color = 0x81090aff, border_radius = 3);
                text!("Leave", x = btn_x + 4, y = btn_y + 3);
                let m = mouse(0);
                if m.intersects_abs(btn_x, btn_y, btn_w, btn_h) && m.left.just_pressed() {
                    client::commands::leave_multiplayer_dungeon_lobby::exec(owner);
                }
            }
        }
        // Player is browsing lobby list
        else {
            // Display lobbies
            ctx.selected = false; // reset selected
            #[rustfmt::skip]
            text!("JOIN A PARTY", absolute = true, x = 4, y = 4, font = Font::L);
            let mut y = 16;
            if lobbies.is_empty() {
                text!("No parties available", x = 4, y = y, font = Font::S);
            }
            for (owner, lobby) in lobbies {
                let (btn_x, btn_y, btn_w, btn_h) = (4, y, w - 8, 12u32);
                #[rustfmt::skip]
                rect!(absolute = true, x = btn_x, y = btn_y, w = btn_w, h = btn_h, color = 0x83758bff, border_radius = 3);
                text!("{:.8}'s Party ({})  >", owner, lobby.players.len(); x = btn_x + 4, y = btn_y + 3);
                let m = mouse(0);
                if m.intersects_abs(btn_x, btn_y, btn_w, btn_h) && m.left.just_pressed() {
                    // View the party
                    ctx.selected = true;
                }
                y += 16;
            }

            // Go Back
            let (btn_x, btn_y, btn_w, btn_h) = (4, h as i32 - 16, w / 2 - 4, 12u32);
            #[rustfmt::skip]
            rect!(absolute = true, x = btn_x, y = btn_y, w = btn_w, h = btn_h, color = BTN_SECONDARY_COLOR, border_radius = 3);
            text!("Back", x = btn_x + 4, y = btn_y + 3);
            let m = mouse(0);
            if m.intersects_abs(btn_x, btn_y, btn_w, btn_h) && m.left.just_pressed() {
                state.screen = Screen::SelectMode;
            }

            // Create a party button
            let (btn_x, btn_y, btn_w, btn_h) =
                ((w / 2) as i32 + 2, h as i32 - 16, w / 2 - 4, 12u32);
            #[rustfmt::skip]
            rect!(absolute = true, x = btn_x, y = btn_y, w = btn_w, h = btn_h, color = BTN_PRIMARY_COLOR, border_radius = 3);
            text!("New Party", x = btn_x + 4, y = btn_y + 3);
            let m = mouse(0);
            if m.intersects_abs(btn_x, btn_y, btn_w, btn_h) && m.left.just_pressed() {
                client::commands::create_multiplayer_dungeon_lobby::exec();
            }
        }
    }
}
