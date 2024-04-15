//!

use gbp_linalg::prelude::*;
use ndarray::{concatenate, Axis};

use super::{FactorState, IFactor};
use crate::factorgraph::DOFS;

/// Dynamic factor: constant velocity model
#[derive(Debug)]
pub struct DynamicFactor {
    cached_jacobian: Matrix<Float>,
}

impl DynamicFactor {
    pub const NEIGHBORS: usize = 2;

    #[must_use]
    pub fn new(state: &mut FactorState, delta_t: Float) -> Self {
        let eye = Matrix::<Float>::eye(DOFS / 2);
        let zeros = Matrix::<Float>::zeros((DOFS / 2, DOFS / 2));
        #[allow(clippy::similar_names)]
        let qc_inv = Float::powi(state.strength, -2) * &eye;

        #[allow(clippy::similar_names)]
        let qi_inv = concatenate![
            Axis(0),
            concatenate![
                Axis(1),
                12.0 * Float::powi(delta_t, -3) * &qc_inv,
                -6.0 * Float::powi(delta_t, -2) * &qc_inv
            ],
            concatenate![
                Axis(1),
                -6.0 * Float::powi(delta_t, -2) * &qc_inv,
                (4.0 / delta_t) * &qc_inv
            ]
        ];
        debug_assert_eq!(qi_inv.shape(), &[DOFS, DOFS]);

        state.measurement_precision = qi_inv;

        let cached_jacobian = concatenate![
            Axis(0),
            concatenate![Axis(1), eye, delta_t * &eye, -1.0 * &eye, zeros],
            concatenate![Axis(1), zeros, eye, zeros, -1.0 * &eye]
        ];
        debug_assert_eq!(cached_jacobian.shape(), &[DOFS, DOFS * 2]);

        Self { cached_jacobian }
    }
}

impl IFactor for DynamicFactor {
    #[inline]
    fn name(&self) -> &'static str {
        "DynamicFactor"
    }

    #[inline]
    fn jacobian(&mut self, _state: &FactorState, _x: &Vector<Float>) -> Matrix<Float> {
        self.cached_jacobian.clone()
    }

    #[inline(always)]
    fn measure(&mut self, _state: &FactorState, x: &Vector<Float>) -> Vector<Float> {
        self.cached_jacobian.dot(x)
    }

    #[inline(always)]
    fn skip(&mut self, _state: &FactorState) -> bool {
        false
    }

    #[inline(always)]
    fn jacobian_delta(&self) -> Float {
        1e-8
    }

    #[inline(always)]
    fn linear(&self) -> bool {
        true
    }
}
