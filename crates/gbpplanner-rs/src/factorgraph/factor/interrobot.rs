use std::{borrow::Cow, ops::Sub};

use bevy::log::info;
use gbp_linalg::prelude::*;
use ndarray::s;
use typed_floats::StrictlyPositiveFinite;

use super::{Factor, FactorState};
use crate::factorgraph::{
    factorgraph::{FactorGraphId, VariableIndex},
    DOFS,
};

/// Identifier for a external variable, i.e. a variable in another factorgraph
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

/// Interrobot factor: for avoidance of other robots
/// This factor results in a high energy or cost if two robots are planning to
/// be in the same position at the same timestep (collision). This factor is
/// created between variables of two robots. The factor has 0 energy if the
/// variables are further away than the safety distance.
#[derive(Debug, Clone)]
pub struct InterRobotFactor {
    safety_distance: Float,
    robot_radius: Float,
    skip: bool,
    pub external_variable: ExternalVariableId,
}

impl InterRobotFactor {
    pub const DEFAULT_SAFETY_DISTANCE_MULTIPLIER: Float = 2.2;
    pub const NEIGHBORS: usize = 2;

    #[must_use]
    pub fn new(
        robot_radius: StrictlyPositiveFinite<Float>,
        external_variable: ExternalVariableId,
        safety_distance_multiplier: Option<StrictlyPositiveFinite<Float>>,
    ) -> Self {
        let robot_radius = robot_radius.get();
        // let epsilon = 0.2 * robot_radius;
        //
        // Self {
        //     safety_distance: 2.0f64.mul_add(robot_radius, epsilon),
        //     skip: false,
        //     external_variable,
        // }

        let safety_distance_multiplier = safety_distance_multiplier
            .map_or(Self::DEFAULT_SAFETY_DISTANCE_MULTIPLIER, |x| x.get());
        // .unwrap_or(2.2.try_into().expect("2.2 > 0.0"))
        // .get();
        let safety_distance = safety_distance_multiplier * robot_radius;

        Self {
            safety_distance,
            robot_radius,
            skip: false,
            external_variable,
        }
    }

    /// Get the safety distance
    #[inline(always)]
    pub const fn safety_distance(&self) -> Float {
        self.safety_distance
    }

    pub fn update_safety_distance(&mut self, multiplier: StrictlyPositiveFinite<Float>) {
        self.safety_distance = multiplier.get() * self.robot_radius
    }
}

impl Factor for InterRobotFactor {
    #[inline(always)]
    fn name(&self) -> &'static str {
        "InterRobotFactor"
    }

    fn jacobian(&self, state: &FactorState, x: &Vector<Float>) -> Cow<'_, Matrix<Float>> {
        // PERF: reuse allocation by
        let mut jacobian = Matrix::<Float>::zeros((state.initial_measurement.len(), DOFS * 2));

        let x_diff = {
            let offset = DOFS / 2;
            let mut x_diff = x.slice(s![..offset]).sub(&x.slice(s![DOFS..DOFS + offset]));

            // NOTE: In gbplanner, they weight this by the robot id, why they do this is
            // unclear as a robot id should be unique, and not have any
            // semantics of distance/weight.
            for i in 0..offset {
                // Add a tiny random offset to avoid div/0 errors
                x_diff[i] += 1e-6 * Float::from(self.external_variable.factorgraph_id.index());
            }
            x_diff
        };

        let radius = x_diff.euclidean_norm();
        if radius <= self.safety_distance {
            // TODO: why do we change the Jacobian if we are not outside the safety
            // distance?

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

    fn measure(&self, state: &FactorState, x: &Vector<Float>) -> Vector<Float> {
        let mut h = Vector::<Float>::zeros(state.initial_measurement.len());
        let x_diff = {
            let offset = DOFS / 2;
            let mut x_diff = x.slice(s![..offset]).sub(&x.slice(s![DOFS..DOFS + offset]));
            // NOTE: In gbplanner, they weight this by the robot id, why they do this is
            // unclear as a robot id should be unique, and not have any
            // semantics of distance/weight.
            for i in 0..offset {
                // Add a tiny random offset to avoid div/0 errors
                x_diff[i] += 1e-6 * Float::from(self.external_variable.factorgraph_id.index());
            }
            x_diff
        };

        let radius = x_diff.euclidean_norm();
        if radius <= self.safety_distance {
            if self.skip {
                info!(
                    "within safety distance, radius = {}, setting self.skip to false",
                    radius
                );
            }

            // gbpplanner: h(0) = 1.f*(1 - r/safety_distance_);
            // NOTE: in Eigen, indexing a matrix with a single index corresponds to indexing
            // the matrix as a flattened array in column-major order.
            // h[(0, 0)] = 1.0 * (1.0 - radius / self.safety_distance);
            h[0] = 1.0 * (1.0 - radius / self.safety_distance);
        }

        h
    }

    #[inline(always)]
    fn jacobian_delta(&self) -> Float {
        1e-2
    }

    fn skip(&self, state: &FactorState) -> bool {
        let offset = DOFS / 2;
        // [..offset] is the position of the first variable
        // [dofs..dofs + offset] is the position of the other variable
        let difference_between_estimated_positions = state
            .linearisation_point
            .slice(s![..offset])
            .sub(&state.linearisation_point.slice(s![DOFS..DOFS + offset]));
        let squared_norm = difference_between_estimated_positions
            .mapv(|x| x.powi(2))
            .sum();

        let skip = squared_norm >= self.safety_distance.powi(2);
        skip
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
