use crate::ising_engine::ExternalField;
use std::f32::consts::TAU;

#[derive(Clone, Copy, PartialEq)]
pub enum FieldEquation {
    Sinusoidal {
        x_cycles: f32,
        y_cycles: f32,
    },
    Polynomial {
        x_linear: f32,
        x_quadratic: f32,
        y_linear: f32,
        y_quadratic: f32,
    },
}

#[derive(Clone, Copy, PartialEq)]
pub struct FieldConfig {
    pub strength: f32,
    pub equation: FieldEquation,
}

impl Default for FieldConfig {
    fn default() -> Self {
        Self {
            strength: 0.02,
            equation: FieldEquation::Sinusoidal {
                x_cycles: 3.0,
                y_cycles: 4.0,
            },
        }
    }
}

impl FieldConfig {
    pub fn generate(&self, field: &mut [ExternalField], width: usize, height: usize) {
        assert_eq!(field.len(), width * height);
        for (i, value) in field.iter_mut().enumerate() {
            let x = (i % width) as f32 / width as f32;
            let y = (i / width) as f32 / height as f32;
            let (fx, fy) = match self.equation {
                FieldEquation::Sinusoidal { x_cycles, y_cycles } => {
                    ((TAU * x_cycles * x).sin(), (TAU * y_cycles * y).cos())
                }
                FieldEquation::Polynomial {
                    x_linear,
                    x_quadratic,
                    y_linear,
                    y_quadratic,
                } => (
                    x_linear * x + x_quadratic * x * x,
                    y_linear * y + y_quadratic * y * y,
                ),
            };
            value.x = self.strength * fx;
            value.y = self.strength * fy;
        }
    }
}
