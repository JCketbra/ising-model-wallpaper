use crate::field::{FieldConfig, FieldEquation};
use crate::ising_engine::IsingModelParameters;
use display_info::DisplayInfo;
use macroquad::{
    prelude::*,
    ui::{hash, root_ui},
};

const BUTTON_WIDTH: f32 = 100.0;
const BUTTON_HEIGHT: f32 = 30.0;
const MASK_VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
varying lowp vec2 uv;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    uv = texcoord;
}"#;
const MASK_FRAGMENT: &str = r#"#version 100
precision lowp float;
varying lowp vec2 uv;
uniform sampler2D Texture;
void main() {
    gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0 - texture2D(Texture, uv).a);
}"#;

fn button_mask() -> (RenderTarget, Material) {
    let target = render_target(BUTTON_WIDTH as u32, BUTTON_HEIGHT as u32);
    target.texture.set_filter(FilterMode::Linear);
    set_camera(&Camera2D {
        render_target: Some(target.clone()),
        ..Camera2D::from_display_rect(Rect::new(0.0, 0.0, BUTTON_WIDTH, BUTTON_HEIGHT))
    });
    clear_background(BLANK);
    let text = "Controls";
    let font_size = 22;
    let size = measure_text(text, None, font_size, 1.0);
    // Thicken the cutout once when creating the mask, not every frame.
    for dx in -1..=1 {
        for dy in -1..=1 {
            draw_text(
                text,
                (BUTTON_WIDTH - size.width) * 0.5 + dx as f32,
                (BUTTON_HEIGHT + size.height) * 0.5 + dy as f32,
                font_size as f32,
                WHITE,
            );
        }
    }
    set_default_camera();
    let material = load_material(
        ShaderSource::Glsl {
            vertex: MASK_VERTEX,
            fragment: MASK_FRAGMENT,
        },
        MaterialParams {
            pipeline_params: macroquad::miniquad::PipelineParams {
                color_blend: Some(macroquad::miniquad::BlendState::new(
                    macroquad::miniquad::Equation::Add,
                    macroquad::miniquad::BlendFactor::Value(
                        macroquad::miniquad::BlendValue::SourceAlpha,
                    ),
                    macroquad::miniquad::BlendFactor::OneMinusValue(
                        macroquad::miniquad::BlendValue::SourceAlpha,
                    ),
                )),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .expect("failed to create button mask shader");
    (target, material)
}

pub struct ControlPanel {
    open: bool,
    mask: RenderTarget,
    mask_material: Material,
    cell_size: f32,
    sinusoidal: FieldEquation,
    polynomial: FieldEquation,
}

impl ControlPanel {
    pub fn new(cell_size: f32) -> Self {
        let (mask, mask_material) = button_mask();
        Self {
            open: false,
            mask,
            mask_material,
            cell_size,
            sinusoidal: FieldConfig::default().equation,
            polynomial: FieldEquation::Polynomial {
                x_linear: 1.0,
                x_quadratic: 0.0,
                y_linear: 1.0,
                y_quadratic: 0.0,
            },
        }
    }

    pub fn draw(
        &mut self,
        display: &DisplayInfo,
        params: &mut IsingModelParameters,
        field: &mut FieldConfig,
        target_tps: &mut f32,
        current_tps: f64,
    ) {
        let x = (screen_width() - BUTTON_WIDTH) * 0.5;
        let rect = Rect::new(x, 0.0, BUTTON_WIDTH, BUTTON_HEIGHT);
        if is_mouse_button_pressed(MouseButton::Left) && rect.contains(mouse_position().into()) {
            self.open = !self.open;
        }
        gl_use_material(&self.mask_material);
        draw_texture_ex(
            &self.mask.texture,
            x,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(BUTTON_WIDTH, BUTTON_HEIGHT)),
                flip_y: true,
                ..Default::default()
            },
        );
        gl_use_default_material();
        if !self.open {
            return;
        }

        let mut ui = root_ui();
        let width = 360.0_f32.min(screen_width());
        ui.window(
            hash!(),
            vec2((screen_width() - width) * 0.5, 30.0),
            vec2(width, 510.0),
            |ui| {
                ui.label(
                    None,
                    &format!(
                        "Display: {} ({} x {})",
                        display.name, display.width, display.height
                    ),
                );
                ui.label(
                    None,
                    &format!(
                        "DPI scale: {:.2}   Grid: {} x {}",
                        display.scale_factor, params.matrix_x, params.matrix_y
                    ),
                );
                ui.label(None, &format!("Current TPS: {:.1}", current_tps));
                ui.slider(hash!(), "Target TPS", 1.0..480.0, target_tps);
                ui.slider(hash!(), "Temperature", 0.0..5.0, &mut params.temperature);
                ui.slider(hash!(), "Coupling", -5.0..5.0, &mut params.coupling_j);
                ui.slider(hash!(), "Field strength", 0.0..2.0, &mut field.strength);
                if ui.button(None, "Sinusoidal") {
                    field.equation = self.sinusoidal;
                }
                if ui.button(None, "Polynomial") {
                    field.equation = self.polynomial;
                }
                match &mut field.equation {
                    FieldEquation::Sinusoidal { x_cycles, y_cycles } => {
                        ui.label(None, "Equation: sinusoidal");
                        ui.slider(hash!(), "Field X cycles", 0.0..20.0, x_cycles);
                        ui.slider(hash!(), "Field Y cycles", 0.0..20.0, y_cycles);
                        self.sinusoidal = field.equation;
                    }
                    FieldEquation::Polynomial {
                        x_linear,
                        x_quadratic,
                        y_linear,
                        y_quadratic,
                    } => {
                        ui.label(None, "Equation: polynomial");
                        ui.slider(hash!(), "X linear", -5.0..5.0, x_linear);
                        ui.slider(hash!(), "X quadratic", -5.0..5.0, x_quadratic);
                        ui.slider(hash!(), "Y linear", -5.0..5.0, y_linear);
                        ui.slider(hash!(), "Y quadratic", -5.0..5.0, y_quadratic);
                        self.polynomial = field.equation;
                    }
                }
                ui.label(
                    None,
                    &format!("Cell size: {:.1} (set at startup)", self.cell_size),
                );
            },
        );
    }
}
