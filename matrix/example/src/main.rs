use std::sync::{LazyLock, Mutex};

use macroquad::prelude::*;
use common_matrix::prelude::*;

#[macroquad::main(conf)]
async fn main() {
    default_main(vec![Box::new(make_vec), Box::new(decompose_vec), Box::new(interactive_decomposition)]).await;
}

fn make_vec(frame_data: FrameData) -> bool {
    let time = frame_data.time;
    let transform = frame_data.transform;
    let mut check = FrameCheck::new(time);

    display_background(transform);
    transform.point_of_interest(vec2(-3.0, -3.0));
    transform.point_of_interest(vec2(3.0, 3.0));

    if let Some(frac) = check.smoother_test(1.5) {
        display_vec(frac * vec2(3.0, 5.0), transform, "");
    } else { return true; }
    false
}

fn decompose_vec(frame_data: FrameData) -> bool {
    let time = frame_data.time;
    let transform = frame_data.transform;
    let mut check = FrameCheck::new(time);

    display_background(transform);
    transform.point_of_interest(vec2(-3.0, -3.0));
    transform.point_of_interest(vec2(3.0, 3.0));
    display_vec(vec2(3.0, 5.0), transform, "");

    if let Some(frac) = check.smooth_test(1.5) {
        display_vec_offset(frac * vec2(-3.0, 0.0), vec2(3.0, 5.0), transform, "");
        display_vec_offset(frac * vec2(0.0, -5.0), vec2(3.0, 5.0), transform, "");
    } else if let Some(frac) = check.smooth_test(2.0) {
        display_vec_offset((1.0 - frac) * vec2(-3.0, 0.0), vec2((1.0 - frac) * 3.0, 5.0), transform, "");
        display_vec_offset((1.0 - frac) * vec2(0.0, -5.0), vec2(3.0, (1.0 - frac) * 5.0), transform, "");
        display_point(vec2(0.0, 5.0), transform, "5.0", RED, frac * 5.0);
        display_point(vec2(3.0, 0.0), transform, "3.0", RED, frac * 5.0);
    } else { return true; }
    false
}

fn interactive_decomposition(frame_data: FrameData) -> bool {
    static VEC: LazyLock<Mutex<Vec2>> = LazyLock::new(|| Mutex::new(vec2(3.0, 5.0)));
    let mut vec = VEC.lock().unwrap();

    let time = frame_data.time;
    let transform = frame_data.transform;

    display_background(transform);
    transform.point_of_interest(vec2(-3.0, -3.0));
    transform.point_of_interest(vec2(3.0, 3.0));
    display_vec(*vec, transform, "");
    display_point(vec2(0.0, vec.y), transform, &format!("{:.1}", vec.y), RED, 5.0);
    display_point(vec2(vec.x, 0.0), transform, &format!("{:.1}", vec.x), RED, 5.0);

    if !frame_data.whiteboard.is_enabled() && is_mouse_button_down(MouseButton::Left) {
        *vec = transform.screen_to_world(mouse_vec())
    }

    time > 0.0
}