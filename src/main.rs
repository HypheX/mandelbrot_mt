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
fn index_to_complex(i: usize, scale: f64, dim: WindowDimensions, offset: Complex) -> Complex {
    let [r, i] = usizex2::from_array([i % dim.width, i / dim.height])
        .cast::<isize>()
        .sub(isizex2::splat(isize::try_from(dim.height / 2).unwrap()))
        .cast::<f64>()
        .mul(f64x2::splat(scale))
        .to_array();

    Complex { r, i } + offset
}

#[inline]
fn generate_buffer(
    threads: usize,
    scale: f64,
    buffer: &mut [u32],
    dim: WindowDimensions,
    offset: Complex,
) {
    let buf = UncheckedSyncArray::from_slice(buffer);
    let out_buf = &buf;

    rayon::scope(|s| {
        for thread_id in 0..threads {
            s.spawn(move |_| {
                let mut pixel = thread_id;

                while pixel < out_buf.len() {
                    let mut z = Complex::default();
                    let c = index_to_complex(pixel, scale, dim, offset);

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

#[allow(dead_code)] // not currently called
fn generate_frame(config: &Config, frame: u64, buffer: &mut [u32]) {
    let mut scale = config.starting_scale;

    #[allow(clippy::cast_precision_loss)] // this is fine, we hit fp error way before frames cap out
    {
        scale *= config.scaling_factor.powf(frame as f64);
    }

    generate_buffer(config.threads, scale, buffer, config.dims, config.offset);
}

fn insert_frame_counter(frame: u64, buf: &mut [u32], dim: WindowDimensions) {
    let digits = pixel::Digit::from_u64(frame);

    let mut offset = 1;
    // cut off top
    let buf = &mut buf[dim.width..];

    for d in digits {
        d.render_6_10(buf, dim, offset);

        offset += 8;
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let conf = Config::generate()?;

    let mut buffer: Vec<u32> = vec![0; conf.dims.flat_length()];

    let mut window = Window::new(
        "Mandelbrot",
        conf.dims.width,
        conf.dims.height,
        WindowOptions::default(),
    )?;

    window.limit_update_rate(Some(std::time::Duration::from_millis(33)));

    let mut frame = 0;
    let mut scale = conf.starting_scale;

    let start = std::time::Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        generate_buffer(conf.threads, scale, &mut buffer, conf.dims, conf.offset);
        insert_frame_counter(frame, &mut buffer, conf.dims);

        scale *= conf.scaling_factor;
        frame += 1;

        window.update_with_buffer(&buffer, conf.dims.width, conf.dims.height)?;

        if frame % 100 == 0 {
            println!("{frame} frames in {:?}", start.elapsed());
        }
    }

    Ok(())
}

struct Config {
    dims: WindowDimensions,
    starting_scale: f64,
    scaling_factor: f64,
    offset: Complex,
    threads: usize,
}

impl Config {
    fn generate() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            threads: thread::available_parallelism()?.into(),
            dims: WindowDimensions::default(),
            starting_scale: 4.0 / 450.0,
            scaling_factor: 0.95,
            offset: Complex {
                r: -1.781_050_04,
                i: 0.0,
            },
        })
    }
}

#[derive(Copy, Clone)]
pub struct WindowDimensions {
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
