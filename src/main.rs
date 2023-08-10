use minifb::{Key, Window, WindowOptions};
use std::{error::Error, thread};

mod complex;
mod pixel;
mod unchecked_array;

use complex::Complex;
use pixel::RGB;
use unchecked_array::UncheckedSyncArray;

const ITER_MAX: i32 = 600;

fn index_to_complex(i: usize, scale: f64, dim: WindowDimensions) -> Complex {
    const OFFSET: Complex = Complex {
        r: -1.78105004,
        i: 0.0,
    };

    let x: isize = (i % dim.width) as isize - (dim.width / 2) as isize;
    let y: isize = (i / dim.height) as isize - (dim.height / 2) as isize;

    let r = x as f64 * scale;
    let i = y as f64 * scale;
    Complex { i, r } + OFFSET
}

fn generate_buffer(threads: usize, scale: f64, buffer: &mut [u32], dim: WindowDimensions) {
    let max_pixel = dim.width * dim.height - 1;

    let buf = UncheckedSyncArray::from_slice(buffer);
    let out_buf = &buf;

    rayon::scope(|s| {
        for thread_id in 0..threads {
            s.spawn(move |_| {
                let mut pixel = thread_id as usize;

                while pixel <= max_pixel {
                    let mut z = Complex { r: 0.0, i: 0.0 };
                    let c = index_to_complex(pixel, scale, dim);
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

fn main() -> Result<(), Box<dyn Error>> {
    let threads: usize = thread::available_parallelism()?.into();
    let mut scale: f64 = 4.0 / 450.0;
    let dimensions = WindowDimensions::default();

    let mut buffer: Vec<u32> = vec![0; dimensions.flat_length()];

    let mut window = Window::new(
        "Mandelbrot",
        dimensions.width,
        dimensions.height,
        WindowOptions::default(),
    )?;

    window.limit_update_rate(Some(std::time::Duration::from_millis(33)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        generate_buffer(threads, scale, &mut buffer, dimensions);

        scale = scale * 0.95;

        window.update_with_buffer(&buffer, dimensions.width, dimensions.height)?;
    }

    Ok(())
}

#[derive(Copy, Clone)]
struct WindowDimensions {
    width: usize,
    height: usize,
}

impl Default for WindowDimensions {
    fn default() -> Self {
        const WIDTH: usize = 1000;
        const HEIGHT: usize = 1000;

        Self {
            width: WIDTH,
            height: HEIGHT,
        }
    }
}

impl WindowDimensions {
    fn flat_length(&self) -> usize {
        self.width * self.height
    }
}

#[test]
fn ensure_threadsafe() {
    let threads: usize = thread::available_parallelism().unwrap().into();
    let mut scale: f64 = 4.0 / 450.0;
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    generate_buffer(threads, scale, &mut buffer);
}
