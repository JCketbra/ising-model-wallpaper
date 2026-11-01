use macroquad::prelude::*;
use std::f32::consts::{FRAC_PI_2, PI, TAU};

const PADDING: f32 = 0.12;
const ROUNDING: f32 = 0.25;

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        1.0,
    )
}

fn spin_to_color(angle: f32) -> Color {
    let angle = angle.rem_euclid(TAU);

    let c0 = Color::new(0.10, 0.08, 0.12, 1.0);
    let c90 = Color::new(0.95, 0.72, 0.50, 1.0);
    let c180 = Color::new(1.00, 0.55, 0.78, 1.0);
    let c270 = Color::new(0.72, 0.58, 0.95, 1.0);

    let base = if angle < FRAC_PI_2 {
        lerp_color(c0, c90, angle / FRAC_PI_2)
    } else if angle < PI {
        lerp_color(c90, c180, (angle - FRAC_PI_2) / FRAC_PI_2)
    } else if angle < 3.0 * FRAC_PI_2 {
        lerp_color(c180, c270, (angle - PI) / FRAC_PI_2)
    } else {
        lerp_color(c270, c0, (angle - 3.0 * FRAC_PI_2) / FRAC_PI_2)
    };

    let brightness = 1.0 - (angle - PI).abs() / PI;

    Color::new(
        base.r * brightness,
        base.g * brightness,
        base.b * brightness,
        1.0,
    )
}

pub fn update_image(image: &mut Image, spins: &[f32]) {
    let pixels = image.get_image_data_mut();
    assert_eq!(pixels.len(), spins.len());

    for (pixel, &angle) in pixels.iter_mut().zip(spins.iter()) {
        *pixel = spin_to_color(angle).into();
    }
}

// Shader wierzchołków
const VERTEX_SHADER: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;

varying highp vec2 uv;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    uv = texcoord;
}
"#;

const FRAGMENT_SHADER: &str = r#"#version 100
precision highp float;

varying vec2 uv;

uniform sampler2D Texture;
uniform vec2 GridSize;
uniform float CellPixels;
uniform float Padding;
uniform float Radius;

void main() {
    // Indeks spinu w teksturze
    vec2 index = floor(uv * GridSize);
    vec2 sample_uv = (index + 0.5) / GridSize;

    vec3 color = texture2D(Texture, sample_uv).rgb;

    // Przy bardzo małych komórkach nie dodajemy odstępów
    if (CellPixels < 4.0) {
        gl_FragColor = vec4(color, 1.0);
        return;
    }

    // Pozycja lokalna wewnątrz pojedynczej komórki
    vec2 p = fract(uv * GridSize) - 0.5;

    float half_size = 0.5 - Padding * 0.5;
    float r = min(Radius, half_size);

    // Signed Distance Field zaokrąglonego kwadratu
    vec2 q = abs(p) - vec2(half_size - r);
    float distance = length(max(q, 0.0))
                   + min(max(q.x, q.y), 0.0) - r;

    // Wygładzenie krawędzi
    float aa = 0.5 / CellPixels;
    float mask = 1.0 - smoothstep(-aa, aa, distance);

    // Tło pomiędzy komórkami
    vec3 background = vec3(0.06, 0.06, 0.08);

    gl_FragColor = vec4(mix(background, color, mask), 1.0);
}
"#;

pub fn create_material(width: usize, height: usize) -> Material {
    let material = load_material(
        ShaderSource::Glsl {
            vertex: VERTEX_SHADER,
            fragment: FRAGMENT_SHADER,
        },
        MaterialParams {
            uniforms: vec![
                UniformDesc::new("GridSize", UniformType::Float2),
                UniformDesc::new("CellPixels", UniformType::Float1),
                UniformDesc::new("Padding", UniformType::Float1),
                UniformDesc::new("Radius", UniformType::Float1),
            ],
            ..Default::default()
        },
    )
    .expect("Nie można skompilować shadera");

    material.set_uniform("GridSize", vec2(width as f32, height as f32));
    material.set_uniform("Padding", PADDING);
    material.set_uniform("Radius", ROUNDING);

    material
}

pub fn render(texture: &Texture2D, material: &Material, width: usize, height: usize) {
    let cell_size = (screen_width() / width as f32).min(screen_height() / height as f32);

    let render_width = width as f32 * cell_size;
    let render_height = height as f32 * cell_size;

    let x = (screen_width() - render_width) * 0.5;
    let y = (screen_height() - render_height) * 0.5;

    material.set_uniform("CellPixels", cell_size);

    gl_use_material(material);

    draw_texture_ex(
        texture,
        x,
        y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(render_width, render_height)),
            ..Default::default()
        },
    );

    gl_use_default_material();
}
