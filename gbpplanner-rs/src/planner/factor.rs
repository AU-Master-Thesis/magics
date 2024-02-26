use bevy::{ecs::entity::Entity, log::info};

// use nalgebra::{Matrix, Vector, Matrix, Vector};
use ndarray::{array, s, Axis, Slice};
use ndarray_linalg::Norm;
use petgraph::prelude::NodeIndex;
use std::{
    collections::HashMap,
    ops::{AddAssign, Sub},
    sync::Arc,
};

use super::{
    factorgraph::{Graph, Inbox, Message},
    Matrix, Scalar, Vector,
};

trait Model {
    // TODO: maybe just return &Matrix<f32>
    fn jacobian(&mut self, state: &FactorState, x: &Vector<f32>) -> Matrix<f32> {
        self.first_order_jacobian(state, x.clone())
    }
    /// Measurement function
    /// **Note**: This method takes a mutable reference to self, because the interrobot factor
    fn measure(&mut self, state: &FactorState, x: &Vector<f32>) -> Vector<f32>;
    fn first_order_jacobian(
        &mut self,
        state: &FactorState,
        x: Vector<f32>,
    ) -> Matrix<f32> {
        // Eigen::MatrixXd Factor::jacobianFirstOrder(const Eigen::VectorXd& X0){
        //     return jac_out;
        // };

        // Eigen::MatrixXd h0 = h_func_(X0);    // Value at lin point
        let h0 = self.measure(state, &x);
        // Eigen::MatrixXd jac_out = Eigen::MatrixXd::Zero(h0.size(),X0.size());
        let mut jacobian = Matrix::<f32>::zeros((h0.len(), x.len()));

        //     for (int i=0; i<X0.size(); i++){
        //         Eigen::VectorXd X_copy = X0;                                    // Copy of lin point
        //         X_copy(i) += delta_jac;                                         // Perturb by delta
        //         jac_out(Eigen::all, i) = (h_func_(X_copy) - h0) / delta_jac;    // Derivative (first order)
        //     }
        for i in 0..x.len() {
            let mut copy_of_x = x.clone();
            copy_of_x[i] += self.jacobian_delta();
            let column = (self.measure(state, &copy_of_x) - &h0) / self.jacobian_delta();
            // jacobian.set_column(i, &column);
            jacobian
                .slice_axis_mut(Axis(0), Slice::from(0..jacobian.dim().0))
                .assign(&column);
        }

        jacobian
    }

    fn jacobian_delta(&self) -> f32;

    /// Whether to skip this factor in the update step
    /// In gbpplanner, this is only used for the interrobot factor.
    /// The other factors are always included in the update step.
    fn skip(&mut self, state: &FactorState) -> bool;

    fn linear(&self) -> bool;
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
    pub id_of_robot_connected_with: Entity,
}

impl InterRobotFactor {
    pub fn new(
        safety_distance: f32,
        robot_radius: f32,
        skip: bool,
        id_of_robot_connected_with: Entity,
    ) -> Self {
        let epsilon = 0.2 * robot_radius;

        Self {
            safety_distance: 2.0 * robot_radius + epsilon,
            skip,
            id_of_robot_connected_with,
        }
    }
}

impl Model for InterRobotFactor {
    fn jacobian(&mut self, state: &FactorState, x: &Vector<f32>) -> Matrix<f32> {
        let mut jacobian = Matrix::zeros((state.measurement.len(), state.dofs * 2));
        let x_diff = {
            let offset = state.dofs / 2;
            // let mut x_diff = x.rows(0, offset) - x.rows(state.dofs, offset);
            let mut x_diff = x.slice_axis(Axis(0), Slice::from(0..offset)).sub(
                &x.slice_axis(Axis(0), Slice::from(state.dofs..(state.dofs + offset))),
            );
            for i in 0..offset {
                x_diff[i] += 1e-6; // Add a tiny random offset to avoid div/0 errors
            }
            x_diff
        };
        let radius = x_diff.norm();
        if radius <= self.safety_distance {
            // TODO: why do we change the Jacobian if we are not outside the safety distance?
            // jacobian
            //     .view_mut((0, 0), (0, state.dofs / 2))
            //     .copy_from(&(-1.0 / self.safety_distance / radius * &x_diff));
            // jacobian.slice_mut(s![0..(state.dofs / 2), 0..(state.dofs / 2)]) =
            //     &(-1.0 / self.safety_distance / radius * &x_diff);
            jacobian
                .slice_mut(s![0..(state.dofs / 2), 0..(state.dofs / 2)])
                .assign(&(-1.0 / self.safety_distance / radius * &x_diff));

            // jacobian
            //     .view_mut((0, state.dofs), (0, state.dofs + state.dofs / 2))
            //     .copy_from(&(1.0 / self.safety_distance / radius * &x_diff));
            // jacobian.slice_mut(s![
            //     0..(state.dofs / 2),
            //     state.dofs..state.dofs + (state.dofs / 2)
            // ]) = &(1.0 / self.safety_distance / radius * &x_diff);
            jacobian
                .slice_mut(s![
                    0..(state.dofs / 2),
                    state.dofs..state.dofs + (state.dofs / 2)
                ])
                .assign(&(1.0 / self.safety_distance / radius * &x_diff));
        }
        jacobian
    }

    fn measure(&mut self, state: &FactorState, x: &Vector<f32>) -> Vector<f32> {
        // let mut h = Matrix::zeros(state.measurement.nrows(), state.measurement.ncols());
        let mut h = Vector::zeros(state.measurement.len());
        let x_diff = {
            let offset = state.dofs / 2;

            // let mut x_diff = x.rows(0, offset) - x.rows(state.dofs, offset);
            // let mut x_diff = x.slice_axis(Axis(0), s![0..offset])
            //     - x.slice_axis(Axis(0), s![state.dofs..(state.dofs + offset)]);
            let mut x_diff = x.slice_axis(Axis(0), Slice::from(0..offset)).sub(
                &x.slice_axis(Axis(0), Slice::from(state.dofs..(state.dofs + offset))),
            );
            // let mut x_diff = x
            //     .slice_axis(Axis(0), Slice::from(0..offset))
            //     .sub(&x.slice(s![state.dofs..(state.dofs + offset), ..]));
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
        // let dontknow = (state.linearisation_point.rows(0, offset)
        //     - state.linearisation_point.rows(state.dofs, offset))
        let dontknow = state
            .linearisation_point
            .slice_axis(Axis(0), Slice::from(0..offset))
            .sub(
                &state
                    .linearisation_point
                    .slice_axis(Axis(0), Slice::from(state.dofs..(state.dofs + offset))),
            )
            .norm();
        self.skip = dontknow >= f32::powi(self.safety_distance, 2);

        self.skip
    }

    fn linear(&self) -> bool {
        false
    }
}

// TODO: use proper error handling here with an Error type
// TODO: move into module
// TODO: write unit test cases
fn insert_block_matrix<T: Scalar>(
    matrix: &mut Matrix<T>,
    start: (usize, usize),
    block: &Matrix<T>,
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
#[derive(Debug, Clone)]
pub struct DynamicFactor {
    cached_jacobian: Matrix<f32>,
    /// defined at src/Robot.cpp:64
    pub delta_t: f32,
}

impl DynamicFactor {
    #[must_use]
    pub fn new(state: &mut FactorState, delta_t: f32) -> Self {
        let (eye, zeros) = {
            let (nrows, ncols) = (state.dofs / 2, state.dofs / 2);
            let eye = Matrix::<f32>::eye(nrows);
            let zeros = Matrix::<f32>::zeros((nrows, ncols));
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
            let mut qi_inv = Matrix::<f32>::zeros((nrows, ncols));
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
            let mut jacobian = Matrix::<f32>::zeros((state.dofs, state.dofs * 2));
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

impl Model for DynamicFactor {
    fn jacobian(&mut self, _state: &FactorState, x: &Vector<f32>) -> Matrix<f32> {
        self.cached_jacobian.clone()
    }

    fn measure(&mut self, _state: &FactorState, x: &Vector<f32>) -> Vector<f32> {
        self.cached_jacobian.dot(x)
    }

    fn skip(&mut self, _state: &FactorState) -> bool {
        false
    }

    fn jacobian_delta(&self) -> f32 {
        1e-2
    }

    fn linear(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone)]
pub struct PoseFactor;

impl Model for PoseFactor {
    /// Default jacobian is the first order taylor series jacobian
    fn jacobian(&mut self, state: &FactorState, x: &Vector<f32>) -> Matrix<f32> {
        self.first_order_jacobian(state, x.clone())
    }

    /// Default meaurement function is the identity function
    fn measure(&mut self, _state: &FactorState, x: &Vector<f32>) -> Vector<f32> {
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

#[derive(Debug, Clone)]
struct ObstacleFactor {
    obstacle_sdf: Arc<image::RgbImage>,
    /// Copy of the `WORLD_SZ` setting from **gbpplanner**, that we store a copy of here since
    /// `ObstacleFactor` needs this information to calculate `.jacobian_delta()` and `.measurement()`
    world_size: f32,
}

impl ObstacleFactor {
    /// Creates a new [`ObstacleFactor`].
    #[must_use]
    fn new(obstacle_sdf: Arc<image::RgbImage>, world_size: f32) -> Self {
        Self {
            obstacle_sdf,
            world_size,
        }
    }
}

impl Model for ObstacleFactor {
    fn jacobian(&mut self, state: &FactorState, x: &Vector<f32>) -> Matrix<f32> {
        // Same as PoseFactor
        self.first_order_jacobian(state, x.clone())
    }

    fn measure(&mut self, state: &FactorState, x: &Vector<f32>) -> Vector<f32> {
        // White areas are obstacles, so h(0) should return a 1 for these regions.
        let scale = self.obstacle_sdf.width() as f32 / self.world_size;
        let pixel_x = ((x[0] + self.world_size / 2.0) * scale) as u32;
        let pixel_y = ((x[1] + self.world_size / 2.0) * scale) as u32;
        let pixel = self.obstacle_sdf[(pixel_x, pixel_y)].0;
        let hsv_value = pixel[0] as f32 / 255.0;

        array![hsv_value]
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

#[derive(Debug, Clone)]
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
    pub fn is_obstacle(&self) -> bool {
        matches!(self, Self::Obstacle(..))
    }

    /// Returns `true` if the factor kind is [`Dynamic`].
    ///
    /// [`Dynamic`]: FactorKind::Dynamic
    #[must_use]
    pub fn is_dynamic(&self) -> bool {
        matches!(self, Self::Dynamic(..))
    }

    /// Returns `true` if the factor kind is [`InterRobot`].
    ///
    /// [`InterRobot`]: FactorKind::InterRobot
    #[must_use]
    pub fn is_inter_robot(&self) -> bool {
        matches!(self, Self::InterRobot(..))
    }

    /// Returns `true` if the factor kind is [`Pose`].
    ///
    /// [`Pose`]: FactorKind::Pose
    #[must_use]
    pub fn is_pose(&self) -> bool {
        matches!(self, Self::Pose(..))
    }
}

impl Model for FactorKind {
    fn jacobian(&mut self, state: &FactorState, x: &Vector<f32>) -> Matrix<f32> {
        match self {
            FactorKind::Pose(f) => f.jacobian(state, x),
            FactorKind::InterRobot(f) => f.jacobian(state, x),
            FactorKind::Dynamic(f) => f.jacobian(state, x),
            FactorKind::Obstacle(f) => f.jacobian(state, x),
        }
    }

    fn measure(&mut self, state: &FactorState, x: &Vector<f32>) -> Vector<f32> {
        match self {
            FactorKind::Pose(f) => f.measure(state, x),
            FactorKind::InterRobot(f) => f.measure(state, x),
            FactorKind::Dynamic(f) => f.measure(state, x),
            FactorKind::Obstacle(f) => f.measure(state, x),
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

// TODO: make generic over f32 | f64
// <T: nalgebra::Scalar + Copy>
#[derive(Debug, Clone)]
pub struct FactorState {
    /// called `z_` in **gbpplanner**
    pub measurement: Vector<f32>,
    /// called `meas_model_lambda_` in **gbpplanner**
    pub measurement_precision: Matrix<f32>,
    /// Stored linearisation point
    /// called X_ in gbpplanner, they use Eigen::MatrixXd instead
    pub linearisation_point: Vector<f32>,
    /// Strength of the factor. Called `sigma` in gbpplanner.
    /// The factor precision $Lambda = sigma^-2 * Identify$
    pub strength: f32,
    /// Number of degrees of freedom e.g. 4 [x, y, x', y']
    pub dofs: usize,

    /// Cached value of the factors jacobian function
    /// called `J_` in **gbpplanner**
    pub cached_jacobian: Matrix<f32>,

    /// Cached value of the factors jacobian function
    /// called `h_` in **gbpplanner**
    pub cached_measurement: Vector<f32>,
}

impl FactorState {
    fn new(
        measurement: Vector<f32>,
        // linearisation_point: Vector<f32>,
        strength: f32,
        dofs: usize,
    ) -> Self {
        // Initialise precision of the measurement function
        // this->meas_model_lambda_ = Eigen::MatrixXd::Identity(z_.rows(), z_.rows()) / pow(sigma,2.);
        let measurement_precision =
            Matrix::<f32>::eye(measurement.len()) / f32::powi(strength, 2);

        Self {
            measurement,
            measurement_precision,
            linearisation_point: array![],
            strength,
            dofs,
            cached_jacobian: array![[]],
            cached_measurement: array![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Factor {
    /// Unique identifier that associates the variable with a factorgraph/robot.
    pub node_index: Option<NodeIndex>,
    /// State common between all factor kinds
    pub state: FactorState,
    /// Variant storing the specialized behavior of each Factor kind.
    pub kind: FactorKind,
    /// Mailbox for incoming message storage
    inbox: Inbox,

    /// Set to true after the first call to self.update()
    /// TODO: move to FactorState
    initialized: bool,
}

impl Factor {
    fn new(state: FactorState, kind: FactorKind) -> Self {
        Self {
            node_index: None,
            state,
            kind,
            inbox: Inbox::new(),
            initialized: false,
        }
    }

    pub fn new_dynamic_factor(
        strength: f32,
        measurement: &Vector<f32>,
        dofs: usize,
        delta_t: f32,
    ) -> Self {
        let mut state = FactorState::new(measurement.to_owned(), strength, dofs);
        let dynamic_factor = DynamicFactor::new(&mut state, delta_t);
        let kind = FactorKind::Dynamic(dynamic_factor);
        Self::new(state, kind)
    }

    pub fn new_interrobot_factor(
        strength: f32,
        measurement: Vector<f32>,
        dofs: usize,
        safety_radius: f32,
        id_of_robot_connected_with: Entity,
    ) -> Self {
        let state = FactorState::new(measurement, strength, dofs);
        let interrobot_factor = InterRobotFactor::new(
            safety_radius,
            strength,
            false,
            id_of_robot_connected_with,
        );
        let kind = FactorKind::InterRobot(interrobot_factor);

        Self::new(state, kind)
    }

    pub fn new_pose_factor() -> Self {
        todo!()
    }

    pub fn new_obstacle_factor(
        strength: f32,
        measurement: Vector<f32>,
        dofs: usize,
        obstacle_sdf: Arc<image::RgbImage>,
        world_size: f32,
    ) -> Self {
        let state = FactorState::new(measurement, strength, dofs);
        let obstacle_factor = ObstacleFactor::new(obstacle_sdf, world_size);
        let kind = FactorKind::Obstacle(obstacle_factor);
        Self::new(state, kind)
    }

    pub fn jacobian(&mut self, x: &Vector<f32>) -> Matrix<f32> {
        self.kind.jacobian(&self.state, x)
    }

    pub fn measure(&mut self, x: &Vector<f32>) -> Vector<f32> {
        self.kind.measure(&self.state, x)
    }

    pub fn set_node_index(&mut self, node_index: NodeIndex) {
        if self.node_index.is_some() {
            panic!("The node index is already set");
        }
        self.node_index = Some(node_index);
    }

    pub fn get_node_index(&self) -> NodeIndex {
        if self.node_index.is_none() {
            panic!("The node index has not been set");
        }
        self.node_index.expect("I checked it was there 3 lines ago")
    }

    pub fn send_message(&mut self, from: NodeIndex, message: Message) {
        let _ = self.inbox.insert(from, message);
    }

    pub fn read_message_from(&mut self, from: NodeIndex) -> Option<&Message> {
        self.inbox.get(&from)
    }

    fn residual(&self) -> Vector<f32> {
        &self.state.measurement - &self.state.cached_measurement
    }

    // Main section: Factor update:
    // Messages from connected variables are aggregated. The beliefs are used to create the linearisation point X_.
    // The Factor potential is calculated using h_func_ and J_func_
    // The factor precision and information is created, and then marginalised to create outgoing messages to its connected variables.
    // pub fn update(&mut self, factor_index: NodeIndex, graph: &mut Graph) -> bool {
    pub fn update(
        &mut self,
        factor_index: NodeIndex,
        adjacent_variables: &[NodeIndex],
        graph: &Graph,
    ) -> HashMap<NodeIndex, Message> {
        // // Messages from connected variables are aggregated.
        // // The beliefs are used to create the linearisation point X_.
        // int idx = 0; int n_dofs;
        // for (int v=0; v<variables_.size(); v++){
        //     n_dofs = variables_[v]->n_dofs_;
        //     auto& [_, __, mu_belief] = this->inbox_[variables_[v]->key_];
        //     X_(seqN(idx, n_dofs)) = mu_belief;
        //     idx += n_dofs;
        // }
        // let adjacent_variables = graph.neighbors(factor_index);
        let mut idx = 0;
        for variable_index in adjacent_variables {
            let variable = graph[*variable_index]
                .as_variable()
                .expect("The variable_index should point to a Variable in the graph");

            idx += variable.dofs;
            let message = self.inbox.get(variable_index).expect(
                "There should be a message from each variable connected to this factor",
            );

            // TODO: how do we ensure/know state.linearisation_point is long enough to fit all message means concatenated
            // utils::nalgebra::insert_subvector(
            //     &mut self.state.linearisation_point,
            //     idx..idx + variable.dofs,
            //     &message.mean(),
            // );
            self.state
                .linearisation_point
                .slice_mut(s![idx..idx + variable.dofs])
                .assign(&message.mean());
        }

        // TODO: implement the rest of the update method

        if self.kind.skip(&self.state) {
            info!("skipping factor update early for factor with index: {factor_index:?}");
            let messages_to_variables = adjacent_variables
                .iter()
                .map(|variable_index| {
                    let message = Message::with_dofs(idx);
                    (*variable_index, message)
                })
                .collect::<HashMap<_, _>>();

            return messages_to_variables;
        }

        //  Update factor precision and information with incoming messages from connected variables.
        let measurement = self
            .kind
            .measure(&self.state, &self.state.linearisation_point);

        let linearisation_point_clone = self.state.linearisation_point.clone();
        let jacobian = if self.kind.linear() && self.initialized {
            self.state.cached_jacobian.clone()
        } else {
            self.jacobian(&linearisation_point_clone)
        };

        // Eigen::MatrixXd factor_lam_potential = J_.transpose() * meas_model_lambda_ * J_;
        // Eigen::VectorXd factor_eta_potential = (J_.transpose() * meas_model_lambda_) * (J_ * X_ + residual());
        // this->initialised_ = true;

        // Eigen::MatrixXd factor_lam_potential = J_.transpose() * meas_model_lambda_ * J_;
        // Eigen::VectorXd factor_eta_potential = (J_.transpose() * meas_model_lambda_) * (J_ * X_ + residual());

        let factor_lam_potential = jacobian
            .t()
            .dot(&self.state.measurement_precision)
            .dot(&jacobian);
        let factor_eta_potential = (jacobian.t().dot(&self.state.measurement_precision))
            .dot(&jacobian.dot(&(linearisation_point_clone + self.residual())));

        self.initialized = true;

        //  Update factor precision and information with incoming messages from connected variables.
        let mut marginalisation_idx = 0usize;
        let mut messages_to_variables =
            HashMap::<NodeIndex, Message>::with_capacity(adjacent_variables.len());

        for variable_index in adjacent_variables.iter() {
            let mut factor_eta = factor_eta_potential.to_owned();
            let mut factor_lam = factor_lam_potential.to_owned();

            let variable = graph[*variable_index]
                .as_variable()
                .expect("The variable_index should point to a Variable in the graph");

            // Combine the factor with the belief from other variables apart from the receiving variable

            let mut index_offset = 0usize;
            for other_variable_index in adjacent_variables.iter() {
                let other_variable = graph[*other_variable_index]
                    .as_variable()
                    .expect("The variable_index should point to a Variable in the graph");
                if variable_index != other_variable_index {
                    let message = self.read_message_from(*other_variable_index).expect("There should be a message from each variable connected to this factor");
                    // let slice = index_offset..index_offset + variable.dofs;
                    // factor_eta(seqN(idx_v, n_dofs)) += eta_belief;
                    // gbp_linalg::vector::add_assign_subvector(
                    //     &mut factor_eta,
                    //     index_offset,
                    //     &message.0.information_vector,
                    // );
                    // TODO: KRISTOFFER CHECK THIS
                    factor_eta
                        .slice_mut(s![index_offset..index_offset + variable.dofs])
                        .add_assign(&message.0.information_vector);
                    // .add_assign(&message.0.mean());
                    // factor_lam(seqN(idx_v, n_dofs), seqN(idx_v, n_dofs)) += lam_belief;
                    // gbp_linalg::matrix::add_assign_submatrix(
                    //     &mut factor_lam,
                    //     (index_offset, index_offset + variable.dofs),
                    //     &message.0.precision_matrix,
                    // );
                    // TODO: KRISTOFFER CHECK THIS
                    factor_lam
                        .slice_mut(s![
                            index_offset..index_offset + variable.dofs,
                            index_offset..index_offset + variable.dofs
                        ])
                        .add_assign(&message.0.precision_matrix);
                }
                index_offset += other_variable.dofs;
            }

            // // Marginalise the Factor Precision and Information to send to the relevant variable
            // outbox_[var_out->key_] = marginalise_factor_dist(factor_eta, factor_lam, v_out_idx, marginalisation_idx);
            // marginalisation_idx += var_out->n_dofs_;
            let message_to_send = self.marginalise_factor_distance(
                factor_eta,
                factor_lam,
                variable_index.index(),
                marginalisation_idx,
            );
            marginalisation_idx += variable.dofs;
            messages_to_variables.insert(*variable_index, message_to_send);
        }

        messages_to_variables
    }

    pub fn skip(&mut self) -> bool {
        self.kind.skip(&self.state)
    }

    // /// Marginalise the factor precision and information and create the outgoing message to the variable.
    // pub fn marginalise_factor_distance(
    //     &mut self,
    //     information_vector: Vector<f32>,
    //     precision_matrix: Matrix<f32>,
    //     // variable_index: usize
    //     var_idx: usize,
    //     marg_idx: usize,
    // ) -> Message {
    //     let dofs = self
    //         .adjacent_variables
    //         .get(var_idx)
    //         .expect("var_idx is within [0, len)")
    //         .dofs;

    //     if information_vector.len() == dofs {
    //         return Message::new(information_vector, precision_matrix);
    //     }

    //     // eta_a = eta(seqN(marg_idx, n_dofs));
    //     let eta_a = information_vector.rows(marg_idx, dofs);
    //     // eta_b << eta(seq(0, marg_idx - 1)), eta(seq(marg_idx + n_dofs, last));
    //     let eta_b = {
    //         let mut v = Vector::<f32>::zeros(information_vector.len() - dofs);
    //         v.view_mut((0, 0), (marg_idx - 1, 0))
    //             .copy_from(&information_vector.rows(0, marg_idx - 1));
    //         v.view_mut((marg_idx, 0), (v.len(), 0)).copy_from(
    //             &information_vector.rows(marg_idx + dofs, information_vector.len()),
    //         );
    //         v
    //     };

    //     // TODO: create some declarative macros to do this

    //     let mut lam_aa = Matrix::<f32>::zeros(dofs, dofs);
    //     let mut lam_ab = Matrix::<f32>::zeros(dofs, precision_matrix.ncols() - dofs);
    //     let mut lam_ba = Matrix::<f32>::zeros(precision_matrix.nrows() - dofs, dofs);
    //     let mut lam_bb = Matrix::<f32>::zeros(
    //         precision_matrix.nrows() - dofs,
    //         precision_matrix.ncols() - dofs,
    //     );

    //     let lam_bb_inv = lam_bb.try_inverse().expect("The matrix is invertible");

    //     // let marginalised_message = Message {};

    //     // marginalised_message
    //     todo!()
    // }

    fn marginalise_factor_distance(
        &self,
        information_vector: Vector<f32>,
        precision_matrix: Matrix<f32>,
        dofs_of_variable: usize,
        marginalisation_idx: usize,
    ) -> Message {
        if information_vector.len() == dofs_of_variable {
            return Message::new(information_vector, precision_matrix);
        }

        // Eigen::VectorXd eta_a(n_dofs);
        // eta_a = eta(seqN(marg_idx, n_dofs));
        let eta_a = {
            let mut v = Vector::<f32>::zeros(dofs_of_variable);
            // gbp_linalg::vector::override_subvector(
            //     &mut v,
            //     marginalisation_idx,
            //     &information_vector,
            // );
            v.slice_mut(s![marginalisation_idx..])
                .assign(&information_vector);
            v
        };
        // eta_b(eta.size()-n_dofs);
        // eta_b << eta(seq(0, marg_idx - 1)), eta(seq(marg_idx + n_dofs, last));
        let eta_b = {
            let mut v = Vector::<f32>::zeros(information_vector.len() - dofs_of_variable);
            // gbp_linalg::vector::override_subvector(
            //     &mut v,
            //     0,
            //     gbp_linalg::vector_view!(information_vector, 0..marginalisation_idx - 1),
            // );
            v.slice_mut(s![0..marginalisation_idx])
                .assign(&information_vector.slice(s![0..marginalisation_idx]));
            v
        };

        todo!()
    }
}
