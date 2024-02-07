use nalgebra::{DMatrix, DVector};
use std::rc::Rc;

#[derive(Debug, thiserror::Error)]
pub enum LossError {}

// Robust losses are implemented by scaling the Gaussian covariance
pub trait Loss: std::fmt::Debug {
    fn effective_covariance(&mut self, residual: Option<&DVector<f64>>) -> &DMatrix<f64>;
    fn covariance(&self) -> &DMatrix<f64>;
    fn robust(&self) -> bool;
}

/// Defines squared loss functions that correspond to Gaussians
#[derive(Debug)]
pub struct SquaredLoss {
    covariance: DMatrix<f64>,
    effective_covariance: DMatrix<f64>,
}

impl SquaredLoss {
    pub fn new(dofs: usize, diagonal_coveriance: DVector<f64>) -> Self {
        // TODO: use dofs
        let covariance = DMatrix::<f64>::from_diagonal(&diagonal_coveriance);
        let effective_covariance = covariance.to_owned();
        Self {
            covariance,
            effective_covariance,
        }
    }
}

impl Loss for SquaredLoss {
    fn effective_covariance(&mut self, _residual: Option<&DVector<f64>>) -> &DMatrix<f64> {
        self.effective_covariance = self.covariance.clone();
        &self.effective_covariance
    }

    fn covariance(&self) -> &DMatrix<f64> {
        &self.covariance
    }

    fn robust(&self) -> bool {
        // TODO: is this tuned right?
        let eps = -1e8;
        let max_relative = -1e8;
        self.covariance
            .relative_eq(&self.effective_covariance, eps, max_relative)
    }
}

#[derive(Debug)]
pub struct HuberLoss {
    covariance: DMatrix<f64>,
    effective_covariance: DMatrix<f64>,
    standard_deviation_transition: f64,
}

impl HuberLoss {
    pub fn new(
        // dofs: usize,
        diagonal_covariance: DVector<f64>,
        standard_deviation_transition: f64,
    ) -> Self {
        // TODO: change to result type and return error
        // assert_eq!(diagonal_covariance.len(), dofs);
        // let mut covariance = DMatrix::<f64>::zeros(dofs, dofs);
        // covariance
        //     .view_mut((0, 0), (dofs, dofs))
        //     .add_assign(DMatrix::<f64>::from_diagonal(&diagonal_covariance));
        let covariance = DMatrix::<f64>::from_diagonal(&diagonal_covariance);
        let effective_covariance = covariance.to_owned();
        Self {
            covariance,
            effective_covariance,
            standard_deviation_transition,
        }
    }
}

impl Loss for HuberLoss {
    fn effective_covariance(&mut self, residual: Option<&DVector<f64>>) -> &DMatrix<f64> {
        // TODO: do not like this
        let Some(residual) = residual else {
            panic!("residual has to be Some")
        };

        let mahalanobis_distance: f64 = (residual
            * (self.covariance.clone().try_inverse().unwrap() * residual))
            .index((0, 0))
            .sqrt();
        // .iter()
        // .map(f64::sqrt)
        // .collect();
        if mahalanobis_distance > self.standard_deviation_transition {
            self.effective_covariance = &self.covariance * f64::powi(mahalanobis_distance, 2)
                / (2.0 * self.standard_deviation_transition * mahalanobis_distance
                    - f64::powi(self.standard_deviation_transition, 2));
        }

        &self.effective_covariance
    }

    fn covariance(&self) -> &DMatrix<f64> {
        &self.covariance
    }

    fn robust(&self) -> bool {
        // TODO: is this tuned right?
        let eps = -1e8;
        let max_relative = -1e8;
        self.covariance
            .relative_eq(&self.effective_covariance, eps, max_relative)
    }
}

#[derive(Debug)]
pub struct TukeyLoss {
    covariance: DMatrix<f64>,
    effective_covariance: DMatrix<f64>,
    standard_deviation_transition: f64,
}

impl crate::factorgraph::measurement_model::TukeyLoss {
    pub fn new(
        // dofs: usize,
        diagonal_covariance: DVector<f64>,
        standard_deviation_transition: f64,
    ) -> Self {
        // TODO: use dofs
        let covariance = DMatrix::<f64>::from_diagonal(&diagonal_covariance);
        let effective_covariance = covariance.to_owned();
        Self {
            covariance,
            effective_covariance,
            standard_deviation_transition,
        }
    }
}

impl Loss for crate::factorgraph::measurement_model::TukeyLoss {
    fn effective_covariance(&mut self, residual: Option<&DVector<f64>>) -> &DMatrix<f64> {
        // TODO: do not like this
        let Some(residual) = residual else {
            panic!("residual has to be Some")
        };

        let mahalanobis_distance: f64 = (residual
            * (self.covariance.clone().try_inverse().unwrap() * residual))
            .index((0, 0))
            .sqrt();
        //
        // let mahalanobis_distance: f64 =
        //     (residual * self.covariance.try_inverse().unwrap() * residual)
        //         .iter()
        //         .map(f64::sqrt)
        //         .collect();
        if mahalanobis_distance > self.standard_deviation_transition {
            self.effective_covariance = &self.covariance * f64::powi(mahalanobis_distance, 2)
                / f64::powi(self.standard_deviation_transition, 2);
        }

        &self.effective_covariance
    }

    fn covariance(&self) -> &DMatrix<f64> {
        &self.covariance
    }

    fn robust(&self) -> bool {
        // TODO: is this tuned right?
        let eps = -1e8;
        let max_relative = -1e8;
        self.covariance
            .relative_eq(&self.effective_covariance, eps, max_relative)
    }
}

pub type Jacobian = dyn Fn(&DVector<f64>) -> DMatrix<f64>;
pub type Measurement = dyn Fn(&DVector<f64>) -> DMatrix<f64>;

// enum Loss {
//     Squared(SquaredLoss),
//     Huber(HuberLoss),
//     Tukey(TukeyLoss),
// }

// trait

pub enum MeasurementModelKind {
    Linear,
    NonLinear,
}

pub struct MeasurementModel<L: Loss> {
    pub loss: L,
    pub kind: MeasurementModelKind,
    jacobian: Rc<Jacobian>,
    measurement: Rc<Measurement>,
}

impl<L> MeasurementModel<L>
where
    L: Loss,
{
    pub fn new(
        loss: L,
        kind: MeasurementModelKind,
        jacobian: Rc<Jacobian>,
        measurement: Rc<Measurement>,
    ) -> Self {
        Self {
            loss,
            kind,
            jacobian,
            measurement,
        }
    }

    pub fn jacobian(&self, x: &DVector<f64>) -> DMatrix<f64> {
        (self.jacobian)(x)
    }

    pub fn measurement(&self, x: &DVector<f64>) -> DMatrix<f64> {
        (self.measurement)(x)
    }
}
