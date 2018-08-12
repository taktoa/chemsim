#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(unused_parens)]
#![feature(duration_as_u128)]
#![feature(use_extern_macros)]
#![feature(const_fn)]

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
extern crate conrod;

pub mod matrix;
pub mod convolver;
pub mod lbm;
pub mod display;
pub mod render;
pub mod theme;
pub mod preconditioned;
// pub mod record;
