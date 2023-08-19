#![warn(clippy::pedantic)]

use crossbeam_channel::{unbounded, Receiver, Sender, TryRecvError};

use mandelbrot_mt_f64::{generate_buffer, insert_frame_counter, Config};
use minifb::{Key, Window, WindowOptions};
use std::error::Error;

struct TwoWayPressureValve<T, C: FnMut() -> T> {
    spawned: usize,
    spawn_limit: usize,
    backpressure_limit: usize,
    recv_ch: Receiver<T>,
    creator: C,
}

impl<T, C: FnMut() -> T> TwoWayPressureValve<T, C> {
    fn new(recv_ch: Receiver<T>, creator: C) -> Self {
        Self {
            spawned: 0,
            spawn_limit: 100,
            backpressure_limit: 2,
            recv_ch,
            creator,
        }
    }
}

impl<T, C: FnMut() -> T> Iterator for TwoWayPressureValve<T, C> {
    type Item = T;

    /// dynamically allocates or frees vecs in the pipe depending on pressure
    fn next(&mut self) -> Option<T> {
        match self.recv_ch.try_recv() {
            Ok(buf) => Some(if self.recv_ch.len() > self.backpressure_limit {
                drop(buf);
                self.spawned -= 1;
                let Ok(buf) = self.recv_ch.recv() else {
                    return None;
                };
                buf
            } else {
                buf
            }),
            Err(TryRecvError::Disconnected) => None,
            Err(TryRecvError::Empty) => Some(if self.spawned < self.spawn_limit {
                self.spawned += 1;
                (self.creator)()
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
    let (return_send_ch, recv_ch) = unbounded();
    let (send_ch, return_recv_ch) = unbounded();

    std::thread::spawn(move || {
        let mut scale = conf.starting_scale;

        let iter = TwoWayPressureValve::new(recv_ch, || vec![0; conf.dims.flat_length()]);

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
