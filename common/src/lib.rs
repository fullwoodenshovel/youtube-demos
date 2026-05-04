pub mod mat2;
use std::collections::HashSet;
use std::f32;
mod parse;
pub use parse::{FloatEx, MatEx, VecEx, Ex, Obj, resolve_ex};
pub use parse::for_each::{ExPointer, for_each, resolve_indexed};
pub use parse::visualise::{visualise, display_background, visualise_obj};
pub mod transform;
use transform::{Transform, get_screen_dims};
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

pub struct ExVisual {
    pub speed: f32,
    pub transform: Transform,
    anim_done: bool,
    pub ex: Ex,
    pub ignored: HashSet<(usize, usize, usize)>,
    time: f32,
    index: usize,
    order: Vec<usize>,
    display_speed: Option<f64>
}

pub enum VisualControl {
    Backwards,
    Forwards,
    Current
}

impl ExVisual {
    /// I can potentially get rid of ignored parameter by just taking the complement of order.
    pub fn new(ex: Ex, order: Vec<usize>, ignored: HashSet<(usize, usize, usize)>, speed: Option<f32>, transform: Option<Transform>) -> Self {
        Self {
            speed: speed.unwrap_or(1.0),
            transform: transform.unwrap_or(Transform::new(vec2(0.0, 0.0), 1.0)),
            anim_done: false,
            ex,
            ignored,
            time: 0.0,
            index: 0, // Check this
            order,
            display_speed: None
        }
    }

    const SPEEDS: [f32; 9] = [0.1, 0.2, 0.5, 0.8, 1.0, 1.2, 1.5, 2.0, 2.5];

    pub fn visualise_ex(&mut self) -> VisualControl {
        use VisualControl::*;

        clear_background(BLACK);
        self.transform.move_camera();
        self.transform.screen_dims = get_screen_dims();
        display_background(&self.transform);

        let vec = for_each(&self.ex, false, true);
        if self.index < self.order.len() {
            let current_ex = for_each(&self.ex, true, false)[self.order[self.index]].0;
            let new_index = vec.iter().position(|d| d.0.pointer_eq(current_ex)).unwrap();
            let mut target_depth = if self.time == 0.0 {
                vec[new_index].1 + 1
            } else {
                vec[new_index].1
            };
            for (index, (ex, depth)) in vec.range(new_index + 1..).enumerate() {
                let index = index + new_index + 1;
                if target_depth >= *depth {
                    target_depth = *depth;
                    let mut parent_shown = true;
                    for (index, (_, parent_depth)) in vec.range(..index).enumerate().rev() {
                        if parent_depth < depth {
                            let index = vec.len() - index - 1;
                            parent_shown = !self.ignored.iter().any(|(_, _, target_index)| *target_index == index);
                            break
                        }
                    }
                    if parent_shown {
                        visualise_obj(ex.resolve(), &mut self.transform, true);
                    }
                }
            }
        }
    
        if self.time > 0.0 {
            if !self.anim_done {
                if self.ignored.iter().any(|(_, _, ignored_index)| *ignored_index == self.order[self.index]) {
                    self.anim_done = true;
                } else {
                    self.anim_done = visualise(self.order[self.index], self.time, &self.ex, &mut self.transform);
                }
                self.time += get_frame_time() * self.speed;
            }
            if self.anim_done {
                self.index += 1;
                self.time = 0.0;
            }
        }
    
        if self.time == 0.0 {
            let obj = resolve_indexed(self.order[self.index - 1], &self.ex);
            visualise_obj(obj, &mut self.transform, false);
    
            let text = obj.to_string();
            let w = measure_text(&text, None, 18, 1.0).width;
            draw_text(&text, self.transform.screen_dims.x / 2.0 - w / 2.0, 26.0, 18.0, WHITE);
        }
        
        if left_pressed() && self.index == 0 {
            return Backwards
        } else if left_pressed() {
            if self.time == 0.0 {
                self.index -= 1;
                if self.index == 0 {
                    return Backwards
                }
            } else {
                self.time = 0.0;
            }
        } else if right_pressed() {
            if self.index == self.order.len() {
                return Forwards
            } else if self.time > 0.0 {
                self.index += 1;
                self.time = 0.0;
            } else {
                self.time += get_frame_time() * self.speed;
                self.anim_done = false;
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
            self.display_speed = Some(get_time());
        }

        if let Some(time) = self.display_speed && time + 1.5 > get_time() {
            let text = format!("{}x", self.speed);
            let w = measure_text(&text, None, 18, 1.0).width;
            draw_text(&text, self.transform.screen_dims.x / 2.0 - w / 2.0, 58.0, 18.0, WHITE);
        } else {
            self.display_speed = None
        }
    
        if is_key_down(KeyCode::K) {
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color { r: 0.0, g: 0.0, b: 0.0, a: 0.6});
            let offset = if self.time > 0.0 {
                0
            } else {
                1
            };
            draw_tree(&self.ex, &mut self.ignored, Some(self.order[self.index - offset]));
        }
    
        Current
    }
}

pub fn get_total(ex: &Ex) -> Vec<usize> {
    let mut depths = Vec::new();

    for (_, depth) in for_each(ex, true, false) {
        while depths.len() <= depth {
            depths.push(0);
        }
        depths[depth] += 1;
    }

    depths
}

pub fn draw_tree(ex: &Ex, ignored: &mut HashSet<(usize, usize, usize)>, current_ex_index: Option<usize>) -> Vec<usize> {
    let mouse = if is_mouse_button_pressed(MouseButton::Left) {
        let mouse = mouse_position();
        Some(vec2(mouse.0, mouse.1))
    } else {
        None
    };
    let (width, height) = (screen_width(), screen_height());
    let totals = get_total(ex);
    let spacing = *[
        (width as usize - 20) / *totals.iter().max().expect("ex shouldn't be empty"),
        (height as usize - 20) / totals.len(),
        300,
        300
    ].iter().min().unwrap() as f32;

    let polygon_lines = (spacing * 0.5).round().clamp(8.0, 128.0) as u8;

    let mut indicies = Vec::new();

    let x_offset = width / 2.0;
    let y_offset = spacing / 2.0;

    let mut order = Vec::new();

    let vec = for_each(ex, true, false);
    for (index, (ex, depth)) in vec.iter().enumerate() {
        let depth = *depth;
        while indicies.len() <= depth {
            indicies.push(0);
        }

        let i = indicies[depth];
        indicies[depth] += 1;
        
        
        let x = x_offset + spacing * (totals[depth] as f32 / -2.0 + 0.5 + i as f32);
        let y = y_offset + spacing * depth as f32;
        
        if depth != 0 {
            let j = indicies[depth - 1];
            let parent_x = x_offset + spacing * (totals[depth - 1] as f32 / -2.0 + 0.5 + j as f32);
            let parent_y = y_offset + spacing * (depth - 1) as f32;

            draw_line(x, y, parent_x, parent_y, (spacing / 100.0).clamp(2.0, f32::INFINITY), LIGHTGRAY);
        };

        if let Some(mouse) = mouse && mouse.distance_squared(vec2(x, y)) < (spacing / 3.0).powf(2.0) {
            ignored.insert((depth, i, index));
            ignored.retain(|(ignored_depth, index, _)| !(depth > *ignored_depth && indicies[*ignored_depth] == *index));
        }

        let is_ignored = ignored.iter().any(|(ignored_depth, index, _)| depth > *ignored_depth && indicies[*ignored_depth] == *index);

        // This can be optimised by only activating on the tick after left mouse clicked.
        if is_ignored {
            ignored.insert((depth, i, index));
        }

        let colour = if let Some(current_ex_index) = current_ex_index && current_ex_index == index {
            MAGENTA
        } else if is_ignored {
            BROWN
        } else if ignored.contains(&(depth, i, index)) ||
        matches!(ex, ExPointer::Float(FloatEx::Literal(_)) | ExPointer::Vec(VecEx::Literal(_)) | ExPointer::Mat(MatEx::Literal(_))){
            ORANGE
        } else {
            YELLOW
        };
        draw_poly(x, y, polygon_lines, spacing / 3.0, 0.0, colour);

        let text = &ex.to_string();
        let mut scale = (spacing / 3.0) as u16;
        let TextDimensions { width: mut text_width, mut offset_y, .. } = measure_text(text, None, scale, 1.0);

        while
            let TextDimensions { width: test_width, offset_y: test_y, .. } = measure_text(text, None, scale, 1.0) &&
            test_width > spacing / 1.7
        {
            scale -= 1; // could be optimised with binary search instead
            text_width = test_width;
            offset_y = test_y;
        };

        draw_text(text, x - text_width / 2.0, y + offset_y / 3.0, scale as f32, BLUE);

        if !is_ignored {
            order.push(index);
        }
    }

    order
}

// Example of vec:
// [
//     (
//         Mat(MatMul(
//             Literal([1.0, 0.5, -2.0, 0.5]),
//             MatAdd(
//                 Literal([1.0, 2.0, -3.0, 3.0]),
//                 Literal([0.5, -1.0, 1.0, 0.5])
//             )
//         )),
//         0
//     ),
//     (
//         Mat(MatAdd(
//             Literal([1.0, 2.0, -3.0, 3.0]),
//             Literal([0.5, -1.0, 1.0, 0.5])
//         )),
//         1
//     ),
//     (
//         Mat(Literal([0.5, -1.0, 1.0, 0.5])),
//         2
//     ),
//     (
//         Mat(Literal([1.0, 2.0, -3.0, 3.0])),
//         2
//     ),
//     (
//         Mat(Literal([1.0, 0.5, -2.0, 0.5])),
//         1
//     )
// ]