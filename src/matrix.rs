extern crate num;

use std;
use arrayfire as af;
use std::marker::PhantomData;

pub use self::num::Complex;
pub use num_traits::identities::One;

#[derive(Clone)]
pub struct Matrix<Element> {
    array:  af::Array,
    marker: PhantomData<Element>,
}

#[derive(Debug, Clone, Copy)]
pub enum Error {
    /// The slice given to `Matrix::new` had the wrong size.
    InvalidSliceSize,
}

pub type Result<T> = std::result::Result<T, Error>;

impl<Element: af::HasAfEnum + Copy> Matrix<Element> {
    pub fn new(slice: &[Element], dims: (usize, usize)) -> Result<Self> {
        let (w, h) = dims;
        if slice.len() != w * h { Err(Error::InvalidSliceSize)?; }
        let dim4 = af::Dim4::new(&[w as u64, h as u64, 1, 1]);
        let arr = af::transpose(&af::Array::new(slice, dim4), false);
        Ok(Matrix::unsafe_new(arr))
    }

    pub fn unsafe_new(array: af::Array) -> Self {
        assert_eq!(Element::get_af_dtype(), array.get_type());
        let dims = array.dims();
        assert_eq!(dims[2], 1);
        assert_eq!(dims[3], 1);
        Matrix { array: array, marker: PhantomData }
    }

    pub fn new_filled(value: Element, dims: (usize, usize)) -> Self {
        let (w, h) = dims;
        let mut vec: Vec<Element> = Vec::new();
        vec.resize(w * h, value);
        Matrix::new(&vec[..], dims).unwrap()
    }

    pub fn new_diag(diagonal: &[Element], offset: i32) -> Self {
        let vector = Matrix::new(diagonal, (diagonal.len(), 1)).unwrap();
        Matrix::unsafe_new(af::diag_create(&vector.array, offset))
    }

    pub fn new_identity(dims: (usize, usize)) -> Self {
        let (w, h) = dims;
        let dim4 = af::Dim4::new(&[h as u64, w as u64, 1, 1]);
        Matrix::unsafe_new(af::identity::<Element>(dim4))
    }

    pub fn new_random(dims: (usize, usize)) -> Self {
        let r_engine = af::RandomEngine::new(af::DEFAULT_RANDOM_ENGINE, None);
        let (w, h) = dims;
        let dim4 = af::Dim4::new(&[h as u64, w as u64, 1, 1]);
        Matrix::unsafe_new(af::random_normal::<Element>(dim4, r_engine))
    }

    pub fn get_width(&self)  -> usize { self.array.dims()[1] as usize }
    pub fn get_height(&self) -> usize { self.array.dims()[0] as usize }

    pub fn get_shape(&self) -> (usize, usize) {
        let w = self.get_width();
        let h = self.get_height();
        (w, h)
    }

    pub fn get_array(&self) -> &af::Array { &self.array }

    pub fn get_array_mut(&mut self) -> &mut af::Array { &mut self.array }

    pub fn cast<T: af::HasAfEnum + Copy>(&self) -> Matrix<T> {
        Matrix::unsafe_new(self.array.cast::<T>())
    }

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

    pub fn from_scalar(&self) -> Option<Element> {
        if !self.array.is_scalar() { return None; }
        let mut vec = Vec::with_capacity(1);
        self.array.host(&mut vec[..]);
        Some(vec[0])
    }

    pub fn from_row(&self) -> Option<Vec<Element>> {
        if !self.array.is_vector() { return None; }
        let mut vec = Vec::with_capacity(self.get_width());
        self.array.host(&mut vec[..]);
        Some(vec)
    }

    pub fn from_col(&self) -> Option<Vec<Element>> {
        self.transpose().from_row()
    }

    pub fn get_underlying(&self) -> Vec<Element> {
        let mut vec = Vec::new();
        let num_elements = self.get_width() * self.get_height();
        unsafe { vec.resize(num_elements, std::mem::zeroed()); }
        self.transpose().array.host(&mut vec);
        vec
    }

    pub fn get_diagonal(&self, offset: i32) -> Vec<Element> {
        let diag = af::diag_extract(&self.array, offset);
        Matrix::unsafe_new(diag).from_row().unwrap()
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

    pub fn conj(&self) -> Self {
        match self.array.get_type() {
            af::DType::C32 => Matrix::unsafe_new(af::conjg(&self.array)),
            af::DType::C64 => Matrix::unsafe_new(af::conjg(&self.array)),
            _              => self.clone(),
        }
    }

    pub fn abs(&self) -> Self {
        self.hadamard(&self.conj()).sqrt()
    }

    pub fn recip(&self) -> Self where Element: One {
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

    pub fn z_score(&self) -> Matrix<f64> {
        let avg = af::mean_all(self.get_array()).0;
        let std = af::stdev_all(self.get_array()).0;
        (self.cast::<f64>() - Matrix::new_filled(avg, self.get_shape()))
            .scale(1.0 / std)
    }

    pub fn logistic(&self) -> Self {
        Matrix::unsafe_new(af::sigmoid(&self.array))
    }
}

// -----------------------------------------------------------------------------

impl Matrix<f32> {
    pub fn shift(&self, shifter: f32) -> Self {
        Matrix::unsafe_new(&self.array + shifter)
    }

    pub fn scale(&self, scalar: f32) -> Self {
        Matrix::unsafe_new(&self.array * scalar)
    }

    pub fn clamp(&self, min: f32, max: f32) -> Self {
        Matrix::unsafe_new(af::clamp(&self.array, &min, &max, true))
    }
}

impl Matrix<f64> {
    pub fn shift(&self, shifter: f64) -> Self {
        Matrix::unsafe_new(&self.array + shifter)
    }

    pub fn scale(&self, scalar: f64) -> Self {
        Matrix::unsafe_new(&self.array * scalar)
    }

    pub fn clamp(&self, min: f64, max: f64) -> Self {
        Matrix::unsafe_new(af::clamp(&self.array, &min, &max, true))
    }
}

impl Matrix<Complex<f32>> {
    pub fn scale(&self, scalar: Complex<f32>) -> Self {
        Matrix::unsafe_new(&self.array * scalar)
    }
}

impl Matrix<Complex<f64>> {
    pub fn scale(&self, scalar: Complex<f64>) -> Self {
        Matrix::unsafe_new(&self.array * scalar)
    }
}

// -----------------------------------------------------------------------------

impl Matrix<Complex<f32>> {
    pub fn real(&self) -> Matrix<f32> {
        Matrix::unsafe_new(af::real(self.get_array()))
    }

    pub fn imag(&self) -> Matrix<f32> {
        Matrix::unsafe_new(af::imag(self.get_array()))
    }

    pub fn magnitude(&self) -> Matrix<f32> {
        self.hadamard(&self.conj()).real().sqrt()
    }

    /// Returned numbers are in `[-π, π]`
    pub fn phase(&self) -> Matrix<f32> {
        Matrix::unsafe_new(af::arg(self.get_array()))
    }
}

impl Matrix<Complex<f64>> {
    pub fn real(&self) -> Matrix<f64> {
        Matrix::unsafe_new(af::real(self.get_array()))
    }

    pub fn imag(&self) -> Matrix<f64> {
        Matrix::unsafe_new(af::imag(self.get_array()))
    }

    pub fn magnitude(&self) -> Matrix<f64> {
        self.hadamard(&self.conj()).real().sqrt()
    }

    /// Returned numbers are in `[-π, π]`
    pub fn phase(&self) -> Matrix<f64> {
        Matrix::unsafe_new(af::arg(self.get_array()))
    }
}

// -----------------------------------------------------------------------------

impl Matrix<f32> {
    pub fn dft(&self, norm_factor: f64) -> Matrix<Complex<f32>> {
        Matrix::unsafe_new(af::fft2(&self.array,
                                    norm_factor,
                                    self.get_height() as i64,
                                    self.get_width()  as i64))
    }

    pub fn inverse_dft(&self, norm_factor: f64) -> Matrix<Complex<f32>> {
        Matrix::unsafe_new(af::ifft2(&self.array,
                                     norm_factor,
                                     self.get_height() as i64,
                                     self.get_width()  as i64))
    }
}

impl Matrix<Complex<f32>> {
    pub fn dft(&self, norm_factor: f64) -> Matrix<Complex<f32>> {
        Matrix::unsafe_new(af::fft2(&self.array,
                                    norm_factor,
                                    self.get_height() as i64,
                                    self.get_width()  as i64))
    }

    pub fn inverse_dft(&self, norm_factor: f64) -> Matrix<Complex<f32>> {
        Matrix::unsafe_new(af::ifft2(&self.array,
                                     norm_factor,
                                     self.get_height() as i64,
                                     self.get_width()  as i64))
    }
}

impl Matrix<f64> {
    pub fn dft(&self, norm_factor: f64) -> Matrix<Complex<f64>> {
        Matrix::unsafe_new(af::fft2(&self.array,
                                    norm_factor,
                                    self.get_height() as i64,
                                    self.get_width()  as i64))
    }

    pub fn inverse_dft(&self, norm_factor: f64) -> Matrix<Complex<f64>> {
        Matrix::unsafe_new(af::ifft2(&self.array,
                                     norm_factor,
                                     self.get_height() as i64,
                                     self.get_width()  as i64))
    }
}

impl Matrix<Complex<f64>> {
    pub fn dft(&self, norm_factor: f64) -> Matrix<Complex<f64>> {
        Matrix::unsafe_new(af::fft2(&self.array,
                                    norm_factor,
                                    self.get_height() as i64,
                                    self.get_width()  as i64))
    }

    pub fn inverse_dft(&self, norm_factor: f64) -> Matrix<Complex<f64>> {
        Matrix::unsafe_new(af::ifft2(&self.array,
                                     norm_factor,
                                     self.get_height() as i64,
                                     self.get_width()  as i64))
    }
}

// -----------------------------------------------------------------------------

use std::ops::AddAssign;

impl<T: af::HasAfEnum + Copy> AddAssign<Matrix<T>> for Matrix<T> {
    fn add_assign(&mut self, rhs: Matrix<T>) {
        self.array += rhs.array;
    }
}

// -----------------------------------------------------------------------------

use std::ops::Add;

impl<T: af::HasAfEnum + Copy> Add<Matrix<T>> for Matrix<T> {
    type Output = Matrix<T>;
    fn add(self, rhs: Matrix<T>) -> Matrix<T> {
        Matrix::unsafe_new(self.array + rhs.array)
    }
}

impl<'a, T: af::HasAfEnum + Copy> Add<&'a Matrix<T>> for Matrix<T> {
    type Output = Matrix<T>;
    fn add(self, rhs: &'a Matrix<T>) -> Matrix<T> {
        Matrix::unsafe_new(self.array + &rhs.array)
    }
}

impl<'a, T: af::HasAfEnum + Copy> Add<Matrix<T>> for &'a Matrix<T> {
    type Output = Matrix<T>;
    fn add(self, rhs: Matrix<T>) -> Matrix<T> {
        Matrix::unsafe_new(&self.array + rhs.array)
    }
}

impl<'a, 'b, T: af::HasAfEnum + Copy> Add<&'a Matrix<T>> for &'b Matrix<T> {
    type Output = Matrix<T>;
    fn add(self, rhs: &'a Matrix<T>) -> Matrix<T> {
        Matrix::unsafe_new(&self.array + &rhs.array)
    }
}

// -----------------------------------------------------------------------------

use std::ops::Sub;

impl<T: af::HasAfEnum + Copy> Sub<Matrix<T>> for Matrix<T> {
    type Output = Matrix<T>;
    fn sub(self, rhs: Matrix<T>) -> Matrix<T> {
        Matrix::unsafe_new(self.array - rhs.array)
    }
}

impl<'a, T: af::HasAfEnum + Copy> Sub<&'a Matrix<T>> for Matrix<T> {
    type Output = Matrix<T>;
    fn sub(self, rhs: &'a Matrix<T>) -> Matrix<T> {
        Matrix::unsafe_new(self.array - &rhs.array)
    }
}

impl<'a, T: af::HasAfEnum + Copy> Sub<Matrix<T>> for &'a Matrix<T> {
    type Output = Matrix<T>;
    fn sub(self, rhs: Matrix<T>) -> Matrix<T> {
        Matrix::unsafe_new(&self.array - rhs.array)
    }
}

impl<'a, 'b, T: af::HasAfEnum + Copy> Sub<&'a Matrix<T>> for &'b Matrix<T> {
    type Output = Matrix<T>;
    fn sub(self, rhs: &'a Matrix<T>) -> Matrix<T> {
        Matrix::unsafe_new(&self.array - &rhs.array)
    }
}

// -----------------------------------------------------------------------------
