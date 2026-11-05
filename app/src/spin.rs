use std::f32::consts::TAU;
use std::ops::{Add, AddAssign, Sub, SubAssign, Mul};
use rand::RngExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spin(u16);

impl Spin {
    const SCALE: f32 = 65536.0;

    pub fn new(angle: f32) -> Self {
        let normalized = angle.rem_euclid(TAU);
        let value = (normalized * Self::SCALE / TAU) as u16;
        Self(value)
    }

    pub fn angle(self) -> f32 {
        self.0 as f32 * TAU / Self::SCALE
    }

    pub fn sin(self) -> f32 {
        self.angle().sin()
    }

    pub fn cos(self) -> f32 {
        self.angle().cos()
    }

    pub fn raw(self) -> u16 {
        self.0
    }

    pub fn from_raw(value: u16) -> Self {
        Self(value)
    }

    pub fn random_range(
        rng: &mut impl rand::Rng,
        min: i16,
        max: i16,
    ) -> Self {
        Self(rng.random_range(min..max) as u16)
    }
}

// Spin + Spin
impl Add for Spin {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0.wrapping_add(rhs.0))
    }
}

// Spin + f32
impl Add<f32> for Spin {
    type Output = Self;

    fn add(self, rhs: f32) -> Self {
        Self::new(self.angle() + rhs)
    }
}

// Spin - Spin
impl Sub for Spin {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(self.0.wrapping_sub(rhs.0))
    }
}

// Spin - f32
impl Sub<f32> for Spin {
    type Output = Self;

    fn sub(self, rhs: f32) -> Self {
        Self::new(self.angle() - rhs)
    }
}

// Spin += Spin
impl AddAssign for Spin {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0.wrapping_add(rhs.0);
    }
}

// Spin += f32
impl AddAssign<f32> for Spin {
    fn add_assign(&mut self, rhs: f32) {
        *self = *self + rhs;
    }
}

// Spin -= Spin
impl SubAssign for Spin {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = self.0.wrapping_sub(rhs.0);
    }
}

// Spin -= f32
impl SubAssign<f32> for Spin {
    fn sub_assign(&mut self, rhs: f32) {
        *self = *self - rhs;
    }
}

// Spin * f32
impl Mul<f32> for Spin {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Self::new(self.angle() * rhs)
    }
}

// Konwersja do f32
impl From<Spin> for f32 {
    fn from(spin: Spin) -> f32 {
        spin.angle()
    }
}
