use macroquad::prelude::*;
use std::f32::consts::{FRAC_PI_2, PI, TAU};

mod ising_engine;
mod renderer;

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);

    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

fn wrap_angle(angle: f32) -> f32 {
    angle.rem_euclid(TAU)
}

fn spin_to_color(angle: f32) -> Color {
    let angle = wrap_angle(angle);

    // Punkty bazowe koloru wokół okręgu
    let c0 = Color::new(0.10, 0.08, 0.12, 1.0);
    let c90 = Color::new(0.95, 0.72, 0.50, 1.0);
    let c180 = Color::new(1.00, 0.55, 0.78, 1.0);
    let c270 = Color::new(0.72, 0.58, 0.95, 1.0);

    // Interpolacja po ćwiartkach
    let base = if angle < FRAC_PI_2 {
        let t = angle / FRAC_PI_2;
        lerp_color(c0, c90, t)
    } else if angle < PI {
        let t = (angle - FRAC_PI_2) / FRAC_PI_2;
        lerp_color(c90, c180, t)
    } else if angle < 3.0 * FRAC_PI_2 {
        let t = (angle - PI) / FRAC_PI_2;
        lerp_color(c180, c270, t)
    } else {
        let t = (angle - 3.0 * FRAC_PI_2) / FRAC_PI_2;
        lerp_color(c270, c0, t)
    };

    let brightness = 1.0 - ((angle - PI).abs() / PI);

    Color::new(
        base.r * brightness,
        base.g * brightness,
        base.b * brightness,
        1.0,
    )
}

fn draw_rounded_rect(x: f32, y: f32, w: f32, h: f32, radius: f32, color: Color) {
    let r = radius.min(w * 0.5).min(h * 0.5);

    // środek
    draw_rectangle(x + r, y, w - 2.0 * r, h, color);
    draw_rectangle(x, y + r, w, h - 2.0 * r, color);

    // narożniki
    draw_circle(x + r, y + r, r, color);
    draw_circle(x + w - r, y + r, r, color);
    draw_circle(x + r, y + h - r, r, color);
    draw_circle(x + w - r, y + h - r, r, color);
}

fn render_spin_matrix(
    spin_matrix: &[f32],
    matrix_x: usize,
    matrix_y: usize,
    start_x: f32,
    start_y: f32,
    cell_size: f32,
    padding: f32,
    rounding: f32,
) {
    for y in 0..matrix_y {
        for x in 0..matrix_x {
            let idx = y * matrix_x + x;
            let angle = spin_matrix[idx];
            let color = spin_to_color(angle);

            let px = start_x + x as f32 * cell_size + padding * 0.5;
            let py = start_y + y as f32 * cell_size + padding * 0.5;
            let size = cell_size - padding;

            draw_rounded_rect(px, py, size, size, rounding, color);
        }
    }
}

const WINDOW_X: i32 = 1200;
const WINDOW_Y: i32 = 1200;

const MATRIX_X: usize = 100;
const MATRIX_Y: usize = 100;

const CELL_PADDING: f32 = 0.12;
const CELL_ROUNDING: f32 = 0.35;

fn window_conf() -> Conf {
    Conf {
        window_title: "XY Model".to_owned(),
        window_width: WINDOW_X,
        window_height: WINDOW_Y,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut spin_matrix = ising_engine::generate_initial_state(MATRIX_X, MATRIX_Y);
    println!("Pierwsze spiny: {:?}", &spin_matrix[..10]);

    let mut image = Image::gen_image_color(MATRIX_X as u16, MATRIX_Y as u16, BLACK);
    renderer::update_image(&mut image, &spin_matrix);

    let texture = Texture2D::from_image(&image);
    texture.set_filter(FilterMode::Nearest);

    let material = renderer::create_material(MATRIX_X, MATRIX_Y);

    // External Field
    const H: f32 = 0.010;
    const A: f32 = 6.0;
    const B: f32 = 4.0;

    let external_field: Box<[ising_engine::ExternalField]> = (0..MATRIX_X * MATRIX_Y)
        .map(|i| {
            let x = (i % MATRIX_X) as f32 / MATRIX_X as f32;
            let y = (i / MATRIX_X) as f32 / MATRIX_Y as f32;

            ising_engine::ExternalField {
                x: H * (TAU * A * x).sin(),
                y: H * (TAU * B * y).cos(),
            }
        })
        .collect();

    let params = ising_engine::IsingModelParameters {
        temperature: 0.1,
        coupling_j: 1.0,
        matrix_x: MATRIX_X,
        matrix_y: MATRIX_Y,
    };

    loop {
        ising_engine::ising_step(&mut spin_matrix, &external_field, &params);

        renderer::update_image(&mut image, &spin_matrix);
        texture.update(&image);

        clear_background(BLACK);
        renderer::render(&texture, &material, MATRIX_X, MATRIX_Y);

        next_frame().await;
    }
}
