// -----------------------------------------------------------------------------

use std;
use std::io::{self};
use std::path::Path;
use std::fs::{File, OpenOptions};
use image::{self, RgbaImage, ImageBuffer};
use webm::mux::{self, Track};
use vpx;
use ffmpeg;

// -----------------------------------------------------------------------------

pub fn record(
    size:    (usize, usize),
    path:    &Path,
    render:  &mut (FnMut() -> RgbaImage),
    frames:  usize,
    bitrate: usize,
) -> io::Result<()> {
    // let (width, height) = size;
    //
    // let mut open_options = OpenOptions::new();
    // open_options.write(true);
    // open_options.create_new(true);
    //
    // let out = match open_options.open(path) {
    //     Ok(file) => file,
    //     Err(err) => {
    //         if err.kind() == io::ErrorKind::AlreadyExists {
    //             File::create(path)?
    //         } else {
    //             return Err(err.into());
    //         }
    //     },
    // };
    //
    // let mut webm = mux::Segment::new(mux::Writer::new(out))
    //     .expect("Could not initialize the multiplexer.");
    //
    // let mut vt = webm.add_video_track(width as u32, height as u32,
    //                                   None, mux::VideoCodecId::VP9);
    // vt.set_color(8, (false, false), true);
    //
    // let mut vpx = self::Encoder::new(self::Config {
    //     width:    width  as u32,
    //     height:   height as u32,
    //     timebase: [1, 1000],
    //     bitrate:  bitrate as u32,
    // });
    //
    // {
    //     let mut yuv = Vec::new();
    //
    //     for pts in 0 .. frames {
    //         let rgba_image = (render)();
    //
    //         let mut rgba: Vec<u8> = Vec::new();
    //         for pixel in rgba_image.pixels() {
    //             let r = pixel.data[0];
    //             let g = pixel.data[1];
    //             let b = pixel.data[2];
    //             let a = pixel.data[3];
    //             rgba.push(b);
    //             rgba.push(g);
    //             rgba.push(r);
    //             rgba.push(a);
    //             // rgba.push(((219.0 * (b as f64 / 256.0)) + 16.0).round().min(235.0).max(16.0) as u8);
    //             // rgba.push(((219.0 * (g as f64 / 256.0)) + 16.0).round().min(235.0).max(16.0) as u8);
    //             // rgba.push(((219.0 * (r as f64 / 256.0)) + 16.0).round().min(235.0).max(16.0) as u8);
    //             // rgba.push(255);
    //         }
    //
    //         bgra_to_i420(width as usize, height as usize, &rgba, &mut yuv);
    //
    //         for frame in vpx.encode((pts * 20) as i64, &yuv) {
    //             vt.add_frame(frame.data,
    //                          frame.pts as u64 * 1_000_000,
    //                          frame.key);
    //         }
    //     }
    // }
    //
    // let mut frames = vpx.finish();
    // while let Some(frame) = frames.next() {
    //     vt.add_frame(frame.data, frame.pts as u64 * 1_000_000, frame.key);
    // }
    //
    // let _ = webm.finalize(None);

    Ok(())
}

// -----------------------------------------------------------------------------

pub mod transcode {
    use std;
    use ffmpeg;
    use image::{self, RgbaImage};

    // -------------------------------------------------------------------------

    #[derive(Debug)]
    pub enum TranscodeError {
        NoFrameCompressed,
        NoFrameDecompressed,
        IOError(std::io::Error),
        FFError(ffmpeg::util::error::Error),
    }

    impl std::convert::From<std::io::Error> for TranscodeError {
        fn from(e: std::io::Error) -> TranscodeError {
            TranscodeError::IOError(e)
        }
    }

    impl std::convert::From<ffmpeg::util::error::Error> for TranscodeError {
        fn from(e: ffmpeg::util::error::Error) -> TranscodeError {
            TranscodeError::FFError(e)
        }
    }

    // -------------------------------------------------------------------------

    pub type Result<T> = std::result::Result<T, TranscodeError>;

    // -------------------------------------------------------------------------

    // pub mod io_context {
    //     use libc;
    //     use ffmpeg::{self, ffi::AVIOContext};
    //     use std::rc::Rc;
    //
    //     pub struct IOContext {
    //         ptr:  *mut AVIOContext,
    //         dtor: Rc<Destructor>,
    //     }
    //
    //     impl IOContext {
    //         pub fn new(
    //             buffer:       &mut[u8],
    //             write_flag:   i32,
    //             opaque:       *mut libc::c_void,
    //             read_packet:  Option<unsafe extern "C" fn(*mut libc::c_void, *mut u8, i32) -> i32>,
    //             write_packet: Option<unsafe extern "C" fn(*mut libc::c_void, *mut u8, i32) -> i32>,
    //             seek:         Option<unsafe extern "C" fn(*mut libc::c_void, i64, i32) -> i64>,
    //         ) -> Self {
    //             // unsafe { ffmpeg::ffi::avio_alloc_context(...); }
    //             unimplemented!()
    //         }
    //     }
    //
    //     struct Destructor {
    //         ptr:  *mut ffmpeg::ffi::AVIOContext,
    //     }
    //
    //     impl Destructor {
    //         pub unsafe fn new(ptr: *mut AVIOContext) -> Self {
    //             Destructor { ptr: ptr }
    //         }
    //     }
    //
    //     impl Drop for Destructor {
    //         fn drop(&mut self) {
    //             unimplemented!()
    //         }
    //     }
    // }

    // -------------------------------------------------------------------------

    pub struct Transcoder {
        pts:     usize,
        output:  ffmpeg::format::context::Output,
        decoder: ffmpeg::codec::decoder::Video,
        encoder: ffmpeg::codec::encoder::Video,
    }

    impl Transcoder {
        pub fn new(path: &std::path::Path) -> self::Result<Self> {
            let mut octx = ffmpeg::format::output(&path)?;
            octx.set_metadata(ffmpeg::Dictionary::new());
            octx.write_header()?;
            let decoder_id = ffmpeg::codec::id::Id::BMP;
            let encoder_id = ffmpeg::codec::id::Id::BMP;
            use ffmpeg::codec::traits::{Decoder, Encoder};
            let decoder_codec
                = ffmpeg::codec::decoder::find(decoder_id).unwrap();
            let encoder_codec
                = ffmpeg::codec::encoder::find(encoder_id).unwrap();
            let output = octx.add_stream(encoder_codec)?;
            let decoder = output.codec().decoder().video()?;
            let encoder = output.codec().encoder().video()?;
            // Ok(Transcoder {
            //     pts:     0,
            //     output:  octx,
            //     decoder: decoder,
            //     encoder: encoder,
            // })
            unimplemented!()
        }

        pub fn add_frame(mut self, img: &RgbaImage) -> self::Result<Self> {
            use ffmpeg::codec::packet::Packet;
            use ffmpeg::util::format::pixel::Pixel;
            use ffmpeg::util::frame::video::Video as Frame;
            let bmp: Vec<u8> = rgba_to_bmp(img)?;
            let input_packet = Packet::copy(&bmp[..]);
            let mut frame = Frame::new(Pixel::RGB24, img.width(), img.height());
            if !(self.decoder.decode(&input_packet, &mut frame)?) {
                return Err(TranscodeError::NoFrameDecompressed);
            }
            let mut output_packet = Packet::empty();
            if !(self.encoder.encode(&frame, &mut output_packet)?) {
                return Err(TranscodeError::NoFrameCompressed);
            }
            output_packet.set_stream(0);
            output_packet.write_interleaved(&mut self.output)?;
            self.pts += 1;
            Ok(self)
        }

        pub fn finalize(self) -> self::Result<()> {
            Ok(())
        }
    }

    fn rgba_to_bmp(image: &RgbaImage) -> std::io::Result<Vec<u8>> {
        let mut cursor = std::io::Cursor::new(Vec::new());
        {
            let mut encoder = image::bmp::BMPEncoder::new(&mut cursor);
            use image::Pixel;
            // encoder.encode(
            //     image.
            //     image::Rgba::color_type(),
            // );
        }
        Ok(cursor.into_inner())
    }
}

// -----------------------------------------------------------------------------
