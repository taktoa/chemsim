#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(non_snake_case)]

extern crate piston;
extern crate graphics;
extern crate opengl_graphics;
extern crate glutin_window;
extern crate timer;
extern crate chrono;
extern crate image;
extern crate arrayfire;
extern crate num_complex;

pub mod matrix;
pub mod lbm;
pub mod display;

pub fn main() {
    // use arrayfire as af;
    // let signal = af::Array::new(&[0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0],
    //                             af::Dim4::new(&[8, 1, 1, 1]));
    // let kernel = af::Array::new(&[0.5, 0.5, 0.5],
    //                             af::Dim4::new(&[3, 1, 1, 1]));
    // let convolved = af::convolve1(&signal, &kernel,
    //                               af::ConvMode::EXPAND,
    //                               af::ConvDomain::SPATIAL);
    // af::print(&signal);
    // af::print(&kernel);
    // af::print(&convolved);
    display::example();
}
