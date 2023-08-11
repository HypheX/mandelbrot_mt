#![warn(clippy::pedantic)]
#![feature(portable_simd)]

use crossbeam_channel::{unbounded as channel, Receiver, Sender, TryRecvError};

use mandelbrot_mt_f64::{generate_buffer, insert_frame_counter, Config, WindowDimensions};
use minifb::{Key, Window, WindowOptions};
use std::error::Error;

struct TwoWayPressureValve {
    spawned: usize,
    pressure: i64,
    recv_ch: Receiver<Vec<u32>>,
    dims: WindowDimensions,
}

impl Iterator for TwoWayPressureValve {
    type Item = Vec<u32>;

    /// dynamically allocates or frees vecs in the pipe depending on pressure
    fn next(&mut self) -> Option<Vec<u32>> {
        match self.recv_ch.try_recv() {
            Ok(buf) => {
                self.pressure = -i64::try_from(self.recv_ch.len()).unwrap();

                Some(if self.pressure < -2 {
                    drop(buf);
                    self.spawned -= 1;
                    let Ok(buf) = self.recv_ch.recv() else {
                        return None;
                    };
                    buf
                } else {
                    buf
                })
            }
            Err(TryRecvError::Disconnected) => None,
            Err(TryRecvError::Empty) => Some(if self.spawned < 100 {
                self.spawned += 1;
                vec![0; self.dims.flat_length()]
            } else {
                let Ok(buf) = self.recv_ch.recv() else {
                    return None;
                };
                buf
            }),
        }
    }
}

fn mandelbrot_generator(conf: Config) -> (Sender<Vec<u32>>, Receiver<Vec<u32>>) {
    let (return_send_ch, recv_ch) = channel();
    let (send_ch, return_recv_ch) = channel();

    //for _ in 0..5 {
    //    return_send_ch
    //        .send(vec![0; conf.dims.flat_length()])
    //        .unwrap();
    //}

    std::thread::spawn(move || {
        let mut scale = conf.starting_scale;

        let iter = TwoWayPressureValve {
            spawned: 0,
            pressure: 0,
            recv_ch,
            dims: conf.dims,
        };

        for mut buf in iter {
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
