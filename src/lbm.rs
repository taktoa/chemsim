// -----------------------------------------------------------------------------

use std;
use super::matrix;

// -----------------------------------------------------------------------------

pub type Scalar = f32;

// -----------------------------------------------------------------------------

#[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
pub struct Vector(Scalar, Scalar);

impl Vector {
    #[inline(always)]
    pub fn to_complex(&self) -> matrix::Complex<Scalar> {
        matrix::Complex::new(self.0, self.1)
    }

    #[inline(always)]
    pub fn to_pair(&self) -> (Scalar, Scalar) {
        (self.0, self.1)
    }
}

impl std::ops::Add for Vector {
    type Output = Vector;
    fn add(self, rhs: Vector) -> Vector {
        Vector(self.0 + rhs.0, self.1 + rhs.1)
    }
}

// -----------------------------------------------------------------------------

pub type Matrix = matrix::Matrix<Scalar>;

// -----------------------------------------------------------------------------

#[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
pub struct Discretization {
    delta_x: Scalar,
    delta_t: Scalar,
}

impl Discretization {
    pub fn isothermal_speed_of_sound(&self) -> Scalar {
        self.delta_x / (Scalar::sqrt(3.0) * self.delta_t)
    }
}

// -----------------------------------------------------------------------------

#[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
pub struct Direction {
    w_scalar: Scalar,
    c_vector: Vector,
}

// -----------------------------------------------------------------------------

pub type Population = Matrix;

// -----------------------------------------------------------------------------

pub type Populations = Vec<(Direction, Population)>;

// -----------------------------------------------------------------------------

pub trait CollisionOperator {
    fn evaluate(&self, populations: &Populations) -> Vec<Matrix>;
    fn kinematic_shear_viscosity(&self, disc: &Discretization) -> Scalar;
    fn kinematic_bulk_viscosity(&self, disc: &Discretization) -> Scalar {
        2.0 * self.kinematic_shear_viscosity(disc) / 3.0
    }
}

// -----------------------------------------------------------------------------

pub struct BGK {
    tau: Scalar,
}

impl CollisionOperator for BGK {
    fn evaluate(&self, populations: &Populations) -> Vec<Matrix> {
        unimplemented!()
    }

    fn kinematic_shear_viscosity(&self, disc: &Discretization) -> Scalar {
        let (dx, dt) = (disc.delta_x, disc.delta_t);
        (dx * dx / (3.0 * dt * dt)) * (self.tau - dt / 2.0)
    }
}

// -----------------------------------------------------------------------------

pub struct Lattice {
    size:        (usize, usize),
    collision:   Box<CollisionOperator>,
    populations: Populations,
}

impl Lattice {
    pub fn new_D2Q9(
        populations: &[Population; 9],
        collision:   Box<CollisionOperator>,
    ) -> Self {
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

        Lattice {
            size:        size,
            populations: vec,
            collision:   collision,
        }
    }
}

// -----------------------------------------------------------------------------

pub struct State {
    lattice:        Lattice,
    discretization: Discretization,
}

impl State {
    pub fn stream(&mut self) {
        // for (dir, f_i) in &self.lattice.populations {
        // }
        unimplemented!()
    }

    pub fn collide(&mut self) {
        let omega = self.lattice.collision.evaluate(&self.lattice.populations);
        let mut result = Vec::with_capacity(self.lattice.populations.len());
        for (pair, omega_i) in self.lattice.populations.iter().zip(omega) {
            let (dir, f_i) = pair;
            result.push((dir.clone(), f_i + omega_i));
        }
        self.lattice.populations = result;
    }

    #[inline(always)]
    pub fn size(&self) -> (usize, usize) {
        self.lattice.size
    }

    #[inline(always)]
    pub fn delta_x(&self) -> Scalar {
        self.discretization.delta_x
    }

    #[inline(always)]
    pub fn delta_t(&self) -> Scalar {
        self.discretization.delta_t
    }

    #[inline(always)]
    pub fn populations(&self) -> &Populations {
        &self.lattice.populations
    }

    #[inline(always)]
    pub fn isothermal_speed_of_sound(&self) -> Scalar {
        self.discretization.isothermal_speed_of_sound()
    }

    pub fn density(&self) -> Matrix {
        let mut result = matrix::Matrix::new_filled(0.0, self.lattice.size);
        for (_, pop) in self.populations() { result += pop.clone(); }
        result
    }

    #[inline(always)]
    pub fn pressure(&self) -> Matrix {
        let cs = self.isothermal_speed_of_sound();
        self.density().scale(cs * cs)
    }

    pub fn velocity(&self) -> (Matrix, Matrix) {


        // let inverse_density = self.density();
        // let mut c_matrices: Vec<matrix::Matrix<matrix::Complex<Scalar>>>
        //     = Vec::with_capacity(self.populations.len());
        // for (dir, _) in self.populations() {
        //     let c = dir.c_vector.to_complex();
        //     c_matrices.push(matrix::Matrix::new_filled(c, self.size));
        // }
        // assert_eq!(c_matrices.len(), self.populations.len());
        unimplemented!()
    }

    pub fn equilibrium(&self) -> Populations {
        let density = self.density();
        let (vx, vy) = self.velocity();
        let v2 = vx.hadamard(&vx) + vy.hadamard(&vy);
        let cs = self.isothermal_speed_of_sound();
        let cs2 = cs * cs;
        let cs4 = cs2 * cs2;
        let mut result = Vec::with_capacity(self.populations().len());
        for (dir, _) in self.populations() {
            let (cx, cy) = dir.c_vector.to_pair();
            let vc = vx.scale(cx) + vy.scale(cy);
            let vc2 = vc.hadamard(&vc);
            let sum: Matrix
                = Matrix::new_filled(1.0, self.size())
                + vc.scale(1.0 / cs2)
                + vc2.scale(1.0 / (2.0 * cs4))
                + v2.scale(-1.0 / (2.0 * cs2));
            let pop = self.density().scale(dir.w_scalar).hadamard(&sum);
            result.push((dir.clone(), pop));
        }
        result
    }

    pub fn non_equilibrium(&self) -> Populations {
        let f = self.populations();
        let f_eq = self.equilibrium();
        let mut f_neq = Vec::with_capacity(f.len());
        for (pair, pair_eq) in f.iter().zip(f_eq) {
            let dir = pair.0;
            let pop = pair.1.clone();
            let pop_eq = pair_eq.1;
            f_neq.push((dir, pop - pop_eq));
        }
        f_neq
    }
}

// -----------------------------------------------------------------------------
