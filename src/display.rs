use arrayfire as af;
use piston::window::{Window, WindowSettings};
use opengl_graphics::{GlGraphics, OpenGL, Texture, TextureSettings};
use graphics::{Image, clear};
use graphics::rectangle::*;
use piston::input::Event;
use std::path::Path;
use piston::input::{RenderEvent, ButtonEvent, UpdateEvent};
use piston::event_loop::*;
use timer::Timer;
use std::sync::mpsc::sync_channel;
use std::time::{Duration, Instant};
use image;
use chrono;
use glutin_window;
use conrod::{self, widget, Colorable, Positionable, Widget};
use conrod::backend::piston as conrod_piston;
use opengl_graphics;

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

pub trait Simulation {
    fn size(&self) -> (usize, usize);
    fn handle(&mut self, input: &Event);
    fn step(&mut self, elapsed: &Duration);
    fn render<D: Drawable>(&self, buf: &mut D);
}

pub fn example<S: Simulation>(initial: S) {
    let (w, h) = initial.size();

    let opengl = OpenGL::V3_2;
    let window_settings
        = WindowSettings::new("Example", [w as u32, h as u32])
        .srgb(false)
        .vsync(true)
        .opengl(opengl)
        // .fullscreen(true)
        .exit_on_esc(true);
    let mut window
        = glutin_window::GlutinWindow::new(&window_settings)
        .expect("Failed to make window");
    let mut gl = GlGraphics::new(opengl);

    let image = Image::new().rect([0.0, 0.0, w as f64, h as f64]);
    let mut rgba_image: image::RgbaImage
        = image::ImageBuffer::new(w as u32, h as u32);

    for (_, _, pixel) in rgba_image.enumerate_pixels_mut() {
        pixel.data = [0, 0, 0, 255];
    }

    let mut texture = Texture::from_image(&rgba_image, &TextureSettings::new());
    let mut state = initial;
    let mut last_draw = Instant::now();
    let mut events = Events::new(EventSettings::new());

    while let Some(e) = events.next(&mut window) {
        state.handle(&e);

        if let Some(b) = e.button_args() {
            use input::{Button, ButtonState};
            use input::keyboard::Key;
            if let Button::Keyboard(k) = b.button {
                // println!("Key received: {:?}", k);
                if (k == Key::Q) && (b.state == ButtonState::Release) {
                    println!("Quitting!");
                    window.set_should_close(true);
                }
            }
        }

        if let Some(r) = e.render_args() {
            state.step(&last_draw.elapsed());
            last_draw = Instant::now();
            state.render(&mut rgba_image);
            texture.update(&rgba_image);
            gl.draw(r.viewport(), |c, gl| {
                clear([0.0, 0.0, 0.0, 1.0], gl);
                image.draw(&texture, &Default::default(), c.transform, gl);
            });
        }
    }
}

extern crate gif;
use std::io::Result;
use std::io::Write;

pub fn record<S: Simulation, W: Write>(
    initial: S,
    steps:   usize,
    encoder: &mut gif::Encoder<W>
) -> Result<()>
{
    let (w, h) = initial.size();

    let mut rgba_image: image::RgbaImage
        = image::ImageBuffer::new(w as u32, h as u32);

    for (_, _, pixel) in rgba_image.enumerate_pixels_mut() {
        pixel.data = [0, 0, 0, 255];
    }

    let mut state = initial;
    let mut last_draw = Instant::now();

    for _ in 0 .. steps {
        state.step(&last_draw.elapsed());
        last_draw = Instant::now();
        state.render(&mut rgba_image);
        let frame = gif::Frame::from_rgba(w as u16, h as u16,
                                          &mut (rgba_image.clone().into_raw()));
        encoder.write_frame(&frame)?;
    }

    Ok(())
}

extern crate ttf_noto_sans;

pub fn conrod() {
    let w: usize = 500;
    let h: usize = 500;

    // Create the window.
    let opengl = OpenGL::V3_2;
    let window_settings
        = WindowSettings::new("Example", [w as u32, h as u32])
        .srgb(false)
        .vsync(true)
        .opengl(opengl)
        .exit_on_esc(true);
    let mut window
        = glutin_window::GlutinWindow::new(&window_settings)
        .expect("Failed to make window");
    let mut gl = GlGraphics::new(opengl);

    let mut ui = conrod::UiBuilder::new([w as f64, h as f64])
        .theme(super::theme::theme())
        .build();

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    use conrod::text::FontCollection;
    ui.fonts.insert(FontCollection::from_bytes(ttf_noto_sans::REGULAR)
                    .expect("failed to FontCollection::from_bytes")
                    .into_font()
                    .expect("failed to into_font"));

    // Create a texture to use for efficiently caching text on the GPU.
    let mut text_vertex_data = Vec::new();
    let (mut glyph_cache, mut text_texture_cache) = {
        let scale_tolerance:    f32 = 0.1;
        let position_tolerance: f32 = 0.1;
        let cache = conrod::text::GlyphCache::new(w as u32, h as u32,
                                                  scale_tolerance,
                                                  position_tolerance);
        let init = vec![128; w * h];
        let settings = TextureSettings::new();
        let texture = Texture::from_memory_alpha(&init, w as u32, h as u32, &settings).unwrap();
        (cache, texture)
    };

    // Instantiate the generated list of widget identifiers.
    let ids = super::theme::Ids::new(ui.widget_id_generator());

    // Load the rust logo from file to a Texture.
    let rust_logo: Texture = {
        let path = Path::new("/home/remy/Downloads/rust.png");
        let settings = TextureSettings::new();
        Texture::from_path(&path, &settings).unwrap()
    };

    // Create our `conrod::image::Map` which describes each of our widget->image mappings.
    // In our case we only have one image, however the macro may be used to list multiple.
    let mut image_map = conrod::image::Map::new();
    let rust_logo = image_map.insert(rust_logo);

    // A demonstration of some state that we'd like to control with the App.
    let mut app = super::theme::DemoApp::new(rust_logo);

    let mut events = Events::new(EventSettings::new());

    while let Some(event) = events.next(&mut window) {
        let size = window.size();
        let (win_w, win_h) = (size.width  as conrod::Scalar,
                              size.height as conrod::Scalar);
        if let Some(e) = conrod_piston::event::convert(event.clone(), win_w, win_h) {
            ui.handle_event(e);
        }

        event.update(|_| {
            let mut ui = ui.set_widgets();
            super::theme::gui(&mut ui, &ids, &mut app);
        });

        if let Some(r) = event.render_args() {
            gl.draw(r.viewport(), |context, graphics| {
                if let Some(primitives) = ui.draw_if_changed() {
                    use conrod::text::rt::Rect;
                    let cache_queued_glyphs
                        = |graphics: &mut GlGraphics, cache: &mut Texture, rect: Rect<u32>, data: &[u8]| {
                            text_vertex_data.clear();
                            text_vertex_data.extend(
                                data.iter()
                                    .flat_map(|&b| vec![255, 255, 255, b])
                            );
                            opengl_graphics::UpdateTexture::update(
                                cache, &mut (),
                                opengl_graphics::Format::Rgba8,
                                &text_vertex_data[..],
                                [rect.min.x, rect.min.y],
                                [rect.width(), rect.height()],
                            ).expect("failed to update texture")
                        };

                    // Specify how to get the drawable texture from the image.
                    // In this case, the image *is* the texture.
                    fn texture_from_image<T>(img: &T) -> &T { img }

                    // Draw the conrod `render::Primitives`.
                    conrod::backend::piston::draw::primitives(
                        primitives,
                        context,
                        graphics,
                        &mut text_texture_cache,
                        &mut glyph_cache,
                        &image_map,
                        cache_queued_glyphs,
                        texture_from_image,
                    );
                }
            });
        }
    }
}
