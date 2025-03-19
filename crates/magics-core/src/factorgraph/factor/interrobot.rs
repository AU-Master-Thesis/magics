//! Inter-robot collision avoidance factor
//!
//! This module implements the inter-robot factor for collision avoidance
//! between multiple robots in the planning algorithm.

use std::{borrow::Cow, num::NonZeroUsize, ops::Sub};

use gbp_linalg::prelude::*;
use ndarray::s;
use typed_floats::StrictlyPositiveFinite;

use super::{Factor, FactorState, Measurement};
use crate::factorgraph::{
    factorgraph::{FactorGraphId, VariableIndex},
    DOFS,
};

/// Identifier for an external variable, i.e. a variable in another factorgraph
/// than the one this interrobot factor belongs to
#[derive(Debug, Clone, Copy)]
pub struct ExternalVariableId {
    /// The factorgraph id
    pub factorgraph_id: FactorGraphId,
    /// The variable index
    pub variable_index: VariableIndex,
}

impl ExternalVariableId {
    /// Create a new `ExternalVariableId`
    pub const fn new(factorgraph_id: FactorGraphId, variable_index: VariableIndex) -> Self {
        Self {
            factorgraph_id,
            variable_index,
        }
    }
}

/// Inter-robot factor: for avoidance of other robots
/// 
/// This factor results in a high energy or cost if two robots are planning to
/// be in the same position at the same timestep (collision). This factor is
/// created between variables of two robots. The factor has 0 energy if the
/// variables are further away than the safety distance.
#[derive(Debug, Clone)]
pub struct InterRobotFactor {
    /// Safety distance between robots
    safety_distance: Float,
    /// Radius of the robot
    robot_radius: Float,
    /// Whether to skip this factor
    skip: bool,
    /// External variable reference
    pub external_variable: ExternalVariableId,
    /// Small offset to avoid numerical issues
    tiny_offset: Float,
}

impl InterRobotFactor {
    /// Default safety distance multiplier
    pub const DEFAULT_SAFETY_DISTANCE_MULTIPLIER: Float = 2.2;
    /// Number of neighbors this factor connects to
    pub const NEIGHBORS: usize = 2;
    /// Scale factor for numerical stability
    pub const TINY_OFFSET_SCALE: f32 = 1e-6;

    /// Create a new inter-robot factor
    #[must_use]
    pub fn new(
        robot_radius: StrictlyPositiveFinite<Float>,
        external_variable: ExternalVariableId,
        safety_distance_multiplier: Option<StrictlyPositiveFinite<Float>>,
        robot_number: NonZeroUsize,
    ) -> Self {
        let robot_radius = robot_radius.get();
        let safety_distance_multiplier = safety_distance_multiplier
            .map_or(Self::DEFAULT_SAFETY_DISTANCE_MULTIPLIER, |x| x.get());
        let safety_distance = safety_distance_multiplier * robot_radius;

        Self {
            safety_distance,
            robot_radius,
            skip: false,
            external_variable,
            tiny_offset: Float::from(Self::TINY_OFFSET_SCALE) * robot_number.get() as f64,
        }
    }

    /// Get the safety distance
    #[inline(always)]
    pub const fn safety_distance(&self) -> Float {
        self.safety_distance
    }

    /// Update the safety distance
    /// The multiplier is multiplied by the robot radius
    pub fn update_safety_distance(&mut self, multiplier: StrictlyPositiveFinite<Float>) {
        self.safety_distance = multiplier.get() * self.robot_radius
    }

    /// Calculate the difference between the estimated positions of two robots
    fn diff_between_estimated_positions(
        &self,
        linearisation_point: &Vector<Float>,
    ) -> Vector<Float> {
        let offset = DOFS / 2;
        let mut diff_between_estimated_positions = linearisation_point
            .slice(s![..offset])
            .sub(&linearisation_point.slice(s![DOFS..DOFS + offset]));
        
        // Add a tiny random offset to avoid div/0 errors
        for i in 0..offset {
            diff_between_estimated_positions[i] += self.tiny_offset;
        }
        diff_between_estimated_positions
    }
}

impl Factor for InterRobotFactor {
    #[inline(always)]
    fn name(&self) -> &'static str {
        "InterRobotFactor"
    }

    #[inline]
    fn color(&self) -> [u8; 3] {
        // #a6da95
        [166, 218, 149]
    }

    fn jacobian(
        &self,
        state: &FactorState,
        lineraisation_point: &Vector<Float>,
    ) -> Cow<'_, Matrix<Float>> {
        let mut jacobian = Matrix::<Float>::zeros((state.initial_measurement.len(), DOFS * 2));
        let x_diff = self.diff_between_estimated_positions(lineraisation_point);

        let radius = x_diff.euclidean_norm();
        if radius <= self.safety_distance {
            // J(0, seqN(0, n_dofs_ / 2)) = -1.f / safety_distance_ / r * X_diff;
            jacobian
                .slice_mut(s![0, ..DOFS / 2])
                .assign(&(-1.0 / self.safety_distance / radius * &x_diff));

            // J(0, seqN(n_dofs_, n_dofs_ / 2)) = 1.f / safety_distance_ / r * X_diff;
            jacobian
                .slice_mut(s![0, DOFS..DOFS + (DOFS / 2)])
                .assign(&(1.0 / self.safety_distance / radius * &x_diff));
        }
        Cow::Owned(jacobian)
    }

    fn measure(&self, state: &FactorState, lineraisation_point: &Vector<Float>) -> Measurement {
        let mut measurement = Vector::<Float>::zeros(state.initial_measurement.len());
        let x_diff = self.diff_between_estimated_positions(lineraisation_point);

        let radius = x_diff.euclidean_norm();
        if radius <= self.safety_distance {
            if self.skip {
                // Log that we're within safety distance but only in debug builds
                #[cfg(debug_assertions)]
                println!(
                    "within safety distance, radius = {}, setting self.skip to false",
                    radius
                );
            }

            // gbpplanner: h(0) = 1.f*(1 - r/safety_distance_);
            measurement[0] = 1.0 * (1.0 - radius / self.safety_distance);
        }

        Measurement::new(measurement)
    }

    #[inline(always)]
    fn jacobian_delta(&self) -> Float {
        1e-2
    }

    /// Returns true if the distance between the two variables associated with
    /// this interrobot factor is greater than the safety distance
    fn skip(&self, state: &FactorState) -> bool {
        let offset = DOFS / 2;
        // [..offset] is the position of the first variable
        // [dofs..dofs + offset] is the position of the other variable
        let difference_between_estimated_positions = state
            .linearisation_point
            .slice(s![..offset])
            .sub(&state.linearisation_point.slice(s![DOFS..DOFS + offset]));
        let squared_distance = difference_between_estimated_positions
            .mapv(|x| x.powi(2))
            .sum();

        squared_distance >= self.safety_distance.powi(2)
    }

    #[inline(always)]
    fn linear(&self) -> bool {
        false
    }

    #[inline(always)]
    fn neighbours(&self) -> usize {
        Self::NEIGHBORS
    }
}

impl std::fmt::Display for InterRobotFactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "safety_distance: {}", self.safety_distance)
    }
}
