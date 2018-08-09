// -----------------------------------------------------------------------------

extern crate dimensioned;

use std;
use arrayfire as af;
use super::matrix;
use super::lbm::{Scalar, Vector, Matrix};

// -----------------------------------------------------------------------------

#[derive(Clone)]
pub struct Validity2D { pub x_valid: bool, pub y_valid: bool }

impl Validity2D {
    pub fn new(x_valid: bool, y_valid: bool) -> Self {
        Validity2D { x_valid: x_valid, y_valid: y_valid }
    }
}

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
    velocity:         (Validity2D, VelocityField),
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
            velocity:         &self.velocity.1,
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

        {
            let validity = &mut self.velocity.0;
            if !validity.x_valid {
                self.velocity.1.x = (
                    &self.populations[1]
                        + &self.populations[5]
                        + &self.populations[8]
                        - &self.populations[3]
                        - &self.populations[6]
                        - &self.populations[7]
                ).hadamard(self.specific_volume.as_ref().unwrap());
                *(&mut validity.x_valid) = true;
            }
            if !validity.y_valid {
                // self.velocity.1.y = ...;
                *(&mut validity.y_valid) = true;
            }
        }

        assert!({
            let valid = self.velocity.0.clone();
            valid.x_valid && valid.y_valid
        });
    }

    fn compute_momentum_density(&mut self) {
        self.compute_density();
        self.compute_velocity();
        let rho = self.density.as_ref().unwrap();
        let v   = &self.velocity.1;
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
        self.compute_velocity();
        self.compute_momentum_density();
        self.compute_force();
        self.unsafe_view()
    }

    pub fn time_mut(&mut self) -> &mut Time {
        self.density = None;
        self.specific_volume = None;
        self.momentum_density = None;
        self.force = None;
        self.velocity.0.x_valid = false;
        self.velocity.0.y_valid = false;
        &mut self.time
    }

    pub fn population_0_mut(&mut self) -> &mut Population {
        self.density = None;
        self.momentum_density = None;
        &mut self.populations[0]
    }

    pub fn population_1_mut(&mut self) -> &mut Population {
        self.density = None;
        self.momentum_density = None;
        self.velocity.0.x_valid = false;
        &mut self.populations[1]
    }

    pub fn population_2_mut(&mut self) -> &mut Population {
        self.density = None;
        self.momentum_density = None;
        self.velocity.0.y_valid = false;
        &mut self.populations[2]
    }

    pub fn population_3_mut(&mut self) -> &mut Population {
        self.density = None;
        self.momentum_density = None;
        self.velocity.0.x_valid = false;
        &mut self.populations[3]
    }

    pub fn population_4_mut(&mut self) -> &mut Population {
        self.density = None;
        self.momentum_density = None;
        self.velocity.0.y_valid = false;
        &mut self.populations[4]
    }

    pub fn population_5_mut(&mut self) -> &mut Population {
        self.density = None;
        self.momentum_density = None;
        self.velocity.0.x_valid = false;
        self.velocity.0.y_valid = false;
        &mut self.populations[5]
    }

    pub fn population_6_mut(&mut self) -> &mut Population {
        self.density = None;
        self.momentum_density = None;
        self.velocity.0.x_valid = false;
        self.velocity.0.y_valid = false;
        &mut self.populations[6]
    }

    pub fn population_7_mut(&mut self) -> &mut Population {
        self.density = None;
        self.momentum_density = None;
        self.velocity.0.x_valid = false;
        self.velocity.0.y_valid = false;
        &mut self.populations[7]
    }

    pub fn population_8_mut(&mut self) -> &mut Population {
        self.density = None;
        self.momentum_density = None;
        self.velocity.0.x_valid = false;
        self.velocity.0.y_valid = false;
        &mut self.populations[8]
    }
}


// -----------------------------------------------------------------------------
