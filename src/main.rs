use minifb::{Key, Window, WindowOptions};
use std::{
    fmt,
    ops::{Add, Mul},
    thread,
};

const WIDTH: usize = 1000;
const HEIGHT: usize = 1000;
const ITER_MAX: i32 = 600;

#[derive(Clone, Copy)]
struct Complex {
    r: f64,
    i: f64,
}

impl Complex {
    const OFFSET: Complex = Complex {
        r: -1.78105004,
        i: 0.0,
    };

    fn magnitude(&self) -> f64 {
        self.r.hypot(self.i)
    }
}

impl Add for Complex {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        let real: f64 = self.r + other.r;
        let imaginary: f64 = self.i + other.i;

        Self {
            r: real,
            i: imaginary,
        }
    }
}

impl Mul for Complex {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        let real: f64 = (self.r * other.r) - (self.i * other.i);
        let imaginary: f64 = (self.r * other.i) + (self.i * other.r);

        Self {
            r: real,
            i: imaginary,
        }
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

fn index_to_complex(i: usize, scale: f64) -> Complex {
    let x: isize = (i % WIDTH) as isize - (WIDTH / 2) as isize;
    let y: isize = (i / HEIGHT) as isize - (HEIGHT / 2) as isize;

    let r = x as f64 * scale;
    let i = y as f64 * scale;
    Complex { i, r } + Complex::OFFSET
}

struct RGB {
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
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> RGB {
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

    RGB::from_normalized_floats(rgb)
}

fn main() {
    let threads: usize = thread::available_parallelism().unwrap().into();
    let mut scale: f64 = 4.0 / 450.0;
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new("Mandelbrot", WIDTH, HEIGHT, WindowOptions::default())
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

    window.limit_update_rate(Some(std::time::Duration::from_millis(33)));

    let max_pixel = WIDTH * HEIGHT - 1;

    let mut deltas: Vec<Vec<(usize, u32)>> = vec![Vec::new(); threads];

    while window.is_open() && !window.is_key_down(Key::Escape) {

        let start = std::time::Instant::now();

        rayon::scope(|s| {
            for (thread_id, delta) in deltas.iter_mut().enumerate() {
                s.spawn(move |_| {
                    let mut pixel = thread_id as usize;

                    delta.clear();

                    while pixel <= max_pixel {
                        let mut z = Complex { r: 0.0, i: 0.0 };
                        let c = index_to_complex(pixel, scale);
                        let mut iter: i32 = 0;

                        while z.magnitude() <= 2.0 && iter < ITER_MAX {
                            iter += 1;
                            z = (z * z) + c;
                        }

                        if z.magnitude() < 2.0 {
                            delta.push((pixel, 0));
                        } else {
                            let color: RGB = hsv_to_rgb(((iter as f32) / 70.0) % 1.0, 0.5, 1.0);

                            delta.push((pixel, u32::from_be_bytes([0, color.r, color.g, color.b])));
                        }
                        pixel += threads;
                    }
                });
            }
        });

        for handle in &deltas {
            for &(index, color) in handle {
                buffer[index] = color;
            }
        }

        scale = scale * 0.95;

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();

        println!("{:?}", start.elapsed());
    }
}
