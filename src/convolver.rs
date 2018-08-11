use std;
use arrayfire as af;
use super::matrix;
use super::lbm::{Scalar, Vector, Matrix};

pub struct Convolver {
    size:       (usize, usize),
    matrix:     Matrix,
    fft_matrix: Matrix,
}

static M_1: [Scalar; 9] = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
static M_2: [Scalar; 9] = [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
static M_3: [Scalar; 9] = [0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
static M_4: [Scalar; 9] = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
static M_5: [Scalar; 9] = [0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0];
static M_6: [Scalar; 9] = [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0];
static M_7: [Scalar; 9] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0];
static M_8: [Scalar; 9] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0];
static M_9: [Scalar; 9] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0];

fn array_equal(lhs: &af::Array, rhs: &af::Array) -> bool {
    let result = af::all_true_all(&af::eq(lhs, rhs, false));
    (result.0 > 0.0) && (result.1 == 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asin_deg(array: &af::Array) -> af::Array {
        let r2d: f64 = 57.2957795131;
        af::mul(&r2d, &af::asin(&array), false)
    }

    fn acos_deg(array: &af::Array) -> af::Array {
        let r2d: f64 = 57.2957795131;
        af::mul(&r2d, &af::acos(&array), false)
    }

    fn asin_frac(array: &af::Array) -> af::Array {
        let r2f: f64 = 15.9154943092;
        af::mul(&r2f, &af::asin(&array), false)
    }

    fn acos_frac(array: &af::Array) -> af::Array {
        let r2f: f64 = 15.9154943092;
        af::mul(&r2f, &af::acos(&array), false)
    }

    #[test]
    fn it_works() {
        let (w, h) = (10, 30);
        let mA = Matrix::new(&M_3, (3, 3)).unwrap().transpose();
        let mB = Matrix::new(&[
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ], (w, h)).unwrap().scale(666.0);

        // Matrix::new_random((w, h));
        let fft_mA = af::fft2(mA.get_array(), 1.0, h as i64, w as i64);
        let fft_mB = af::fft2(mB.get_array(), 1.0, h as i64, w as i64);
        // let product = &fft_mA * &fft_mB;
        let fft_mA_real = af::real(&fft_mA);
        let fft_mA_imag = af::imag(&fft_mA);
        let fft_mB_real = af::real(&fft_mB);
        let fft_mB_imag = af::imag(&fft_mB);
        let product = af::cplx2(
            &((&fft_mA_real * &fft_mB_real) - (&fft_mA_imag * &fft_mB_imag)),
            &((&fft_mA_real * &fft_mB_imag) + (&fft_mA_imag * &fft_mB_real)),
            false,
        );
        let result =
            af::round(
                &af::real(
                    &af::ifft2(&product, 1.0 / ((w * h) as f64),
                               h as i64, w as i64)
                )
            ).cast::<i32>();
        af::print(&af::real(&fft_mA));
        af::print(&af::imag(&fft_mA));
        af::print(&mB.get_array().cast::<i32>());
        af::print(&result);
        let ground_truth = af::round(&af::fft_convolve2(mB.get_array(), mA.get_array(), af::ConvMode::DEFAULT)).cast::<i32>();
        af::print(&ground_truth);
        assert!(array_equal(&result, &ground_truth));
        // af::save_image("/home/remy/fft.png".to_string(), &fft_m1);
    }
}
