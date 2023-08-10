use std::ops::{Add, Mul};
use std::fmt;
use minifb::{Key, Window, WindowOptions};
use std::thread;
use thread::JoinHandle;
use std::time::Duration;

const WIDTH: usize = 1000;
const HEIGHT: usize = 1000;
const ITER_MAX: i32 = 600;

#[derive(Clone, Copy)]
struct Complex {
    r: f64,
    i: f64,
}

impl Complex {

    const OFFSET: Complex =  Complex{r: -1.78105004, i: 0.0};

    fn magnitude(&self) -> f64 {
        self.r.hypot(self.i)
    }
}

impl Add for Complex {  
    type Output = Self;
    fn add(self, other: Self) -> Self {

        let real: f64 = self.r + other.r;
        let imaginary: f64 = self.i + other.i;

        Self{r: real, i: imaginary}
    }
}

impl Mul for Complex {  
    type Output = Self;
    fn mul(self, other: Self) -> Self {

        let real: f64 = (self.r * other.r)-(self.i * other.i);
        let imaginary: f64 = (self.r * other.i)+(self.i * other.r);

        Self{r: real, i: imaginary}
    }
}

impl fmt::Display for Complex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.i >= 0.0 {
            write!(f, "{0}+{1}i", self.r, self.i)
        } else {
            write!(f, "{0}{1}i", self.r, self.i)
        }
    }
}

fn index_to_complex (i: usize, scale: f64) -> Complex {
    let x: isize = (i%WIDTH)as isize - (WIDTH/2) as isize;
    let y: isize = (i/HEIGHT) as isize - (HEIGHT/2) as isize;
    
    let r = x as f64 * scale;
    let i = y as f64 * scale;
    Complex{i, r} + Complex::OFFSET
}

struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

fn hsv_to_rgb(h : f32, s : f32, v : f32) -> RGB
{
    let r : f32;
    let g : f32;
    let b : f32;

    if s == 0.0
    {
        r = v;
        g = v;
        b = v;
    }
    else
    {
        let mut h = h * 6.0;
        if h == 6.0
        {
            h = 0.0;
        }
        let i = h as i32;
        let f = h - i as f32;
        let p = v * (1.0 - s);
        let q = v * (1.0 - s * f);
        let t = v * (1.0 - s * (1.0 - f));

        match i
        {
            0 => { r = v; g = t; b = p; },
            1 => { r = q; g = v; b = p; },
            2 => { r = p; g = v; b = t; },
            3 => { r = p; g = q; b = v; },
            4 => { r = t; g = p; b = v; },
            5 => { r = v; g = p; b = q; },
            _ => { r = 0.0; g = 0.0; b = 0.0; },
        }
    }

    RGB{r: (r*255.0).round() as u8, g: (g*255.0).round() as u8, b: (b*255.0).round() as u8}
}

fn main() {

    let threads = thread::available_parallelism().unwrap();
    let mut scale: f64 = 4.0/450.0;
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Mandelbrot",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.limit_update_rate(Some(std::time::Duration::from_millis(33)));

    let max_pixel = WIDTH*HEIGHT-1;

    thread::sleep(Duration::from_millis(2000));

    while window.is_open() && !window.is_key_down(Key::Escape) {

        let mut thread_handles: Vec<JoinHandle<Vec<(usize, u32)>>> = Vec::new();

        for thread_id in 0..threads {
            let handle = thread::spawn(move|| {
                let mut pixel = thread_id as usize;
                let mut delta: Vec<(usize, u32)> = Vec::new();
                loop {
                    if pixel > max_pixel {
                        break;
                    }
                    let mut z = Complex{r: 0.0, i: 0.0};
                    let c = index_to_complex(pixel, scale);
                    let mut iter: i32 = 0;
                    while z.magnitude() <= 2.0 && iter < ITER_MAX {
                        iter += 1;
                        z = (z*z) + c;
                    }
                    if z.magnitude() < 2.0 {
                        delta.push((pixel, 0));
                    } else {
                        let color: RGB = hsv_to_rgb(((iter as f32)/70.0)%1.0, 0.5, 1.0);
                        delta.push((pixel, u32::from_be_bytes([0,color.r,color.g,color.b])));
                        //delta.push((pixel, u32::from_be_bytes([0,255,255,255])));
                    }
                    pixel += threads as usize;
                }
                delta
            });
            thread_handles.push(handle);
        }

        for handle in thread_handles {
            let result = handle.join().expect("Error!");
            for (index, color) in result {
                buffer[index] = color;
            }
        }
        
        scale = scale  * 0.95;
/* 
        SCALE = SCALE * 0.95;
        for (index, pixel) in buffer.iter_mut().enumerate() {
            let mut z = Complex{r: 0.0, i: 0.0};
                let c = index_to_complex(index, SCALE);
                let mut iter: i32 = 0;
            while z.magnitude() <= 2.0 && iter < ITER_MAX {
                iter += 1;
                z = (z*z) + c;
            }
            if z.magnitude() < 2.0 {
                *pixel = 0;
            } else {
                let color = ((iter as f32/ITER_MAX as f32 + 1.0).log(1.3) * 255.0).round() as u8;
                *pixel = u32::from_be_bytes([0,color,color,color]);
            }
        }
*/
        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }

}