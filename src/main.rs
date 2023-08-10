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

    fn from_hsv(h: f32, s: f32, v: f32) -> Self {
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

    fn to_u32(&self) -> u32 {
        u32::from_be_bytes([0, self.r, self.g, self.b])
    }
}

struct UncheckedSyncArray<'a, T>(*mut T, usize, core::marker::PhantomData<&'a mut T>);

unsafe impl<'a, T: Send + Sync> Sync for UncheckedSyncArray<'a, T> {}

impl<'a, T> UncheckedSyncArray<'a, T> {
    fn from_slice(v: &'a mut [T]) -> Self {
        UncheckedSyncArray(v.as_mut_ptr(), v.len(), core::marker::PhantomData)
    }

    /// # Safety:
    /// As this has no mechanism to ensure more than 1 thread accesses the same index at a time,
    /// if more than 1 thread accesses the same index at a time UB will occur.
    /// However, this does check for out of bounds accesses
    unsafe fn store_unchecked(&self, idx: usize, item: T) {
        if idx >= self.1 {
            panic!("index out of bounds")
        }

        // SAFETY: no other threads are accessing this index, so we can safely write to it
        // we drop the T given to us by replace, this lets us hack dropping the old T
        unsafe { core::ptr::replace(self.0.add(idx), item) };
    }
}

fn generate_buffer(threads: usize, scale: f64, buffer: &mut [u32]) {
    let max_pixel = WIDTH * HEIGHT - 1;

    let buf = UncheckedSyncArray::from_slice(buffer);
    let out_buf = &buf;

    rayon::scope(|s| {
        for thread_id in 0..threads {
            s.spawn(move |_| {
                let mut pixel = thread_id as usize;

                while pixel <= max_pixel {
                    let mut z = Complex { r: 0.0, i: 0.0 };
                    let c = index_to_complex(pixel, scale);
                    let mut iter: i32 = 0;

                    while z.magnitude() <= 2.0 && iter < ITER_MAX {
                        iter += 1;
                        z = (z * z) + c;
                    }

                    let hsv = if z.magnitude() < 2.0 {
                        0
                    } else {
                        let color = RGB::from_hsv(((iter as f32) / 70.0) % 1.0, 0.5, 1.0);

                        color.to_u32()
                    };

                    // SAFETY: the pixels given to each thread are unique, and cannot overlap
                    unsafe { out_buf.store_unchecked(pixel, hsv) };

                    pixel += threads;
                }
            });
        }
    });
}

#[test]
fn ensure_threadsafe() {
    let threads: usize = thread::available_parallelism().unwrap().into();
    let mut scale: f64 = 4.0 / 450.0;
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    generate_buffer(threads, scale, &mut buffer);
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

    while window.is_open() && !window.is_key_down(Key::Escape) {
        generate_buffer(threads, scale, &mut buffer);

        scale = scale * 0.95;

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
