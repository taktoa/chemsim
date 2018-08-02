use arrayfire as af;
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
use image;
use chrono;
use glutin_window;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy, Hash)]
pub struct PixelPos(pub u32, pub u32);

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy, Hash)]
pub struct RGB(pub u8, pub u8, pub u8);

impl RGB {
    pub fn to_rgba(&self) -> image::Rgba<u8> {
        image::Rgba { data: [self.0, self.1, self.2, 255] }
    }

    pub fn from_rgba(image: &image::Rgba<u8>) -> Self {
        RGB(image.data[0], image.data[1], image.data[2])
    }
}

pub trait Drawable {
    /// Create a new drawable object filled with the given color.
    fn new(size: (u32, u32), default: RGB) -> Self;

    /// Get the dimensions of the drawable object.
    fn dimensions(&self) -> (u32, u32) {
        let w = self.width();
        let h = self.height();
        (w, h)
    }

    /// Get the width of the drawable object.
    fn width(&self) -> u32;

    /// Get the height of the drawable object.
    fn height(&self) -> u32;

    /// Set a pixel in the drawable object at the given position to the given
    /// color value.
    fn set_pixel(&mut self, pos: PixelPos, value: RGB);

    /// Get the color of the pixel at the given position in the given
    /// drawable object.
    fn get_pixel(&self, pos: PixelPos) -> RGB;
}

impl Drawable for image::RgbaImage {
    fn new(size: (u32, u32), default: RGB) -> Self {
        let (w, h) = size;
        image::ImageBuffer::from_pixel(w, h, default.to_rgba())
    }

    fn width(&self)  -> u32 { image::ImageBuffer::width(&self)  }
    fn height(&self) -> u32 { image::ImageBuffer::height(&self) }

    fn set_pixel(&mut self, pos: PixelPos, value: RGB) {
        let PixelPos(x, y) = pos;
        *(self.get_pixel_mut(x, y)) = value.to_rgba();
    }

    fn get_pixel(&self, pos: PixelPos) -> RGB {
        let PixelPos(x, y) = pos;
        RGB::from_rgba(self.get_pixel(x, y))
    }
}

pub struct SimState<State> {
    window:  &'static Window,
    display: GlGraphics,
    buffer:  image::RgbaImage,
    output:  Texture,
    state:   State,
}

//pub enum

pub trait Simulation {
    type State;

    fn initial(
    ) -> Self::State;

    fn step(
        state:   &mut Self::State,
        elapsed: &chrono::Duration,
    );

    fn render<D>(
        state: &Self::State,
        buf:   &mut D,
    ) where D: Drawable;
}

pub fn example() {
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

    let path  = Path::new("/home/remy/Pictures/background/tumblr_p5oizcKURY1uby4koo1_400.jpg");
    let image = Image::new().rect([0.0, 0.0, 378.0, 396.0]);
    let mut rgba_image = image::open(path).ok().unwrap().to_rgba();
    for (x, y, pixel) in rgba_image.enumerate_pixels_mut() {
        let mut value = 1.0;
        value *= ((x as f32) / 378.0).sin();
        value *= ((y as f32) / 396.0).sin();
        let luminance = (value * 255.0) as u8;
        pixel.data = [luminance, luminance, luminance, 255];
    }
    let texture = Texture::from_image(&rgba_image, &TextureSettings::new());


    let timer = Timer::new();
    let (tx, rx) = sync_channel(1);
    let guard = timer.schedule_with_delay(chrono::Duration::milliseconds(1000),
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
