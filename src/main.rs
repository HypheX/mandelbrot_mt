#![feature(portable_simd)]
#![warn(clippy::pedantic)]

use minifb::{Key, Window, WindowOptions};
use std::{
    error::Error,
    ops::{Mul, Sub},
    simd::{f64x2, isizex2, usizex2, SimdInt, SimdUint},
};

mod complex;
mod pixel;

use complex::Complex;
use pixel::Rgb;

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
fn generate_buffer(scale: f64, buffer: &mut [u32], dim: WindowDimensions, offset: Complex) {
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
fn generate_frame(config: &Config, frame: u64, buffer: &mut [u32]) {
    let mut scale = config.starting_scale;

    #[allow(clippy::cast_precision_loss)] // this is fine, we hit fp error way before frames cap out
    {
        scale *= config.scaling_factor.powf(frame as f64);
    }

    generate_buffer(scale, buffer, config.dims, config.offset);
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

use std::sync::mpsc::{sync_channel, Receiver, SyncSender};

fn mandelbrot_generator(conf: Config) -> (SyncSender<Vec<u32>>, Receiver<Vec<u32>>) {
    let (return_send_ch, recv_ch) = sync_channel(20);
    let (send_ch, return_recv_ch) = sync_channel(20);

    for _ in 0..10 {
        return_send_ch
            .send(vec![0; conf.dims.flat_length()])
            .unwrap();
    }

    std::thread::spawn(move || {
        let mut scale = conf.starting_scale;
        for mut buf in recv_ch {

            generate_buffer(scale, &mut buf, conf.dims, conf.offset);

            scale *= conf.scaling_factor;

            send_ch.send(buf).unwrap();
        }
    });

    (return_send_ch, return_recv_ch)
}

fn main() -> Result<(), Box<dyn Error>> {
    let conf = Config::generate();

    let mut window = Window::new(
        "Mandelbrot",
        conf.dims.width,
        conf.dims.height,
        WindowOptions::default(),
    )?;

    window.limit_update_rate(Some(std::time::Duration::from_millis(33)));

    let mut frame = 0;

    let (send, recv) = mandelbrot_generator(conf);

    let start = std::time::Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mut buffer = recv.recv().unwrap();

        insert_frame_counter(frame, &mut buffer, conf.dims);

        frame += 1;

        window.update_with_buffer(&buffer, conf.dims.width, conf.dims.height)?;

        send.send(buffer).unwrap();

        if frame % 100 == 0 {
            println!("{frame} frames in {:?}", start.elapsed());
        }
    }

    Ok(())
}

#[derive(Copy, Clone)]
struct Config {
    dims: WindowDimensions,
    starting_scale: f64,
    scaling_factor: f64,
    offset: Complex,
}

impl Config {
    fn generate() -> Self {
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
