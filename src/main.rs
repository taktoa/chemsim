#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![feature(duration_as_u128)]

extern crate chemsim;
extern crate piston;
extern crate arrayfire;
extern crate conrod;
extern crate gif;

use chemsim::display::{Drawable, RGB, PixelPos};
use chemsim::lbm::{Scalar, Matrix};
use arrayfire as af;
use arrayfire::HasAfEnum;

pub fn draw_matrix<T: Copy + HasAfEnum, D: Drawable>(
    buffer: &mut D,
    matrix: &chemsim::matrix::Matrix<T>,
    shader: &(Fn(T) -> i8),
) {
    let (w, h) = matrix.get_shape();
    let copied = matrix.get_underlying();
    for x in 0 .. w {
        for y in 0 .. h {
            let n = shader(copied[(y * w) + x]);
            let k = 2 * (n.abs().min(127) as u8);
            let value = {
                if n < 0 {
                    RGB(k, 0, 0)
                } else if n > 0 {
                    RGB(0, k, 0)
                } else {
                    RGB(0, 0, 0)
                }
            };
            buffer.set_pixel(PixelPos(x as u32, y as u32), value);
        }
    }
}

pub struct LBMSim {
    size:  (usize, usize),
    state: chemsim::lbm::State,
}

impl chemsim::display::Simulation for LBMSim {
    fn size(&self) -> (usize, usize) { self.size }

    fn handle(&mut self, input: &piston::input::Event) {
        // FIXME: drawing boundaries etc.
    }

    fn step(&mut self, elapsed: &std::time::Duration) {
        let t = std::time::Instant::now();
        self.state.step();
        println!("Step {} took {} ms",
                 self.state.time,
                 t.elapsed().as_millis());
    }

    fn render<D: chemsim::display::Drawable>(&self, buf: &mut D) {
        if self.state.is_unstable() {
            println!("[ERROR] Instability detected!");
        }
        println!("Max speed: {}", self.state.speed().maximum_real());
        // for (i, (_, pop)) in self.state.populations().iter().enumerate() {
        //     let fft = pop.dft(1.0).abs();
        //     let nonzeros = af::count_all(fft.get_array()).0 as usize;
        //     let total    = fft.get_width() * fft.get_height();
        //     assert!(total > nonzeros);
        //     let numerator   = total - nonzeros;
        //     let denominator = total;
        //     let ratio       = (100.0 * numerator as f64) / (denominator as f64);
        //     println!("> > FFT of population {} has {} / {} = {}% zeroes",
        //              i, numerator, denominator, ratio);
        // }
        chemsim::render::render_vector_field(&self.state.momentum_density(), buf);
        chemsim::render::render_geometry(&self.state.geometry, buf);
    }
}

fn initial_state(size: (usize, usize)) -> LBMSim {
    use chemsim::*;

    let (w, h) = size;

    let collision = lbm::BGK { tau: 5.0 };

    let disc = lbm::Discretization { delta_x: 1.0, delta_t: 1.0 };

    let initial_velocity = {
        let mut vec_x = Vec::new();
        let mut vec_y = Vec::new();
        vec_x.resize(w * h, 0.0);
        vec_y.resize(w * h, 0.0);
        for x in 0 .. w {
            for y in 0 .. h {
                let scale = 0.01;
                // vec_x[(y * w) + x] = -(y as Scalar) * scale / (h as Scalar);
                // vec_y[(y * w) + x] =  (x as Scalar) * scale / (w as Scalar);
                vec_x[(y * w) + x] = scale;
                vec_y[(y * w) + x] = 0.0;
            }
        }
        let vx = matrix::Matrix::new(&vec_x, size).unwrap();
        let vy = matrix::Matrix::new(&vec_y, size).unwrap();
        (vx, vy)
    };

    let initial_density = {
        // FIXME: proper initialization
        // matrix::Matrix::new_filled(0.0, size)
        matrix::Matrix::new_filled(1.0, size)
        // matrix::Matrix::new_random(size).abs().scale(10.0)
        // matrix::Matrix::new_identity(size)

        // let sine = {
        //     let mut vec = Vec::new();
        //     vec.resize(w * h, 0.0);
        //     for x in 0 .. w {
        //         for y in 0 .. h {
        //             let mut val = 0.0;
        //             val += Scalar::sin(3.14159 * (x as Scalar) / (w as Scalar));
        //             val += Scalar::sin(3.14159 * (y as Scalar) / (h as Scalar));
        //             vec[(y * w) + x] = 0.001 * val;
        //         }
        //     }
        //     matrix::Matrix::new(&vec, size).unwrap()
        // };
        // matrix::Matrix::new_filled(1.0, size)
        //     + matrix::Matrix::new_random(size).hadamard(&sine)

        // let mut vec = Vec::new();
        // vec.resize(w * h, 0.0);
        // for x in 0 .. w {
        //     for y in 0 .. h {
        //         let mut val = 0.0;
        //         val += 1.0;
        //         val += 0.3 * Scalar::sin(3.0 * (x as Scalar) / (w as Scalar));
        //         val += 0.3 * Scalar::sin(3.0 * (y as Scalar) / (h as Scalar));
        //         vec[(y * w) + x] = val;
        //     }
        // }
        // matrix::Matrix::new(&vec, size).unwrap()
    };

    let pops = &({
        let temp = lbm::compute_equilibrium(
            initial_density,
            initial_velocity,
            &lbm::D2Q9::directions(),
            disc,
        );
        temp.iter().map(|(_, pop)| pop.clone()).collect::<Vec<lbm::Population>>()
    });

    let lattice = lbm::D2Q9::new(pops);

    let geometry = {
        let mut vec = Vec::new();
        vec.resize(w * h, false);

        {
            let mut set = |x: usize, y: usize, val: bool| { vec[y * w + x] = val; };

            for x in 0 .. w {
                for y in 0 .. h {
                    // set(x, y,
                    //     false
                    //     || (x ==     0) || (y ==     0)
                    //     || (x == w - 1) || (y == h - 1));
                    // set(x, y, (y == 0) || (y == h - 1));
                    let mut r = 0.0;
                    r += (x as f64 - (w as f64 / 2.0)).powi(2);
                    r += (y as f64 - (h as f64 / 2.0)).powi(2);
                    r = r.sqrt();
                    if r < 25.0 {
                        set(x, y, true);
                    }
                }
            }
            // for x in (128 - 51) .. (128 + 51) {
            //     for y in (128 - 50) .. (128 + 50) { set(x, y, true); }
            // }
            // for x in (128 - 49) .. (128 + 49) {
            //     for y in (128 - 49) .. (128 + 49) { set(x, y, false); }
            // }
            //
            // set(128, 128 - 50, false);
            // set(128, 128 - 51, false);
        }

        let vec = vec;

        matrix::Matrix::new(&vec, size).unwrap()
    };

    let state = lbm::State::initial(
        Box::new(lattice),
        geometry,
        Box::new(collision),
        disc,
    );

    LBMSim { size: size, state: state }
}


fn main() {
    af::init();
    println!("[NOTE] ArrayFire successfully initialized!");

    let gif = true;
    let (w, h) = (768, 384);

    // -------------------------------------------------------------------------

    let initial = initial_state((w, h));

    if gif {
        use std::fs::File;
        use gif::SetParameter;
        let color_map = &[0xFF, 0xFF, 0xFF, 0, 0, 0];
        let mut image = File::create("output.gif").unwrap();
        let mut encoder
            = gif::Encoder::new(&mut image, w as u16, h as u16, color_map).unwrap();
        encoder.set(gif::Repeat::Infinite).unwrap();
        chemsim::display::record(initial, 400, &mut encoder).unwrap();
    } else {
        chemsim::display::example(initial);
    }
}
