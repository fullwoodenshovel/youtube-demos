mod mat2;
#[cfg(target_family = "unix")]
use input_handler::InputHandler;
#[cfg(not(target_family = "unix"))]
use parse::InputHandler;
use std::collections::HashSet;
use std::{collections::HashMap, f32};
mod parse;
use parse::{FloatEx, MatEx, VecEx, parse_exp, Ex, Line, resolve_ex};
use parse::for_each::{ExPointer, for_each, resolve_indexed};
use parse::visualise::{visualise, display_background, visualise_obj};
mod transform;
use transform::{Transform, get_screen_dims};
use macroquad::prelude::*;
#[cfg(target_arch = "wasm32")]
mod web;

// There are many checks that wont change from most frames to the next, but are checked each frame. these can be optimised.
// For example:
//  - When drawing the tree, finding children of ignored is done every frame
//  - When visualising, figuring out what objects to draw in the background is done every frame
// Using a flamegraph shows that optimisation isn't a big issue. What takes most CPU time is macroquad instead.

fn conf() -> Conf {
    Conf {
        window_title: "Matrix Visualiser".to_string(),
        sample_count: 4,
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {
    let mut vars = HashMap::new();
    vars.entry("pi".to_string()).insert_entry(parse::Obj::Float(f32::consts::PI));
    vars.entry("tau".to_string()).insert_entry(parse::Obj::Float(f32::consts::TAU));
    vars.entry("e".to_string()).insert_entry(parse::Obj::Float(f32::consts::E));
    vars.entry("I".to_string()).insert_entry(parse::Obj::Mat(mat2::I));
    #[cfg(not(target_family = "unix"))]
    let mut handler = InputHandler;
    #[cfg(target_family = "unix")]
    let mut handler = InputHandler::new().expect("Failed to initialise InputHandler");
    display_go_to_term().await;
    loop {
        let Some((line, show)) = parse_exp(&vars, &mut handler) else {
            // On WASM: no input yet — yield one frame so the browser stays
            // responsive instead of spinning. On native this branch is never
            // reached because input() blocks until stdin returns.
            #[cfg(target_arch = "wasm32")]
            next_frame().await;
            continue;
        };

        let mut set_var = None;
        let ex = match line {
            Line::Eval(ex) => ex,
            Line::SetVar(var, ex) => {set_var = Some(var); ex},
            Line::None => continue,
        };

        let result = resolve_ex(&ex);
        
        if let Some(var) = set_var {
            vars.entry(var).insert_entry(result);
        } else {
            #[cfg(not(target_arch = "wasm32"))]
            println!("{result:?}");
            #[cfg(target_arch = "wasm32")]
            web::push_output(format!("{result:?}"));
        }

        if show {
            #[cfg(not(target_arch = "wasm32"))]
            println!("Go to window for visualisation.");
            #[cfg(target_arch = "wasm32")]
            web::set_show_mode(true);

            graphics(&ex).await;

            #[cfg(target_arch = "wasm32")]
            web::set_show_mode(false);

            display_go_to_term().await;
        }
    }
}

fn right_pressed() -> bool {
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

fn left_pressed() -> bool {
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

async fn graphics(ex: &Ex) {
    let mut ignored = HashSet::new();
    const SPEEDS: [f32; 9] = [0.1, 0.2, 0.5, 0.8, 1.0, 1.2, 1.5, 2.0, 2.5];
    let mut speed_index = 4;
    let mut speed = SPEEDS[speed_index];
    'main: loop {
        next_frame().await;

        let order = 'tree: loop {
            clear_background(BLACK);

            let order = draw_tree(ex, &mut ignored, None);
            
            if right_pressed() {
                break 'tree order
            }
            next_frame().await;
        };
        
        next_frame().await;
        
        let mut display_speed = 0.0;
        let mut index = 0;
        let mut time = get_frame_time() * speed;
        let mut anim_done = false;
        let mut transform = Transform::new(Vec2::new(0.0, 0.0), 0.01);

        'visualise: loop {
            clear_background(BLACK);
            transform.move_camera();
            transform.screen_dims = get_screen_dims();
            display_background(&transform);

            let vec = for_each(ex, false, true);

            // This algorithm does the following:
            // walk the tree backwards, until reaching index of target. all these values are special cases and do not need to be displayed.
            // set target_depth to depth of this index, and display.
            // when walking backwards, if target_depth >= depth, display and set target_depth to depth.
            // This doesnt work if order isnt in ascending order.
            // This can be optimised as vec is just the normal for_each reversed. this means the indexing can be identified easily. vec variable is unnecessary
            if index < order.len() {
                let current_ex = for_each(ex, true, false)[order[index]].0;
                let new_index = vec.iter().position(|d| d.0.pointer_eq(current_ex)).unwrap();
                let mut target_depth = if time == 0.0 {
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
                                parent_shown = !ignored.iter().any(|(_, _, target_index)| *target_index == index);
                                break
                            }
                        }
                        if parent_shown {
                            visualise_obj(ex.resolve(), &mut transform, true);
                        }
                    }
                }
            }

            if time > 0.0 {
                if !anim_done {
                    if ignored.iter().any(|(_, _, ignored_index)| *ignored_index == order[index]) {
                        anim_done = true;
                    } else {
                        anim_done = visualise(order[index], time, ex, &mut transform);
                    }
                    time += get_frame_time() * speed;
                }
                if anim_done {
                    index += 1;
                    time = 0.0;
                }
            }

            if time == 0.0 {
                let obj = resolve_indexed(order[index - 1], ex);
                visualise_obj(obj, &mut transform, false);

                let text = obj.to_string();
                let w = measure_text(&text, None, 18, 1.0).width;
                draw_text(&text, transform.screen_dims.x / 2.0 - w / 2.0, 26.0, 18.0, WHITE);
            }
            
            if display_speed + 1.5 > get_time() {
                let text = format!("{speed}x");
                let w = measure_text(&text, None, 18, 1.0).width;
                draw_text(&text, transform.screen_dims.x / 2.0 - w / 2.0, 58.0, 18.0, WHITE);
            }

            if left_pressed() && index == 0 {
                break 'visualise
            } else if left_pressed() {
                if time == 0.0 {
                    index -= 1;
                    if index == 0 {
                        break 'visualise
                    }
                } else {
                    time = 0.0;
                }
            } else if right_pressed() {
                if index == order.len() {
                    loop {
                        draw_text("End of visualisation. Click right to continue.", 50.0, 50.0, 30.0, WHITE);
                        next_frame().await;
                        if left_pressed() {
                            break
                        } else if right_pressed() {
                            break 'main
                        }
                    }
                } else if time > 0.0 {
                    index += 1;
                    time = 0.0;
                } else {
                    time += get_frame_time() * speed;
                    anim_done = false;
                }
            } else if let up = is_key_pressed(KeyCode::Up) && (up || is_key_pressed(KeyCode::Down)) {
                if up {
                    if speed_index != 8 {
                        speed_index += 1
                    }
                } else {
                    speed_index = speed_index.saturating_sub(1)
                };
                speed = SPEEDS[speed_index];
                display_speed = get_time();
            }

            if is_key_down(KeyCode::K) {
                draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color { r: 0.0, g: 0.0, b: 0.0, a: 0.6});
                let offset = if time > 0.0 {
                    0
                } else {
                    1
                };
                draw_tree(ex, &mut ignored, Some(order[index - offset]));
            }

            next_frame().await;
        }
    }
}

fn get_total(ex: &Ex) -> Vec<usize> {
    let mut depths = Vec::new();

    for (_, depth) in for_each(ex, true, false) {
        while depths.len() <= depth {
            depths.push(0);
        }
        depths[depth] += 1;
    }

    depths
}

fn draw_tree(ex: &Ex, ignored: &mut HashSet<(usize, usize, usize)>, current_ex_index: Option<usize>) -> Vec<usize> {
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

async fn display_go_to_term() {
    clear_background(BLACK);
    #[cfg(target_arch = "wasm32")]
    draw_text("End of visualisation. Click right to continue.", 50.0, 50.0, 30.0, WHITE);
    #[cfg(not(target_arch = "wasm32"))]
    draw_text("Enter input in terminal", 50.0, 50.0, 30.0, WHITE);
    next_frame().await;
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