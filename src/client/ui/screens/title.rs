use super::*;

pub fn render(_state: &mut LocalState, is_logged_in: bool) {
    // Get canvas width and height
    let [w, h] = canvas_size!();

    // Handle user input
    let gp = gamepad(0);
    if gp.start.just_pressed() {
        client::commands::create_new_dungeon::exec(true);
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

    if is_logged_in {
        if mouse(0).left.just_pressed() {
            client::commands::create_new_dungeon::exec(true);
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

    // text!("PRESS START {:?}", os::client::user_id(););
    // text!("PRESS START");
}