#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![feature(duration_as_u128)]

extern crate chemsim;
extern crate piston;
extern crate arrayfire;

use chemsim::display::Drawable;
use chemsim::lbm::{Scalar, Matrix};
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
        draw_matrix(buf, &matrix, &(|value: Scalar| -> u8 {
            (256.0 / (1.0 + Scalar::exp(-value))) as u8
        }));
    }
}

fn initial_state(size: (usize, usize)) -> LBMSim {
    use chemsim::*;

    let collision = lbm::BGK { tau: 2.0 };

    let initial_pop = |i: usize| {
        // FIXME: proper initialization
        // matrix::Matrix::new_filled(0.0, size)
        matrix::Matrix::new_random(size).abs().scale(0.0001)
        // matrix::Matrix::new_identity(size)
    };

    let pops = &[
        initial_pop(0),
        initial_pop(1),
        initial_pop(2),
        initial_pop(3),
        initial_pop(4),
        initial_pop(5),
        initial_pop(6),
        initial_pop(7),
        initial_pop(8),
    ];

    let lattice = lbm::Lattice::new_D2Q9(pops, Box::new(collision));

    let disc = lbm::Discretization { delta_x: 1.0, delta_t: 1.0 };

    let state = lbm::State::initial(lattice, disc);

    LBMSim { size: size, state: state }
}

fn main() {
    arrayfire::init();
    chemsim::display::example(initial_state((300, 300)));
}
