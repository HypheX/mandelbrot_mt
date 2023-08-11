#![warn(clippy::pedantic)]

pub mod complex;

mod pixel;

use complex::Complex;
use pixel::Rgb;

const ITER_MAX: u16 = 600;

#[inline]
#[allow(clippy::missing_panics_doc, clippy::cast_precision_loss)]
#[must_use]
pub fn index_to_complex(i: usize, scale: f64, dim: WindowDimensions, offset: Complex) -> Complex {
    let hd2 = isize::try_from(dim.height / 2).unwrap();

    let r = (isize::try_from(i % dim.width).unwrap() - hd2) as f64 * scale;
    let i = (isize::try_from(i / dim.height).unwrap() - hd2) as f64 * scale;

    Complex { r, i } + offset
}

#[inline]
pub fn generate_buffer(scale: f64, buffer: &mut [u32], dim: WindowDimensions, offset: Complex) {
    rayon::scope(|s| {
        for (h_axis, pixels) in buffer.chunks_mut(dim.width).enumerate() {
            s.spawn(move |_| {
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
            });
        }
    });
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
    #[must_use]
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
    #[must_use]
    pub fn flat_length(&self) -> usize {
        self.width * self.height
    }
}
