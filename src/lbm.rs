// -----------------------------------------------------------------------------

use std;
use arrayfire as af;
use super::matrix;

// -----------------------------------------------------------------------------

pub use super::matrix::Matrix;

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

    #[inline(always)]
    fn add(self, rhs: Vector) -> Vector {
        Vector(self.0 + rhs.0, self.1 + rhs.1)
    }
}

// -----------------------------------------------------------------------------

pub fn compute_equilibrium(
    density:        Matrix,
    velocity:       (Matrix, Matrix),
    directions:     &[Direction],
    discretization: Discretization,
) -> Populations {
    let size = density.get_shape();
    let (vx, vy) = velocity;
    assert_eq!(size, vx.get_shape());
    assert_eq!(size, vy.get_shape());
    let v2 = vx.hadamard(&vx) + vy.hadamard(&vy);
    let cs = discretization.isothermal_speed_of_sound();
    let cs2 = cs * cs;
    let cs4 = cs2 * cs2;
    let mut result = Vec::with_capacity(directions.len());
    for dir in directions {
        let (cx, cy) = dir.c_vector.to_pair();
        let vc = vx.scale(cx) + vy.scale(cy);
        let vc2 = vc.hadamard(&vc);
        let sum: Matrix
            = Matrix::new_filled(1.0, size)
            + vc.scale(1.0 / cs2)
            + vc2.scale(1.0 / (2.0 * cs4))
            + v2.scale(-1.0 / (2.0 * cs2));
        let pop = density.scale(dir.w_scalar).hadamard(&sum);
        result.push((dir.clone(), pop));
    }
    result
}

// -----------------------------------------------------------------------------

#[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
pub struct Discretization {
    pub delta_x: Scalar,
    pub delta_t: Scalar,
}

impl Discretization {
    #[inline(always)]
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

pub type Geometry = af::Array<bool>;

// -----------------------------------------------------------------------------

pub type Population = Matrix;

// -----------------------------------------------------------------------------

pub type Populations = Vec<(Direction, Population)>;

// -----------------------------------------------------------------------------

pub trait Lattice {
    fn size(&self) -> (usize, usize);
    fn populations(&self) -> &Populations;
    fn populations_mut(&mut self) -> &mut Populations;
    fn swap_populations(&self) -> Populations;

    fn density(&self) -> Matrix {
        let mut result = Matrix::new_filled(0.0, self.size());
        for (_, pop) in self.populations() { result += pop.clone(); }
        result
    }

    fn momentum_density(&self) -> (Matrix, Matrix) {
        let mut md_x = Matrix::new_filled(0.0, self.size());
        let mut md_y = Matrix::new_filled(0.0, self.size());
        for (dir, f_i) in self.populations() {
            md_x = md_x + f_i.scale(dir.c_vector.0);
            md_y = md_y + f_i.scale(dir.c_vector.1);
        }
        (md_x, md_y)
    }

    fn velocity(&self) -> (Matrix, Matrix) {
        let inverse_density = self.density().recip();
        let (md_x, md_y) = self.momentum_density();
        let v_x = inverse_density.hadamard(&md_x);
        let v_y = inverse_density.hadamard(&md_y);
        (v_x, v_y)
    }

    fn speed(&self) -> Matrix {
        let (v_x, v_y) = self.velocity();
        (v_x.hadamard(&v_x) + v_y.hadamard(&v_y)).sqrt()
    }

    fn equilibrium(&self, disc: &Discretization) -> Populations {
        let directions: Vec<Direction>
            = self.populations().iter().map(|(dir, _)| dir.clone()).collect();
        compute_equilibrium(self.density(), self.velocity(), &directions, *disc)
    }

    fn non_equilibrium(&self, disc: &Discretization) -> Populations {
        let f = self.populations();
        let f_eq = self.equilibrium(disc);
        let mut f_neq = Vec::with_capacity(f.len());
        for (pair, pair_eq) in f.iter().zip(f_eq) {
            let dir = pair.0.clone();
            let pop = pair.1.clone();
            let pop_eq = pair_eq.1;
            f_neq.push((dir, pop - pop_eq));
        }
        f_neq
    }

    fn swap_equilibrium(&self, disc: &Discretization) -> Populations;
}

// -----------------------------------------------------------------------------

#[derive(Clone)]
pub struct D2Q9 {
    size:        (usize, usize),
    populations: Populations,
}

impl D2Q9 {
    pub fn new(populations: &[Population]) -> Self {
        assert!(populations.len() == 9);

        let size = populations[0].get_shape();
        for pop in populations { assert_eq!(size, pop.get_shape()); }
        let directions = Self::directions();

        let mut vec = Vec::new();
        for (dir, pop) in directions.iter().zip(populations) {
            vec.push((dir.clone(), pop.clone()))
        }

        D2Q9 { size: size, populations: vec }
    }

    pub fn directions() -> [Direction; 9] {
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

        [
            (Direction { w_scalar: ws[0], c_vector: cs[0], stencil: ms[0].clone() }),
            (Direction { w_scalar: ws[1], c_vector: cs[1], stencil: ms[1].clone() }),
            (Direction { w_scalar: ws[2], c_vector: cs[2], stencil: ms[2].clone() }),
            (Direction { w_scalar: ws[3], c_vector: cs[3], stencil: ms[3].clone() }),
            (Direction { w_scalar: ws[4], c_vector: cs[4], stencil: ms[4].clone() }),
            (Direction { w_scalar: ws[5], c_vector: cs[5], stencil: ms[5].clone() }),
            (Direction { w_scalar: ws[6], c_vector: cs[6], stencil: ms[6].clone() }),
            (Direction { w_scalar: ws[7], c_vector: cs[7], stencil: ms[7].clone() }),
            (Direction { w_scalar: ws[8], c_vector: cs[8], stencil: ms[8].clone() }),
        ]
    }
}

impl Lattice for D2Q9 {
    fn size(&self) -> (usize, usize) {
        self.size.clone()
    }

    fn populations(&self) -> &Populations {
        &self.populations
    }

    fn populations_mut(&mut self) -> &mut Populations {
        &mut self.populations
    }

    fn swap_populations(&self) -> Populations {
        let mut new_pops = self.populations.clone();
        new_pops[1].1 = self.populations[3].1.clone();
        new_pops[2].1 = self.populations[4].1.clone();
        new_pops[3].1 = self.populations[1].1.clone();
        new_pops[4].1 = self.populations[2].1.clone();
        new_pops[5].1 = self.populations[7].1.clone();
        new_pops[6].1 = self.populations[8].1.clone();
        new_pops[7].1 = self.populations[5].1.clone();
        new_pops[8].1 = self.populations[6].1.clone();
        new_pops
    }

    fn swap_equilibrium(&self, disc: &Discretization) -> Populations {
        let mut new_pops = self.equilibrium(disc).clone();
        new_pops[1].1 = self.populations[3].1.clone();
        new_pops[2].1 = self.populations[4].1.clone();
        new_pops[3].1 = self.populations[1].1.clone();
        new_pops[4].1 = self.populations[2].1.clone();
        new_pops[5].1 = self.populations[7].1.clone();
        new_pops[6].1 = self.populations[8].1.clone();
        new_pops[7].1 = self.populations[5].1.clone();
        new_pops[8].1 = self.populations[6].1.clone();
        new_pops
    }
}

// -----------------------------------------------------------------------------

pub trait CollisionOperator {
    fn evaluate(
        &self,
        lattice:        &Lattice,
        equilibrium:    &Populations,
        discretization: &Discretization,
    ) -> Vec<Matrix>;

    fn kinematic_shear_viscosity(&self, disc: &Discretization) -> Scalar;

    #[inline(always)]
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
        lattice:        &Lattice,
        equilibrium:    &Populations,
        discretization: &Discretization,
    ) -> Vec<Matrix>
    {
        let factor = -discretization.delta_t / self.tau;
        let mut result = Vec::with_capacity(lattice.populations().len());
        for (pair, pair_eq) in lattice.populations().iter().zip(equilibrium) {
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

pub struct TRT {
    pub tau_minus: Scalar,
    pub tau_plus:  Scalar,
}

impl TRT {
    pub fn new(
        lambda:       Scalar,
        ks_viscosity: Scalar,
        disc:         &Discretization,
    ) -> Self {
        let (dx, dt) = (disc.delta_x, disc.delta_t);
        let cs = disc.isothermal_speed_of_sound();
        let tau_plus  = dt * ((ks_viscosity / (cs * cs)) + 0.5);
        let tau_minus = dt * ((lambda / ((tau_plus / dt) - 0.5)) + 0.5);
        TRT { tau_minus: tau_minus, tau_plus: tau_plus }
    }

    pub fn lambda(&self, disc: &Discretization) -> Scalar {
        let (dx, dt) = (disc.delta_x, disc.delta_t);
        let mut result = 1.0;
        result *= ((self.tau_plus  / dt) - 0.5);
        result *= ((self.tau_minus / dt) - 0.5);
        result
    }
}

impl CollisionOperator for TRT {
    fn evaluate(
        &self,
        lattice:        &Lattice,
        equilibrium:    &Populations,
        discretization: &Discretization,
    ) -> Vec<Matrix>
    {
        let f            = lattice.populations();
        let f_swapped    = lattice.swap_populations();
        let f_eq         = lattice.equilibrium(discretization);
        let f_eq_swapped = lattice.swap_equilibrium(discretization);

        let mut f_pm = Vec::new();
        for (pair, pair_swapped) in f.iter().zip(f_swapped) {
            let f_p = (&pair.1 + &pair_swapped.1).scale(0.5);
            let f_m = (&pair.1 - &pair_swapped.1).scale(0.5);
            f_pm.push((f_p, f_m));
        }

        let mut f_eq_pm = Vec::new();
        for (pair, pair_swapped) in f_eq.iter().zip(f_eq_swapped) {
            let f_eq_p = (&pair.1 + &pair_swapped.1).scale(0.5);
            let f_eq_m = (&pair.1 - &pair_swapped.1).scale(0.5);
            f_eq_pm.push((f_eq_p, f_eq_m));
        }

        let (dx, dt) = (discretization.delta_x, discretization.delta_t);
        let omega_m = 1.0 / self.tau_minus;
        let omega_p = 1.0 / self.tau_plus;

        let mut result = Vec::with_capacity(lattice.populations().len());
        for (f_pm_i, f_eq_pm_i) in f_pm.iter().zip(f_eq_pm) {
            let (f_p_i, f_m_i) = f_pm_i;
            let (f_eq_p_i, f_eq_m_i) = f_eq_pm_i;
            result.push((
                (f_p_i - f_eq_p_i).scale(omega_p)
                    + (f_m_i - f_eq_m_i).scale(omega_m)
            ).scale(-dt));
        }
        result
    }

    fn kinematic_shear_viscosity(&self, disc: &Discretization) -> Scalar {
        let (dx, dt) = (disc.delta_x, disc.delta_t);
        let cs = disc.isothermal_speed_of_sound();
        cs * cs * (self.tau_plus / dt - 0.5)
    }
}

// -----------------------------------------------------------------------------

pub struct State {
    pub time:           Scalar,
    pub lattice:        Box<Lattice>,
    pub geometry:       Geometry,
    pub collision:      Box<CollisionOperator>,
    pub discretization: Discretization,
}

impl State {
    pub fn initial(
        lattice:        Box<Lattice>,
        geometry:       Geometry,
        collision:      Box<CollisionOperator>,
        discretization: Discretization,
    ) -> Self {
        State {
            time:           0.0,
            lattice:        lattice,
            geometry:       geometry,
            collision:      collision,
            discretization: discretization,
        }
    }

    pub fn step(&mut self) {
        {
            let timer = std::time::Instant::now();
            self.stream();
            println!("> Streaming took {} ms", timer.elapsed().as_millis());
        }

        {
            let timer = std::time::Instant::now();
            self.bounce_back();
            println!("> Bounce-back took {} ms", timer.elapsed().as_millis());
        }

        {
            let timer = std::time::Instant::now();
            self.collide();
            println!("> Colliding took {} ms", timer.elapsed().as_millis());
        }

        self.time += self.discretization.delta_t;
    }

    pub fn stream(&mut self) {
        for pair in self.lattice.populations_mut() {
            let transposed = pair.0.stencil.transpose();
            let new_f_i = {
                let f_i = pair.1.get_array();
                let stencil = transposed.get_array();
                let arr = af::convolve2(&f_i, &stencil,
                                        af::ConvMode::DEFAULT,
                                        af::ConvDomain::AUTO);
                Matrix::unsafe_new(arr)
            };
            *(&mut pair.1) = new_f_i;
        }
    }

    pub fn collide(&mut self) {
        use std::borrow::Borrow;
        let omega = self.collision.evaluate(
            self.lattice.borrow(),
            &self.equilibrium(),
            &self.discretization,
        );
        // af::eval_multiple(omega.iter().map(|m| m.get_array()).collect());
        let mut result
            = Vec::with_capacity(self.lattice.populations().len());
        for (pair, omega_i) in self.lattice.populations().iter().zip(omega) {
            let (dir, f_i) = pair;
            result.push((dir.clone(), f_i + omega_i));
        }
        *(self.lattice.populations_mut()) = result;
    }

    pub fn bounce_back(&mut self) {
        let mut sw_pops = self.lattice.swap_populations();
        for (pair, mut sw_pair) in self.populations().iter().zip(&mut sw_pops) {
            let (dir, pop, sw_pop) = (pair.0.clone(), &pair.1, &mut sw_pair.1);
            af::replace(sw_pop.get_array_mut(),
                        &self.geometry,
                        pop.get_array());
        }
        *(self.lattice.populations_mut()) = sw_pops;
    }

    #[inline(always)]
    pub fn size(&self) -> (usize, usize) {
        self.lattice.size()
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
        self.lattice.populations()
    }

    #[inline(always)]
    pub fn isothermal_speed_of_sound(&self) -> Scalar {
        self.discretization.isothermal_speed_of_sound()
    }

    #[inline(always)]
    pub fn density(&self) -> Matrix {
        self.lattice.density()
    }

    #[inline(always)]
    pub fn pressure(&self) -> Matrix {
        let cs = self.isothermal_speed_of_sound();
        self.density().scale(cs * cs)
    }

    #[inline(always)]
    pub fn momentum_density(&self) -> (Matrix, Matrix) {
        self.lattice.momentum_density()
    }

    #[inline(always)]
    pub fn velocity(&self) -> (Matrix, Matrix) {
        self.lattice.velocity()
    }

    #[inline(always)]
    pub fn speed(&self) -> Matrix {
        self.lattice.speed()
    }

    // FIXME: viscous stress tensor is defined as
    //   σxx ≈ ((Δt / 2τ) - 1) · Σᵢ (c_{ix} c_{ix} fᵢ^neq)
    //   σxy ≈ ((Δt / 2τ) - 1) · Σᵢ (c_{ix} c_{iy} fᵢ^neq)
    //   σyx ≈ ((Δt / 2τ) - 1) · Σᵢ (c_{iy} c_{ix} fᵢ^neq)
    //   σyy ≈ ((Δt / 2τ) - 1) · Σᵢ (c_{iy} c_{iy} fᵢ^neq)

    #[inline(always)]
    pub fn equilibrium(&self) -> Populations {
        self.lattice.equilibrium(&self.discretization)
    }

    #[inline(always)]
    pub fn non_equilibrium(&self) -> Populations {
        self.lattice.non_equilibrium(&self.discretization)
    }

    #[inline(always)]
    pub fn is_unstable(&self) -> bool {
        let eq0 = &self.equilibrium()[0].1;
        af::imin_all(eq0.get_array()).0 < 0.0
    }
}

// -----------------------------------------------------------------------------
