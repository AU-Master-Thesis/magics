use std::{borrow::Cow, num::NonZeroUsize, ops::AddAssign};

use bevy::math::Vec2;
use gbp_linalg::{prelude::*, pretty_format_matrix, pretty_format_vector};
use ndarray::{array, s};
use typed_floats::StrictlyPositiveFinite;

use self::{
    dynamic::DynamicFactor, interrobot::InterRobotFactor, obstacle::ObstacleFactor,
    tracking::TrackingFactor,
};
use super::{
    factorgraph::{FactorGraphId, NodeIndex},
    id::VariableId,
    message::MessagesToVariables,
    node::FactorGraphNode,
    prelude::Message,
    MessageCount, MessagesReceived, MessagesSent, DOFS,
};
use crate::{factorgraph::node::RemoveConnectionToError, simulation_loader::SdfImage};

pub(in crate::factorgraph) mod dynamic;
pub(in crate::factorgraph) mod interrobot;
mod marginalise_factor_distance;
pub(crate) mod obstacle;
pub(in crate::factorgraph) mod pose;
pub(in crate::factorgraph) mod tracking;
mod velocity;
// pub(in crate::factorgraph) mod velocity;

use marginalise_factor_distance::marginalise_factor_distance;

pub use crate::factorgraph::factor::interrobot::ExternalVariableId;

/// Common interface for all factors
pub trait Factor: std::fmt::Display {
    /// The name of the factor. Used for debugging and visualization.
    fn name(&self) -> &'static str;
    /// The color of the factor. Used for visualization.
    fn color(&self) -> [u8; 3];

    /// The delta for the jacobian calculation
    fn jacobian_delta(&self) -> Float;

    /// Returns the number of neighbours this factor expects
    fn neighbours(&self) -> usize;

    /// Whether to skip this factor in the update step
    /// In gbpplanner, this is only used for the interrobot factor.
    /// The other factors are always included in the update step.
    fn skip(&self, state: &FactorState) -> bool;

    /// Whether the factor is linear or non-linear
    fn linear(&self) -> bool;

    /// The jacobian of the factor
    #[must_use]
    #[inline]
    // fn jacobian<'a>(&mut self, state: &'a FactorState, x: &Vector<Float>) ->
    // Cow<'a, Matrix<Float>> {
    fn jacobian(
        &self,
        state: &FactorState,
        linearisation_point: &Vector<Float>,
    ) -> Cow<'_, Matrix<Float>> {
        Cow::Owned(self.first_order_jacobian(state, linearisation_point.clone()))
    }

    /// Measurement function
    #[must_use]
    fn measure(&self, state: &FactorState, linearisation_point: &Vector<Float>) -> Vector<Float>;

    /// The first order jacobian
    /// This is a default impl as factor variants should compute the first order
    /// jacobian the same way
    fn first_order_jacobian(
        &self,
        state: &FactorState,
        mut linearization_point: Vector<Float>,
    ) -> Matrix<Float> {
        let h0 = self.measure(state, &linearization_point); // value at linearization point
        let mut jacobian = Matrix::<Float>::zeros((h0.len(), linearization_point.len()));

        let delta = self.jacobian_delta();

        for i in 0..linearization_point.len() {
            linearization_point[i] += delta; // perturb by delta
            let derivatives = (self.measure(state, &linearization_point) - &h0) / delta;
            jacobian.column_mut(i).assign(&derivatives);
            linearization_point[i] -= delta; // reset the perturbation
        }

        jacobian
    }
}

/// Factor node in the factorgraph
#[derive(Debug)]
pub struct FactorNode {
    factorgraph_id: FactorGraphId,
    /// Unique identifier that associates the variable with the factorgraph it
    /// is part of.
    pub node_index: Option<NodeIndex>,
    /// State common between all factor kinds
    pub state:      FactorState,
    /// Variant storing the specialized behavior of each Factor kind.
    pub kind:       FactorKind,
    /// ailbox for incoming message storage
    pub inbox:      MessagesToVariables,

    message_count: MessageCount,
    pub enabled:   bool,
}

impl FactorNode {
    fn new(
        factorgraph_id: FactorGraphId,
        state: FactorState,
        kind: FactorKind,
        enabled: bool,
    ) -> Self {
        Self {
            factorgraph_id,
            node_index: None,
            state,
            kind,
            inbox: MessagesToVariables::new(),
            message_count: MessageCount::default(),
            enabled,
        }
    }

    /// Returns the factorgraph id that the factor belongs to
    #[inline]
    pub fn factorgraph_id(&self) -> FactorGraphId {
        self.factorgraph_id
    }

    /// Returns the node index of the factor
    ///
    /// # Panics
    ///
    /// Panics if the node index has not been set, which should not happen.
    #[inline]
    #[allow(clippy::unwrap_used)]
    pub fn node_index(&self) -> NodeIndex {
        assert!(self.node_index.is_some(), "The node index has not been set");
        self.node_index.unwrap()
    }

    /// Sets the node index of the factor
    ///
    /// # Panics
    ///
    /// Panics if the node index has already been set.
    pub fn set_node_index(&mut self, node_index: NodeIndex) {
        assert!(self.node_index.is_none(), "The node index is already set");
        self.node_index = Some(node_index);
    }

    /// Create a new dynamic factor
    pub fn new_dynamic_factor(
        factorgraph_id: FactorGraphId,
        strength: Float,
        measurement: Vector<Float>,
        delta_t: Float,
        enabled: bool,
    ) -> Self {
        let mut state = FactorState::new(measurement, strength, DynamicFactor::NEIGHBORS);
        let dynamic_factor = DynamicFactor::new(&mut state, delta_t);
        let kind = FactorKind::Dynamic(dynamic_factor);
        Self::new(factorgraph_id, state, kind, enabled)
    }

    /// Create a new interrobot factor
    pub fn new_interrobot_factor(
        factorgraph_id: FactorGraphId,
        strength: Float,
        measurement: Vector<Float>,
        // safety_radius: StrictlyPositiveFinite<Float>,
        robot_radius: StrictlyPositiveFinite<Float>,
        safety_distance_multiplier: StrictlyPositiveFinite<Float>,
        external_variable: ExternalVariableId,
        robot_number: NonZeroUsize,
        enabled: bool,
    ) -> Self {
        let interrobot_factor = InterRobotFactor::new(
            robot_radius,
            external_variable,
            Some(safety_distance_multiplier),
            robot_number,
        );
        let kind = FactorKind::InterRobot(interrobot_factor);
        let state = FactorState::new(measurement, strength, InterRobotFactor::NEIGHBORS);

        Self::new(factorgraph_id, state, kind, enabled)
    }

    // pub fn new_pose_factor() -> Self {
    //     unimplemented!("the pose factor is stored in the variable")
    // }

    /// Create a new obstacle factor
    pub fn new_obstacle_factor(
        factorgraph_id: FactorGraphId,
        strength: Float,
        measurement: Vector<Float>,
        obstacle_sdf: SdfImage,
        world_size: obstacle::WorldSize,
        enabled: bool,
        // world_size_width: Float,
        // world_size_height: Float,
    ) -> Self {
        let state = FactorState::new(measurement, strength, ObstacleFactor::NEIGHBORS);
        let obstacle_factor = ObstacleFactor::new(obstacle_sdf, world_size);
        let kind = FactorKind::Obstacle(obstacle_factor);
        Self::new(factorgraph_id, state, kind, enabled)
    }

    /// Create a new tracking factor
    pub fn new_tracking_factor(
        factorgraph_id: FactorGraphId,
        strength: Float,
        measurement: Vector<Float>,
        rrt_path: Option<min_len_vec::TwoOrMore<Vec2>>,
        enabled: bool,
    ) -> Self {
        let state = FactorState::new(measurement, strength, TrackingFactor::NEIGHBORS);
        let tracking_factor = TrackingFactor::new(rrt_path);
        let kind = FactorKind::Tracking(tracking_factor);
        Self::new(factorgraph_id, state, kind, enabled)
    }

    #[inline(always)]
    fn jacobian(&self, x: &Vector<Float>) -> Cow<'_, Matrix<Float>> {
        self.kind.jacobian(&self.state, x)
    }

    #[inline]
    fn measure(&self, x: &Vector<Float>) -> Vector<Float> {
        self.kind.measure(&self.state, x)
        // self.state.cached_measurement = self.kind.measure(&self.state, x);
        // self.state.cached_measurement.clone()
    }

    /// Check if the factor should be skipped in the update step
    #[inline(always)]
    fn skip(&mut self) -> bool {
        self.kind.skip(&self.state)
    }

    /// Add a message to this factors inbox
    pub fn receive_message_from(&mut self, from: VariableId, message: Message) {
        if !self.enabled {
            return;
        }
        let _ = self.inbox.insert(from, message);
        if from.factorgraph_id == self.factorgraph_id {
            self.message_count.received.internal += 1;
        } else {
            self.message_count.received.external += 1;
        }
    }

    // #[inline(always)]
    // pub fn read_message_from(&mut self, from: VariableId) -> Option<&Message> {
    //     self.inbox.get(&from)
    // }

    /// Calculates the residual between the current measurement and the initial
    /// measurement
    #[inline(always)]
    #[must_use]
    fn residual(&self) -> Vector<Float> {
        &self.state.initial_measurement - &self.state.cached_measurement
    }

    /// Update the factor using the gbp message passing algorithm
    #[must_use]
    pub fn update(&mut self) -> MessagesToVariables {
        // debug_assert_eq!(
        //     self.state.linearisation_point.len(),
        //     DOFS * self.inbox.len()
        // );

        // if self.state.linearisation_point.len() != DOFS * self.inbox.len() {
        //     eprintln!(
        //         "self.state.linearisation_point.len() = {}, DOFS * self.inbox.len() =
        // {}",         self.state.linearisation_point.len(),
        //         DOFS * self.inbox.len()
        //     );
        //     panic!("should not happen");
        // }

        // // let chunks = self.state.linearisation_point.exact_chunks_mut(DOFS);
        // for chunk in
        // self.state.linearisation_point.exact_chunks_mut(DOFS).into_iter() {
        //     // chunk.into_slice_memory_order()
        //     if let Some(data) = chunk.as_slice_memory_order_mut() {
        //         data.assign_elem(0.0);
        //     } else {
        //         panic!("should not happen, is 1d");
        //     }
        // }

        for (i, (_, message)) in self.inbox.iter().enumerate() {
            let mut slice = self
                .state
                .linearisation_point
                .slice_mut(s![i * DOFS..(i + 1) * DOFS]);

            if let Some(mean) = message.mean() {
                slice.assign(mean);
            } else {
                for x in slice {
                    *x = 0.0;
                }
            }
        }

        if self.skip() {
            let mut messages_sent = MessagesSent::new();
            let messages: MessagesToVariables = self
                .inbox
                .keys()
                .map(|variable_id| {
                    if variable_id.factorgraph_id == self.factorgraph_id {
                        messages_sent.internal += 1;
                    } else {
                        messages_sent.external += 1;
                    }

                    (*variable_id, Message::empty())
                })
                .collect();
            self.message_count.sent += messages_sent;
            return messages;
        }

        let measurement = self.measure(&self.state.linearisation_point);
        let jacobian = self.jacobian(&self.state.linearisation_point);

        let potential_precision_matrix = jacobian
            .t()
            .dot(&self.state.measurement_precision)
            .dot(jacobian.as_ref());

        let residual = &self.state.initial_measurement - measurement;

        let potential_information_vec = jacobian
            .t()
            .dot(&self.state.measurement_precision)
            .dot(&(jacobian.dot(&self.state.linearisation_point) + residual));

        self.state.initialized = true;

        let mut marginalisation_idx = 0;
        let mut messages = MessagesToVariables::new();
        // let mut messages = MessagesToVariables::with_capacity(self.inbox.len());

        let mut messages_sent = MessagesSent::new();

        for variable_id in self.inbox.keys() {
            let mut information_vec = potential_information_vec.clone();
            let mut precision_matrix = potential_precision_matrix.clone();

            for (j, (other_variable_id, other_message)) in self.inbox.iter().enumerate() {
                if other_variable_id == variable_id {
                    // Do not aggregate data from the variable we're sending to
                    continue;
                }

                if other_message.is_empty() {
                    continue;
                }

                // let message_eta = other_message.information_vector().expect("it better be
                // there"); information_vec
                //     .slice_mut(s![j * DOFS..(j + 1) * DOFS])
                //     .add_assign(message_eta);

                if let Some(message_information) = other_message.information_vector() {
                    information_vec
                        .slice_mut(s![j * DOFS..(j + 1) * DOFS])
                        .add_assign(message_information);
                }

                if let Some(message_precision) = other_message.precision_matrix() {
                    precision_matrix
                        .slice_mut(s![j * DOFS..(j + 1) * DOFS, j * DOFS..(j + 1) * DOFS])
                        .add_assign(message_precision);
                }
            }

            let message =
                marginalise_factor_distance(information_vec, precision_matrix, marginalisation_idx);
            messages.insert(*variable_id, message);

            if variable_id.factorgraph_id == self.factorgraph_id {
                messages_sent.internal += 1;
            } else {
                messages_sent.external += 1;
            }

            marginalisation_idx += DOFS;
        }

        self.message_count.sent += messages_sent;
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

    // /// Check if the factor is a [`PoseFactor`]
    // #[inline(always)]
    // pub fn is_pose(&self) -> bool {
    //     self.kind.is_pose()
    // }
}

/// Static dispatch enum for the various factors in the factorgraph
/// Used instead of dynamic dispatch
#[allow(missing_docs)]
#[derive(Debug, derive_more::IsVariant, strum_macros::EnumTryAs)]
pub enum FactorKind {
    // Pose(PoseFactor),
    /// `InterRobotFactor`
    InterRobot(InterRobotFactor),
    /// `DynamicFactor`
    Dynamic(DynamicFactor),
    /// `ObstacleFactor`
    Obstacle(ObstacleFactor),
    /// `TrackingFactor`
    Tracking(TrackingFactor),
}

impl std::fmt::Display for FactorKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InterRobot(f) => f.fmt(formatter),
            Self::Dynamic(f) => f.fmt(formatter),
            Self::Obstacle(f) => f.fmt(formatter),
            Self::Tracking(f) => f.fmt(formatter),
        }
    }
}

impl Factor for FactorKind {
    fn name(&self) -> &'static str {
        match self {
            Self::InterRobot(f) => f.name(),
            Self::Dynamic(f) => f.name(),
            Self::Obstacle(f) => f.name(),
            Self::Tracking(f) => f.name(),
        }
    }

    fn color(&self) -> [u8; 3] {
        match self {
            Self::InterRobot(f) => f.color(),
            Self::Dynamic(f) => f.color(),
            Self::Obstacle(f) => f.color(),
            Self::Tracking(f) => f.color(),
        }
    }

    fn jacobian(
        &self,
        state: &FactorState,
        linearisation_point: &Vector<Float>,
    ) -> Cow<'_, Matrix<Float>> {
        match self {
            Self::Dynamic(f) => f.jacobian(state, linearisation_point),
            Self::InterRobot(f) => f.jacobian(state, linearisation_point),
            Self::Obstacle(f) => f.jacobian(state, linearisation_point),
            Self::Tracking(f) => f.jacobian(state, linearisation_point),
        }
    }

    fn measure(&self, state: &FactorState, linearisation_point: &Vector<Float>) -> Vector<Float> {
        match self {
            // Self::Pose(f) => f.measure(state, x),
            Self::Dynamic(f) => f.measure(state, linearisation_point),
            Self::InterRobot(f) => f.measure(state, linearisation_point),
            Self::Obstacle(f) => f.measure(state, linearisation_point),
            Self::Tracking(f) => f.measure(state, linearisation_point),
        }
    }

    fn skip(&self, state: &FactorState) -> bool {
        match self {
            Self::Dynamic(f) => f.skip(state),
            Self::InterRobot(f) => f.skip(state),
            Self::Obstacle(f) => f.skip(state),
            Self::Tracking(f) => f.skip(state),
        }
    }

    fn jacobian_delta(&self) -> Float {
        match self {
            Self::Dynamic(f) => f.jacobian_delta(),
            Self::InterRobot(f) => f.jacobian_delta(),
            Self::Obstacle(f) => f.jacobian_delta(),
            Self::Tracking(f) => f.jacobian_delta(),
        }
    }

    // TODO: not used so maybe just remove
    fn linear(&self) -> bool {
        match self {
            Self::Dynamic(f) => f.linear(),
            Self::InterRobot(f) => f.linear(),
            Self::Obstacle(f) => f.linear(),
            Self::Tracking(f) => f.linear(),
        }
    }

    fn neighbours(&self) -> usize {
        match self {
            FactorKind::InterRobot(f) => f.neighbours(),
            FactorKind::Dynamic(f) => f.neighbours(),
            FactorKind::Obstacle(f) => f.neighbours(),
            FactorKind::Tracking(f) => f.neighbours(),
        }
    }
}

/// The state of the factor
/// Struct encapsulating all the internal state of a factor, than every variant
/// shares
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
    /// TODO: only used in `DynamicFactor` maybe move it there
    pub strength: Float,

    /// Cached value of the factors jacobian function
    /// called `J_` in **gbpplanner**
    /// TODO: wrap in Option<>
    /// TODO: not used anywhere remove
    pub cached_jacobian: Matrix<Float>,

    /// Cached value of the factors jacobian function
    /// called `h_` in **gbpplanner**
    /// TODO: wrap in Option<>
    /// TODO: not used anywhere remove
    pub cached_measurement: Vector<Float>,
    /// Set to true after the first call to `self.update()`
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

impl std::fmt::Display for FactorState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write!(
        //     f,
        //     "initial_measurement:\n{}",
        //     self.initial_measurement.pretty_format()
        // )?;

        write!(
            f,
            "{}",
            pretty_format_vector!("initial measurement", &self.initial_measurement, None)
        )?;

        // write!(
        //     f,
        //     "measurement_precision:\n{}",
        //     self.measurement_precision.pretty_format()
        // )?;

        write!(
            f,
            "{}",
            pretty_format_matrix!("measurement precision", &self.measurement_precision, None)
        )?;
        write!(
            f,
            "{}",
            pretty_format_vector!("linearisation point", &self.linearisation_point, None)
        )?;
        // write!(
        //     f,
        //     "linearisation_point:\n{}",
        //     self.linearisation_point.pretty_format()
        // )?;
        write!(
            f,
            "cached_jacobian:\n{}",
            self.cached_jacobian.pretty_format()
        )?;
        write!(
            f,
            "cached_measurement:\n{}",
            self.cached_measurement.pretty_format()
        )?;
        writeln!(f, "strength: {:?}", self.strength)?;
        writeln!(f, "initialized: {:?}", self.initialized)
    }
}

impl FactorGraphNode for FactorNode {
    fn remove_connection_to(
        &mut self,
        factorgraph_id: super::factorgraph::FactorGraphId,
    ) -> Result<(), RemoveConnectionToError> {
        let connections_before = self.inbox.len();
        self.inbox
            .retain(|variable_id, _| variable_id.factorgraph_id != factorgraph_id);
        let connections_after = self.inbox.len();

        let no_connections_removed = connections_before == connections_after;
        if no_connections_removed {
            Err(RemoveConnectionToError)
        } else {
            Ok(())
        }
    }

    #[inline(always)]
    fn messages_sent(&self) -> MessagesSent {
        self.message_count.sent
    }

    #[inline(always)]
    fn messages_received(&self) -> MessagesReceived {
        self.message_count.received
    }

    #[inline(always)]
    fn reset_message_count(&mut self) {
        self.message_count.reset();
    }
}

// impl std::fmt::Display for FactorNode {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "factorgraph_id: {:?}", self.factorgraph_id)?;
//         write!(f, "node_index: {:?}", self.node_index)?;
//     }
// }
