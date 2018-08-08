#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![feature(duration_as_u128)]

extern crate chemsim;
extern crate piston;
extern crate arrayfire;
extern crate conrod;

use chemsim::display::Drawable;
use chemsim::lbm::{Scalar, Matrix};
use arrayfire as af;
use arrayfire::HasAfEnum;

pub fn draw_matrix<T: Copy + HasAfEnum, D: Drawable>(
    buffer: &mut D,
    matrix: &chemsim::matrix::Matrix<T>,
    shader: &(Fn(T) -> u8),
) {
    use chemsim::lbm::Scalar;
    use chemsim::display::{RGB, PixelPos};
    let (w, h) = matrix.get_shape();
    let copied = matrix.get_underlying();
    for x in 0 .. w {
        for y in 0 .. h {
            let n = shader(copied[(y * w) + x]);
            let value = RGB(n, n, n);
            buffer.set_pixel(PixelPos(x as u32, y as u32), value);
        }
    }
}

pub struct LBMSim {
    size:  (usize, usize),
    state: chemsim::lbm::State,
}

impl LBMSim {
    pub fn draw(&self) -> chemsim::lbm::Matrix {
        if self.state.is_unstable() {
            panic!("[ERROR] Instability detected!");
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
        self.state.density()
    }
}

impl chemsim::display::Simulation for LBMSim {
    fn size(&self) -> (usize, usize) {
        self.size
    }

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
        let matrix = self.draw();
        assert_eq!(matrix.get_shape(), self.size);
        // let max = f64::max(
        //     matrix.abs().maximum_real() as Scalar,
        //     0.1,
        // );
        let max = matrix.abs().maximum_real() as Scalar;
        draw_matrix(buf, &matrix, &(|value: Scalar| -> u8 {
            let f = 256.0 * Scalar::abs(value / max);
            f.min(255.0).max(0.0) as u8
        }));
    }
}

fn initial_state(size: (usize, usize)) -> LBMSim {
    use chemsim::*;

    let collision = lbm::BGK { tau: 4.0 };

    let initial_density = {
        // FIXME: proper initialization
        // matrix::Matrix::new_filled(0.0, size)
        // matrix::Matrix::new_filled(1.0, size)
        // matrix::Matrix::new_random(size).abs().scale(10.0)
        // matrix::Matrix::new_identity(size)

        // matrix::Matrix::new_filled(1.0, size)
        //     + matrix::Matrix::new_random(size).scale(0.1)

        let (w, h) = size;
        let mut vec = Vec::new();
        vec.resize(w * h, 0.0);
        for x in 0 .. w {
            for y in 0 .. h {
                let mut val = 0.0;
                val += 1.0;
                val += 0.3 * Scalar::sin(3.0 * (x as Scalar) / (w as Scalar));
                val += 0.3 * Scalar::sin(3.0 * (y as Scalar) / (h as Scalar));
                vec[(y * w) + x] = val;
            }
        }
        matrix::Matrix::new(&vec, size).unwrap()
    };

    let pops = &[
        initial_density.scale(16.0 / 36.0),
        initial_density.scale(4.0  / 36.0),
        initial_density.scale(4.0  / 36.0),
        initial_density.scale(4.0  / 36.0),
        initial_density.scale(4.0  / 36.0),
        initial_density.scale(1.0  / 36.0),
        initial_density.scale(1.0  / 36.0),
        initial_density.scale(1.0  / 36.0),
        initial_density.scale(1.0  / 36.0),
    ];

    let lattice = lbm::Lattice::new_D2Q9(pops);

    let disc = lbm::Discretization { delta_x: 1.0, delta_t: 1.0 };

    let state = lbm::State::initial(lattice, disc, Box::new(collision));

    LBMSim { size: size, state: state }
}

extern crate gif;
use std::fs::File;

fn main() {
    // chemsim::display::conrod();
    af::init();
    chemsim::display::example(initial_state((300, 300)));

    // use gif::SetParameter;
    // let (w, h) = (500, 500);
    // let initial = initial_state((w, h));
    // let color_map = &[0xFF, 0xFF, 0xFF, 0, 0, 0];
    // let mut image = File::create("output.gif").unwrap();
    // let mut encoder
    //     = gif::Encoder::new(&mut image, w as u16, h as u16, color_map).unwrap();
    // encoder.set(gif::Repeat::Infinite).unwrap();
    // chemsim::display::record(initial, 350, &mut encoder).unwrap();
}
