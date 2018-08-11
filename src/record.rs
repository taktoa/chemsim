// -----------------------------------------------------------------------------

use std::io::{self};
use std::path::Path;
use std::fs::{File, OpenOptions};
use image::{self, RgbaImage, ImageBuffer};
use webm::mux::{self, Track};
use vpx;

// -----------------------------------------------------------------------------

pub fn record(
    size:    (usize, usize),
    path:    &Path,
    render:  &mut (FnMut() -> RgbaImage),
    frames:  usize,
    bitrate: usize,
) -> io::Result<()> {
    let (width, height) = size;

    let mut open_options = OpenOptions::new();
    open_options.write(true);
    open_options.create_new(true);

    let out = match open_options.open(path) {
        Ok(file) => file,
        Err(err) => {
            if err.kind() == io::ErrorKind::AlreadyExists {
                File::create(path)?
            } else {
                return Err(err.into());
            }
        },
    };

    let mut webm = mux::Segment::new(mux::Writer::new(out))
        .expect("Could not initialize the multiplexer.");

    let mut vt = webm.add_video_track(width as u32, height as u32,
                                      None, mux::VideoCodecId::VP9);
    vt.set_color(8, (false, false), true);

    let mut vpx = self::Encoder::new(self::Config {
        width:    width  as u32,
        height:   height as u32,
        timebase: [1, 1000],
        bitrate:  bitrate as u32,
    });

    {
        let mut yuv = Vec::new();

        for pts in 0 .. frames {
            let rgba_image = (render)();

            let mut rgba: Vec<u8> = Vec::new();
            for pixel in rgba_image.pixels() {
                let r = pixel.data[0];
                let g = pixel.data[1];
                let b = pixel.data[2];
                let a = pixel.data[3];
                rgba.push(b);
                rgba.push(g);
                rgba.push(r);
                rgba.push(a);
                // rgba.push(((219.0 * (b as f64 / 256.0)) + 16.0).round().min(235.0).max(16.0) as u8);
                // rgba.push(((219.0 * (g as f64 / 256.0)) + 16.0).round().min(235.0).max(16.0) as u8);
                // rgba.push(((219.0 * (r as f64 / 256.0)) + 16.0).round().min(235.0).max(16.0) as u8);
                // rgba.push(255);
            }

            bgra_to_i420(width as usize, height as usize, &rgba, &mut yuv);

            for frame in vpx.encode((pts * 20) as i64, &yuv) {
                vt.add_frame(frame.data,
                             frame.pts as u64 * 1_000_000,
                             frame.key);
            }
        }
    }

    let mut frames = vpx.finish();
    while let Some(frame) = frames.next() {
        vt.add_frame(frame.data, frame.pts as u64 * 1_000_000, frame.key);
    }

    let _ = webm.finalize(None);

    Ok(())
}

// -----------------------------------------------------------------------------

use std::os::raw::{c_int, c_uint, c_ulong};
use std::{ptr, slice};
use vpx_sys::vp8e_enc_control_id::*;
use vpx_sys::vpx_codec_cx_pkt_kind::VPX_CODEC_CX_FRAME_PKT;
use vpx_sys::*;

// -----------------------------------------------------------------------------

const ABI_VERSION: c_int = 12;
const DEADLINE: c_ulong = 0;

// -----------------------------------------------------------------------------

pub struct Encoder {
    ctx: vpx_codec_ctx_t,
    width: usize,
    height: usize,
}

impl Encoder {
    pub fn new(config: Config) -> Self {
        let mut err = vpx_codec_err_t::VPX_CODEC_OK;

        let i = unsafe { vpx_codec_vp9_cx() };

        assert!(config.width % 2 == 0);
        assert!(config.height % 2 == 0);

        let mut c = Default::default();
        err = unsafe { vpx_codec_enc_config_default(i, &mut c, 0) };
        if err != vpx_codec_err_t::VPX_CODEC_OK {
            panic!("libvpx encountered an error: {:?}", err);
        }

        c.g_w = config.width;
        c.g_h = config.height;
        c.g_timebase.num = config.timebase[0];
        c.g_timebase.den = config.timebase[1];
        c.rc_target_bitrate = config.bitrate;

        c.g_threads = 8;
        c.g_error_resilient = VPX_ERROR_RESILIENT_DEFAULT;

        let mut ctx = Default::default();

        err = unsafe { vpx_codec_enc_init_ver(&mut ctx, i, &c, 0, ABI_VERSION) };
        if err != vpx_codec_err_t::VPX_CODEC_OK {
            panic!("SOMETHING IS WRONG AT {:?}!!! {:?}", line!(), err);
        }

        err = unsafe { vpx_codec_control_(&mut ctx, VP8E_SET_CPUUSED as _, 6 as c_int) };
        if err != vpx_codec_err_t::VPX_CODEC_OK {
            panic!("libvpx encountered an error: {:?}", err);
        }

        // vpx_codec_control_(&mut ctx, VP9E_SET_ROW_MT  as _, 1 as c_int);

        Self {
            ctx,
            width:  config.width  as usize,
            height: config.height as usize,
        }
    }

    pub fn encode(&mut self, pts: i64, data: &[u8]) -> Packets {
        let mut err = vpx_codec_err_t::VPX_CODEC_OK;

        assert!(2 * data.len() >= 3 * self.width * self.height);

        let mut image = Default::default();
        unsafe {
            vpx_img_wrap(
                &mut image,
                vpx_img_fmt::VPX_IMG_FMT_I420,
                self.width as _,
                self.height as _,
                1,
                data.as_ptr() as _,
            );
            // image.range = vpx_color_range::VPX_CR_FULL_RANGE;
        }

        err = unsafe {
            vpx_codec_encode(
                &mut self.ctx,
                &image,
                pts,
                1, // Alignment
                0, // Flags
                DEADLINE,
            )
        };

        if err != vpx_codec_err_t::VPX_CODEC_OK {
            panic!("libvpx encountered an error: {:?}", err);
        }

        Packets {
            ctx: &mut self.ctx,
            iter: ptr::null(),
        }
    }

    pub fn finish(mut self) -> Finish {
        let mut err = vpx_codec_err_t::VPX_CODEC_OK;

        err = unsafe {
            vpx_codec_encode(
                &mut self.ctx,
                ptr::null(),
                -1, // PTS
                1,  // Alignment
                0,  // Flags
                DEADLINE,
            )
        };

        if err != vpx_codec_err_t::VPX_CODEC_OK {
            panic!("libvpx encountered an error: {:?}", err);
        }

        Finish {
            enc: self,
            iter: ptr::null(),
        }
    }
}

impl Drop for Encoder {
    fn drop(&mut self) {
        unsafe {
            let _ = vpx_codec_destroy(&mut self.ctx);
        }
    }
}

// -----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct Frame<'a> {
    /// Compressed data.
    pub data: &'a [u8],
    /// Whether the frame is a keyframe.
    pub key: bool,
    /// Presentation timestamp (in timebase units).
    pub pts: i64,
}

// -----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct Config {
    /// The width (in pixels).
    pub width: c_uint,
    /// The height (in pixels).
    pub height: c_uint,
    /// The timebase (in seconds).
    pub timebase: [c_int; 2],
    /// The target bitrate (in kilobits per second).
    pub bitrate: c_uint,
}

// -----------------------------------------------------------------------------

pub struct Packets<'a> {
    ctx: &'a mut vpx_codec_ctx_t,
    iter: vpx_codec_iter_t,
}

impl<'a> Iterator for Packets<'a> {
    type Item = Frame<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            unsafe {
                let pkt = vpx_codec_get_cx_data(self.ctx, &mut self.iter);

                if pkt.is_null() {
                    return None;
                } else if (*pkt).kind == VPX_CODEC_CX_FRAME_PKT {
                    let f = &(*pkt).data.frame;
                    return Some(Frame {
                        data: slice::from_raw_parts(f.buf as _, f.sz),
                        key: (f.flags & VPX_FRAME_IS_KEY) != 0,
                        pts: f.pts,
                    });
                } else {
                    // Ignore the packet.
                }
            }
        }
    }
}

// -----------------------------------------------------------------------------

pub struct Finish {
    enc: Encoder,
    iter: vpx_codec_iter_t,
}

impl Finish {
    pub fn next(&mut self) -> Option<Frame> {
        let mut err = vpx_codec_err_t::VPX_CODEC_OK;

        let mut tmp = Packets {
            ctx: &mut self.enc.ctx,
            iter: self.iter,
        };

        if let Some(packet) = tmp.next() {
            self.iter = tmp.iter;
            Some(packet)
        } else {
            err = unsafe {
                vpx_codec_encode(
                    tmp.ctx,
                    ptr::null(),
                    -1, // PTS
                    1,  // Alignment
                    0,  // Flags
                    DEADLINE,
                )
            };

            if err != vpx_codec_err_t::VPX_CODEC_OK {
                panic!("libvpx encountered an error: {:?}", err);
            }

            tmp.iter = ptr::null();
            if let Some(packet) = tmp.next() {
                self.iter = tmp.iter;
                Some(packet)
            } else {
                None
            }
        }
    }
}

// -----------------------------------------------------------------------------

pub fn bgra_to_i420(width: usize, height: usize, src: &[u8], dest: &mut Vec<u8>) {
    fn clamp(x: i32) -> u8 { x.min(255).max(0) as u8 }

    let stride = src.len() / height;

    dest.clear();

    for y in 0..height {
        for x in 0..width {
            let o = y * stride + 4 * x;

            let b = src[o] as i32;
            let g = src[o + 1] as i32;
            let r = src[o + 2] as i32;

            let y = (66 * r + 129 * g + 25 * b + 128) / 256 + 16;
            dest.push(clamp(y));
        }
    }

    for y in (0..height).step_by(2) {
        for x in (0..width).step_by(2) {
            let o = y * stride + 4 * x;

            let b = src[o] as i32;
            let g = src[o + 1] as i32;
            let r = src[o + 2] as i32;

            let u = (-38 * r - 74 * g + 112 * b + 128) / 256 + 128;
            dest.push(clamp(u));
        }
    }

    for y in (0..height).step_by(2) {
        for x in (0..width).step_by(2) {
            let o = y * stride + 4 * x;

            let b = src[o] as i32;
            let g = src[o + 1] as i32;
            let r = src[o + 2] as i32;

            let v = (112 * r - 94 * g - 18 * b + 128) / 256 + 128;
            dest.push(clamp(v));
        }
    }
}

// -----------------------------------------------------------------------------
