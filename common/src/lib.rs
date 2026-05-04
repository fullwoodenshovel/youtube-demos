pub mod mat2;
pub mod transform;
use macroquad::prelude::*;
#[cfg(target_arch = "wasm32")]
pub mod web;

pub fn conf() -> Conf {
    Conf {
        window_title: "Demo".to_string(),
        sample_count: 4,
        ..Default::default()
    }
}

pub fn right_pressed() -> bool {
    #[cfg(feature = "touchscreen")]
    let center = vec2(screen_width() * 0.95, screen_height() / 2.0);

    #[cfg(feature = "touchscreen")]
    {
        draw_triangle(center, center + vec2(-30.0, 40.0), center + vec2(-30.0, -40.0), WHITE);
    }

    if is_key_pressed(KeyCode::Right) {
        return true
    }

    #[cfg(feature = "touchscreen")]
    if is_mouse_button_pressed(MouseButton::Left) &&
        let mouse = mouse_position() &&
        let mouse = vec2(mouse.0, mouse.1) &&
        let rect = Rect::new(center.x - 30.0, center.y - 40.0, 30.0, 80.0) &&
        rect.contains(mouse)
    {
        return true
    }
    false
    
}

pub fn left_pressed() -> bool {
    #[cfg(feature = "touchscreen")]
    let center = vec2(screen_width() * 0.05, screen_height() / 2.0);

    #[cfg(feature = "touchscreen")]
    {
        draw_triangle(center, center + vec2(30.0, 40.0), center + vec2(30.0, -40.0), WHITE);
    }

    if is_key_pressed(KeyCode::Left) {
        return true
    }

    #[cfg(feature = "touchscreen")]
    if is_mouse_button_pressed(MouseButton::Left) &&
        let mouse = mouse_position() &&
        let mouse = vec2(mouse.0, mouse.1) &&
        let rect = Rect::new(center.x, center.y - 40.0, 30.0, 80.0) &&
        rect.contains(mouse)
    {
        return true
    }
    false
    
}
