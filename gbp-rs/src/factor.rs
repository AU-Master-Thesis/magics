use nalgebra::{dvector, DMatrix, DVector};

use crate::{robot::RobotId, variable::Variable, Key, Mailbox, Message};

use std::rc::Rc;

// TODO: make generic over f32 | f64
// <T: nalgebra::Scalar + Copy>
#[derive(Debug)]
struct FactorState {
    /// called `z_` in **gbpplanner**
    measurement: DVector<f32>,
    /// called `meas_model_lambda_` in **gbpplanner**
    measurement_precision: DMatrix<f32>,
    /// Stored linearisation point
    /// called X_ in gbpplanner, they use Eigen::MatrixXd instead
    linearisation_point: DVector<f32>,
    /// Strength of the factor. Called `sigma` in gbpplanner.
    /// The factor precision $Lambda = sigma^-2 * Identify$
    strength: f32,
    /// Number of degrees of freedom e.g. 4 [x, y, x', y']
    dofs: usize,
}

impl FactorState {
    fn new(
        measurement: DVector<f32>,
        // linearisation_point: DVector<f32>,
        strength: f32,
        dofs: usize,
    ) -> Self {
        // Initialise precision of the measurement function
        // this->meas_model_lambda_ = Eigen::MatrixXd::Identity(z_.rows(), z_.rows()) / pow(sigma,2.);
        let measurement_precision =
            DMatrix::<f32>::identity(measurement.nrows(), measurement.ncols())
                / f32::powi(strength, 2);

        Self {
            measurement,
            measurement_precision,
            linearisation_point: dvector![],
            strength,
            dofs,
        }
    }
}

#[derive(Debug)]
pub struct Factor {
    /// Unique identifier that associates the variable with a factorgraph/robot.
    pub key: Key,
    /// Vector of pointers to the connected variables. Order of variables matters␍
    /// in **gbpplanner** `std::vector<std::shared_ptr<Variable>> variables_{};`
    pub adjacent_variables: Vec<Rc<Variable>>,
    /// State common between all factor kinds
    pub state: FactorState,
    /// Variant storing the specialized behavior of each Factor kind.
    pub kind: FactorKind,
    /// Mailbox for incoming message storage
    pub inbox: Mailbox,
    /// Mailbox for outgoing message storage
    pub outbox: Mailbox,
}

impl Factor {
    fn new(
        key: Key,
        adjacent_variables: Vec<Rc<Variable>>,
        state: FactorState,
        kind: FactorKind,
    ) -> Self {
        Self {
            key,
            adjacent_variables,
            state,
            kind,
            inbox: Mailbox::new(),
            outbox: Mailbox::new(),
        }
    }

    // Message Factor::marginalise_factor_dist(const Eigen::VectorXd &eta, const Eigen::MatrixXd &Lam, int var_idx, int marg_idx){
    //     // Marginalisation only needed if factor is connected to >1 variables
    //     int n_dofs = variables_[var_idx]->n_dofs_;
    //     if (eta.size() == n_dofs) return Message {eta, Lam};

    //     Eigen::VectorXd eta_a(n_dofs), eta_b(eta.size()-n_dofs);
    //     eta_a = eta(seqN(marg_idx, n_dofs));
    //     eta_b << eta(seq(0, marg_idx - 1)), eta(seq(marg_idx + n_dofs, last));

    //     Eigen::MatrixXd lam_aa(n_dofs, n_dofs), lam_ab(n_dofs, Lam.cols()-n_dofs);
    //     Eigen::MatrixXd lam_ba(Lam.rows()-n_dofs, n_dofs), lam_bb(Lam.rows()-n_dofs, Lam.cols()-n_dofs);
    //     lam_aa << Lam(seqN(marg_idx, n_dofs), seqN(marg_idx, n_dofs));
    //     lam_ab << Lam(seqN(marg_idx, n_dofs), seq(0, marg_idx - 1)), Lam(seqN(marg_idx, n_dofs), seq(marg_idx + n_dofs, last));
    //     lam_ba << Lam(seq(0, marg_idx - 1), seq(marg_idx, marg_idx + n_dofs - 1)), Lam(seq(marg_idx + n_dofs, last), seqN(marg_idx, n_dofs));
    //     lam_bb << Lam(seq(0, marg_idx - 1), seq(0, marg_idx - 1)), Lam(seq(0, marg_idx - 1), seq(marg_idx + n_dofs, last)),
    //             Lam(seq(marg_idx + n_dofs, last), seq(0, marg_idx - 1)), Lam(seq(marg_idx + n_dofs, last), seq(marg_idx + n_dofs, last));

    //     Eigen::MatrixXd lam_bb_inv = lam_bb.inverse();
    //     Message marginalised_msg(n_dofs);
    //     marginalised_msg.eta = eta_a - lam_ab * lam_bb_inv * eta_b;
    //     marginalised_msg.lambda = lam_aa - lam_ab * lam_bb_inv * lam_ba;
    //     if (!marginalised_msg.lambda.allFinite()) marginalised_msg.setZero();

    //     return marginalised_msg;
    // }

    /// Marginalise the factor precision and information and create the outgoing message to the variable.
    pub fn marginalise_factor_distance(
        &mut self,
        information_vector: DVector<f32>,
        precision_matrix: DMatrix<f32>,
        // variable_index: usize
        var_idx: usize,
        marg_idx: usize,
    ) -> Message {
        let dofs = self
            .adjacent_variables
            .get(var_idx)
            .expect("var_idx is within [0, len)")
            .dofs;

        if information_vector.len() == dofs {
            return Message::new(information_vector, precision_matrix);
        }

        // eta_a = eta(seqN(marg_idx, n_dofs));
        let eta_a = information_vector.rows(marg_idx, dofs);
        // eta_b << eta(seq(0, marg_idx - 1)), eta(seq(marg_idx + n_dofs, last));
        let eta_b = {
            let mut v = DVector::<f32>::zeros(information_vector.len() - dofs);
            v.view_mut((0, 0), (marg_idx - 1, 0))
                .copy_from(&information_vector.rows(0, marg_idx - 1));
            v.view_mut((marg_idx, 0), (v.len(), 0)).copy_from(
                &information_vector.rows(marg_idx + dofs, information_vector.len()),
            );
            v
        };

        // TODO: create some declarative macros to do this

        let mut lam_aa = DMatrix::<f32>::zeros(dofs, dofs);
        let mut lam_ab = DMatrix::<f32>::zeros(dofs, precision_matrix.ncols() - dofs);
        let mut lam_ba = DMatrix::<f32>::zeros(precision_matrix.nrows() - dofs, dofs);
        let mut lam_bb = DMatrix::<f32>::zeros(
            precision_matrix.nrows() - dofs,
            precision_matrix.ncols() - dofs,
        );

        let lam_bb_inv = lam_bb.try_inverse().expect("The matrix is invertible");

        // let marginalised_message = Message {};

        // marginalised_message
        todo!()
    }

    pub fn new_dynamic_factor(
        key: Key,
        adjacent_variables: Vec<Rc<Variable>>,
        strength: f32,
        measurement: &DVector<f32>,
        dofs: usize,
        delta_t: f32,
    ) -> Self {
        let mut state = FactorState::new(measurement.clone_owned(), strength, dofs);
        let dynamic_factor = DynamicFactor::new(&mut state, delta_t);
        let kind = FactorKind::Dynamic(dynamic_factor);
        Self::new(key, adjacent_variables, state, kind)
    }
    pub fn new_interrobot_factor(
        key: Key,
        adjacent_variables: Vec<Rc<Variable>>,
        strength: f32,
        measurement: DVector<f32>,
        dofs: usize,
        safety_radius: f32,
        id_of_robot_connected_with: RobotId,
    ) -> Self {
        let state = FactorState::new(measurement, strength, dofs);
        let interrobot_factor = InterRobotFactor::new(
            safety_radius,
            strength,
            false,
            id_of_robot_connected_with,
        );
        let kind = FactorKind::InterRobot(interrobot_factor);

        Self::new(key, adjacent_variables, state, kind)
    }
    pub fn new_pose_factor(key: Key, adjacent_variables: Vec<Rc<Variable>>) -> Self {
        todo!()
    }
    pub fn new_obstacle_factor(
        key: Key,
        adjacent_variables: Vec<Rc<Variable>>,
        strength: f32,
        measurement: DVector<f32>,
        dofs: usize,
        obstacle_sdf: Rc<image::RgbImage>,
        world_size: f32,
    ) -> Self {
        let state = FactorState::new(measurement, strength, dofs);
        let obstacle_factor = ObstacleFactor::new(obstacle_sdf, world_size);
        let kind = FactorKind::Obstacle(obstacle_factor);
        Self::new(key, adjacent_variables, state, kind)
    }

    // Main section: Factor update:
    // Messages from connected variables are aggregated. The beliefs are used to create the linearisation point X_.
    // The Factor potential is calculated using h_func_ and J_func_
    // The factor precision and information is created, and then marginalised to create outgoing messages to its connected variables.

    ///
    pub fn update(&mut self) -> bool {
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
        for variable in self.adjacent_variables.iter() {
            idx += variable.dofs;
            let message = self
                .inbox
                .get(&variable.key)
                .expect("there should be a message");
            // self.state.linearisation_point
        }

        // // *Depending on the problem*, we may need to skip computation of this factor.
        // // eg. to avoid extra computation, factor may not be required if two connected variables are too far apart.
        // // in which case send out a Zero Message.
        // if (this->skip_factor()){
        //     for (auto var : variables_){
        //         this->outbox_[var->key_] = Message(var->n_dofs_);
        //     }
        //     return false;
        // }

        if self.kind.skip(&self.state) {
            for variable in self.adjacent_variables.iter() {
                self.outbox
                    .insert(variable.key, Message::with_dofs(variable.dofs));
            }

            return false;
        }

        true
    }
}

/// Interrobot factor: for avoidance of other robots
/// This factor results in a high energy or cost if two robots are planning to be in the same
/// position at the same timestep (collision). This factor is created between variables of two robots.
/// The factor has 0 energy if the variables are further away than the safety distance.
#[derive(Debug, Clone, Copy)]
pub struct InterRobotFactor {
    // TODO: constrain to be positive
    pub safety_distance: f32,
    ///
    skip: bool,
    pub id_of_robot_connected_with: RobotId,
}

impl InterRobotFactor {
    pub fn new(
        safety_distance: f32,
        robot_radius: f32,
        skip: bool,
        id_of_robot_connected_with: RobotId,
    ) -> Self {
        let epsilon = 0.2 * robot_radius;

        Self {
            safety_distance: 2.0 * robot_radius + epsilon,
            skip,
            id_of_robot_connected_with,
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
    /// defined at src/Robot.cpp:64
    pub delta_t: f32,
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

        #[allow(clippy::similar_names)]
        let qc_inv = f32::powi(state.strength, -2) * &eye;

        #[allow(clippy::similar_names)]
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
            insert_block_matrix(
                &mut jacobian,
                (state.dofs * 2 / 2, eye.ncols() * 3),
                &eye,
            );

            jacobian
        };

        Self {
            cached_jacobian,
            delta_t,
        }
    }
}

#[derive(Debug)]
pub struct PoseFactor;

// TODO: obstacle factor, in gbpplanner uses a pointer to an image, which contains an SDF of the obstacles in the environment

#[derive(Debug)]
pub enum FactorKind {
    Pose(PoseFactor),
    InterRobot(InterRobotFactor),
    Dynamic(DynamicFactor),
    Obstacle(ObstacleFactor),
}

impl FactorKind {
    /// Returns `true` if the factor kind is [`Obstacle`].
    ///
    /// [`Obstacle`]: FactorKind::Obstacle
    #[must_use]
    fn is_obstacle(&self) -> bool {
        matches!(self, Self::Obstacle(..))
    }

    /// Returns `true` if the factor kind is [`Dynamic`].
    ///
    /// [`Dynamic`]: FactorKind::Dynamic
    #[must_use]
    fn is_dynamic(&self) -> bool {
        matches!(self, Self::Dynamic(..))
    }

    /// Returns `true` if the factor kind is [`InterRobot`].
    ///
    /// [`InterRobot`]: FactorKind::InterRobot
    #[must_use]
    fn is_inter_robot(&self) -> bool {
        matches!(self, Self::InterRobot(..))
    }

    /// Returns `true` if the factor kind is [`Pose`].
    ///
    /// [`Pose`]: FactorKind::Pose
    #[must_use]
    fn is_pose(&self) -> bool {
        matches!(self, Self::Pose(..))
    }
}

impl Model for PoseFactor {
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

    fn linear(&self) -> bool {
        false
    }
}

trait Model {
    // TODO: maybe just return &DMatrix<f32>
    fn jacobian(&mut self, state: &FactorState, x: &DVector<f32>) -> DMatrix<f32> {
        self.first_order_jacobian(state, x)
    }
    /// Measurement function
    /// **Note**: This method takes a mutable reference to self, because the interrobot factor
    fn measurement(&mut self, state: &FactorState, x: &DVector<f32>) -> DVector<f32>;
    fn first_order_jacobian(
        &mut self,
        state: &FactorState,
        x: &DVector<f32>,
    ) -> DMatrix<f32> {
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
            let column =
                (self.measurement(state, &copy_of_x) - &h0) / self.jacobian_delta();
            jacobian.set_column(i, &column);
        }

        jacobian
    }

    fn jacobian_delta(&self) -> f32;

    fn linear(&self) -> bool;

    /// Whether to skip this factor in the update step
    /// In gbpplanner, this is only used for the interrobot factor.
    /// The other factors are always included in the update step.
    fn skip(&mut self, state: &FactorState) -> bool;
}

impl Model for InterRobotFactor {
    fn jacobian(&mut self, state: &FactorState, x: &DVector<f32>) -> DMatrix<f32> {
        // TODO: switch to ndarray
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

    fn linear(&self) -> bool {
        false
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

    // #[inline(always)]
    fn linear(&self) -> bool {
        true
    }
}

#[derive(Debug)]
struct ObstacleFactor {
    obstacle_sdf: Rc<image::RgbImage>,
    /// Copy of the `WORLD_SZ` setting from **gbpplanner**, that we store a copy of here since
    /// `ObstacleFactor` needs this information to calculate `.jacobian_delta()` and `.measurement()`
    world_size: f32,
}

impl ObstacleFactor {
    /// Creates a new [`ObstacleFactor`].
    #[must_use]
    fn new(obstacle_sdf: Rc<image::RgbImage>, world_size: f32) -> Self {
        Self {
            obstacle_sdf,
            world_size,
        }
    }
}

impl Model for ObstacleFactor {
    fn jacobian(&mut self, state: &FactorState, x: &DVector<f32>) -> DMatrix<f32> {
        // Same as PoseFactor
        self.first_order_jacobian(state, x)
    }

    fn measurement(&mut self, state: &FactorState, x: &DVector<f32>) -> DVector<f32> {
        // White areas are obstacles, so h(0) should return a 1 for these regions.
        let scale = self.obstacle_sdf.width() as f32 / self.world_size;
        let pixel_x = ((x[0] + self.world_size / 2.0) * scale) as u32;
        let pixel_y = ((x[1] + self.world_size / 2.0) * scale) as u32;
        let pixel = self.obstacle_sdf[(pixel_x, pixel_y)].0;
        let hsv_value = pixel[0] as f32 / 255.0;

        dvector![hsv_value]
    }

    fn jacobian_delta(&self) -> f32 {
        self.world_size / self.obstacle_sdf.width() as f32
    }

    fn skip(&mut self, state: &FactorState) -> bool {
        false
    }

    fn linear(&self) -> bool {
        false
    }
}

impl Model for FactorKind {
    fn jacobian(&mut self, state: &FactorState, x: &DVector<f32>) -> DMatrix<f32> {
        match self {
            FactorKind::Pose(f) => f.jacobian(state, x),
            FactorKind::InterRobot(f) => f.jacobian(state, x),
            FactorKind::Dynamic(f) => f.jacobian(state, x),
            FactorKind::Obstacle(f) => f.jacobian(state, x),
        }
    }

    fn measurement(&mut self, state: &FactorState, x: &DVector<f32>) -> DVector<f32> {
        match self {
            FactorKind::Pose(f) => f.measurement(state, x),
            FactorKind::InterRobot(f) => f.measurement(state, x),
            FactorKind::Dynamic(f) => f.measurement(state, x),
            FactorKind::Obstacle(f) => f.measurement(state, x),
        }
    }

    fn skip(&mut self, state: &FactorState) -> bool {
        match self {
            FactorKind::Pose(f) => f.skip(state),
            FactorKind::InterRobot(f) => f.skip(state),
            FactorKind::Dynamic(f) => f.skip(state),
            FactorKind::Obstacle(f) => f.skip(state),
        }
    }

    fn jacobian_delta(&self) -> f32 {
        match self {
            FactorKind::Pose(f) => f.jacobian_delta(),
            FactorKind::InterRobot(f) => f.jacobian_delta(),
            FactorKind::Dynamic(f) => f.jacobian_delta(),
            FactorKind::Obstacle(f) => f.jacobian_delta(),
        }
    }

    fn linear(&self) -> bool {
        match self {
            FactorKind::Pose(f) => f.linear(),
            FactorKind::InterRobot(f) => f.linear(),
            FactorKind::Dynamic(f) => f.linear(),
            FactorKind::Obstacle(f) => f.linear(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn test_marginalize_factor_distance() {
        // let
    }
}
