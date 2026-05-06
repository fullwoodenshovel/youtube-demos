use std::sync::LazyLock;

use macroquad::{miniquad::{BlendFactor, BlendState, Equation}, prelude::*};

static MATERIAL: LazyLock<Material> = LazyLock::new(make_erase_material);

fn make_erase_material() -> Material {
    load_material(
        ShaderSource::Glsl {
            // Standard passthrough shaders — the blend mode does the work
            vertex: "
                #version 100
                attribute vec3 position;
                attribute vec4 color0;
                uniform mat4 Model;
                uniform mat4 Projection;
                varying vec4 color;
                void main() {
                    color = color0 / 255.0;
                    gl_Position = Projection * Model * vec4(position, 1.0);
                }
            ",
            fragment: "
                #version 100
                void main() {
                    gl_FragColor = vec4(0.0, 0.0, 0.0, 0.0);
                }
            ",
        },
        MaterialParams {
            pipeline_params: PipelineParams {
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Zero, // src factor
                    BlendFactor::Zero, // dst factor → zeroes out existing pixels
                )),
                alpha_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Zero,
                    BlendFactor::Zero,
                )),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .unwrap()
}

/// Makes the camera surface whatever it was, but all draw calls become an erase
pub fn erase_target() -> EraseLock {
    gl_use_material(&MATERIAL);
    EraseLock
}

pub struct EraseLock;

impl Drop for EraseLock {
    fn drop(&mut self) {
        gl_use_default_material();
    }
}