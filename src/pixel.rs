pub struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

impl RGB {
    fn from_normalized_floats((r, g, b): (f32, f32, f32)) -> Self {
        Self {
            r: (r * 255.) as u8,
            g: (g * 255.) as u8,
            b: (b * 255.) as u8,
        }
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
            let p = v * (1.0 - s);
            let q = v * (1.0 - s * f);
            let t = v * (1.0 - s * (1.0 - f));

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

    pub fn to_u32(&self) -> u32 {
        u32::from_be_bytes([0, self.r, self.g, self.b])
    }
}
