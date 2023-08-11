use std::{
    fmt,
    ops::{Add, Mul},
};

#[derive(Clone, Copy, Default)]
pub struct Complex {
    pub r: f64,
    pub i: f64,
}

impl Complex {
    #[must_use]
    pub fn magnitude(&self) -> f64 {
        self.r.hypot(self.i)
    }

    pub fn mandelbrot_iter(&mut self, c: Complex) {
        *self *= *self;
        *self += c;
    }

    #[must_use]
    pub fn mandelbrot_escaped(&self) -> bool {
        self.magnitude() > 2.
    }
}

impl Add for Complex {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let r = self.r + other.r;
        let i = self.i + other.i;

        Self { r, i }
    }
}

impl core::ops::AddAssign for Complex {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Mul for Complex {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        let r = self.r * other.r + (self.i * -other.i);
        let i = self.r * other.i + (self.i * other.r);

        Self { r, i }
    }
}

impl core::ops::MulAssign for Complex {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl fmt::Display for Complex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.i >= 0.0 {
            write!(f, "{0}+{1}i", self.r, self.i)
        } else {
            write!(f, "{0}{1}i", self.r, self.i)
        }
    }
}
