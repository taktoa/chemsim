// -----------------------------------------------------------------------------

use std;
use arrayfire;
use super::matrix;

// -----------------------------------------------------------------------------

pub type Scalar = f64;

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
    pub delta_x: Scalar,
    pub delta_t: Scalar,
}

impl Discretization {
    pub fn isothermal_speed_of_sound(&self) -> Scalar {
        self.delta_x / (Scalar::sqrt(3.0) * self.delta_t)
    }
}

// -----------------------------------------------------------------------------

#[derive(Clone)]
pub struct Direction {
    w_scalar: Scalar,
    c_vector: Vector,
    stencil:  Matrix,
}

// -----------------------------------------------------------------------------

pub type Population = Matrix;

// -----------------------------------------------------------------------------

pub type Populations = Vec<(Direction, Population)>;

// -----------------------------------------------------------------------------

pub trait CollisionOperator {
    fn evaluate(
        &self,
        populations:    &Populations,
        equilibrium:    &Populations,
        discretization: &Discretization,
    ) -> Vec<Matrix>;
    fn kinematic_shear_viscosity(&self, disc: &Discretization) -> Scalar;
    fn kinematic_bulk_viscosity(&self, disc: &Discretization) -> Scalar {
        2.0 * self.kinematic_shear_viscosity(disc) / 3.0
    }
}

// -----------------------------------------------------------------------------

pub struct BGK {
    pub tau: Scalar,
}

impl CollisionOperator for BGK {
    fn evaluate(
        &self,
        populations:    &Populations,
        equilibrium:    &Populations,
        discretization: &Discretization,
    ) -> Vec<Matrix>
    {
        let factor = -discretization.delta_t / self.tau;
        let mut result = Vec::with_capacity(populations.len());
        for (pair, pair_eq) in populations.iter().zip(equilibrium) {
            let (f_i, f_eq_i) = (pair.1.clone(), pair_eq.1.clone());
            result.push((f_i - f_eq_i).scale(factor));
        }
        result
    }

    fn kinematic_shear_viscosity(&self, disc: &Discretization) -> Scalar {
        let (dx, dt) = (disc.delta_x, disc.delta_t);
        (dx * dx / (3.0 * dt * dt)) * (self.tau - dt / 2.0)
    }
}

// -----------------------------------------------------------------------------

pub struct Lattice {
    size:        (usize, usize),
    populations: Populations,
}

impl Lattice {
    pub fn new_D2Q9(populations: &[Population; 9]) -> Self {
        let size = populations[0].get_shape();
        for pop in populations { assert_eq!(size, pop.get_shape()); }

        let make_m = |vec: &[i8; 9]| -> Matrix {
            let mut temp = Vec::new();
            for x in vec { temp.push(*x as Scalar); }
            Matrix::new(&temp, (3, 3)).unwrap()
        };

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

        let ms: Vec<Matrix> = vec![
            make_m(&[ 0, 0, 0,
                      0, 1, 0,
                      0, 0, 0, ]),

            make_m(&[ 0, 0, 0,
                      1, 0, 0,
                      0, 0, 0, ]),

            make_m(&[ 0, 0, 0,
                      0, 0, 0,
                      0, 1, 0, ]),

            make_m(&[ 0, 0, 0,
                      0, 0, 1,
                      0, 0, 0, ]),

            make_m(&[ 0, 1, 0,
                      0, 0, 0,
                      0, 0, 0, ]),

            make_m(&[ 0, 0, 0,
                      0, 0, 0,
                      1, 0, 0, ]),

            make_m(&[ 0, 0, 0,
                      0, 0, 0,
                      0, 0, 1, ]),

            make_m(&[ 0, 0, 1,
                      0, 0, 0,
                      0, 0, 0, ]),

            make_m(&[ 1, 0, 0,
                      0, 0, 0,
                      0, 0, 0, ]),
        ];

        let mut vec = Vec::new();
        for (((w, c), m), pop) in ws.iter().zip(cs).zip(ms).zip(populations) {
            let dir = Direction {
                w_scalar: w.clone(),
                c_vector: c.clone(),
                stencil:  m.clone(),
            };
            vec.push((dir, pop.clone()))
        }

        Lattice { size: size, populations: vec }
    }
}

// -----------------------------------------------------------------------------

pub struct State {
    pub time:           Scalar,
    pub lattice:        Lattice,
    pub discretization: Discretization,
    pub collision:      Box<CollisionOperator>,
}

impl State {
    pub fn initial(
        lattice:        Lattice,
        discretization: Discretization,
        collision:      Box<CollisionOperator>,
    ) -> Self {
        State {
            time:           0.0,
            lattice:        lattice,
            discretization: discretization,
            collision:      collision,
        }
    }

    pub fn step(&mut self) {
        {
            let timer = std::time::Instant::now();
            self.collide();
            println!("> Colliding took {} ms", timer.elapsed().as_millis());
        }

        {
            let timer = std::time::Instant::now();
            self.stream();
            println!("> Streaming took {} ms", timer.elapsed().as_millis());
        }

        self.time += self.discretization.delta_t;
    }

    pub fn stream(&mut self) {
        use arrayfire as af;
        for pair in &mut self.lattice.populations {
            let dir = &pair.0;
            let transposed = dir.stencil.transpose();
            let new_f_i = {
                let f_i = pair.1.get_array();
                let stencil = transposed.get_array();
                // let arr = af::fft_convolve2(&f_i, &stencil,
                //                             af::ConvMode::DEFAULT);
                let arr = af::convolve2(&f_i, &stencil,
                                        af::ConvMode::DEFAULT,
                                        af::ConvDomain::FREQUENCY);
                arrayfire::eval_multiple(vec![&arr]);
                Matrix::unsafe_new(arr)
            };
            *(&mut pair.1) = new_f_i;
        }
    }

    pub fn collide(&mut self) {
        let omega = self.collision.evaluate(
            &self.lattice.populations,
            &self.equilibrium(),
            &self.discretization,
        );
        arrayfire::eval_multiple(omega.iter().map(|m| m.get_array()).collect());
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
        let mut result = Matrix::new_filled(0.0, self.lattice.size);
        for (_, pop) in self.populations() { result += pop.clone(); }
        result
    }

    #[inline(always)]
    pub fn pressure(&self) -> Matrix {
        let cs = self.isothermal_speed_of_sound();
        self.density().scale(cs * cs)
    }

    pub fn momentum_density(&self) -> (Matrix, Matrix) {
        let mut md_x = Matrix::new_filled(0.0, self.lattice.size);
        let mut md_y = Matrix::new_filled(0.0, self.lattice.size);
        for (dir, f_i) in self.populations() {
            md_x = md_x + f_i.scale(dir.c_vector.0);
            md_y = md_y + f_i.scale(dir.c_vector.1);
        }
        (md_x, md_y)
    }

    pub fn velocity(&self) -> (Matrix, Matrix) {
        let inverse_density = self.density().recip();
        let (md_x, md_y) = self.momentum_density();
        let v_x = inverse_density.hadamard(&md_x);
        let v_y = inverse_density.hadamard(&md_y);
        (v_x, v_y)
    }

    pub fn speed(&self) -> Matrix {
        let (v_x, v_y) = self.velocity();
        (v_x.hadamard(&v_x) + v_y.hadamard(&v_y)).sqrt()
    }

    // FIXME: viscous stress tensor is defined as
    //   σxx ≈ ((Δt / 2τ) - 1) · Σᵢ (c_{ix} c_{ix} fᵢ^neq)
    //   σxy ≈ ((Δt / 2τ) - 1) · Σᵢ (c_{ix} c_{iy} fᵢ^neq)
    //   σyx ≈ ((Δt / 2τ) - 1) · Σᵢ (c_{iy} c_{ix} fᵢ^neq)
    //   σyy ≈ ((Δt / 2τ) - 1) · Σᵢ (c_{iy} c_{iy} fᵢ^neq)

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
            let dir = pair.0.clone();
            let pop = pair.1.clone();
            let pop_eq = pair_eq.1;
            f_neq.push((dir, pop - pop_eq));
        }
        f_neq
    }

    pub fn is_unstable(&self) -> bool {
        for (_, f_eq_i) in &self.equilibrium() {
            if arrayfire::imin_all(f_eq_i.get_array()).0 < 0.0 { return true; }
        }
        // for (_, f_i) in self.populations() {
        //     if arrayfire::imin_all(f_i.get_array()).0 < 0.0 { return true; }
        // }
        false
    }
}

// -----------------------------------------------------------------------------
