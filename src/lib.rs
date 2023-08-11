#![feature(portable_simd)]
#![warn(clippy::pedantic)]

pub mod complex;

use std::{
    ops::{Mul, Sub},
    simd::{f64x2, isizex2, usizex2, SimdInt, SimdUint},
};

mod pixel;

use complex::Complex;
use pixel::Rgb;

const ITER_MAX: u16 = 600;

#[inline]
pub fn index_to_complex(i: usize, scale: f64, dim: WindowDimensions, offset: Complex) -> Complex {
    let [r, i] = usizex2::from_array([i % dim.width, i / dim.height])
        .cast::<isize>()
        .sub(isizex2::splat(isize::try_from(dim.height / 2).unwrap()))
        .cast::<f64>()
        .mul(f64x2::splat(scale))
        .to_array();

    Complex { r, i } + offset
}

#[inline]
pub fn generate_buffer(scale: f64, buffer: &mut [u32], dim: WindowDimensions, offset: Complex) {
    for (h_axis, pixels) in buffer.chunks_mut(dim.width).enumerate() {
        for (idx, pix) in pixels.iter_mut().enumerate() {
            let mut z = Complex::default();
            let c = index_to_complex(idx + (h_axis * dim.width), scale, dim, offset);

            let mut iter = None;

            for i in 0..ITER_MAX {
                if z.mandelbrot_escaped() {
                    iter = Some(i);
                    break;
                }

                z.mandelbrot_iter(c);
            }

            let rgb = iter.map_or(0, |rgb| {
                Rgb::from_hsv((f32::from(rgb) / 70.0) % 1.0, 0.5, 1.0).to_u32()
            });

            *pix = rgb;
        }
    }
}

#[allow(dead_code)] // not currently called
pub fn generate_frame(config: &Config, frame: u64, buffer: &mut [u32]) {
    let mut scale = config.starting_scale;

    #[allow(clippy::cast_precision_loss)] // this is fine, we hit fp error way before frames cap out
    {
        scale *= config.scaling_factor.powf(frame as f64);
    }

    generate_buffer(scale, buffer, config.dims, config.offset);
}

pub fn insert_frame_counter(frame: u64, buf: &mut [u32], dim: WindowDimensions) {
    let digits = pixel::Digit::from_u64(frame);

    let mut offset = 1;
    // cut off top
    let buf = &mut buf[dim.width..];

    for d in digits {
        d.render_6_10(buf, dim, offset);

        offset += 8;
    }
}

#[derive(Copy, Clone)]
pub struct Config {
    pub dims: WindowDimensions,
    pub starting_scale: f64,
    pub scaling_factor: f64,
    pub offset: Complex,
}

impl Config {
    pub fn generate() -> Self {
        Self {
            dims: WindowDimensions::default(),
            starting_scale: 4.0 / 450.0,
            scaling_factor: 0.95,
            offset: Complex {
                r: -1.781_050_04,
                i: 0.0,
            },
        }
    }
}

#[derive(Copy, Clone)]
pub struct WindowDimensions {
    pub width: usize,
    pub height: usize,
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
    pub fn flat_length(&self) -> usize {
        self.width * self.height
    }
}
