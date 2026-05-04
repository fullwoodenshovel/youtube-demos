use std::fmt::Debug;
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, Default)]
pub struct Transform {
    offset: Vec2,
    /// The screen width / height of one in-game unit. how many pixels in one unit
    pub scale: f32,
    pub screen_dims: Vec2,
    pub target_rect: Rect,
    prev_actual: Option<Rect>
}

pub fn get_screen_dims() -> Vec2 {
    let width = screen_width();
    let height = screen_height();

    vec2(width, height)
}

impl Transform {
    pub fn new(offset: Vec2, scale: f32) -> Self {
        Self {
            offset,
            scale,
            screen_dims: get_screen_dims(),
            target_rect: Rect::new(-1.0, -1.0, 2.0, 2.0),
            prev_actual: None
        }
    }

    pub fn world_to_screen(&self, world: Vec2) -> Vec2 {
        let result = world * self.scale - self.offset;
        vec2(result.x, self.screen_dims[1] - result.y)
    }

    pub fn screen_to_world(&self, screen: Vec2) -> Vec2 {
        let screen = vec2(screen.x, self.screen_dims[1] - screen.y);
        (screen + self.offset) / self.scale
    }

    pub fn rect(&self) -> Rect {
        let p1 = self.screen_to_world(vec2(0.0, self.screen_dims[1]));
        let p2 = self.screen_to_world(vec2(self.screen_dims[0], 0.0));
        let wh = p2 - p1;
        Rect::new(p1.x, p1.y, wh.x, wh.y)
    }

    pub fn rect_conservative(&self) -> Rect {
        let mut result = self.rect();
        result.x -= result.w * 0.001;
        result.y -= result.h * 0.001;
        result.w += result.w * 0.002;
        result.h += result.h * 0.002;
        result
    }

    pub fn modify_from_rect(&mut self, rect: Rect) {
        self.scale = (self.screen_dims[0] / rect.w).min(self.screen_dims[1] / rect.h);
        self.offset = (rect.point() + (rect.size() - self.screen_dims / self.scale) / 2.0) * self.scale;
    }

    pub fn move_camera(&mut self) {
        if self.target_rect.w == 0.0 && self.target_rect.h == 0.0 {
            self.target_rect.h = if self.target_rect.x == 0.0 && self.target_rect.y == 0.0 {
                1.0
            } else if self.target_rect.x == 0.0 {
                self.target_rect.h
            } else {
                self.target_rect.x
            }
        }
        
        const SPEED: f32 = 0.03;

        let prev_actual = self.prev_actual.unwrap_or(Rect::new(-1.0, -1.0, 2.0, 2.0));
        let actual = Rect::new(
            self.target_rect.x * SPEED + prev_actual.x * (1.0 - SPEED),
            self.target_rect.y * SPEED + prev_actual.y * (1.0 - SPEED),
            self.target_rect.w * SPEED + prev_actual.w * (1.0 - SPEED),
            self.target_rect.h * SPEED + prev_actual.h * (1.0 - SPEED)
        );
        
        self.prev_actual = Some(actual);
        let conservative = Rect::new(
            actual.x - actual.w / 2.0,
            actual.y - actual.h / 2.0,
            2.0 * actual.w,
            2.0 * actual.h
        );
        self.modify_from_rect(conservative);

        self.target_rect = Rect::new(0.0, 0.0, 0.0, 0.0);

    }
    
    pub fn point_of_interest(&mut self, vec: Vec2) {
        #[cfg(feature = "debug_points")]
        {
            let screen = self.world_to_screen(vec);
            draw_circle_lines(screen.x, screen.y, 4.0, 1.0, WHITE);
        }
        if vec.x < self.target_rect.x {
            self.target_rect.w += self.target_rect.x - vec.x;
            self.target_rect.x = vec.x;
        } else if vec.x > self.target_rect.x + self.target_rect.w {
            self.target_rect.w = vec.x - self.target_rect.x;
        }
        if vec.y < self.target_rect.y {
            self.target_rect.h += self.target_rect.y - vec.y;
            self.target_rect.y = vec.y;
        } else if vec.y > self.target_rect.y + self.target_rect.h {
            self.target_rect.h = vec.y - self.target_rect.y;
        }
    }

    /// This is in world coordinates
    pub fn draw_line(&self, p1: Vec2, p2: Vec2, width: f32, colour: Color) -> bool {
        let rect = self.rect_conservative();
        if let Some((p1, p2)) = line_rect_intersections(rect, p1, p2) {
            let p1 = self.world_to_screen(p1);
            let p2 = self.world_to_screen(p2);
            draw_line(p1.x, p1.y, p2.x, p2.y, width, colour);
            true
        } else {
            false
        }
    }
}

/// Finds the two points on rect where the line through the origin and p intersect it.
pub fn line_rect_intersections(rect: Rect, p1: Vec2, p2: Vec2) -> Option<(Vec2, Vec2)> {
    let dir = p2 - p1;

    let mut t_min = f32::NEG_INFINITY;
    let mut t_max = f32::INFINITY;

    // Helper to update interval
    let mut clip = |p: f32, q: f32| -> bool {
        if p.abs() < f32::EPSILON {
            // Line parallel to edge
            return q >= 0.0;
        }

        let t = q / p;

        if p < 0.0 {
            t_min = t_min.max(t);
        } else {
            t_max = t_max.min(t);
        }

        t_min <= t_max
    };

    // Left:   x >= rect.x
    if !clip(-dir.x, p1.x - rect.x) {
        return None;
    }
    // Right:  x <= rect.x + w
    if !clip(dir.x, rect.x + rect.w - p1.x) {
        return None;
    }
    // Top:    y >= rect.y
    if !clip(-dir.y, p1.y - rect.y) {
        return None;
    }
    // Bottom: y <= rect.y + h
    if !clip(dir.y, rect.y + rect.h - p1.y) {
        return None;
    }

    let a = p1 + dir * t_min;
    let b = p1 + dir * t_max;

    Some((a, b))
}