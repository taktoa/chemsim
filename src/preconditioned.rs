// -----------------------------------------------------------------------------

extern crate dimensioned;

use std;
use arrayfire as af;
use super::matrix;
use super::lbm::{Scalar, Vector, Matrix};

// -----------------------------------------------------------------------------

// This value is T
const TRANSFORMATION_MATRIX: [[i8; 9]; 9] = [
    [ 1,  1,  1,  1,  1,  1,  1,  1,  1],
    [-4, -1, -1, -1, -1,  2,  2,  2,  2],
    [ 4, -2, -2, -2, -2,  1,  1,  1,  1],
    [ 0,  1,  0, -1,  0,  1, -1, -1,  1],
    [ 0, -2,  0,  2,  0,  1, -1, -1,  1],
    [ 0,  0,  1,  0, -1,  1,  1, -1, -1],
    [ 0,  0, -2,  0,  2,  1,  1, -1, -1],
    [ 0,  1, -1,  1, -1,  0,  0,  0,  0],
    [ 0,  0,  0,  0,  0,  1, -1,  1, -1],
];

// This value is 36 * T^-1
const INVERSE_TRANSFORMATION_MATRIX: [[i8; 9]; 9] = [
    [ 4, -4,  4,  0,  0,  0,  0,  0,  0],
    [ 4, -1, -2,  6, -6,  0,  0,  9,  0],
    [ 4, -1, -2,  0,  0,  6, -6, -9,  0],
    [ 4, -1, -2, -6,  6,  0,  0,  9,  0],
    [ 4, -1, -2,  0,  0, -6,  6, -9,  0],
    [ 4,  2,  1,  6,  3,  6,  3,  0,  9],
    [ 4,  2,  1, -6, -3,  6,  3,  0, -9],
    [ 4,  2,  1, -6, -3, -6, -3,  0,  9],
    [ 4,  2,  1,  6,  3, -6, -3,  0, -9],
];

// -----------------------------------------------------------------------------

pub type ScalarField = Matrix;

// -----------------------------------------------------------------------------

#[derive(Clone)]
pub struct VectorField { pub x: ScalarField, pub y: ScalarField }

impl VectorField {
    pub fn scale(&self, scalar: Scalar) -> Self {
        VectorField { x: self.x.scale(scalar), y: self.y.scale(scalar) }
    }

    pub fn scale_pointwise(&self, scalar_field: &ScalarField) -> Self {
        VectorField {
            x: self.x.hadamard(scalar_field),
            y: self.y.hadamard(scalar_field),
        }
    }

    pub fn magnitude(&self) -> ScalarField {
        let x2 = self.x.hadamard(&self.x);
        let y2 = self.y.hadamard(&self.y);
        x2 + y2
    }

    pub fn direction(&self) -> ScalarField {
        Matrix::unsafe_new(af::atan2(self.y.get_array(),
                                     self.x.get_array(),
                                     true))
    }
}

// -----------------------------------------------------------------------------

pub type Time                 = Scalar;
pub type RelaxationTime       = Scalar;
pub type PreconditionFactor   = Scalar;
pub type Viscosity            = Scalar;
pub type Population           = ScalarField;
pub type DensityField         = ScalarField;
pub type SpecificVolumeField  = ScalarField;
pub type SpeedField           = ScalarField;
pub type VelocityField        = VectorField;
pub type MomentumDensityField = VectorField;
pub type ForceField           = VectorField;

// -----------------------------------------------------------------------------

#[derive(Clone)]
pub struct StateView<'a> {
    pub time:             Time,
    pub populations:      &'a [Population; 9],
    pub density:          &'a DensityField,
    pub specific_volume:  &'a SpecificVolumeField,
    pub velocity:         &'a VelocityField,
    pub momentum_density: &'a MomentumDensityField,
    pub force:            &'a Option<ForceField>,
}

// -----------------------------------------------------------------------------

pub struct Parameters {
    relaxation_times:    [RelaxationTime; 3],
    precondition_factor: PreconditionFactor,
    bulk_viscosity:      Viscosity,
    shear_viscosity:     Viscosity,
    update_force:        Box<for<'a> Fn(StateView<'a>) -> ForceField>,
}

// -----------------------------------------------------------------------------

pub struct State {
    time:             Time,
    parameters:       Parameters,
    populations:      [Population; 9],
    density:          Option<DensityField>,
    specific_volume:  Option<SpecificVolumeField>,
    velocity:         Option<VelocityField>,
    momentum_density: Option<MomentumDensityField>,
    force:            Option<ForceField>,
}

impl State {
    fn unsafe_view<'a>(&'a self) -> StateView<'a> {
        StateView {
            time:             self.time,
            populations:      &self.populations,
            density:          self.density.as_ref().unwrap(),
            specific_volume:  self.specific_volume.as_ref().unwrap(),
            velocity:         self.velocity.as_ref().unwrap(),
            momentum_density: self.momentum_density.as_ref().unwrap(),
            force:            &self.force,
        }
    }

    fn compute_density(&mut self) {
        if self.density.is_none() {
            self.density = Some(&self.populations[0]
                                + &self.populations[1]
                                + &self.populations[2]
                                + &self.populations[3]
                                + &self.populations[4]
                                + &self.populations[5]
                                + &self.populations[6]
                                + &self.populations[7]
                                + &self.populations[8]);
        }
        assert!(self.density.is_some());
    }

    fn compute_specific_volume(&mut self) {
        self.compute_density();
        if self.specific_volume.is_none() {
            self.specific_volume = Some(
                self.density.as_ref().unwrap().recip());
        }
    }

    fn compute_velocity(&mut self) {
        self.compute_specific_volume();

        if self.velocity.is_none() {
            af::eval_multiple(vec![
                &self.populations[1].get_array(),
                &self.populations[2].get_array(),
                &self.populations[3].get_array(),
                &self.populations[4].get_array(),
                &self.populations[5].get_array(),
                &self.populations[6].get_array(),
                &self.populations[7].get_array(),
                &self.populations[8].get_array(),
            ]);

            let five_minus_seven = &self.populations[5] - &self.populations[7];
            let six_minus_eight  = &self.populations[6] - &self.populations[8];
            af::eval_multiple(vec![
                five_minus_seven.get_array(),
                six_minus_eight.get_array(),
            ]);

            let vx = (
                &five_minus_seven
                    - &six_minus_eight
                    + &self.populations[1]
                    - &self.populations[3]
            ).hadamard(self.specific_volume.as_ref().unwrap());

            let vy = (
                &five_minus_seven
                    + &six_minus_eight
                    + &self.populations[2]
                    - &self.populations[4]
            ).hadamard(self.specific_volume.as_ref().unwrap());

            af::eval_multiple(vec![ vx.get_array(), vy.get_array() ]);

            self.velocity = Some(VectorField { x: vx, y: vy });
        }
    }

    fn compute_momentum_density(&mut self) {
        self.compute_density();
        self.compute_velocity();
        let rho = self.density.as_ref().unwrap();
        let v   = self.velocity.as_ref().unwrap();
        if self.momentum_density.is_none() {
            self.momentum_density = Some(v.scale_pointwise(rho));
        }
    }

    fn compute_force(&mut self) {
        self.compute_density();
        self.compute_velocity();
        self.compute_momentum_density();
        if self.force.is_none() {
            let force = {
                let view = self.unsafe_view();
                (self.parameters.update_force)(view)
            };
            self.force = Some(force);
        }
    }

    pub fn view<'a>(&'a mut self) -> StateView<'a> {
        self.compute_density();
        self.compute_specific_volume();
        self.compute_velocity();
        self.compute_momentum_density();
        self.compute_force();
        self.unsafe_view()
    }

    pub fn time_mut(&mut self) -> &mut Time {
        self.density          = None;
        self.specific_volume  = None;
        self.momentum_density = None;
        self.force            = None;
        self.velocity         = None;
        &mut self.time
    }

    pub fn population_0_mut(&mut self) -> &mut Population {
        self.density          = None;
        self.momentum_density = None;
        &mut self.populations[0]
    }

    pub fn population_1_mut(&mut self) -> &mut Population {
        self.density          = None;
        self.momentum_density = None;
        self.velocity         = None;
        &mut self.populations[1]
    }

    pub fn population_2_mut(&mut self) -> &mut Population {
        self.density          = None;
        self.momentum_density = None;
        self.velocity         = None;
        &mut self.populations[2]
    }

    pub fn population_3_mut(&mut self) -> &mut Population {
        self.density          = None;
        self.momentum_density = None;
        self.velocity         = None;
        &mut self.populations[3]
    }

    pub fn population_4_mut(&mut self) -> &mut Population {
        self.density          = None;
        self.momentum_density = None;
        self.velocity         = None;
        &mut self.populations[4]
    }

    pub fn population_5_mut(&mut self) -> &mut Population {
        self.density          = None;
        self.momentum_density = None;
        self.velocity         = None;
        &mut self.populations[5]
    }

    pub fn population_6_mut(&mut self) -> &mut Population {
        self.density          = None;
        self.momentum_density = None;
        self.velocity         = None;
        &mut self.populations[6]
    }

    pub fn population_7_mut(&mut self) -> &mut Population {
        self.density          = None;
        self.momentum_density = None;
        self.velocity         = None;
        &mut self.populations[7]
    }

    pub fn population_8_mut(&mut self) -> &mut Population {
        self.density          = None;
        self.momentum_density = None;
        self.velocity         = None;
        &mut self.populations[8]
    }

    // pub fn get_density(&mut self) -> &
}


// -----------------------------------------------------------------------------
