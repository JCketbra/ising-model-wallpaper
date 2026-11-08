use rand::RngExt;

use crate::spin::Spin;
pub fn generate_initial_state(matrix_x: usize, matrix_y: usize) -> Box<[Spin]> {
    let mut rng = rand::rng();

    let spin_matrix: Box<[Spin]> = (0..matrix_x * matrix_y)
        .map(|_| Spin::from_raw(rng.random()))
        .collect::<Vec<_>>()
        .into_boxed_slice();

    spin_matrix
}

pub struct ExternalField {
    pub x: f32,
    pub y: f32,
}

pub struct IsingModelParameters {
    pub temperature: f32,
    pub coupling_j: f32,
    pub matrix_x: usize,
    pub matrix_y: usize,
}

pub fn ising_step(
    spin_matrix: &mut [Spin],
    external_field: &[ExternalField],
    params: &IsingModelParameters,
) {
    let width = params.matrix_x;
    let height = params.matrix_y;

    assert!(width >= 3 && height >= 3);
    assert_eq!(spin_matrix.len(), width * height);
    assert_eq!(external_field.len(), width * height);
    assert!(params.temperature >= 0.0);

    let mut rng = rand::rng();

    for _ in 0..spin_matrix.len() {
        let x = rng.random_range(0..width);
        let y = rng.random_range(0..height);
        let i = y * width + x;

        let left = y * width + (x + width - 1) % width;
        let right = y * width + (x + 1) % width;
        let up = ((y + height - 1) % height) * width + x;
        let down = ((y + 1) % height) * width + x;

        let old_angle = spin_matrix[i];
        let delta = Spin::random_range(&mut rng, -8192, 8192);

        let new_angle = old_angle + delta;

        let mut ax = external_field[i].x;
        let mut ay = external_field[i].y;

        for neighbor in [left, right, up, down] {
            let angle = spin_matrix[neighbor];

            ax += params.coupling_j * angle.cos();
            ay += params.coupling_j * angle.sin();
        }

        let delta_energy =
            ax * (old_angle.cos() - new_angle.cos()) + ay * (old_angle.sin() - new_angle.sin());

        if delta_energy <= 0.0
            || (params.temperature > 0.0
                && rng.random::<f32>() < (-delta_energy / params.temperature).exp())
        {
            spin_matrix[i] = new_angle;
        }
    }
}
