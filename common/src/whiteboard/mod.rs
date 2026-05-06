use macroquad::prelude::*;
use crate::helpers::{lock_camera, mouse_vec, screen_size_vec};
mod remove;
use remove::*;

pub struct Whiteboard {
    commands: Vec<Command>,
    redo: Vec<Command>,
    current_command: Option<Command>,
    screen: RenderTarget
}

pub enum Command {
    Stroke {
        col: Color,
        points: Vec<Vec2>,
        size: f32
    },
    Remove {
        points: Vec<Vec2>,
        size: f32
    }
}

pub enum CommandSel {
    Stroke { col: Color, size: f32 },
    Remove { size: f32 }
}

fn clear_target(target: &RenderTarget) {
    let _ = lock_camera(target);
    clear_background(BLANK);
}

fn get_empty_surface() -> RenderTarget {
    let target = render_target(screen_width() as u32, screen_height() as u32);
    target.texture.set_filter(FilterMode::Nearest);
    clear_target(&target);
    target
}

impl CommandSel {
    fn into_command(self) -> Command {
        match self {
            Self::Stroke { col, size } => Command::Stroke { col, points: vec![mouse_vec()], size },
            Self::Remove { size } => Command::Remove { points: vec![mouse_vec()], size },
        }
    }
}

impl Command {
    fn blit_command(&self) {
        fn draw_spline(points: &[Vec2], col: Color, size: f32) {
            for points in points.windows(2) {
                let [p1, p2] = points else { panic!("Vec::windows returned an array of size different to its input.") };
                draw_line(p1.x, p1.y, p2.x, p2.y, size, col);
                
            }
            for point in points {
                draw_circle(point.x, point.y, size / 2.0, col);
            }
        }

        match self {
            Command::Stroke { col, points, size } => draw_spline(points, *col, *size),
            Command::Remove { points, size } => {
                let _lock = erase_target();
                draw_spline(points, WHITE, *size);
            },
        }
    }
}

impl Whiteboard {
    pub fn new() -> Self {
        Self { commands: Vec::new(), redo: Vec::new(), current_command: None, screen: get_empty_surface() }
    }

    /// This performs the default functions for updating and drawing the whiteboard correctly.
    /// This function should be used after previous drawing calls, so it is drawn ontop of everything,
    /// and the implementation of this function can be used as reference for more specific implementations.
    pub fn default_update(&mut self) {
        self.update();
        self.blit_screen();
        self.handle_controls();
    }

    /// # Warning
    /// This stops the previous command
    pub fn start_command(&mut self, command: CommandSel) {
        self.stop_command(); // This is needed or else the previous command would be cancelled / overwritten instead.
        self.current_command = Some(command.into_command());
    }

    pub fn stop_command(&mut self) {
        if let Some(command) = self.current_command.take() {
            let _lock = lock_camera(&self.screen);
            command.blit_command();
            self.commands.push(command);
            self.redo.clear();
        }
    }

    pub fn update(&mut self) {
        if let Some(command) = &mut self.current_command {
            match command {
                Command::Stroke { col: _, points, size: _ } => points.push(mouse_vec()),
                Command::Remove { points, size: _ } => points.push(mouse_vec()),
            }
        }
    }

    /// This should happen after calling update and before calling next_frame().await, but it doesn't matter that much.
    pub fn handle_controls(&mut self) {
        let ctrl = is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl);
        let shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);

        let mousepad_check = if cfg!(feature = "mousepad_controls") { is_mouse_button_pressed(MouseButton::Right) } else { false };
        let undo = mousepad_check || ctrl && !shift && is_key_pressed(KeyCode::Z);

        let redo = (ctrl && is_key_pressed(KeyCode::Y)) || (ctrl && shift && is_key_pressed(KeyCode::Z));

        if undo {
            self.undo();
        } else if redo {
            self.redo();
        }

        if mousepad_check && ctrl {
            let _lock = lock_camera(&self.screen);
            self.clear();
        }



        const DRAW_SIZE: f32 = 5.0;
        const REMOVE_SIZE: f32 = 80.0;
        const DRAW_COL: Color = RED;

        if is_mouse_button_pressed(MouseButton::Left) {
            if shift || is_mouse_button_down(MouseButton::Middle) {
                self.start_command(CommandSel::Remove { size: REMOVE_SIZE });
            } else {
                self.start_command(CommandSel::Stroke { col: DRAW_COL, size: DRAW_SIZE });
            }
        }
        if is_mouse_button_released(MouseButton::Left) {
            self.stop_command();
        }

        let mouse = mouse_vec();
        if shift || is_mouse_button_down(MouseButton::Middle) {
            draw_circle_lines(mouse.x, mouse.y, REMOVE_SIZE / 2.0, 2.0, WHITE);
        } else {
            draw_circle_lines(mouse.x, mouse.y, DRAW_SIZE / 2.0, 2.0, DRAW_COL);
        }
    }

    pub fn blit_screen(&mut self) {
        if self.screen.texture.size() != screen_size_vec() {
            let surface = get_empty_surface();
            self.screen = surface;
            let _lock = lock_camera(&self.screen);
            self.blit_all_commands();
        }

        draw_texture(&self.screen.texture, 0.0, 0.0, WHITE);
        if let Some(command) = &self.current_command {
            command.blit_command();
        }
    }

    pub fn undo(&mut self) {
        let _lock = lock_camera(&self.screen);
        clear_background(BLANK);
        if let Some(command) = self.commands.pop() { self.redo.push(command) };
        self.blit_all_commands();
    }
    
    /// CAREFUL. You need to lock the camera to self.screen
    fn blit_all_commands(&mut self) {
        for command in &self.commands {
            command.blit_command();
        }
    }

    pub fn redo(&mut self) {
        let _lock = lock_camera(&self.screen);
        if let Some(command) = self.redo.pop() {
            command.blit_command();
            self.commands.push(command)
        };
    }

    /// CAERFUL. You need to lock the camera to self.screen
    pub fn clear(&mut self) {
        clear_background(BLANK);
        self.redo.clear();
        self.commands.clear();
    }

    pub fn cancel_command(&mut self) {
        self.current_command = None
    }
}

impl Default for Whiteboard {
    fn default() -> Self {
        Self::new()
    }
}