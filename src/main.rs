#![warn(clippy::pedantic)]
#![feature(portable_simd)]

use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};

use mandelbrot_mt_f64::{generate_buffer, insert_frame_counter, Config};
use minifb::{Key, Window, WindowOptions};
use std::error::Error;

fn mandelbrot_generator(conf: Config) -> (Sender<Vec<u32>>, Receiver<Vec<u32>>) {
    let (return_send_ch, recv_ch) = channel();
    let (send_ch, return_recv_ch) = channel();

    for _ in 0..5 {
        return_send_ch
            .send(vec![0; conf.dims.flat_length()])
            .unwrap();
    }

    std::thread::spawn(move || {
        let mut scale = conf.starting_scale;
        let mut spawned = 5;
        loop {
            let mut buf = match recv_ch.try_recv() {
                Ok(buf) => buf,
                Err(TryRecvError::Disconnected) => break,
                Err(TryRecvError::Empty) => {
                    if spawned < 100 {
                        spawned += 1;
                        vec![0; conf.dims.flat_length()]
                    } else {
                        let Ok(buf) = recv_ch.recv() else { break };
                        buf
                    }
                }
            };

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
