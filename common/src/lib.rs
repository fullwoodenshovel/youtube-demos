pub mod mat2;
pub mod transform;
pub mod whiteboard;
pub mod helpers;
#[cfg(target_arch = "wasm32")]
pub mod web;

pub mod prelude {
    pub use super::transform::{Transform, get_screen_dims};
    pub use super::helpers::{FrameCheck, FrameData, conf, default_main, mouse_vec, smooth_step};
}