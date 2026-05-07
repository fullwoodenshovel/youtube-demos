use macroquad::prelude::*;
use common_matrix::prelude::*;

#[macroquad::main(conf)]
async fn main() {
    default_main(vec![Box::new(slide1), Box::new(slide2)]).await;
}

fn slide1(transform: &mut Transform, time: f32) -> bool {
    let mut clamp = FrameCheck::new(time);
    display_background(transform);
    transform.point_of_interest(vec2(-3.0, -3.0));
    transform.point_of_interest(vec2(3.0, 3.0));
    if let Some(frac) = clamp.smoother_test(1.5) {
        display_vec(frac * vec2(3.0, 5.0), transform, "");
    } else { return true; }
    false
}

fn slide2(transform: &mut Transform, time: f32) -> bool {
    let mut check = FrameCheck::new(time);
    display_background(transform);
    transform.point_of_interest(vec2(-3.0, -3.0));
    transform.point_of_interest(vec2(3.0, 3.0));
    display_vec(vec2(3.0, 5.0), transform, "");

    if let Some(frac) = check.smoother_test(1.5) {
        display_vec_offset(frac * vec2(-3.0, 0.0), vec2(3.0, 5.0), transform, "");
        display_vec_offset(frac * vec2(0.0, -5.0), vec2(3.0, 5.0), transform, "");
    } else if let Some(frac) = check.smoother_test(2.0) {
        display_vec_offset((1.0 - frac) * vec2(-3.0, 0.0), vec2((1.0 - frac) * 3.0, 5.0), transform, "");
        display_vec_offset((1.0 - frac) * vec2(0.0, -5.0), vec2(3.0, (1.0 - frac) * 5.0), transform, "");
    } else { return true; }
    false
}