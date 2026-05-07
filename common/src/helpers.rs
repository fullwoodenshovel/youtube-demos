use macroquad::{miniquad::window::screen_size, prelude::*};

use std::sync::{LazyLock, Mutex};

use super::*;

use crate::transform::Transform;

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

pub struct Slideshow {
    pub visual_number: usize,
    pub time: f32,
    speed: f32,
    display_speed: f64
}


impl Slideshow {
    pub fn new() -> Self {
        Self { visual_number: 0, time: 0.0, speed: 1.0, display_speed: -f64::INFINITY }
    }

    const SPEEDS: [f32; 9] = [0.1, 0.2, 0.5, 0.8, 1.0, 1.2, 1.5, 2.0, 2.5];

    pub fn finish_slide(&mut self) {
        self.visual_number += 1;
        self.time = 0.0;
    }

    pub fn update(&mut self, transform: &mut Transform) {
        clear_background(BLACK);
        transform.move_camera();
        transform.screen_dims = screen_size_vec();
    
        if self.time > 0.0 {
            self.time += get_frame_time() * self.speed;
        }
        
        if self.display_speed + 1.5 > get_time() {
            let text = format!("{}x", self.speed);
            let w = measure_text(&text, None, 18, 1.0).width;
            draw_text(&text, transform.screen_dims.x / 2.0 - w / 2.0, 58.0, 18.0, WHITE);
        }
    
        if left_pressed() && self.visual_number != 0 {
            if self.time == 0.0 {
                self.visual_number -= 1;
            } else {
                self.time = 0.0;
            }
        } else if right_pressed() {
            if self.time > 0.0 {
                self.visual_number += 1;
                self.time = 0.0;
            } else {
                self.time += get_frame_time() * self.speed;
            }
        } else if let up = is_key_pressed(KeyCode::Up) && (up || is_key_pressed(KeyCode::Down)) {
            let mut speed_index = Self::SPEEDS.binary_search_by(|b| b.partial_cmp(&self.speed).unwrap()).unwrap();
            if up {
                if speed_index != 8 {
                    speed_index += 1
                }
            } else {
                speed_index = speed_index.saturating_sub(1)
            };
            self.speed = Self::SPEEDS[speed_index];
            self.display_speed = get_time();
        }
    }
}

impl Default for Slideshow {
    fn default() -> Self {
        Self::new()
    }
}

static WHITEBOARD: LazyLock<Mutex<whiteboard::Whiteboard>> = LazyLock::new(|| Mutex::new(whiteboard::Whiteboard::new()));

fn update_whiteboard() {
    WHITEBOARD.lock().unwrap().default_update();
}

type SlideType = dyn FnMut(&mut Transform,f32) -> bool;

pub async fn default_main(mut slides: Vec<Box<SlideType>>) {
    let mut slideshow = Slideshow::new();
    let mut transform = Transform::default();
    loop {
        slideshow.update(&mut transform);
        if slides.len() <= slideshow.visual_number {
            break
        }
        while slides[slideshow.visual_number](&mut transform, slideshow.time) {
            slideshow.finish_slide();
        };
        update_whiteboard();
        next_frame().await;
    }
}

pub struct FrameCheck {
    time: f32
}

impl FrameCheck {
    pub fn new(time: f32) -> Self {
        FrameCheck { time }
    }

    pub fn test(&mut self, duration: f32) -> Option<f32> {
        if self.time > duration {
            self.time -= duration;
            None
        } else {
            Some(self.time / duration)
        }
    }

    pub fn smooth_test(&mut self, duration: f32) -> Option<f32> {
        Some(smooth_step(self.test(duration)?))
    }

    pub fn smoother_test(&mut self, duration: f32) -> Option<f32> {
        Some(smoother_step(self.test(duration)?))
    }
}


pub fn smooth_step(frac: f32) -> f32 {
    frac * frac * (3.0 - 2.0 * frac)
}

pub fn smoother_step(frac: f32) -> f32 {
    smooth_step(smooth_step(frac))
}