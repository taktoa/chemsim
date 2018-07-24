extern crate piston;
extern crate graphics;
extern crate opengl_graphics;
extern crate glutin_window;
extern crate timer;
extern crate chrono;

use piston::window::{Window, WindowSettings};
use opengl_graphics::{GlGraphics, OpenGL, Texture, TextureSettings};
use graphics::{Image, clear};
use graphics::rectangle::*;
use piston::input::Event;
use std::path::Path;
use piston::input::RenderEvent;
use piston::event_loop::*;
use timer::Timer;
use std::sync::mpsc::sync_channel;
use std::time::Duration;

fn main() {
    let opengl = OpenGL::V3_2;
    let window_settings
        = WindowSettings::new("Example", [600, 400])
        .srgb(false)
        .vsync(true)
        .opengl(opengl)
        .fullscreen(true)
        .exit_on_esc(true);
    let mut window
        = glutin_window::GlutinWindow::new(&window_settings)
        .expect("Failed to make window");
    let mut gl = GlGraphics::new(opengl);

    let path    = Path::new("/home/remy/Pictures/background/tumblr_p5oizcKURY1uby4koo1_400.jpg");
    let image   = Image::new().rect([0.0, 0.0, 378.0, 396.0]);
    let texture = Texture::from_path(path, &TextureSettings::new()).unwrap();

    let timer = Timer::new();
    let (tx, rx) = sync_channel(1);
    let guard = timer.schedule_with_delay(chrono::Duration::seconds(1),
                                          move || { tx.send(()).unwrap(); });
    guard.ignore();
    
    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Ok(()) = rx.recv_timeout(Duration::from_millis(10)) {
            window.set_should_close(true);            
        }
        if let Some(r) = e.render_args() {
            gl.draw(r.viewport(), |c, gl| {
                clear([0.0, 0.0, 0.0, 1.0], gl);
                image.draw(&texture, &Default::default(), c.transform, gl);
            });
        }
    }
}
