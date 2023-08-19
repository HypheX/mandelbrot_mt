#[derive(Clone, Copy)]
pub struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

impl Rgb {
    fn from_normalized_floats((r, g, b): (f32, f32, f32)) -> Self {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let [r, g, b] = [r, g, b].map(|x| (x * 255.) as u8);

        Self { r, g, b }
    }

    #[allow(clippy::many_single_char_names)]
    pub fn from_hsv(h: f32, s: f32, v: f32) -> Self {
        let rgb: (f32, f32, f32) = if s == 0.0 {
            (v, v, v)
        } else {
            let mut h = h * 6.0;

            // this code is witchcraft just let it be
            #[allow(clippy::float_cmp)]
            if h == 6.0 {
                h = 0.0;
            }

            #[allow(clippy::cast_possible_truncation)]
            let i = h as i32;
            #[allow(clippy::cast_precision_loss)]
            let f = h - i as f32;

            let p = v - (s * 1.);
            let q = v - (s * f);
            let t = v - (s * (1. - f));

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

    pub fn to_u32(self) -> u32 {
        u32::from_be_bytes([0, self.r, self.g, self.b])
    }
}

pub struct Digit(u8);

#[rustfmt::skip]
const ART: [[[u8; 3]; 5]; 10] = [
    [
        [0, 1, 0],
        [1, 0, 1],
        [1, 0, 1],
        [1, 0, 1],
        [0, 1, 0],
    ],
    [
        [0, 1, 0],
        [1, 1, 0],
        [0, 1, 0],
        [0, 1, 0],
        [1, 1, 1],
    ],
    [
        [0, 1, 0],
        [1, 0, 1],
        [0, 0, 1],
        [0, 1, 0],
        [1, 1, 1],
    ],
    [
        [1, 1, 0],
        [0, 0, 1],
        [0, 1, 0],
        [0, 0, 1],
        [1, 1, 0],
    ],
    [
        [1, 0, 1],
        [1, 0, 1],
        [0, 1, 1],
        [0, 0, 1],
        [0, 0, 1],
    ],
    [
        [1, 1, 1],
        [1, 0, 0],
        [1, 1, 0],
        [0, 0, 1],
        [1, 1, 1],
    ],
    [
        [0, 1, 1],
        [1, 0, 0],
        [1, 1, 0],
        [1, 0, 1],
        [0, 1, 0],
    ],
    [
        [1, 1, 1],
        [0, 0, 1],
        [0, 1, 0],
        [0, 1, 0],
        [0, 1, 0],
    ],
    [
        [0, 1, 0],
        [1, 0, 1],
        [0, 1, 0],
        [1, 0, 1],
        [0, 1, 0],
    ],
    [
        [0, 1, 0],
        [1, 0, 1],
        [0, 1, 1],
        [0, 0, 1],
        [0, 1, 0],
    ],
];

struct Grid<'a, T>(&'a mut [T], usize);

impl<T> Grid<'_, T> {
    fn subslice_range(&self, idx: usize) -> core::ops::Range<usize> {
        let base_idx = idx * self.1;

        base_idx..(base_idx + self.1)
    }
}

impl<T> core::ops::Index<usize> for Grid<'_, T> {
    type Output = [T];

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[self.subslice_range(index)]
    }
}

impl<T> core::ops::IndexMut<usize> for Grid<'_, T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let range = self.subslice_range(index);
        &mut self.0[range]
    }
}

impl Digit {
    pub fn from_u64(v: u64) -> Vec<Digit> {
        let mut digits = Vec::new();

        for c in v.to_string().as_bytes() {
            digits.push(Digit(c - b'0'));
        }

        digits
    }

    pub fn render_6_10(&self, buf: &mut [u32], dim: super::WindowDimensions, horiz_offset: usize) {
        let mut buf = Grid(buf, dim.width);

        let pixels = ART[self.0 as usize];

        for (y, inner) in pixels.into_iter().enumerate() {
            for (x, pix) in inner.into_iter().enumerate() {
                if pix == 1 {
                    let offsets: [(usize, usize); 4] = [(0, 0), (0, 1), (1, 0), (1, 1)];

                    for (yoff, xoff) in offsets {
                        let pix: &mut u32 = &mut buf[(y * 2) + yoff][(x * 2) + horiz_offset + xoff];

                        // see inversion_is_inverted for proof this works
                        *pix = !*pix;
                    }
                }
            }
        }
    }
}

#[test]
fn inversion_is_inverted() {
    for b in 0..=u8::MAX {
        assert_eq!(!b, 255 - b);
    }
}
