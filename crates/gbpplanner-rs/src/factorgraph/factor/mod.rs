use std::ops::AddAssign;

use bevy::render::texture::Image;
use gbp_linalg::prelude::*;
use ndarray::{array, concatenate, prelude::*, s, Axis};
use typed_floats::StrictlyPositiveFinite;

use self::{
    dynamic::DynamicFactor,
    interrobot::{InterRobotFactor, InterRobotFactorConnection},
    obstacle::ObstacleFactor,
    pose::PoseFactor,
};
use super::{
    factorgraph::NodeIndex,
    id::VariableId,
    message::{MessagesToFactors, MessagesToVariables},
    node::FactorGraphNode,
    prelude::Message,
    MessageCount, DOFS,
};
use crate::factorgraph::node::RemoveConnectionToError;

pub(in crate::factorgraph) mod dynamic;
pub(in crate::factorgraph) mod interrobot;
mod marginalise_factor_distance;
pub(in crate::factorgraph) mod obstacle;
pub(in crate::factorgraph) mod pose;

use marginalise_factor_distance::marginalise_factor_distance;

// TODO: make generic over f32 | f64
// TODO: hide the state parameter from the public API, by having the `Factor`
// struct expose similar methods that dispatch to the `FactorState` struct.
pub trait IFactor {
    /// The name of the factor. Used for debugging and visualization.
    fn name(&self) -> &'static str;

    fn jacobian_delta(&self) -> Float;

    /// Whether to skip this factor in the update step
    /// In gbpplanner, this is only used for the interrobot factor.
    /// The other factors are always included in the update step.
    fn skip(&mut self, state: &FactorState) -> bool;

    /// Whether the factor is linear or non-linear
    fn linear(&self) -> bool;

    #[must_use]
    #[inline]
    fn jacobian(&mut self, state: &FactorState, x: &Vector<Float>) -> Matrix<Float> {
        self.first_order_jacobian(state, x.clone())
    }

    /// Measurement function
    /// **Note**: This method takes a mutable reference to self, because the
    /// interrobot factor
    #[must_use]
    fn measure(&mut self, state: &FactorState, x: &Vector<Float>) -> Vector<Float>;

    fn first_order_jacobian(&mut self, state: &FactorState, mut x: Vector<Float>) -> Matrix<Float> {
        let h0 = self.measure(state, &x); // value at linearization point
        let mut jacobian = Matrix::<Float>::zeros((h0.len(), x.len()));

        let delta = self.jacobian_delta();

        for i in 0..x.len() {
            x[i] += delta; // perturb by delta
            let derivatives = (self.measure(state, &x) - &h0) / delta;
            jacobian.column_mut(i).assign(&derivatives);
            x[i] -= delta; // reset the perturbation
        }

        jacobian
    }
}

#[derive(Debug)]
pub struct Factor {
    /// Unique identifier that associates the variable with the factorgraph it
    /// is part of.
    pub node_index: Option<NodeIndex>,
    /// State common between all factor kinds
    pub state:      FactorState,
    /// Variant storing the specialized behavior of each Factor kind.
    pub kind:       FactorKind,
    /// Mailbox for incoming message storage
    pub inbox:      MessagesToVariables,

    message_count: MessageCount,
}

impl Factor {
    fn new(state: FactorState, kind: FactorKind) -> Self {
        Self {
            node_index: None,
            state,
            kind,
            inbox: MessagesToVariables::new(),
            message_count: MessageCount::default(),
        }
    }

    /// Returns the node index of the factor
    ///
    /// # Panics
    ///
    /// Panics if the node index has not been set, which should not happen.
    #[inline]
    pub fn node_index(&self) -> NodeIndex {
        if self.node_index.is_none() {
            panic!("The node index has not been set");
        }
        self.node_index.unwrap()
    }

    /// Sets the node index of the factor
    ///
    /// # Panics
    ///
    /// Panics if the node index has already been set.
    pub fn set_node_index(&mut self, node_index: NodeIndex) {
        if self.node_index.is_some() {
            panic!("The node index is already set");
        }
        self.node_index = Some(node_index);
    }

    pub fn new_dynamic_factor(strength: Float, measurement: Vector<Float>, delta_t: Float) -> Self {
        let mut state = FactorState::new(measurement, strength, DynamicFactor::NEIGHBORS);
        let dynamic_factor = DynamicFactor::new(&mut state, delta_t);
        let kind = FactorKind::Dynamic(dynamic_factor);
        Self::new(state, kind)
    }

    pub fn new_interrobot_factor(
        strength: Float,
        measurement: Vector<Float>,
        safety_radius: StrictlyPositiveFinite<Float>,
        connection: InterRobotFactorConnection,
    ) -> Self {
        let interrobot_factor = InterRobotFactor::new(safety_radius, connection);
        let kind = FactorKind::InterRobot(interrobot_factor);
        let state = FactorState::new(measurement, strength, InterRobotFactor::NEIGHBORS);

        Self::new(state, kind)
    }

    pub fn new_pose_factor() -> Self {
        unimplemented!("the pose factor is stored in the variable")
    }

    pub fn new_obstacle_factor(
        strength: Float,
        measurement: Vector<Float>,
        obstacle_sdf: &'static Image,
        world_size: Float,
    ) -> Self {
        let state = FactorState::new(measurement, strength, ObstacleFactor::NEIGHBORS);
        let obstacle_factor = ObstacleFactor::new(obstacle_sdf, world_size);
        let kind = FactorKind::Obstacle(obstacle_factor);
        Self::new(state, kind)
    }

    #[inline(always)]
    pub fn variant(&self) -> &'static str {
        self.kind.name()
    }

    #[inline(always)]
    fn jacobian(&mut self, x: &Vector<Float>) -> Matrix<Float> {
        self.kind.jacobian(&self.state, x)
    }

    fn measure(&mut self, x: &Vector<Float>) -> Vector<Float> {
        self.state.cached_measurement = self.kind.measure(&self.state, x);
        self.state.cached_measurement.clone()
    }

    /// Check if the factor should be skipped in the update step
    #[inline(always)]
    fn skip(&mut self) -> bool {
        self.kind.skip(&self.state)
    }

    pub fn receive_message_from(&mut self, from: VariableId, message: Message) {
        let _ = self.inbox.insert(from, message);
        self.message_count.received += 1;
    }

    #[inline(always)]
    pub fn read_message_from(&mut self, from: VariableId) -> Option<&Message> {
        self.inbox.get(&from)
    }

    /// Calculates the residual between the current measurement and the initial
    /// measurement
    #[inline(always)]
    #[must_use]
    fn residual(&self) -> Vector<Float> {
        &self.state.initial_measurement - &self.state.cached_measurement
    }

    #[must_use]
    pub fn update(&mut self) -> MessagesToVariables {
        debug_assert_eq!(
            self.state.linearisation_point.len(),
            DOFS * self.inbox.len()
        );

        let zero_mean = Vector::<Float>::zeros(DOFS);

        for (i, (_, message)) in self.inbox.iter().enumerate() {
            let mean = message.mean().unwrap_or(&zero_mean);
            self.state
                .linearisation_point
                .slice_mut(s![i * DOFS..(i + 1) * DOFS])
                .assign(mean);
        }

        if self.skip() {
            let messages: MessagesToVariables = self
                .inbox
                .iter()
                .map(|(variable_id, _)| (*variable_id, Message::empty()))
                .collect();
            self.message_count.sent += messages.len();
            return messages;
        }

        let meas = self.measure(&self.state.linearisation_point.clone());
        let jacobian = self.jacobian(&self.state.linearisation_point.clone());

        let potential_precision_matrix = jacobian
            .t()
            .dot(&self.state.measurement_precision)
            .dot(&jacobian);
        let potential_information_vec = jacobian
            .t()
            .dot(&self.state.measurement_precision)
            .dot(&(jacobian.dot(&self.state.linearisation_point) + self.residual()));

        self.state.initialized = true;

        let mut marginalisation_idx = 0;
        let mut messages = MessagesToVariables::new();

        let zero_precision = Matrix::<Float>::zeros((DOFS, DOFS));

        for variable_id in self.inbox.keys() {
            let mut information_vec = potential_information_vec.clone();
            let mut precision_matrix = potential_precision_matrix.clone();

            for (j, (other_variable_id, other_message)) in self.inbox.iter().enumerate() {
                if other_variable_id == variable_id {
                    // Do not aggregate data from the variable we're sending to
                    continue;
                }

                let message_eta = other_message
                    .information_vector()
                    .expect("it better be there");

                // let message_precision =
                // other_message.precision_matrix().unwrap_or(&zero_precision);
                let message_precision = other_message
                    .precision_matrix()
                    .unwrap_or_else(|| &zero_precision);
                // .unwrap_or_else(|| &Matrix::<Float>::zeros((DOFS, DOFS)));

                information_vec
                    .slice_mut(s![j * DOFS..(j + 1) * DOFS])
                    .add_assign(message_eta);
                precision_matrix
                    .slice_mut(s![j * DOFS..(j + 1) * DOFS, j * DOFS..(j + 1) * DOFS])
                    .add_assign(message_precision);
            }

            let message =
                marginalise_factor_distance(information_vec, precision_matrix, marginalisation_idx)
                    .expect("marginalise_factor_distance should not fail");
            messages.insert(*variable_id, message);
            marginalisation_idx += DOFS;
        }

        self.message_count.sent += messages.len();
        messages
    }

    /// Check if the factor is an [`InterRobotFactor`]
    #[inline(always)]
    pub fn is_inter_robot(&self) -> bool {
        self.kind.is_inter_robot()
    }

    /// Check if the factor is a [`DynamicFactor`]
    #[inline(always)]
    pub fn is_dynamic(&self) -> bool {
        self.kind.is_dynamic()
    }

    /// Check if the factor is an [`ObstacleFactor`]
    #[inline(always)]
    pub fn is_obstacle(&self) -> bool {
        self.kind.is_obstacle()
    }

    /// Check if the factor is a [`PoseFactor`]
    #[inline(always)]
    pub fn is_pose(&self) -> bool {
        self.kind.is_pose()
    }
}

#[derive(Debug, derive_more::IsVariant)]
pub enum FactorKind {
    Pose(PoseFactor),
    InterRobot(InterRobotFactor),
    Dynamic(DynamicFactor),
    Obstacle(ObstacleFactor),
}

impl FactorKind {
    /// Returns `Some(&InterRobotFactor)` if self is [`InterRobot`], otherwise
    pub fn as_inter_robot(&self) -> Option<&InterRobotFactor> {
        if let Self::InterRobot(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `Some(&DynamicFactor)` if self is [`Dynamic`], otherwise
    pub fn as_dynamic(&self) -> Option<&DynamicFactor> {
        if let Self::Dynamic(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `Some(&ObstacleFactor)` if self is [`Obstacle`], otherwise
    pub fn as_obstacle(&self) -> Option<&ObstacleFactor> {
        if let Self::Obstacle(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `Some(&PoseFactor)` if self is [`Pose`], otherwise `None`.
    pub fn as_pose(&self) -> Option<&PoseFactor> {
        if let Self::Pose(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

impl IFactor for FactorKind {
    fn name(&self) -> &'static str {
        match self {
            FactorKind::Pose(f) => f.name(),
            FactorKind::InterRobot(f) => f.name(),
            FactorKind::Dynamic(f) => f.name(),
            FactorKind::Obstacle(f) => f.name(),
        }
    }

    fn jacobian(&mut self, state: &FactorState, x: &Vector<Float>) -> Matrix<Float> {
        match self {
            FactorKind::Pose(f) => f.jacobian(state, x),
            FactorKind::InterRobot(f) => f.jacobian(state, x),
            FactorKind::Dynamic(f) => f.jacobian(state, x),
            FactorKind::Obstacle(f) => f.jacobian(state, x),
        }
    }

    fn measure(&mut self, state: &FactorState, x: &Vector<Float>) -> Vector<Float> {
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

    fn jacobian_delta(&self) -> Float {
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

#[derive(Debug, Clone)]
pub struct FactorState {
    /// called `z_` in **gbpplanner**
    pub initial_measurement: Vector<Float>,
    /// called `meas_model_lambda_` in **gbpplanner**
    pub measurement_precision: Matrix<Float>,
    /// Stored linearisation point
    /// called `X_` in **gbpplanner**, they use `Eigen::MatrixXd` instead
    pub linearisation_point: Vector<Float>,
    /// Strength of the factor. Called `sigma` in gbpplanner.
    /// The factor precision $Lambda = sigma^-2 * Identify$
    pub strength: Float,

    /// Cached value of the factors jacobian function
    /// called `J_` in **gbpplanner**
    /// TODO: wrap in Option<>
    pub cached_jacobian: Matrix<Float>,

    /// Cached value of the factors jacobian function
    /// called `h_` in **gbpplanner**
    /// TODO: wrap in Option<>
    pub cached_measurement: Vector<Float>,
    /// Set to true after the first call to self.update()
    initialized: bool,
}

impl FactorState {
    /// Create a new [`FactorState`]
    fn new(initial_measurement: Vector<Float>, strength: Float, neighbor_amount: usize) -> Self {
        // Initialise precision of the measurement function
        // this->meas_model_lambda_ = Eigen::MatrixXd::Identity(z_.rows(), z_.rows()) /
        // pow(sigma,2.);
        let measurement_precision =
            Matrix::<Float>::eye(initial_measurement.len()) / Float::powi(strength, 2);

        Self {
            initial_measurement,
            measurement_precision,
            linearisation_point: Vector::<Float>::zeros(DOFS * neighbor_amount),
            strength,
            cached_jacobian: array![[]],
            cached_measurement: array![],
            initialized: false,
        }
    }
}

impl FactorGraphNode for Factor {
    fn remove_connection_to(
        &mut self,
        factorgraph_id: super::factorgraph::FactorGraphId,
    ) -> Result<(), RemoveConnectionToError> {
        let connections_before = self.inbox.len();
        self.inbox
            .retain(|variable_id, v| variable_id.factorgraph_id != factorgraph_id);
        let connections_after = self.inbox.len();

        let no_connections_removed = connections_before == connections_after;
        if no_connections_removed {
            Err(RemoveConnectionToError)
        } else {
            Ok(())
        }
    }

    #[inline(always)]
    fn messages_sent(&self) -> usize {
        self.message_count.sent
    }

    #[inline(always)]
    fn messages_received(&self) -> usize {
        self.message_count.received
    }

    #[inline(always)]
    fn reset_message_count(&mut self) {
        self.message_count.reset();
    }
}
