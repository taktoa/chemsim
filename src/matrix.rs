use std;
use arrayfire as af;
use std::marker::PhantomData;

pub use num_complex::Complex;

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
        Ok(Matrix::unsafe_new(af::Array::new(slice, dim4)))
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
        let mut vec: Vec<Element> = Vec::with_capacity(w * h);
        for el in vec.iter_mut() { *el = value; }
        Matrix::new(&vec[..], dims).unwrap()
    }

    pub fn get_width(&self)  -> usize { self.array.dims()[0] as usize }
    pub fn get_height(&self) -> usize { self.array.dims()[1] as usize }

    pub fn get_shape(&self) -> (usize, usize) {
        let w = self.get_width();
        let h = self.get_height();
        (w, h)
    }
    
    pub fn get_array(&self) -> &af::Array { &self.array }

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
        af::transpose_inplace(&mut self.array, false);
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

    pub fn multiply(a: &Self, b: &Self) -> Self {
        assert_eq!(a.get_width(), b.get_height());
        Matrix::unsafe_new(af::matmul(&a.array, &b.array,
                                      af::MatProp::NONE, af::MatProp::NONE))
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
