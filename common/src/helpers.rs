use macroquad::{miniquad::window::screen_size, prelude::*};

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

pub fn mouse_vec() -> Vec2 {
    let mouse = mouse_position();
    vec2(mouse.0, mouse.1)
}

pub fn screen_size_vec() -> Vec2 {
    let size = screen_size();
    vec2(size.0, size.1)
}

pub fn lock_camera(target: &RenderTarget) -> CameraLock {
    let (w, h) = (target.texture.width(), target.texture.height());
    set_camera(&Camera2D {
        zoom: vec2(2.0 / w, 2.0 / h),
        target: vec2(w / 2.0, h / 2.0),
        render_target: Some(Clone::clone(target)),
        ..Default::default()
    });
    CameraLock
}

pub struct CameraLock;

impl Drop for CameraLock {
    fn drop(&mut self) {
        set_default_camera();
    }
}