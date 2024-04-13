use bevy::log::info;
use gbp_linalg::prelude::*;
use ndarray::s;
use typed_floats::StrictlyPositiveFinite;

use super::{FactorState, IFactor};
use crate::factorgraph::{
    factorgraph::NodeIndex,
    id::{FactorId, VariableId},
    DOFS,
};

// TODO: add to and from
#[derive(Debug, Clone, Copy)]
pub struct InterRobotFactorConnection {
    pub interrobot:     FactorId,
    pub other_variable: VariableId,
    // pub id_of_robot_connected_with: RobotId,
    // pub index_of_connected_variable_in_other_robots_factorgraph: NodeIndex,
}

impl InterRobotFactorConnection {
    #[must_use]
    pub fn new(
        interrobot: FactorId,
        other_variable: VariableId,
        // id_of_robot_connected_with: RobotId,
        // index_of_connected_variable_in_other_robots_factorgraph: NodeIndex,
    ) -> Self {
        Self {
            interrobot,
            other_variable,
            // id_of_robot_connected_with,
            // index_of_connected_variable_in_other_robots_factorgraph,
        }
    }
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub struct Skip(bool);

/// Interrobot factor: for avoidance of other robots
/// This factor results in a high energy or cost if two robots are planning to
/// be in the same position at the same timestep (collision). This factor is
/// created between variables of two robots. The factor has 0 energy if the
/// variables are further away than the safety distance.
#[derive(Debug, Clone)]
pub struct InterRobotFactor {
    safety_distance: Float,
    skip: bool,
    pub connection: InterRobotFactorConnection,
}

// #[derive(Debug, thiserror::Error)]
// pub enum InterRobotFactorError {
//     #[error("The robot radius must be positive, but it was {0}")]
//     NegativeRobotRadius(Float),
// }

impl InterRobotFactor {
    pub const NEIGHBORS: usize = 2;

    #[must_use]
    pub fn new(
        robot_radius: StrictlyPositiveFinite<Float>,
        connection: InterRobotFactorConnection,
        // ) -> Result<Self, InterRobotFactorError> {
    ) -> Self {
        // if robot_radius < 0.0 {
        //     return Err(InterRobotFactorError::NegativeRobotRadius(robot_radius));
        // }
        let robot_radius = robot_radius.get();
        let epsilon = 0.2 * robot_radius;

        Self {
            safety_distance: 2.0 * robot_radius + epsilon,
            skip: false,
            connection,
        }
    }

    /// Get the safety distance
    #[inline(always)]
    pub fn safety_distance(&self) -> Float {
        self.safety_distance
    }
}

impl IFactor for InterRobotFactor {
    #[inline(always)]
    fn name(&self) -> &'static str {
        "InterRobotFactor"
    }

    fn jacobian(&mut self, state: &FactorState, x: &Vector<Float>) -> Matrix<Float> {
        let mut jacobian = Matrix::<Float>::zeros((state.initial_measurement.len(), DOFS * 2));
        let x_diff = {
            let offset = DOFS / 2;
            let mut x_diff = x.slice(s![..offset]).sub(&x.slice(s![DOFS..DOFS + offset]));

            // NOTE: In gbplanner, they weight this by the robot id, why they do this is
            // unclear as a robot id should be unique, and not have any
            // semantics of distance/weight.
            for i in 0..offset {
                // x_diff[i] += 1e-6 * self.connection.id_of_robot_connected_with.index() as
                // Float;
                x_diff[i] += 1e-6 * self.connection.other_variable.factorgraph_id.index() as Float;
                // Add a tiny random offset to avoid div/0 errors
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
        jacobian
    }

    fn measure(&mut self, state: &FactorState, x: &Vector<Float>) -> Vector<Float> {
        let mut h = Vector::<Float>::zeros(state.initial_measurement.len());
        let x_diff = {
            let offset = DOFS / 2;
            let mut x_diff = x.slice(s![..offset]).sub(&x.slice(s![DOFS..DOFS + offset]));
            // NOTE: In gbplanner, they weight this by the robot id, why they do this is
            // unclear as a robot id should be unique, and not have any
            // semantics of distance/weight.
            for i in 0..offset {
                // Add a tiny random offset to avoid div/0 errors
                x_diff[i] += 1e-6 * self.connection.id_of_robot_connected_with.index() as Float;
            }
            x_diff
        };

        let radius = x_diff.euclidean_norm();
        let within_safety_distance = radius <= self.safety_distance;
        // match (self.skip, within_safety_distance) {
        //     (Skip(true), true) => {}
        //     (Skip(true), false) => {}
        //     (Skip(false), true) => {
        //         self.skip = Skip(true);
        //         info!("skip = true, radius = {}", radius);
        //     }
        //     (Skip(false), false) => {}
        // };
        if radius <= self.safety_distance {
            if self.skip {
                info!(
                    "within safety distance, radius = {}, setting self.skip to false",
                    radius
                );
            }
            self.skip = false;
            // gbpplanner: h(0) = 1.f*(1 - r/safety_distance_);
            // NOTE: in Eigen, indexing a matrix with a single index corresponds to indexing
            // the matrix as a flattened array in column-major order.
            // h[(0, 0)] = 1.0 * (1.0 - radius / self.safety_distance);
            h[0] = 1.0 * (1.0 - radius / self.safety_distance);
            // println!("h = {}", h);
        } else {
            if !self.skip {
                // info!(
                //     "outside safety distance, radius = {}, setting self.skip
                // to true",     radius
                // );
            }
            self.skip = true;
        }

        h
    }

    #[inline(always)]
    fn jacobian_delta(&self) -> Float {
        1e-2
    }

    fn skip(&mut self, state: &FactorState) -> bool {
        // this->skip_flag = ( (X_(seqN(0,n_dofs_/2)) - X_(seqN(n_dofs_,
        // n_dofs_/2))).squaredNorm() >= safety_distance_*safety_distance_ );
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
        // let skip = squared_norm >= Float::powi(self.safety_distance, 2);
        if self.skip != skip {
            // warn!(
            //     "skip = {}, squared_norm = {} safety_distance^2 = {}",
            //     skip,
            //     squared_norm,
            //     Float::powi(self.safety_distance, 2)
            // );
        }
        self.skip = skip;
        // self.skip = squared_norm >= Float::powi(self.safety_distance, 2);
        self.skip
    }

    #[inline(always)]
    fn linear(&self) -> bool {
        false
    }
}
