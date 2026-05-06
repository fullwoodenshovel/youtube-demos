use std::sync::{LazyLock, Mutex};

use common_matrix::*;
use common_matrix::helpers::conf;
use macroquad::prelude::*;

static WHITEBOARD: LazyLock<Mutex<whiteboard::Whiteboard>> = LazyLock::new(|| Mutex::new(whiteboard::Whiteboard::new()));

fn update_whiteboard() {
    WHITEBOARD.lock().unwrap().default_update();
}

#[macroquad::main(conf)]
async fn main() {
    loop {
        update_whiteboard();
        next_frame().await;
    }
}