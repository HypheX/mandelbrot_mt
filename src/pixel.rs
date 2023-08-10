use std::{
    ops::{Mul, Sub},
    simd::f32x4,
};

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy)]
pub struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

impl RGB {
    #[inline(always)]
    fn from_normalized_floats((r, g, b): (f32, f32, f32)) -> Self {
        let [r, g, b, _] = f32x4::from_array([r, g, b, 0.0])
            .mul(f32x4::splat(255.0))
            .cast::<u8>()
            .to_array();

        Self { r, g, b }
    }

    pub fn from_hsv(h: f32, s: f32, v: f32) -> Self {
        let rgb: (f32, f32, f32) = if s == 0.0 {
            (v, v, v)
        } else {
            let mut h = h * 6.0;
            if h == 6.0 {
                h = 0.0;
            }
            let i = h as i32;
            let f = h - i as f32;
            let [p, q, t, _] = *f32x4::splat(v)
                .sub(f32x4::splat(s).mul(f32x4::from_array([1.0, f, (1.0 - f), 0.0])))
                .as_array();

            match i {
                0 => (v, t, p),
                1 => (q, v, p),
                2 => (p, v, t),
                3 => (p, q, v),
                4 => (t, p, v),
                5 => (v, p, q),
                _ => (0., 0., 0.),
            }
        };

        Self::from_normalized_floats(rgb)
    }

    #[inline(always)]
    pub fn to_u32(self) -> u32 {
        u32::from_be_bytes([0, self.r, self.g, self.b])
    }
}
