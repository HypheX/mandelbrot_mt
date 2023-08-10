use std::{
    fmt,
    ops::{Add, Mul},
};

#[derive(Clone, Copy)]
pub struct Complex {
    pub r: f64,
    pub i: f64,
}

impl Complex {
    pub fn magnitude(&self) -> f64 {
        self.r.hypot(self.i)
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

impl Mul for Complex {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        let r = (self.r * other.r) - (self.i * other.i);
        let i = (self.r * other.i) + (self.i * other.r);

        Self { r, i }
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
