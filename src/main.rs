#![feature(portable_simd)]
#![warn(clippy::pedantic)]

use minifb::{Key, Window, WindowOptions};
use std::{
    error::Error,
    ops::{Mul, Sub},
    simd::{f64x2, isizex2, usizex2, SimdInt, SimdUint},
    thread,
};

mod complex;
mod pixel;
mod unchecked_array;

use complex::Complex;
use pixel::Rgb;
use unchecked_array::UncheckedSyncArray;

const ITER_MAX: u16 = 600;

#[inline]
fn index_to_complex(i: usize, scale: f64, dim: WindowDimensions) -> Complex {
    const OFFSET: Complex = Complex {
        r: -1.781_050_04,
        i: 0.0,
    };

    let [r, i] = usizex2::from_array([i % dim.width, i / dim.height])
        .cast::<isize>()
        .sub(isizex2::splat(isize::try_from(dim.height / 2).unwrap()))
        .cast::<f64>()
        .mul(f64x2::splat(scale))
        .to_array();

    Complex { r, i } + OFFSET
}

#[inline]
fn generate_buffer(threads: usize, scale: f64, buffer: &mut [u32], dim: WindowDimensions) {
    let max_pixel = dim.width * dim.height - 1;

    let buf = UncheckedSyncArray::from_slice(buffer);
    let out_buf = &buf;

    rayon::scope(|s| {
        for thread_id in 0..threads {
            s.spawn(move |_| {
                let mut pixel = thread_id;

                while pixel <= max_pixel {
                    let mut z = Complex::default();
                    let c = index_to_complex(pixel, scale, dim);

                    let mut iter: u16 = 0;

                    for i in 0..ITER_MAX {
                        if z.magnitude() > 2. {
                            iter = i;
                            break;
                        }

                        z = (z * z) + c;
                    }

                    let hsv = u32::from(z.magnitude() >= 2.0)
                        * Rgb::from_hsv((f32::from(iter) / 70.0) % 1.0, 0.5, 1.0).to_u32();

                    // SAFETY: the pixels given to each thread are unique, and cannot overlap,
                    // pixels are also impossible to be OOB as there are less pixels than the
                    // allocated capacity of the array.
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

        scale *= 0.95;

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
    #[inline]
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
