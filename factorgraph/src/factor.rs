use nalgebra::{DMatrix, DVector};

use crate::{message::Payload, variable::Variable};

use std::rc::Rc;

// TODO: make generic over f32 | f64
// <T: nalgebra::Scalar + Copy>
#[derive(Debug)]
struct FactorState {
    measurement: DVector<f32>, // called z_ in gbpplanner
    measurement_precision: DMatrix<f32>,
    /// Stored linearisation point
    linearisation_point: DVector<f32>, // called X_ in gbpplanner, they use Eigen::MatrixXd instead
    /// Strength of the factor. Called `sigma` in gbpplanner.
    /// The factor precision $Lambda = sigma^-2 * Identify$
    strength: f32,
    /// Number of degrees of freedom e.g. 4 [x, y, x', y']
    dofs: usize,
}

#[derive(Debug)]
pub struct Factor {
    // TODO: are these only variables that belongs to the same factorgraph/robot as self?
    pub adjacent_variables: Vec<Rc<Variable>>,
    // TODO: document when a factor can be valid/invalid
    pub valid: bool,
    pub kind: FactorKind,

    pub state: FactorState,
}

impl Factor {
    // Main section: Factor update:
    // Messages from connected variables are aggregated. The beliefs are used to create the linearisation point X_.
    // The Factor potential is calculated using h_func_ and J_func_
    // The factor precision and information is created, and then marginalised to create outgoing messages to its connected variables.

    ///
    pub fn update(&mut self) {
        // // Messages from connected variables are aggregated.
        // // The beliefs are used to create the linearisation point X_.
        // int idx = 0; int n_dofs;
        // for (int v=0; v<variables_.size(); v++){
        //     n_dofs = variables_[v]->n_dofs_;
        //     auto& [_, __, mu_belief] = this->inbox_[variables_[v]->key_];
        //     X_(seqN(idx, n_dofs)) = mu_belief;
        //     idx += n_dofs;
        // }

        let mut idx = 0;
        for var in self.adjacent_variables.iter() {
            idx += var.dofs;
        }

        todo!()
    }

    /// Marginalise the factor precision and information and create the outgoing message to the variable.
    pub fn marginalise_factor_distance(&mut self) -> Payload {
        todo!()
    }
}

/// Interrobot factor: for avoidance of other robots
/// This factor results in a high energy or cost if two robots are planning to be in the same
/// position at the same timestep (collision). This factor is created between variables of two robots.
/// The factor has 0 energy if the variables are further away than the safety distance.
#[derive(Debug, Clone, Copy)]
pub struct InterRobotFactor {
    // TODO: constrain to be positive
    safety_distance: f32,
    ///
    skip: bool,
}

impl InterRobotFactor {
    // TODO: refactor to use a bbox model
    pub fn new(safety_distance: f32, robot_radius: f32, skip: bool) -> Self {
        let epsilon = 0.2 * robot_radius;

        Self {
            safety_distance: 2.0 * robot_radius + epsilon,
            skip,
        }
    }
}

// #[derive(Debug)]
// struct MatrixShape {
//     nrows: usize,
//     ncols: usize,
// }

// impl std::fmt::Display for MatrixShape {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{:?}", self)
//     }
// }

// #[derive(Debug)]
// enum InsertBlockMatrixError {
//     BlockLargerThanMatrix {
//         matrix_dims: MatrixShape,
//         block_dims: MatrixShape
//     },
//     StartOutsideMatrix {
//         start: (usize, usize),
//         matrix_dims: MatrixShape,
//     },
//     StartPlusBlockExceedsMatrix {
//         start: (usize, usize),
//         matrix_dims: MatrixShape,
//         block_dims: MatrixShape
//     }
// }

// impl std::fmt::Display for InsertBlockMatrixError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         use InsertBlockMatrixError::*;
//         match self {
//             BlockLargerThanMatrix { matrix_dims, block_dims } => write!(f, "The dimensions of the block matrix ({}) exceeds the dimensions of the matrix ({})", block_dims, matrix_dims),
//             StartOutsideMatrix { start, matrix_dims } => write!(f, "The start offset ({}, {}) is outside the dimensions of the matrix ({})", start.0, start.1, matrix_dims ),
//             StartPlusBlockExceedsMatrix { start, matrix_dims, block_dims } => write!(f, ""),
//         }
//     }
// }

// impl std::error::Error for InsertBlockMatrixError {}

// TODO: use proper error handling here with an Error type
// TODO: move into module
// TODO: write unit test cases
fn insert_block_matrix<T: nalgebra::Scalar + Copy>(
    matrix: &mut DMatrix<T>,
    start: (usize, usize),
    block: &DMatrix<T>,
) {
    debug_assert!(
        start.0 <= matrix.nrows() && start.1 <= matrix.ncols(),
        "start: ({}, {}) not inside matrix dims: ({}, {})",
        start.0,
        start.1,
        matrix.nrows(),
        matrix.ncols()
    );
    debug_assert!(
        block.nrows() <= matrix.nrows() && block.ncols() <= block.nrows(),
        "block's dims ({}, {}) exceeds the matrix's ({}, {})",
        block.nrows(),
        block.ncols(),
        matrix.nrows(),
        matrix.ncols()
    );

    debug_assert!(
        matrix.nrows() - start.0 >= block.nrows() || matrix.ncols() - start.1 >= block.ncols(),
        "inserting block with dims ({}, {}) at ({}, {}) would exceed the matrix's dims ({}, {})",
        block.nrows(),
        block.ncols(),
        start.0,
        start.1,
        matrix.nrows(),
        matrix.ncols()
    );

    for r in 0..block.nrows() {
        for c in 0..block.ncols() {
            matrix[(r + start.0, c + start.1)] = block[(r, c)];
        }
    }
}

/// Dynamic factor: constant velocity model
#[derive(Debug)]
pub struct DynamicFactor {
    cached_jacobian: DMatrix<f32>,
    pub delta_t: f32, // defined at src/Robot.cpp:64
}

impl DynamicFactor {
    #[must_use]
    pub fn new(state: &mut FactorState, delta_t: f32) -> Self {
        let (eye, zeros) = {
            let (nrows, ncols) = (state.dofs / 2, state.dofs / 2);
            let eye = DMatrix::<f32>::identity(nrows, ncols);
            let zeros = DMatrix::<f32>::zeros(nrows, ncols);
            (eye, zeros)
        };

        let qc_inv = f32::powi(state.strength, -2) * &eye;

        let qi_inv = {
            let upper_left = 12.0 * f32::powi(delta_t, -3) * &qc_inv;
            let upper_right = -6.0 * f32::powi(delta_t, -2) * &qc_inv;
            let lower_left = -6.0 * f32::powi(delta_t, -2) * &qc_inv;
            let lower_right = (4.0 / delta_t) * &qc_inv;

            // Construct as a block matrix
            let (nrows, ncols) = (state.dofs, state.dofs);
            let mut qi_inv = DMatrix::<f32>::zeros(nrows, ncols);
            insert_block_matrix(&mut qi_inv, (0, 0), &upper_left);
            insert_block_matrix(&mut qi_inv, (0, ncols / 2), &upper_right);
            insert_block_matrix(&mut qi_inv, (nrows / 2, 0), &lower_left);
            insert_block_matrix(&mut qi_inv, (nrows / 2, ncols / 2), &lower_right);

            qi_inv
        };

        state.measurement_precision = qi_inv;

        let cached_jacobian = {
            // J_ = Eigen::MatrixXd::Zero(n_dofs_, n_dofs_*2);
            // J_ << I, dt*I, -1*I,    O,
            //      O,    I,    O, -1*I;
            let mut jacobian = DMatrix::<f32>::zeros(state.dofs, state.dofs * 2);
            insert_block_matrix(&mut jacobian, (0, 0), &eye);
            insert_block_matrix(&mut jacobian, (0, eye.ncols()), &(delta_t * &eye));
            insert_block_matrix(&mut jacobian, (0, eye.ncols() * 2), &(-1.0 * &eye));
            insert_block_matrix(&mut jacobian, (state.dofs / 2, eye.ncols()), &eye);
            insert_block_matrix(&mut jacobian, (state.dofs * 2 / 2, eye.ncols() * 3), &eye);

            jacobian
        };

        Self {
            cached_jacobian,
            delta_t,
        }
    }
}

#[derive(Debug)]
pub struct DefaultFactor;

// TODO: obstacle factor, in gbpplanner uses a pointer to an image, which contains an SDF of the obstacles in the environment

#[derive(Debug)]
enum FactorKind {
    Default(DefaultFactor),
    InterRobot(InterRobotFactor),
    Dynamic(DynamicFactor),
}

trait Model {
    // TODO: maybe just return &DMatrix<f32>
    fn jacobian(&mut self, state: &FactorState, x: &DVector<f32>) -> DMatrix<f32> {
        self.first_order_jacobian(state, x)
    }
    /// Measurement function
    /// **Note**: This method takes a mutable reference to self, because the interrobot factor
    fn measurement(&mut self, state: &FactorState, x: &DVector<f32>) -> DVector<f32>;
    fn first_order_jacobian(&mut self, state: &FactorState, x: &DVector<f32>) -> DMatrix<f32> {
        // Eigen::MatrixXd Factor::jacobianFirstOrder(const Eigen::VectorXd& X0){
        //     return jac_out;
        // };

        // Eigen::MatrixXd h0 = h_func_(X0);    // Value at lin point
        let h0 = self.measurement(state, x);
        // Eigen::MatrixXd jac_out = Eigen::MatrixXd::Zero(h0.size(),X0.size());
        let mut jacobian = DMatrix::<f32>::zeros(h0.len(), x.len());

        //     for (int i=0; i<X0.size(); i++){
        //         Eigen::VectorXd X_copy = X0;                                    // Copy of lin point
        //         X_copy(i) += delta_jac;                                         // Perturb by delta
        //         jac_out(Eigen::all, i) = (h_func_(X_copy) - h0) / delta_jac;    // Derivative (first order)
        //     }
        for i in 0..x.len() {
            let mut copy_of_x = x.clone();
            copy_of_x[i] += self.jacobian_delta();
            let column = (self.measurement(state, &copy_of_x) - &h0) / self.jacobian_delta();
            jacobian.set_column(i, &column);
        }

        jacobian
    }

    fn jacobian_delta(&self) -> f32;

    /// Whether to skip this factor in the update step
    /// In gbpplanner, this is only used for the interrobot factor.
    /// The other factors are always included in the update step.
    fn skip(&mut self, state: &FactorState) -> bool;
}

impl Model for DefaultFactor {
    /// Default jacobian is the first order taylor series jacobian
    fn jacobian(&mut self, state: &FactorState, x: &DVector<f32>) -> DMatrix<f32> {
        self.first_order_jacobian(state, x)
    }

    /// Default meaurement function is the identity function
    fn measurement(&mut self, _state: &FactorState, x: &DVector<f32>) -> DVector<f32> {
        x.clone()
    }

    fn skip(&mut self, _state: &FactorState) -> bool {
        false
    }

    fn jacobian_delta(&self) -> f32 {
        1e-8
    }
}

impl Model for InterRobotFactor {
    fn jacobian(&mut self, state: &FactorState, x: &DVector<f32>) -> DMatrix<f32> {
        let mut jacobian = DMatrix::zeros(state.measurement.nrows(), state.dofs * 2);
        let x_diff = {
            let offset = state.dofs / 2;
            let mut x_diff = x.rows(0, offset) - x.rows(state.dofs, offset);
            for i in 0..offset {
                x_diff[i] += 1e-6; // Add a tiny random offset to avoid div/0 errors
            }
            x_diff
        };
        let radius = x_diff.norm();
        if radius <= self.safety_distance {
            // TODO: why do we change the Jacobian if we are not outside the safety distance?
            jacobian
                .view_mut((0, 0), (0, state.dofs / 2))
                .copy_from(&(-1.0 / self.safety_distance / radius * &x_diff));
            jacobian
                .view_mut((0, state.dofs), (0, state.dofs + state.dofs / 2))
                .copy_from(&(1.0 / self.safety_distance / radius * &x_diff));
        }
        jacobian
    }

    fn measurement(&mut self, state: &FactorState, x: &DVector<f32>) -> DVector<f32> {
        // let mut h = DMatrix::zeros(state.measurement.nrows(), state.measurement.ncols());
        let mut h = DVector::zeros(state.measurement.nrows());
        let x_diff = {
            let offset = state.dofs / 2;

            let mut x_diff = x.rows(0, offset) - x.rows(state.dofs, offset);
            // NOTE: In gbplanner, they weight this by the robot id, why they do this is unclear
            // as a robot id should be unique, and not have any semantics of distance/weight.
            for i in 0..offset {
                x_diff[i] += 1e-6; // Add a tiny random offset to avoid div/0 errors
            }
            x_diff
        };

        let radius = x_diff.norm();
        if radius <= self.safety_distance {
            self.skip = false;
            // gbpplanner: h(0) = 1.f*(1 - r/safety_distance_);
            // NOTE: in Eigen, indexing a matrix with a single index corresponds to indexing the matrix as a flattened array in column-major order.

            // h[(0, 0)] = 1.0 * (1.0 - radius / self.safety_distance);
            h[0] = 1.0 * (1.0 - radius / self.safety_distance);
        } else {
            self.skip = true;
        }

        h
    }

    fn jacobian_delta(&self) -> f32 {
        1e-2
    }

    fn skip(&mut self, state: &FactorState) -> bool {
        // this->skip_flag = ( (X_(seqN(0,n_dofs_/2)) - X_(seqN(n_dofs_, n_dofs_/2))).squaredNorm() >= safety_distance_*safety_distance_ );␍
        let offset = state.dofs / 2;
        // TODO: give a better name to this term of the inequality
        let dontknow = (state.linearisation_point.rows(0, offset)
            - state.linearisation_point.rows(state.dofs, offset))
        .norm_squared();
        self.skip = dontknow >= f32::powi(self.safety_distance, 2);

        self.skip
    }
}

impl Model for DynamicFactor {
    fn jacobian(&mut self, _state: &FactorState, x: &DVector<f32>) -> DMatrix<f32> {
        self.cached_jacobian.clone()
    }

    fn measurement(&mut self, _state: &FactorState, x: &DVector<f32>) -> DVector<f32> {
        &self.cached_jacobian * x
    }

    fn skip(&mut self, _state: &FactorState) -> bool {
        false
    }

    fn jacobian_delta(&self) -> f32 {
        1e-2
    }
}

impl Model for FactorKind {
    fn jacobian(&mut self, state: &FactorState, x: &DVector<f32>) -> DMatrix<f32> {
        match self {
            FactorKind::Default(f) => f.jacobian(state, x),
            FactorKind::InterRobot(f) => f.jacobian(state, x),
            FactorKind::Dynamic(f) => f.jacobian(state, x),
        }
    }

    fn measurement(&mut self, state: &FactorState, x: &DVector<f32>) -> DVector<f32> {
        match self {
            FactorKind::Default(f) => f.measurement(state, x),
            FactorKind::InterRobot(f) => f.measurement(state, x),
            FactorKind::Dynamic(f) => f.measurement(state, x),
        }
    }

    fn skip(&mut self, state: &FactorState) -> bool {
        match self {
            FactorKind::Default(f) => f.skip(state),
            FactorKind::InterRobot(f) => f.skip(state),
            FactorKind::Dynamic(f) => f.skip(state),
        }
    }

    fn jacobian_delta(&self) -> f32 {
        match self {
            FactorKind::Default(f) => f.jacobian_delta(),
            FactorKind::InterRobot(f) => f.jacobian_delta(),
            FactorKind::Dynamic(f) => f.jacobian_delta(),
        }
    }
}
