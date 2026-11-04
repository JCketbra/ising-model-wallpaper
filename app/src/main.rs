use macroquad::prelude::*;
use std::f32::consts::TAU;

mod ising_engine;
mod renderer;

const WINDOW_X: i32 = 1200;
const WINDOW_Y: i32 = 1200;

const MATRIX_X: usize = 100;
const MATRIX_Y: usize = 100;

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
    const H: f32 = 0.02;
    const A: f32 = 3.0;
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
        temperature: 0.2,
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
