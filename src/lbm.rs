use std;
use super::matrix;

pub type Scalar = f32;

#[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
pub struct Vector(Scalar, Scalar);

impl Vector {
    #[inline(always)]
    pub fn to_complex(&self) -> matrix::Complex<Scalar> {
        matrix::Complex::new(self.0, self.1)
    }
}

impl std::ops::Add for Vector {
    type Output = Vector;
    fn add(self, rhs: Vector) -> Vector {
        Vector(self.0 + rhs.0, self.1 + rhs.1)
    }
}

pub type Matrix = matrix::Matrix<Scalar>;

pub struct Direction {
    w_scalar: Scalar,
    c_vector: Vector,
}

pub type Population = Matrix;

pub struct Lattice {
    size:        (usize, usize),
    populations: Vec<(Direction, Population)>,
}

impl Lattice {
    pub fn new_D2Q9(populations: &[Population; 9]) -> Self {
        let size = populations[0].get_shape();
        for pop in populations {
            assert_eq!(size, pop.get_shape());
        }
        
        let ws: Vec<Scalar> = vec![
            16.0 / 36.0,
            4.0  / 36.0,
            4.0  / 36.0,
            4.0  / 36.0,
            4.0  / 36.0,
            1.0  / 36.0,
            1.0  / 36.0,
            1.0  / 36.0,
            1.0  / 36.0,
        ];

        let cs: Vec<Vector> = vec![
            Vector( 0.0,  0.0),
            Vector( 1.0,  0.0),
            Vector( 0.0,  1.0),
            Vector(-1.0,  0.0),
            Vector( 0.0, -1.0),
            Vector( 1.0,  1.0),
            Vector(-1.0,  1.0),
            Vector(-1.0, -1.0),
            Vector( 1.0, -1.0),
        ];

        let mut vec = Vec::new();
        for ((w, c), pop) in ws.iter().zip(cs).zip(populations) {
            let dir = Direction { w_scalar: w.clone(), c_vector: c.clone() };
            vec.push((dir, pop.clone()))
        }

        Lattice { size: size, populations: vec }
    }
    
    pub fn density(&self) -> Matrix {
        let mut result = matrix::Matrix::new_filled(0.0, self.size);
        for (_, pop) in &self.populations { result += pop.clone(); }
        result
    }

    pub fn velocity(&self) -> Matrix {
        let density = self.density();
        let mut c_matrices: Vec<matrix::Matrix<matrix::Complex<Scalar>>>
            = Vec::with_capacity(self.populations.len());
        unimplemented!()
        // for (dir, _) in 
        // matrix::Matrix::new_filled(self.size)
    }
}

pub struct State {
    lattice: Lattice,
    delta_x: Scalar,
    delta_t: Scalar,
}
