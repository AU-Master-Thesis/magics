//! Pose factor

use std::borrow::Cow;

use gbp_linalg::prelude::*;

use super::{Factor, FactorState};

#[derive(Debug)]
pub struct PoseFactor;

impl PoseFactor {
    pub const NEIGHBORS: usize = 1;
}

impl Factor for PoseFactor {
    #[inline(always)]
    fn name(&self) -> &'static str {
        "PoseFactor"
    }

    #[inline(always)]
    fn jacobian_delta(&self) -> Float {
        1e-8
    }

    #[inline(always)]
    fn skip(&mut self, _state: &FactorState) -> bool {
        false
    }

    #[inline(always)]
    fn linear(&self) -> bool {
        false
    }

    /// Default jacobian is the first order taylor series jacobian
    #[inline]
    fn jacobian(&self, state: &FactorState, x: &Vector<Float>) -> Cow<'_, Matrix<Float>> {
        Cow::Owned(self.first_order_jacobian(state, x.clone()))
    }

    /// Default measurement function is the identity function
    #[inline(always)]
    fn measure(&self, _state: &FactorState, x: &Vector<Float>) -> Vector<Float> {
        x.clone()
    }

    #[inline(always)]
    fn neighbours(&self) -> usize {
        Self::NEIGHBORS
    }
}
