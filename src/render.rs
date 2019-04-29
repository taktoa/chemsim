use std;
use arrayfire as af;
use super::lbm::{Matrix, Geometry};
use super::matrix::{self};
use super::display::{Drawable, RGB, PixelPos};

pub fn render_geometry<D: Drawable>(geometry: &Geometry, buf: &mut D) {
    let (w, h) = buf.dimensions();
    let mut vec = Vec::new();
    let dims = geometry.dims();
    assert_eq!(dims[2], 1);
    assert_eq!(dims[3], 1);
    unsafe { vec.resize((dims[0] * dims[1]) as usize, std::mem::zeroed()); }
    af::transpose(geometry, false).host(&mut vec);
    for x in 0 .. w {
        for y in 0 .. h {
            let i = ((y * w) + x) as usize;
            if vec[i] { buf.set_pixel(PixelPos(x, y), RGB(0, 0, 255)); }
        }
    }
}

pub fn render_scalar_field<D: Drawable>(field: &Matrix, buf: &mut D) {
    let size = {
        let dimensions = buf.dimensions();
        (dimensions.0 as usize, dimensions.1 as usize)
    };

    assert_eq!(size, field.get_shape());

    let hsv_array: af::Array<f32> = {
        let hue: matrix::Matrix = {
            matrix::Matrix::new_filled(0.0, size)
        };

        let sat: matrix::Matrix = {
            matrix::Matrix::new_filled(1.0, size)
        };

        let val: matrix::Matrix = {
            let avg = af::mean_all(field.get_array()).0;
            let std = af::stdev_all(field.get_array()).0;
            let shape = field.get_shape();
            (field - Matrix::new_filled(avg as f32, shape))
                .scale(1.0 / std as f32)
                .logistic()
        };

        assert_eq!(size, hue.get_shape());
        assert_eq!(size, sat.get_shape());
        assert_eq!(size, val.get_shape());

        af::join_many(2, vec![
            hue.clamp(0.0, 1.0).get_array(),
            sat.clamp(0.0, 1.0).get_array(),
            val.clamp(0.0, 1.0).get_array(),
        ])
    };

    let rgb_array: af::Array<f32> = af::hsv2rgb(&hsv_array);

    let r_matrix = matrix::Matrix::unsafe_new(af::slice(&rgb_array, 0));
    let g_matrix = matrix::Matrix::unsafe_new(af::slice(&rgb_array, 1));
    let b_matrix = matrix::Matrix::unsafe_new(af::slice(&rgb_array, 2));

    assert_eq!(size, r_matrix.get_shape());
    assert_eq!(size, g_matrix.get_shape());
    assert_eq!(size, b_matrix.get_shape());

    {
        let (w, h) = size;
        let r_vec = r_matrix.get_underlying();
        let g_vec = g_matrix.get_underlying();
        let b_vec = b_matrix.get_underlying();
        for x in 0 .. w {
            for y in 0 .. h {
                let r_raw = r_vec[(y * w) + x];
                let g_raw = g_vec[(y * w) + x];
                let b_raw = b_vec[(y * w) + x];
                let color = RGB(
                    (256.0 * r_raw).round().min(255.0).max(0.0) as u8,
                    (256.0 * g_raw).round().min(255.0).max(0.0) as u8,
                    (256.0 * b_raw).round().min(255.0).max(0.0) as u8,
                );
                buf.set_pixel(PixelPos(x as u32, y as u32), color);
            }
        }
    }
}

pub fn render_vector_field<D: Drawable>(field: &(Matrix, Matrix), buf: &mut D) {
    // let kernel = af::gaussian_kernel(1, 1, 1.0, 1.0);
    // let vx = Matrix::unsafe_new(
    //     af::convolve2(field.0.get_array(),
    //                   &kernel,
    //                   af::ConvMode::DEFAULT,
    //                   af::ConvDomain::SPATIAL));
    // let vy = Matrix::unsafe_new(
    //     af::convolve2(field.1.get_array(),
    //                   &kernel,
    //                   af::ConvMode::DEFAULT,
    //                   af::ConvDomain::SPATIAL));
    let (vx, vy) = field;

    let size = {
        let dimensions = buf.dimensions();
        (dimensions.0 as usize, dimensions.1 as usize)
    };

    assert_eq!(size, vx.get_shape());
    assert_eq!(size, vy.get_shape());

    let mag = vx.hadamard(&vx) + vy.hadamard(&vy);
    let phase = Matrix::unsafe_new(
        af::arg(&af::cplx2(vx.get_array(), vy.get_array(), true)));

    let hsv_array: af::Array<f32> = {
        let hue: matrix::Matrix = {
            phase
                .shift(std::f32::consts::PI)
                .scale(std::f32::consts::FRAC_1_PI * 0.5)

        };

        let sat: matrix::Matrix = {
            matrix::Matrix::new_filled(0.8, size)
        };

        let val: matrix::Matrix = {
            let avg = af::mean_all(mag.get_array()).0;
            let std = af::stdev_all(mag.get_array()).0;
            let shape = mag.get_shape();
            (mag - Matrix::new_filled(avg as f32, shape))
                .scale(1.0 / std as f32)
                .logistic()
            // mag.clamp(0.0, 1.0)
        };

        assert_eq!(size, hue.get_shape());
        assert_eq!(size, sat.get_shape());
        assert_eq!(size, val.get_shape());

        af::join_many(2, vec![
            hue.clamp(0.0, 1.0).get_array(),
            sat.clamp(0.0, 1.0).get_array(),
            val.clamp(0.0, 1.0).get_array(),
        ])
    };

    let rgb_array: af::Array<f32> = af::hsv2rgb(&hsv_array);

    let r_matrix = matrix::Matrix::unsafe_new(af::slice(&rgb_array, 0));
    let g_matrix = matrix::Matrix::unsafe_new(af::slice(&rgb_array, 1));
    let b_matrix = matrix::Matrix::unsafe_new(af::slice(&rgb_array, 2));

    assert_eq!(size, r_matrix.get_shape());
    assert_eq!(size, g_matrix.get_shape());
    assert_eq!(size, b_matrix.get_shape());

    {
        let (w, h) = size;
        let r_vec = r_matrix.get_underlying();
        let g_vec = g_matrix.get_underlying();
        let b_vec = b_matrix.get_underlying();
        for x in 0 .. w {
            for y in 0 .. h {
                let r_raw = r_vec[(y * w) + x];
                let g_raw = g_vec[(y * w) + x];
                let b_raw = b_vec[(y * w) + x];
                let color = RGB(
                    (256.0 * r_raw).round().min(255.0).max(0.0) as u8,
                    (256.0 * g_raw).round().min(255.0).max(0.0) as u8,
                    (256.0 * b_raw).round().min(255.0).max(0.0) as u8,
                );
                buf.set_pixel(PixelPos(x as u32, y as u32), color);
            }
        }
    }

    // matrix = {
    //     let mut temp
    //         = Matrix::new_filled(0.0, self.size).get_array().clone();
    //     af::replace(&mut temp,
    //                 &self.state.geometry.get_array(),
    //                 matrix.get_array());
    //     Matrix::unsafe_new(temp)
    // };

    // let (w, h) = matrix.get_shape();
    // let copied = matrix.get_underlying();
    // for x in 0 .. w {
    //     for y in 0 .. h {
    //         let n = shader(copied[(y * w) + x]);
    //         let k = 2 * (n.abs().min(127) as u8);
    //         let value = {
    //             if n < 0 {
    //                 RGB(k, 0, 0)
    //             } else if n > 0 {
    //                 RGB(0, k, 0)
    //             } else {
    //                 RGB(0, 0, 0)
    //             }
    //         };
    //         buffer.set_pixel(PixelPos(x as u32, y as u32), value);
    //     }
    // }
}
