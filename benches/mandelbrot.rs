
use criterion::{criterion_group, criterion_main, Criterion, black_box};
use rand::Rng;

use mandelbrot_mt_f64::complex::Complex;
use mandelbrot_mt_f64::*;

fn random_complex() -> Complex {
    let mut rand = rand::thread_rng();
    Complex {
        r: rand.gen_range(-2.0..=2.0),
        i: rand.gen_range(-2.0..=2.0),
    }
}

fn all(c: &mut Criterion) {
    c.bench_function("mandelbrot_iter", |b| {
        let mut z = random_complex();
        let c = random_complex();
        b.iter(|| {
            black_box(z.mandelbrot_iter(c))
        })
    });

    c.bench_function("index_to_complex", |b| {
        let config = Config::generate();
        let mut r = rand::thread_rng();
        let i = r.gen_range(0..config.dims.flat_length());
        let scale = config.starting_scale + r.gen_range(0.0..1.0);
        let dim = config.dims;
        let offset = config.offset;
        b.iter(|| {
            black_box(index_to_complex(i, scale, dim, offset))
        })
    });

    c.bench_function("generate_frame", |b| {
        let config = Config::generate();
        let mut buffer = vec![0; config.dims.flat_length()];
        let frame = rand::thread_rng().gen_range(0..100);
        b.iter(|| {
            black_box(generate_frame(&config, frame, &mut buffer))
        })
    });

    c.bench_function("insert_frame_counter", |b| {
        let config = Config::generate();
        let mut buffer = vec![0; config.dims.flat_length()];
        let dim = config.dims;
        let frame = rand::thread_rng().gen_range(0..1000);
        b.iter(|| {
            black_box(insert_frame_counter(frame, &mut buffer, dim))
        })
    });
}

criterion_group!(benches, all);
criterion_main!(benches);