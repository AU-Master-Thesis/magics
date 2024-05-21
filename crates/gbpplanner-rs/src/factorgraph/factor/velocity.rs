//! Experimental factor that models the ability of the robot
//! to vary its velocity by {ac,de}celerating to avoid obstacles
//! and not change its heading
//! The intention is to minimize the total distance travelled at the
//! cost of taking potentionally more time, as the robot will have to
//! slow down at times
use gbp_linalg::prelude::*;
use ndarray::{concatenate, Axis};

use super::{Factor, FactorState};
use crate::factorgraph::DOFS;

#[derive(Debug)]
pub struct VelocityFactor {
    cached_jacobian: Matrix<Float>,
}

impl VelocityFactor {
    /// Neighbour to variable n and n+1
    pub const NEIGHBORS: usize = 2;
}

impl Factor for VelocityFactor {
    #[inline]
    fn name(&self) -> &'static str {
        "VelocityFactor"
    }

    #[inline]
    fn color(&self) -> [u8; 3] {
        todo!()
    }

    #[inline]
    fn jacobian_delta(&self) -> Float {
        1e-8 // Same as DynamicFactor, for now
    }

    #[inline]
    fn neighbours(&self) -> usize {
        Self::NEIGHBORS
    }

    #[inline]
    fn skip(&self, _state: &FactorState) -> bool {
        false
    }

    #[inline]
    fn linear(&self) -> bool {
        true
    }

    fn measure(&self, _state: &FactorState, _x: &Vector<Float>) -> Vector<Float> {
        todo!()
    }

    fn jacobian(
        &self,
        _state: &FactorState,
        _x: &Vector<Float>,
    ) -> std::borrow::Cow<'_, Matrix<Float>> {
        todo!()
        // std::borrow::Cow::Owned(self.first_order_jacobian(state, x.clone()))
    }
}

impl std::fmt::Display for VelocityFactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
