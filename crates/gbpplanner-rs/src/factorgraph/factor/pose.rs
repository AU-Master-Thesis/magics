//! Pose factor

use gbp_linalg::prelude::*;

use super::{FactorState, IFactor};

#[derive(Debug)]
pub struct PoseFactor;

impl IFactor for PoseFactor {
    #[inline(always)]
    fn name(&self) -> &'static str {
        "PoseFactor"
    }

    /// Default jacobian is the first order taylor series jacobian
    fn jacobian(&mut self, state: &FactorState, x: &Vector<Float>) -> Matrix<Float> {
        self.first_order_jacobian(state, x.clone())
    }

    /// Default measurement function is the identity function
    #[inline(always)]
    fn measure(&mut self, _state: &FactorState, x: &Vector<Float>) -> Vector<Float> {
        x.clone()
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
        false
    }
}
