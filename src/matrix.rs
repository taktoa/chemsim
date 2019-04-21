extern crate num;

use std;
use arrayfire as af;
use arrayfire::HasAfEnum;

pub use self::num::Complex;
pub use num_traits::identities::One;

#[derive(Clone)]
pub struct Matrix {
    array: af::Array<f32>,
}

#[derive(Debug, Clone, Copy)]
pub enum Error {
    /// The slice given to `Matrix::new` had the wrong size.
    InvalidSliceSize,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Matrix {
    pub fn new(slice: &[f32], dims: (usize, usize)) -> Result<Self> {
        let (w, h) = dims;
        if slice.len() != w * h { Err(Error::InvalidSliceSize)?; }
        let dim4 = af::Dim4::new(&[w as u64, h as u64, 1, 1]);
        let arr = af::transpose(&af::Array::new(slice, dim4), false);
        Ok(Matrix::unsafe_new(arr))
    }

    pub fn unsafe_new(array: af::Array<f32>) -> Self {
        assert_eq!(f32::get_af_dtype(), array.get_type());
        let dims = array.dims();
        assert_eq!(dims[2], 1);
        assert_eq!(dims[3], 1);
        Matrix { array: array }
    }

    pub fn new_filled(value: f32, dims: (usize, usize)) -> Self {
        let (w, h) = dims;
        let mut vec: Vec<f32> = Vec::new();
        vec.resize(w * h, value);
        Matrix::new(&vec[..], dims).unwrap()
    }

    pub fn new_diag(diagonal: &[f32], offset: i32) -> Self {
        let vector = Matrix::new(diagonal, (diagonal.len(), 1)).unwrap();
        Matrix::unsafe_new(af::diag_create(&vector.array, offset))
    }

    pub fn new_identity(dims: (usize, usize)) -> Self {
        let (w, h) = dims;
        let dim4 = af::Dim4::new(&[h as u64, w as u64, 1, 1]);
        Matrix::unsafe_new(af::identity::<f32>(dim4))
    }

    pub fn new_random(dims: (usize, usize)) -> Self {
        let r_engine = af::RandomEngine::new(af::DEFAULT_RANDOM_ENGINE, None);
        let (w, h) = dims;
        let dim4 = af::Dim4::new(&[h as u64, w as u64, 1, 1]);
        Matrix::unsafe_new(af::random_normal::<f32>(dim4, &r_engine))
    }

    pub fn get_width(&self)  -> usize { self.array.dims()[1] as usize }
    pub fn get_height(&self) -> usize { self.array.dims()[0] as usize }

    pub fn get_shape(&self) -> (usize, usize) {
        let w = self.get_width();
        let h = self.get_height();
        (w, h)
    }

    pub fn get_array(&self) -> &af::Array<f32> { &self.array }

    pub fn get_array_mut(&mut self) -> &mut af::Array<f32> { &mut self.array }

    /// Transpose of a matrix.
    pub fn transpose(&self) -> Self {
        Matrix::unsafe_new(af::transpose(&self.array, false))
    }

    /// Conjugate transpose of a matrix.
    pub fn conjugate_transpose(&self) -> Self {
        Matrix::unsafe_new(af::transpose(&self.array, true))
    }

    /// In-place transpose of a matrix.
    pub fn transpose_in_place(&mut self) {
        af::transpose_inplace(&mut self.array, false);
    }

    /// In-place conjugate transpose of a matrix.
    pub fn conjugate_transpose_in_place(&mut self) {
        af::transpose_inplace(&mut self.array, true);
    }

    pub fn is_empty(&self)  -> bool { self.array.is_empty()  }
    pub fn is_scalar(&self) -> bool { self.array.is_scalar() }
    pub fn is_row(&self)    -> bool { self.array.is_vector() }
    pub fn is_col(&self)    -> bool { self.array.is_column() }

    pub fn from_scalar(&self) -> Option<f32> {
        if !self.array.is_scalar() { return None; }
        let mut vec = Vec::with_capacity(1);
        self.array.host(&mut vec[..]);
        Some(vec[0])
    }

    pub fn from_row(&self) -> Option<Vec<f32>> {
        if !self.array.is_vector() { return None; }
        let mut vec = Vec::with_capacity(self.get_width());
        self.array.host(&mut vec[..]);
        Some(vec)
    }

    pub fn from_col(&self) -> Option<Vec<f32>> {
        self.transpose().from_row()
    }

    pub fn get_underlying(&self) -> Vec<f32> {
        let mut vec = Vec::new();
        let num_elements = self.get_width() * self.get_height();
        unsafe { vec.resize(num_elements, std::mem::zeroed()); }
        self.transpose().array.host(&mut vec);
        vec
    }

    pub fn get_diagonal(&self, offset: i32) -> Vec<f32> {
        let diag = af::diag_extract(&self.array, offset);
        Matrix::unsafe_new(diag).from_row().unwrap()
    }

    pub fn recip(&self) -> Self {
        use num_traits::identities::one;
        Matrix::new_filled(one(), self.get_shape()).divide(self)
    }

    pub fn sum(&self) -> f64 {
        af::sum_all(&self.array).0
    }

    pub fn sum_complex(&self) -> Complex<f64> {
        let (real, imag) = af::sum_all(&self.array);
        Complex::new(real, imag)
    }

    pub fn sqrt(&self) -> Self {
        Matrix::unsafe_new(af::sqrt(&self.array))
    }

    pub fn maximum_complex(&self) -> Complex<f64> {
        let (re, im) = af::max_all(&self.array);
        Complex::new(re, im)
    }

    pub fn maximum_real(&self) -> f64 {
        self.maximum_complex().re
    }

    pub fn maximum_imag(&self) -> f64 {
        self.maximum_complex().im
    }

    // pub fn z_score(&self) -> Matrix {
    //     let avg = af::mean_all(self.get_array()).0;
    //     let std = af::stdev_all(self.get_array()).0;
    //     (self.cast::<f64>() - Matrix::new_filled(avg, self.get_shape()))
    //         .scale(1.0 / std)
    // }

    pub fn logistic(&self) -> Self {
        Matrix::unsafe_new(af::sigmoid(&self.array))
    }


    pub fn shift(&self, shifter: f32) -> Self {
        Matrix::unsafe_new(&self.array + shifter)
    }

    pub fn scale(&self, scalar: f32) -> Self {
        Matrix::unsafe_new(&self.array * scalar)
    }

    pub fn clamp(&self, min: f32, max: f32) -> Self {
        Matrix::unsafe_new(af::clamp(&self.array, &min, &max, true))
    }

    pub fn multiply(a: &Self, b: &Self) -> Self {
        assert_eq!(a.get_width(), b.get_height());
        Matrix::unsafe_new(af::matmul(&a.array, &b.array,
                                      af::MatProp::NONE, af::MatProp::NONE))
    }

    pub fn hadamard(&self, rhs: &Self) -> Self {
        assert_eq!(self.get_shape(), rhs.get_shape());
        Matrix::unsafe_new(af::mul(&self.array, &rhs.array, true))
    }

    pub fn divide(&self, rhs: &Self) -> Self {
        assert_eq!(self.get_shape(), rhs.get_shape());
        Matrix::unsafe_new(af::div(&self.array, &rhs.array, true))
    }
}

// -----------------------------------------------------------------------------

use std::ops::AddAssign;

impl AddAssign<Matrix> for Matrix {
    fn add_assign(&mut self, rhs: Matrix) {
        self.array += rhs.array;
    }
}

// -----------------------------------------------------------------------------

use std::ops::Add;

impl Add<Matrix> for Matrix {
    type Output = Matrix;
    fn add(self, rhs: Matrix) -> Matrix {
        Matrix::unsafe_new(self.array + rhs.array)
    }
}

impl<'a> Add<&'a Matrix> for Matrix {
    type Output = Matrix;
    fn add(self, rhs: &'a Matrix) -> Matrix {
        Matrix::unsafe_new(self.array + &rhs.array)
    }
}

impl<'a> Add<Matrix> for &'a Matrix {
    type Output = Matrix;
    fn add(self, rhs: Matrix) -> Matrix {
        Matrix::unsafe_new(&self.array + rhs.array)
    }
}

impl<'a, 'b> Add<&'a Matrix> for &'b Matrix {
    type Output = Matrix;
    fn add(self, rhs: &'a Matrix) -> Matrix {
        Matrix::unsafe_new(&self.array + &rhs.array)
    }
}

// -----------------------------------------------------------------------------

use std::ops::Sub;

impl Sub<Matrix> for Matrix {
    type Output = Matrix;
    fn sub(self, rhs: Matrix) -> Matrix {
        Matrix::unsafe_new(self.array - rhs.array)
    }
}

impl<'a> Sub<&'a Matrix> for Matrix {
    type Output = Matrix;
    fn sub(self, rhs: &'a Matrix) -> Matrix {
        Matrix::unsafe_new(self.array - &rhs.array)
    }
}

impl<'a> Sub<Matrix> for &'a Matrix {
    type Output = Matrix;
    fn sub(self, rhs: Matrix) -> Matrix {
        Matrix::unsafe_new(&self.array - rhs.array)
    }
}

impl<'a, 'b> Sub<&'a Matrix> for &'b Matrix {
    type Output = Matrix;
    fn sub(self, rhs: &'a Matrix) -> Matrix {
        Matrix::unsafe_new(&self.array - &rhs.array)
    }
}

// -----------------------------------------------------------------------------
