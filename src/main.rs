extern crate piston_window;

use piston_window::*;

pub fn main() {
    let mut window: PistonWindow
        = WindowSettings::new("Hello Piston!", (640, 480))
        .build()
        .unwrap_or_else(|e| { panic!("Failed to build PistonWindow: {}", e) });
    while let Some(e) = window.next() {
        window.draw_2d(&e, |_c, g| {
            clear([0.5, 1.0, 0.5, 1.0], g);
        });
    }
    
    println!("Hello, world!");
}
