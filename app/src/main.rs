use display_info::DisplayInfo;
use macroquad::prelude::*;

mod control_panel;
mod field;
mod ising_engine;
mod renderer;
mod spin;

const WINDOW_X: i32 = 1200;
const WINDOW_Y: i32 = 1200;

// Size of one simulation cell in logical pixels on the selected display.
const CELL_SIZE: f32 = 20.0;

fn get_display_info() -> Vec<DisplayInfo> {
    let displays = DisplayInfo::all().expect("failed to enumerate displays");
    for d in &displays {
        println!(
            "{} (id {}): {}x{} at ({}, {}), scale {}, {} Hz, primary: {}",
            d.name, d.id, d.width, d.height, d.x, d.y, d.scale_factor, d.frequency, d.is_primary
        );
    }
    displays
}
fn window_conf() -> Conf {
    let displays = DisplayInfo::all().expect("failed to enumerate displays");
    let width = displays
        .iter()
        .map(|d| d.width as f32 / d.scale_factor)
        .fold(WINDOW_X as f32, f32::min);
    let height = displays
        .iter()
        .map(|d| d.height as f32 / d.scale_factor)
        .fold(WINDOW_Y as f32 + 100.0, f32::min);
    Conf {
        window_title: "XY Model".to_owned(),
        window_width: (width - 40.0).max(1.0) as i32,
        window_height: (height - 100.0).max(1.0) as i32,
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let display = get_display_info()
        .into_iter()
        .max_by_key(|d| u64::from(d.width) * u64::from(d.height))
        .expect("no displays found");
    let mut panel = control_panel::ControlPanel::new(CELL_SIZE);
    let cell_pixels = CELL_SIZE * display.scale_factor;
    let matrix_x = (display.width as f32 / cell_pixels).ceil() as usize;
    let matrix_y = (display.height as f32 / cell_pixels).ceil() as usize;
    assert!(
        matrix_x >= 3 && matrix_y >= 3,
        "simulation grid must be at least 3x3"
    );
    let image_width = u16::try_from(matrix_x).expect("simulation width exceeds image limit");
    let image_height = u16::try_from(matrix_y).expect("simulation height exceeds image limit");

    let mut spin_matrix = ising_engine::generate_initial_state(matrix_x, matrix_y);

    let mut image = Image::gen_image_color(image_width, image_height, BLACK);
    renderer::update_image(&mut image, &spin_matrix);

    let texture = Texture2D::from_image(&image);
    texture.set_filter(FilterMode::Nearest);

    let material = renderer::create_material(matrix_x, matrix_y);

    let mut external_field: Box<[ising_engine::ExternalField]> = (0..matrix_x * matrix_y)
        .map(|_| ising_engine::ExternalField { x: 0.0, y: 0.0 })
        .collect();
    let mut field_config = field::FieldConfig::default();
    field_config.generate(&mut external_field, matrix_x, matrix_y);

    let mut params = ising_engine::IsingModelParameters {
        temperature: 0.2,
        coupling_j: 1.0,
        matrix_x,
        matrix_y,
    };

    const TPS_SAMPLE_SECONDS: f64 = 1.0;
    const TPS_TOLERANCE: f64 = 0.05;
    let mut target_tps = 144.0_f32;
    let mut current_tps = 0.0_f64;
    let mut sample_start = get_time();
    let mut sample_ticks = 0_u32;
    let mut sleep_seconds = 0.0_f64;

    loop {
        ising_engine::ising_step(&mut spin_matrix, &external_field, &params);

        renderer::update_image(&mut image, &spin_matrix);
        texture.update(&image);

        clear_background(BLACK);
        renderer::render(&texture, &material, matrix_x, matrix_y, cell_pixels);

        let old_field = field_config;
        panel.draw(
            &display,
            &mut params,
            &mut field_config,
            &mut target_tps,
            current_tps,
        );
        if old_field != field_config {
            field_config.generate(&mut external_field, matrix_x, matrix_y);
        }

        next_frame().await;
        sample_ticks += 1;
        let now = get_time();
        let elapsed = now - sample_start;
        if elapsed >= TPS_SAMPLE_SECONDS {
            current_tps = f64::from(sample_ticks) / elapsed;
            let target = f64::from(target_tps);
            if current_tps < target * (1.0 - TPS_TOLERANCE)
                || current_tps > target * (1.0 + TPS_TOLERANCE)
            {
                sleep_seconds = (sleep_seconds
                    + 0.5 * (1.0 / target - elapsed / f64::from(sample_ticks)))
                    .max(0.0);
            }
            sample_ticks = 0;
            sample_start = now;
        }
        if sleep_seconds > 0.0 {
            std::thread::sleep(std::time::Duration::from_secs_f64(sleep_seconds));
        }
    }
}
