#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(unused_parens)]
#![feature(const_fn)]
#![feature(duration_float)]

extern crate chemfiles;
extern crate piston;
extern crate graphics;
extern crate opengl_graphics;
extern crate glutin_window;
extern crate timer;
extern crate chrono;
extern crate image;
extern crate arrayfire;
extern crate num_complex;
extern crate num_traits;
extern crate input;
// extern crate webm;
// extern crate vpx;
// extern crate vpx_sys;
// extern crate ffmpeg;

#[macro_use]
extern crate conrod_core as conrod;

extern crate conrod_piston;

pub mod matrix;
pub mod lbm;
pub mod display;
pub mod render;
pub mod theme;
// pub mod preconditioned;
// pub mod record;
pub mod qchem;
