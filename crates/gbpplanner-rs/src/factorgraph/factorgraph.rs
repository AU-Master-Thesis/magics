use std::ops::{Range, RangeBounds};

use bevy::{
    ecs::{component::Component, entity::Entity},
    log::{debug, info},
};
// use gbp_linalg::Float;
use gbp_linalg::prelude::*;
use petgraph::{stable_graph::EdgeReference, visit::EdgeRef, Undirected};
use typed_floats::StrictlyPositiveFinite;

use super::{
    factor::{
        dynamic, interrobot::InterRobotFactor, obstacle::ObstacleFactor, tracking::TrackingFactor,
        Factor, FactorKind, FactorNode,
    },
    id::{FactorId, VariableId},
    message::{FactorToVariableMessage, VariableToFactorMessage},
    node::{FactorGraphNode, Node, NodeKind, RemoveConnectionToError},
    prelude::Message,
    variable::VariableNode,
    MessageCount, MessagesReceived, MessagesSent,
};

/// type alias used to represent the id of the factorgraph
/// Since we use **Bevy** we can use the `Entity` id of the whatever entity the
/// the factorgraph is attached to as a Component, as its unique identifier.
pub type FactorGraphId = Entity;

/// Type parameter setting the upper bound for the size of the graph
/// u16 -> 2^16 -1 = 65535
type IndexSize = u16;
/// The type used to represent indices into the nodes of the factorgraph.
/// This is just a type alias for `petgraph::graph::NodeIndex`, but
/// we make an alias for it here, such that it is easier to use the same
/// index type across modules, as the various node index types `petgraph`
/// are not interchangeable.
pub type NodeIndex = petgraph::stable_graph::NodeIndex<IndexSize>;
/// The type used to represent indices into the nodes of the factorgraph.
pub type EdgeIndex = petgraph::stable_graph::EdgeIndex<IndexSize>;
/// A factorgraph is an undirected graph
pub type Graph = petgraph::stable_graph::StableGraph<Node, (), Undirected, IndexSize>;

/// A newtype used to enforce type safety of the indices of the factors in the
/// factorgraph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, derive_more::From, derive_more::Deref)]
pub struct FactorIndex(pub NodeIndex);

impl From<FactorIndex> for usize {
    fn from(index: FactorIndex) -> Self {
        index.0.index()
    }
}

/// A newtype used to enforce type safety of the indices of the variables in the
/// factorgraph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, derive_more::From, derive_more::Deref)]
pub struct VariableIndex(pub NodeIndex);

impl From<VariableIndex> for usize {
    fn from(index: VariableIndex) -> Self {
        index.0.index()
    }
}

/// A factor graph is a bipartite graph consisting of two types of nodes:
/// factors and variables.
#[derive(Component, Debug)]
// #[cfg_attr(feature = "bevy", derive(Component))]
pub struct FactorGraph {
    /// The id of the factorgraph. We store a copy of it here, for convenience.
    /// **Invariants**:
    /// - The id of the factorgraph is unique among all factorgraphs in the
    ///   system.
    /// - The id does not change during the lifetime of the factorgraph.
    id:    FactorGraphId,
    /// The underlying graph data structure
    graph: Graph,

    message_count:    MessageCount,
    /// In **gbpplanner** the sequence in which variables are inserted/created
    /// in the graph is meaningful. `self.graph` does not capture this
    /// ordering, so we use an extra vector to manage the order in which
    /// variables are inserted/removed from the graph.
    ///
    /// **IMPORTANT** we have  to manually ensure the invariant that
    /// `self.graph` and this field is consistent at all time.
    variable_indices: Vec<NodeIndex>,
    /// List of indices of the factors in the graph. Order is not important.
    /// Used to speed up iteration over factors.
    factor_indices:   Vec<NodeIndex>,

    /// List of indices of the interrobot factors in the graph. Order is not
    /// important. Used to speed up iteration over interrobot factors.
    /// When querying for number of external messages sent
    interrobot_factor_indices: Vec<NodeIndex>,

    /// List of indices of the obstacle factors in the graph.
    /// Order matches the order of variables, such that index `i` in
    /// `obstacle_factor_indices` corresponds to index `i` in
    /// `variable_indices`. Used to speed up iteration over obstacle
    /// factors.
    obstacle_factor_indices: Vec<NodeIndex>,

    /// List of indices of the dynamic factors in the graph.
    /// Used to speed up iteration over dynamic factors.
    dynamic_factor_indices: Vec<NodeIndex>,

    /// List of indices of the tracking factors in the graph.
    /// Used to speed up iteration over tracking factors.
    tracking_factor_indices: Vec<NodeIndex>,
}

// macro_rules! internal_factor_iteration_inner {
//     // ($indices:ident) => {
//     ($indices:expr) => {
//         for i in 0..$indices.len() {
//             let ix = $indices[i];
//             let node = &mut self.graph[ix];
//             let factor = node.factor_mut();
//             let variable_messages = factor.update();
//             let factor_id = FactorId::new(self.id, FactorIndex(ix));

//             for (variable_id, message) in variable_messages {
//                 debug_assert_eq!(
//                     variable_id.factorgraph_id, self.id,
//                     "non interrobot factors can only have variable neighbours
// in the same graph"                 );
//                 let variable = self.variable_mut(variable_id.variable_index);
//                 variable.receive_message_from(factor_id, message);
//             }
//         }
//     };
// }

impl FactorGraph {
    /// Construct a new empty factorgraph with a given id
    #[must_use]
    pub fn new(id: FactorGraphId) -> Self {
        Self {
            id,
            graph: Graph::with_capacity(0, 0),
            message_count: MessageCount::default(),
            variable_indices: Vec::new(),
            factor_indices: Vec::new(),
            interrobot_factor_indices: Vec::new(),
            obstacle_factor_indices: Vec::new(),
            dynamic_factor_indices: Vec::new(),
            tracking_factor_indices: Vec::new(),
        }
    }

    /// Construct a new empty factorgraph with the specified capacity
    /// for nodes and edges.
    #[must_use]
    pub fn with_capacity(id: FactorGraphId, nodes: usize, edges: usize) -> Self {
        Self {
            id,
            graph: Graph::with_capacity(nodes, edges),
            variable_indices: Vec::with_capacity(nodes),
            factor_indices: Vec::with_capacity(edges),
            message_count: MessageCount::default(),
            interrobot_factor_indices: Vec::new(),
            obstacle_factor_indices: Vec::new(),
            dynamic_factor_indices: Vec::new(),
            tracking_factor_indices: Vec::new(),
        }
    }

    /// Returns the `FactorGraphId` of the factorgraph
    #[inline(always)]
    #[must_use]
    pub const fn id(&self) -> FactorGraphId {
        self.id
    }

    /// Adds a variable to the factorgraph
    /// Returns the index of the variable in the factorgraph
    #[allow(clippy::missing_panics_doc)]
    pub fn add_variable(&mut self, variable: VariableNode) -> VariableIndex {
        let node = Node::new(self.id, NodeKind::Variable(variable));
        let node_index = self.graph.add_node(node);
        self.variable_indices.push(node_index);
        self.graph[node_index]
            .as_variable_mut()
            .expect("just added the variable to the graph in the previous statement")
            .set_node_index(node_index);
        debug!(
            "added a variable with node_index: {:?} to factorgraph: {:?}",
            node_index, self.id
        );
        node_index.into()
    }

    #[allow(clippy::missing_panics_doc)]
    /// Adds a factor to the factorgraph
    /// Returns the index of the factor in the factorgraph
    pub fn add_factor(&mut self, factor: FactorNode) -> FactorIndex {
        let node = Node::new(self.id, NodeKind::Factor(factor));
        let node_index = self.graph.add_node(node);

        let factor = self.graph[node_index]
            .as_factor_mut()
            .expect("just added the factor to the graph in the previous statement");
        factor.set_node_index(node_index);

        self.factor_indices.push(node_index);
        match factor.kind {
            FactorKind::InterRobot(_) => self.interrobot_factor_indices.push(node_index),
            FactorKind::Dynamic(_) => self.dynamic_factor_indices.push(node_index),
            FactorKind::Obstacle(_) => self.obstacle_factor_indices.push(node_index),
            FactorKind::Tracking(_) => self.tracking_factor_indices.push(node_index),
        }

        node_index.into()
    }

    /// Number of nodes in the factorgraph
    ///
    /// **Computes in O(1) time**
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.graph.node_count()
    }

    /// Returns true if the factorgraph contains no nodes
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// A count over the number of variables and factors in the factorgraph
    ///
    /// **Computes in O(1) time**
    #[must_use]
    pub fn node_count(&self) -> NodeCount {
        NodeCount {
            factors:   self.factor_indices.len(),
            variables: self.variable_indices.len(),
        }
    }

    /// Number of edges in the factorgraph
    ///
    /// **Computes in O(1) time**
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    /// Returns the number of the different factors in the factorgraph
    /// **Computes in O(1) time**
    pub fn factor_count(&self) -> FactorCount {
        FactorCount {
            obstacle:   self.obstacle_factor_indices.len(),
            interrobot: self.interrobot_factor_indices.len(),
            dynamic:    self.dynamic_factor_indices.len(),
            tracking:   self.tracking_factor_indices.len(),
        }
    }

    /// Returns the number of messages sent and received by the factorgraph
    /// **Computes in O(1) time**
    pub fn message_count(&self) -> MessageCount {
        self.message_count
    }

    /// go through all nodes, and remove their individual connection to the
    /// other factorgraph if none of the nodes has a connection to the other
    /// factorgraph, then return and Error.
    pub fn remove_connection_to(
        &mut self,
        factorgraph_id: FactorGraphId,
    ) -> Result<(), RemoveConnectionToError> {
        let mut connections_removed: usize = 0;
        for node in self.graph.node_weights_mut() {
            if node.remove_connection_to(factorgraph_id).is_ok() {
                connections_removed += 1;
            }
        }

        if connections_removed == 0 {
            Err(RemoveConnectionToError)
        } else {
            Ok(())
        }
    }

    /// Add an edge between nodes `a` and `b` in the factorgraph.
    ///
    /// **invariants**:
    /// - Both `a` and `b` must already be in the factorgraph. Panics if any of
    ///   the nodes does not exist.
    pub fn add_internal_edge(&mut self, variable_id: VariableId, factor_id: FactorId) -> EdgeIndex {
        // let message_to_factor = {
        let Some(variable) = self.graph[variable_id.variable_index.0].as_variable_mut() else {
            panic!("the variable index either does not exist or does not point to a variable node");
        };
        // TODO: explain why we send an empty message
        variable.receive_message_from(factor_id, Message::empty());

        let node = &mut self.graph[factor_id.factor_index.0];
        match node.kind {
            NodeKind::Factor(ref mut factor) => {
                // NOTE: If this message were not empty, half a variable iteration will have
                // happened manually in secret, which is not wanted
                factor.receive_message_from(variable_id, Message::empty());
            }
            NodeKind::Variable(_) => {
                panic!("the factor index either does not exist or does not point to a factor node")
            }
        }

        self.graph
            .add_edge(variable_id.variable_index.0, factor_id.factor_index.0, ())
    }

    /// Add an external edge between a variable in this factorgraph and an
    /// interrobot factor belonging to another factorgraph
    ///
    /// # Panics
    ///
    /// - Panics if the variable index does not point to an existing variable
    /// - Panics if the factor belongs to the this factorgraph, and not an
    ///   external one
    pub fn add_external_edge(&mut self, factor_id: FactorId, nth_variable_index: usize) {
        let variable_index = self
            .nth_variable_index(nth_variable_index)
            .expect("The variable index exist");
        let variable = self.graph[variable_index.0]
            .as_variable_mut()
            .expect("The variable index points to a variable node");

        // debug!(
        //     "adding external edge from {:?} to {:?} in factorgraph {:?}",
        //     variable_index, factor_id, self.id
        // );
        variable.receive_message_from(factor_id, Message::empty());
    }

    /// Get the index of the nth variable in the factorgraph
    /// Returns `None` if the index is out of bounds
    #[inline]
    pub fn nth_variable_index(&self, index: usize) -> Option<VariableIndex> {
        self.variable_indices.get(index).copied().map(VariableIndex)
    }

    /// Get the index and a reference to the nth variable in the factorgraph
    /// Returns `None` if the index is out of bounds
    pub fn nth_variable(&self, index: usize) -> Option<(VariableIndex, &VariableNode)> {
        let variable_index = self.nth_variable_index(index)?;
        let node = &self.graph[variable_index.0];
        let variable = node.as_variable()?;
        Some((variable_index, variable))
    }

    /// Get the index and a mutable reference to the nth variable in the
    /// factorgraph Returns `None` if the index is out of bounds
    pub fn nth_variable_mut(&mut self, index: usize) -> Option<(VariableIndex, &mut VariableNode)> {
        let variable_index = self.nth_variable_index(index)?;
        let node = &mut self.graph[variable_index.0];
        let variable = node.as_variable_mut()?;
        Some((variable_index, variable))
    }

    pub(crate) fn delete_interrobot_factors_connected_to(&mut self, other: FactorGraphId) {
        // ) -> Result<(), &'static str> {
        // 1. Find all interrobot factors connected to the robot with id `other`
        // and remove them from the graph

        let mut factor_indices_to_remove = Vec::new();

        #[allow(clippy::needless_collect)]
        for node_index in self.graph.node_indices().collect::<Vec<_>>() {
            let node = &mut self.graph[node_index];
            if node.is_variable() {
                // 2. remove the the message from the external factor
                let variable = node
                    .as_variable_mut()
                    .expect("A variable index should point to a Variable in the graph");
                variable
                    .inbox
                    .retain(|factor_id, _| factor_id.factorgraph_id != other);
                continue;
            }

            let factor = node
                .as_factor()
                .expect("A factor index should point to a Factor in the graph");
            let Some(interrobot) = factor.kind.try_as_inter_robot_ref() else {
                continue;
            };

            if interrobot.external_variable.factorgraph_id == other {
                self.graph.remove_node(node_index).expect(
                    "The node index was retrieved from the graph in the previous statement",
                );

                self.factor_indices.retain(|&idx| idx != node_index);

                factor_indices_to_remove.push(FactorIndex(node_index));
            }
        }

        #[allow(clippy::needless_collect)]
        for node_index in self.graph.node_indices().collect::<Vec<_>>() {
            let node = &mut self.graph[node_index];
            if !node.is_variable() {
                continue;
            }

            let variable = node
                .as_variable_mut()
                .expect("A variable index should point to a Variable in the graph");

            for factor_index in &factor_indices_to_remove {
                variable
                    .inbox
                    .remove(&FactorId::new(self.id, *factor_index));
            }
        }
    }

    pub(crate) fn delete_messages_from_interrobot_factor_at(&mut self, other: FactorGraphId) {
        // PERF: avoid allocation
        #[allow(clippy::needless_collect)]
        for node_index in self.graph.node_indices().collect::<Vec<_>>() {
            let node = &mut self.graph[node_index];
            let Some(variable) = node.as_variable_mut() else {
                continue;
            };
            variable
                .inbox
                .retain(|factor_id, _| factor_id.factorgraph_id != other);
        }
    }

    pub fn variable_indices_ordered_by_creation(&self) -> impl Iterator<Item = NodeIndex> + '_ {
        self.variable_indices.iter().copied()
    }

    // /// Return an ordered interval of variables indices.
    // /// The indices are ordered by the order in which they are inserted into the
    // /// factorgraph. Returns `None`, if the end of the  **range** exceeds
    // /// the number of variables in the factorgraph.
    // pub fn variable_indices_ordered_by_creation<R: RangeBounds<usize>>(
    //     &self,
    //     range: R, // range: Range<usize>,
    // ) -> Option<Vec<NodeIndex>> {
    //     let start = match range.start_bound() {
    //         std::ops::Bound::Included(start) => *start,
    //         std::ops::Bound::Excluded(_) => unreachable!(),
    //         std::ops::Bound::Unbounded => 0,
    //     };
    //     let end = match range.end_bound() {
    //         std::ops::Bound::Included(end) => end + 1,
    //         std::ops::Bound::Excluded(end) => *end,
    //         std::ops::Bound::Unbounded => self.variable_indices.len(),
    //     };
    //
    //     let within_range = range.end <= self.variable_indices.len();
    //     if within_range {
    //         Some(
    //             self.variable_indices
    //                 .iter()
    //                 .skip(range.start)
    //                 .take(range.end - range.start)
    //                 .copied()
    //                 .collect::<Vec<_>>(),
    //         )
    //     } else {
    //         None
    //     }
    // }

    /// Change the prior of the variable with the given index
    /// Returns the messages to send to any external factors connected to it, if
    /// any
    #[must_use]
    pub fn change_prior_of_variable(
        &mut self,
        variable_index: VariableIndex,
        new_mean: Vector<Float>,
    ) -> Vec<VariableToFactorMessage> {
        let variable_id = VariableId::new(self.id, variable_index);
        let Some(variable) = self.get_variable_mut(variable_id.variable_index) else {
            panic!("the variable index either does not exist or does not point to a variable node");
        };

        let factor_messages = variable.change_prior(&new_mean);
        let mut messages_to_external_factors: Vec<VariableToFactorMessage> = Vec::new();

        for (factor_id, message) in factor_messages {
            let in_internal_graph = factor_id.factorgraph_id == self.id;
            if in_internal_graph {
                // If the factor is an interrobot factor, it can be missing if the robot the
                // graph is connected to despawns, so we only have the factor
                // receive the message if it exists
                if let Some(factor) = self.get_factor_mut(factor_id.factor_index) {
                    factor.receive_message_from(variable_id, message);
                }
            } else {
                messages_to_external_factors.push(VariableToFactorMessage {
                    from: variable_id,
                    to: factor_id,
                    message,
                });
            }
        }

        // PERF: pass a mutable reference to the vec of messages, instead of allocating
        // and returning
        messages_to_external_factors
    }

    /// Returns a refenrence to the factor with the given index.
    /// Returns `None`, if the factor does not exist.
    pub fn get_factor(&self, index: FactorIndex) -> Option<&FactorNode> {
        self.graph
            .node_weight(index.0)
            .and_then(|node| node.as_factor())
    }

    /// Returns a mutable refenrence to the factor with the given index.
    /// Returns `None`, if the factor does not exist.
    pub fn get_factor_mut(&mut self, index: FactorIndex) -> Option<&mut FactorNode> {
        self.graph
            .node_weight_mut(*index)
            .and_then(|node| node.as_factor_mut())
    }

    /// Returns a refenrence to the variable with the given index.
    /// Returns `None`, if the variable does not exist.
    pub fn get_variable(&self, index: VariableIndex) -> Option<&VariableNode> {
        self.graph
            .node_weight(*index)
            .and_then(|node| node.as_variable())
    }

    /// Returns a mutable refenrence to the variable with the given index.
    /// Returns `None`, if the variable does not exist.
    pub fn get_variable_mut(&mut self, index: VariableIndex) -> Option<&mut VariableNode> {
        self.graph
            .node_weight_mut(*index)
            .and_then(|node| node.as_variable_mut())
    }

    /// Returns a refenrence to the variable with the given index.
    ///
    /// # Panics
    ///
    /// Panic if the `index` does not point to an existing variable
    #[inline]
    fn variable(&self, index: VariableIndex) -> &VariableNode {
        self.get_variable(index)
            .expect("variable index points to a variable in the graph")
    }

    /// Returns a mutable refenrence to the variable with the given index.
    ///
    /// # Panics
    ///
    /// Panic if the `index` does not point to an existing variable
    #[inline]
    fn variable_mut(&mut self, index: VariableIndex) -> &mut VariableNode {
        self.get_variable_mut(index)
            .expect("variable index points to a variable in the graph")
    }

    /// Get the index of the first variable in the factorgraph and a reference
    /// to it to it Returns `None` if the factorgraph contains no variables
    #[inline(always)]
    pub fn first_variable(&self) -> Option<(VariableIndex, &VariableNode)> {
        self.nth_variable(0usize)
    }

    /// Get the index of the last variable in the factorgraph and a mutable
    /// reference to it to it Returns `None` if the factorgraph contains no
    /// variables
    #[inline(always)]
    pub fn last_variable(&self) -> Option<(VariableIndex, &VariableNode)> {
        if self.variable_indices.is_empty() {
            None
        } else {
            self.nth_variable(self.variable_indices.len() - 1)
        }
    }

    /// Get the index of the last variable in the factorgraph and a mutable
    /// reference to it to it Returns `None` if the factorgraph contains no
    /// variables
    #[inline(always)]
    pub fn last_variable_mut(&mut self) -> Option<(VariableIndex, &mut VariableNode)> {
        if self.variable_indices.is_empty() {
            None
        } else {
            self.nth_variable_mut(self.variable_indices.len() - 1)
        }
    }

    /// Variable Iteration in Gaussian Belief Propagation (GBP).
    /// For each variable in the factorgraph:
    /// 1. Use received messages from connected factors to update the variable
    ///    belief
    /// 2. Create and send outgoing messages to the connected factors
    /// # Arguments
    /// * `robot_id` - The id of the robot that this factorgraph belongs to
    /// # Returns
    /// Messages that need to be sent to any externally connected factors
    /// This can be empty if there are no externally connected factors
    /// A [`FactorGraph`] does not have a handle to the factorgraphs of other
    /// robots, so it cannot send messages to them. It is up to the caller
    /// of this method to send the messages to the correct robot. # Panics
    /// This method panics if a variable has not received any messages from its
    /// connected factors. As this indicates that the factorgraph is not
    /// correctly constructed.
    #[must_use]
    pub fn variable_iteration(&mut self) -> Vec<VariableToFactorMessage> {
        let mut messages_to_external_factors: Vec<VariableToFactorMessage> = Vec::new();

        for &node_index in &self.variable_indices {
            let node = &mut self.graph[node_index];
            let variable = node.as_variable_mut().expect(
                "self.variable_indices should only contain indices that point to Variables in the \
                 graph",
            );
            let variable_index = VariableIndex(node_index);

            let factor_messages = variable.update_belief_and_create_factor_responses();
            debug_assert!(
                !factor_messages.is_empty(),
                "The factorgraph {:?} with variable {:?} did not receive any messages from its \
                 connected factors",
                self.id,
                variable_index
            );

            let variable_id = VariableId::new(self.id, variable_index);
            for (factor_id, message) in factor_messages {
                let in_internal_graph = factor_id.factorgraph_id == self.id;
                if in_internal_graph {
                    // Send the messages to the connected factors within the same factorgraph
                    // self.graph.
                    if !self.factor_indices.contains(&factor_id.factor_index.0) {
                        info!(
                            "factor_id: {:?} does not exist in the factorgraph {:?}",
                            factor_id, self.id
                        );
                        continue;
                    }

                    self.graph[factor_id.factor_index.0]
                        .as_factor_mut()
                        .expect("A factor can only have variables as neighbours")
                        .receive_message_from(variable_id, message);
                } else {
                    messages_to_external_factors.push(VariableToFactorMessage {
                        from: variable_id,
                        to: factor_id,
                        message,
                    });
                }
            }
        }

        // Return the messages to be sent to the connected factors in other factorgraphs
        // The caller is responsible for sending these messages to the correct
        // factorgraphs
        messages_to_external_factors
    }

    pub fn internal_factor_iteration(&mut self) {
        for i in 0..self.factor_indices.len() {
            let ix = self.factor_indices[i];
            let node = &mut self.graph[ix];
            let factor = node.factor_mut();
            // Ignore if interrobot factor
            if let FactorKind::InterRobot(_) = factor.kind {
                continue;
            }
            if !factor.enabled {
                continue;
            }

            let variable_messages = factor.update();
            let factor_id = FactorId::new(self.id, FactorIndex(ix));

            for (variable_id, message) in variable_messages {
                let variable = self.variable_mut(variable_id.variable_index);
                variable.receive_message_from(factor_id, message);
            }
        }
        // for i in 0..self.dynamic_factor_indices.len() {
        //     let ix = self.dynamic_factor_indices[i];
        //     let node = &mut self.graph[ix];
        //     let factor = node.factor_mut();
        //     let variable_messages = factor.update();
        //     let factor_id = FactorId::new(self.id, FactorIndex(ix));
        //
        //     for (variable_id, message) in variable_messages {
        //         debug_assert_eq!(
        //             variable_id.factorgraph_id, self.id,
        //             "non interrobot factors can only have variable neighbours
        // in the same graph"
        //         );
        //         let variable = self.variable_mut(variable_id.variable_index);
        //         variable.receive_message_from(factor_id, message);
        //     }
        // }
        //
        // for i in 0..self.obstacle_factor_indices.len() {
        //     let ix = self.obstacle_factor_indices[i];
        //     let node = &mut self.graph[ix];
        //     let factor = node.factor_mut();
        //     let variable_messages = factor.update();
        //     let factor_id = FactorId::new(self.id, FactorIndex(ix));
        //
        //     for (variable_id, message) in variable_messages {
        //         debug_assert_eq!(
        //             variable_id.factorgraph_id, self.id,
        //             "non interrobot factors can only have variable neighbours
        // in the same graph"
        //         );
        //         let variable = self.variable_mut(variable_id.variable_index);
        //         variable.receive_message_from(factor_id, message);
        //     }
        // }
    }

    #[must_use]
    pub fn external_factor_iteration(&mut self) -> Vec<FactorToVariableMessage> {
        // Each interrobot factor is connected to an internal variable
        // So we can preallocate a vec of length the number of interrobot factors
        let mut messages_to_external_variables: Vec<FactorToVariableMessage> =
            Vec::with_capacity(self.interrobot_factor_indices.len());

        for i in 0..self.interrobot_factor_indices.len() {
            let ix = self.interrobot_factor_indices[i];
            if !self.graph.contains_node(ix) {
                // TODO: document when this happens
                continue;
            }

            let node = &mut self.graph[ix];
            let factor = node.factor_mut();
            if !factor.enabled {
                continue;
            }

            let variable_messages = factor.update();
            let factor_id = FactorId::new(self.id, FactorIndex(ix));

            // Each interrobot factor is connected to an internal variable
            // and an external variable
            // So half the iterations should enter the if block, and the other half the else
            // block
            for (variable_id, message) in variable_messages {
                let in_internal_graph = variable_id.factorgraph_id == self.id;
                if in_internal_graph {
                    // let variable =
                    // self.variable_mut(variable_id.variable_index);
                    // variable.receive_message_from(factor_id, message);
                } else {
                    messages_to_external_variables.push(FactorToVariableMessage {
                        from: factor_id,
                        to: variable_id,
                        message,
                    });
                }
            }
        }
        messages_to_external_variables
    }

    pub fn internal_variable_iteration(&mut self) {
        for &ix in &self.variable_indices {
            let node = &mut self.graph[ix];
            let variable = node.variable_mut();
            let variable_index = VariableIndex(ix);
            let variable_id = VariableId::new(self.id, variable_index);
            // TODO: do internal only
            let factor_messages = variable.update_belief_and_create_factor_responses();

            for (factor_id, message) in factor_messages {
                let in_internal_graph = factor_id.factorgraph_id == self.id;
                if !in_internal_graph {
                    // TODO: should not happen
                    continue;
                }
                let factor = self.graph[factor_id.factor_index.0]
                    .as_factor_mut()
                    .expect("a factor only has variables as neighbours");

                if !factor.enabled {
                    continue;
                }

                factor.receive_message_from(variable_id, message);
            }
        }
    }

    // TODO(kpbaks): does this method even make sense?
    #[must_use]
    pub fn external_variable_iteration(&mut self) -> Vec<VariableToFactorMessage> {
        let mut messages_to_external_factors: Vec<VariableToFactorMessage> = Vec::new();
        for &ix in &self.variable_indices {
            let node = &mut self.graph[ix];
            let variable = node.variable_mut();
            let variable_index = VariableIndex(ix);
            let variable_id = VariableId::new(self.id, variable_index);
            // TODO: do internal only
            let factor_messages = variable.update_belief_and_create_factor_responses();

            for (factor_id, message) in factor_messages {
                let in_internal_graph = factor_id.factorgraph_id == self.id;
                if !in_internal_graph {
                    messages_to_external_factors.push(VariableToFactorMessage {
                        from: variable_id,
                        to: factor_id,
                        message,
                    });
                    // // TODO: should not happen
                    // continue;
                }
                // let factor = self.graph[factor_id.factor_index.0]
                //     .as_factor_mut()
                //     .expect("a factor only has variables as neighbours");
                //
                // factor.receive_message_from(variable_id, message);
            }
        }

        messages_to_external_factors
    }

    /// Aggregate and marginalise over all adjacent variables, and send.
    /// Aggregation: product of all incoming messages
    #[must_use]
    pub fn factor_iteration(&mut self) -> Vec<FactorToVariableMessage> {
        let mut messages_to_external_variables: Vec<FactorToVariableMessage> = Vec::new();

        for ix in &self.factor_indices {
            let node = &mut self.graph[*ix];
            let factor = node.as_factor_mut().expect(
                "self.factor_indices should only contain indices that point to Factors in the \
                 graph",
            );

            let variable_messages = factor.update();
            let factor_id = FactorId::new(self.id, FactorIndex(*ix));

            for (variable_id, message) in variable_messages {
                let in_internal_graph = variable_id.factorgraph_id == self.id;
                if in_internal_graph {
                    let variable = self.graph[variable_id.variable_index.0]
                        .as_variable_mut()
                        .expect("A factor can only have variables as neighbors");

                    variable.receive_message_from(factor_id, message);
                } else {
                    messages_to_external_variables.push(FactorToVariableMessage {
                        from: factor_id,
                        to: variable_id,
                        message,
                    });
                }
            }
        }

        // Return the messages to be sent to the connected variables in other
        // factorgraphs The caller is responsible for sending these messages to
        // the correct factorgraphs.
        messages_to_external_variables
    }

    // TODO:
    // pub fn receive_message(&mut self, from: NodeId, message: Message) {
    //     // self.messages_sent += 1;
    //     todo!()
    // }

    /// Returns the number of messages sent by all variables and factors
    #[must_use]
    pub fn messages_sent(&self) -> MessagesSent {
        self.graph
            .node_weights()
            .map(|node| node.messages_sent())
            .sum()
    }

    /// Returns the number of messages received by all variables and factors
    #[must_use]
    pub fn messages_received(&self) -> MessagesReceived {
        self.graph
            .node_weights()
            .map(|node| node.messages_received())
            .sum()
    }

    pub fn update_inter_robot_safety_distance_multiplier(
        &mut self,
        safety_distance_multiplier: StrictlyPositiveFinite<Float>,
    ) {
        for ix in &self.interrobot_factor_indices {
            let Some(node) = self.graph.node_weight_mut(*ix) else {
                continue;
            };
            // let node = &mut self.graph[*ix];
            let factor = node.as_factor_mut().expect(
                "self.factor_indices should only contain indices that point to Factors in the \
                 graph",
            );
            let FactorKind::InterRobot(ref mut interrobot) = factor.kind else {
                panic!("Expected an interrobot factor");
            };
            interrobot.update_safety_distance(safety_distance_multiplier);
        }
    }

    // pub fn receive_variable_message_from(&mut self,)
}

/// Record type used to keep track of how many factors and variables
/// there are in the factorgraph. We keep track of these counts internally in
/// the factorgraph, such a query for the counts, is **O(1)**.
#[derive(Debug, Clone, Copy, Default)]
pub struct NodeCount {
    /// Number of `Factor` nodes
    pub factors:   usize,
    /// Number of `Variable` nodes
    pub variables: usize,
}

impl NodeCount {
    /// Return the total number of nodes
    pub fn total(&self) -> usize {
        self.factors + self.variables
    }
}

/// Record type returned by `FactorGraph::factor_count()`.
#[derive(Debug, Clone, Copy, Default)]
pub struct FactorCount {
    /// Number of `ObstacleFactor`s
    pub obstacle:   usize,
    /// Number of `InterRobotFactor`s
    pub interrobot: usize,
    /// Number of `DynamicFactor`s
    pub dynamic:    usize,
    /// Number of `TrackingFactor`s
    pub tracking:   usize,
}

/// Iterator over the factors in the factorgraph.
///
/// Iterator element type is `(FactorIndex, &'a Factor)`.
///
/// Created with [`.factors()`][1]
///
/// [1]: struct.FactorGraph.html#method.factors
pub struct Factors<'fg> {
    graph: &'fg Graph,
    factor_indices: std::slice::Iter<'fg, NodeIndex>,
}

impl<'fg> Factors<'fg> {
    #[must_use]
    fn new(graph: &'fg Graph, factor_indices: &'fg [NodeIndex]) -> Self {
        Self {
            graph,
            factor_indices: factor_indices.iter(),
        }
    }
}

impl FactorGraph {
    /// Returns an iterator over the factors in the factorgraph.
    #[inline]
    #[must_use]
    pub fn factors(&self) -> Factors<'_> {
        Factors::new(&self.graph, &self.factor_indices)
    }
}

impl<'fg> Iterator for Factors<'fg> {
    type Item = (NodeIndex, &'fg FactorNode);

    fn next(&mut self) -> Option<Self::Item> {
        let &index = self.factor_indices.next()?;
        let node = &self.graph[index];
        node.as_factor().map(|factor| (index, factor))
    }
}

// pub struct InternalFactors<'graph> {
//     graph: &'graph Graph,
//     internal_factors: Box<dyn Iterator<Item = &'graph NodeIndex>>,
//     // internal_factors: &'graph dyn Iterator<Item = &'graph NodeIndex>,
// }

// impl<'graph> InternalFactors<'graph> {
//     pub fn new(graph: &'graph Graph, internal_factors: Box<dyn Iterator<Item
// = &'graph NodeIndex>>) -> Self {         // pub fn new(graph: &'graph Graph,
// internal_factors: &'graph dyn Iterator<Item         // = &'graph NodeIndex>)
// -> Self {         Self {
//             graph,
//             internal_factors,
//         }
//     }
// }

// impl<'graph> std::iter::Iterator for InternalFactors<'graph> {
//     type Item = (NodeIndex, &'graph FactorNode);

//     fn next(&mut self) -> Option<Self::Item> {
//         let index = *self.internal_factors.next()?;
//         Some((index, self.graph[index].factor()))
//     }
// }

// impl FactorGraph {
//     #[inline]
//     #[must_use]
//     // pub fn internal_factors<'graph>(&'graph self) ->
// InternalFactors<'graph> {     pub fn internal_factors(&self) ->
// InternalFactors<'_> {         let iter = self
//             .dynamic_factor_indices
//             .iter()
//             .chain(self.obstacle_factor_indices.iter());

//         InternalFactors::new(&self.graph, Box::new(iter))
//     }
// }

/// Iterator over the variables in the factorgraph.
///
/// Iterator element type is `(VariableIndex, &'a Variable)`.
///
/// Created with [`.variables()`][1]
///
/// [1]: struct.FactorGraph.html#method.variables
pub struct Variables<'fg> {
    graph: &'fg Graph,
    variable_indices: std::slice::Iter<'fg, NodeIndex>,
}

impl<'fg> Variables<'fg> {
    fn new(graph: &'fg Graph, variable_indices: &'fg [NodeIndex]) -> Self {
        Self {
            graph,
            variable_indices: variable_indices.iter(),
        }
    }
}

impl<'fg> Iterator for Variables<'fg> {
    type Item = (VariableIndex, &'fg VariableNode);

    fn next(&mut self) -> Option<Self::Item> {
        let &index = self.variable_indices.next()?;
        let node = &self.graph[index];
        node.as_variable()
            .map(|variable| (VariableIndex(index), variable))
    }
}

impl FactorGraph {
    /// Returns an iterator over the variables in the factorgraph.
    #[inline]
    #[must_use]
    pub fn variables(&self) -> Variables<'_> {
        Variables::new(&self.graph, &self.variable_indices)
    }
}

/// Iterator over the interrobot factors in the factorgraph.
///
/// Iterator element type is `(FactorIndex, &'a InterRobotFactor)`.
///
/// Created with [`.inter_robot_factors()`][1]
pub struct InterRobotFactors<'fg> {
    graph: &'fg Graph,
    factor_indices: std::slice::Iter<'fg, NodeIndex>,
}

impl<'fg> InterRobotFactors<'fg> {
    fn new(graph: &'fg Graph, factor_indices: &'fg [NodeIndex]) -> Self {
        Self {
            graph,
            factor_indices: factor_indices.iter(),
        }
    }
}

impl<'fg> Iterator for InterRobotFactors<'fg> {
    type Item = (NodeIndex, &'fg InterRobotFactor);

    fn next(&mut self) -> Option<Self::Item> {
        let &index = self.factor_indices.next()?;
        let node = &self.graph[index];
        node.as_factor()
            .and_then(|factor| factor.kind.try_as_inter_robot_ref())
            .map(|interrobot| (index, interrobot))
    }
}

impl FactorGraph {
    /// Returns an iterator over the interrobot factors in the factorgraph.
    #[inline]
    #[must_use]
    pub fn inter_robot_factors(&self) -> InterRobotFactors<'_> {
        InterRobotFactors::new(&self.graph, &self.interrobot_factor_indices)
    }
}

// pub struct VariableAndTheirInterRobotFactors<'fg,'edges> where 'edges: 'fg {
pub struct VariableAndTheirInterRobotFactors<'fg> {
    graph: &'fg Graph,
    // iter: std::iter::Zip<std::slice::Iter<'fg, NodeIndex>, std::slice::Iter<'fg, NodeIndex>>,
    // iter: impl Iterator<Item = EdgeReference<'fg, (), IndexSize>>,
    iter:  Box<dyn Iterator<Item = EdgeReference<'fg, (), IndexSize>> + 'fg>,
    // iter:  &'fg mut dyn Iterator<Item = EdgeReference<'fg, (), IndexSize>>,
    // variable_indices: std::slice::Iter<'fg, NodeIndex>,
    // edges: petgraph::stable_graph::Edges<'edges, (), Undirected, IndexSize>,
}

// impl <'fg, 'edges> VariableAndTheirInterRobotFactors<'fg, 'edges> where
// 'edges: 'fg {
impl<'fg> VariableAndTheirInterRobotFactors<'fg> {
    fn new(graph: &'fg Graph, variable_indices: &'fg [NodeIndex]) -> Self {
        let iter = variable_indices
            .iter()
            .flat_map(|var_ix| graph.edges(*var_ix));
        // let iter = variable_indices.iter().map(|var_ix|
        // graph.edges(*var_ix)).reduce(|a, b| a.chain(b));
        Self {
            graph,
            // iter,
            iter: Box::new(iter),
            // iter,
            // iter: interrobot_factor_indices.iter().zip(interrobot_factor_indices.iter()),
        }
    }
}

// impl<'fg, 'edges> Iterator for VariableAndTheirInterRobotFactors<'fg, 'edges>
// where 'edges: 'fg {
impl<'fg> Iterator for VariableAndTheirInterRobotFactors<'fg> {
    type Item = (&'fg VariableNode, &'fg InterRobotFactor);

    fn next(&mut self) -> Option<Self::Item> {
        // A variable can be connected to 0 or more interrobot factors
        // Iterate over all the interrobot factors of the current variable before moving
        // to the next variable.

        while let Some(edge_ref) = self.iter.next() {
            let source = edge_ref.source();
            let target = edge_ref.target();
            let Some(interrobot) = self.graph[target]
                .as_factor()
                .and_then(|factor| factor.kind.try_as_inter_robot_ref())
            else {
                continue;
            };

            let variable = self.graph[source].as_variable().unwrap();
            // let factor = self.graph[target].as_factor().unwrap();
            // let interrobot = factor.kind.as_inter_robot().unwrap();
            return Some((variable, interrobot));
        }

        None

        // self.iter.next().map(|edge_ref| {
        //     let source = edge_ref.source();
        //     let target = edge_ref.target();
        //     let variable = self.graph[source].as_variable().unwrap();
        //     let factor = self.graph[target].as_factor().unwrap();
        //     let interrobot = factor.kind.as_inter_robot().unwrap();
        //     (variable, interrobot)
        //
        //     // let var_ix = edge_ref.source();
        //     // let factor_ix = edge_ref.target();
        //     // let variable = self.graph[var_ix].as_variable().unwrap();
        //     // let factor = self.graph[factor_ix].as_factor().unwrap();
        //     // let interrobot = factor.kind.as_inter_robot().unwrap();
        //     // (variable, interrobot)
        // })

        // None
    }
}

impl FactorGraph {
    /// Returns an iterator over the variable and their connected interrobot
    /// factors in the factorgraph
    #[inline]
    #[must_use]
    pub fn variable_and_inter_robot_factors(&self) -> VariableAndTheirInterRobotFactors<'_> {
        VariableAndTheirInterRobotFactors::new(&self.graph, &self.variable_indices)
    }
}

/// Iterator over the variable and their connected obstacle factors in the
/// factorgraph
pub struct VariableAndTheirObstacleFactors<'fg> {
    graph: &'fg Graph,
    // variable_indices: std::slice::Iter<'a, NodeIndex>,
    // obstacle_factor_indices: std::slice::Iter<'a, NodeIndex>,
    pairs: std::iter::Zip<std::slice::Iter<'fg, NodeIndex>, std::slice::Iter<'fg, NodeIndex>>,
}

impl<'fg> VariableAndTheirObstacleFactors<'fg> {
    fn new(
        graph: &'fg Graph,
        variable_indices: &'fg [NodeIndex],
        obstacle_factor_indices: &'fg [NodeIndex],
    ) -> Self {
        Self {
            graph,
            pairs: variable_indices.iter().zip(obstacle_factor_indices.iter()),
        }
    }
}

impl<'fg> Iterator for VariableAndTheirObstacleFactors<'fg> {
    type Item = (&'fg VariableNode, &'fg ObstacleFactor);

    fn next(&mut self) -> Option<Self::Item> {
        let (&variable_index, &factor_index) = self.pairs.next()?;
        let variable = &self.graph[variable_index]
            .as_variable()
            .expect("variable index points to a variable node");
        let obstacle_factor = &self.graph[factor_index]
            .as_factor()
            .expect("factor index points to a factor node")
            .kind
            .try_as_obstacle_ref()
            .expect("factors In VariableAndTheirObstacleFactors are obstacle factors");

        Some((variable, obstacle_factor))
    }
}

/// Iterator over the variable and their connected tracking factors in the
/// factorgraph
pub struct VariableAndTheirTrackingFactors<'fg> {
    graph: &'fg Graph,
    // variable_indices: std::slice::Iter<'a, NodeIndex>,
    // tracking_factor_indices: std::slice::Iter<'a, NodeIndex>,
    pairs: std::iter::Zip<std::slice::Iter<'fg, NodeIndex>, std::slice::Iter<'fg, NodeIndex>>,
}

impl<'fg> VariableAndTheirTrackingFactors<'fg> {
    fn new(
        graph: &'fg Graph,
        variable_indices: &'fg [NodeIndex],
        tracking_factor_indices: &'fg [NodeIndex],
    ) -> Self {
        Self {
            graph,
            pairs: variable_indices.iter().zip(tracking_factor_indices.iter()),
        }
    }
}

impl<'fg> Iterator for VariableAndTheirTrackingFactors<'fg> {
    type Item = (&'fg VariableNode, &'fg TrackingFactor);

    fn next(&mut self) -> Option<Self::Item> {
        let (&variable_index, &factor_index) = self.pairs.next()?;
        let variable = &self.graph[variable_index]
            .as_variable()
            .expect("variable index points to a variable node");
        let tracking_factor = &self.graph[factor_index]
            .as_factor()
            .expect("factor index points to a factor node")
            .kind
            .try_as_tracking_ref()
            .expect("factors In VariableAndTheirTrackingFactors are tracking factors");

        Some((variable, tracking_factor))
    }
}

impl FactorGraph {
    /// Returns an iterator over the variable and their obstacle factors in the
    /// factorgraph.
    #[inline]
    #[must_use]
    pub fn variable_and_their_obstacle_factors(&self) -> VariableAndTheirObstacleFactors<'_> {
        VariableAndTheirObstacleFactors::new(
            &self.graph,
            &self.variable_indices[1..self.variable_indices.len() - 1],
            &self.obstacle_factor_indices,
        )
    }

    /// Returns an iterator over the variable and their tracking factors in the
    /// factorgraph.
    #[inline]
    #[must_use]
    pub fn variable_and_their_tracking_factors(&self) -> VariableAndTheirTrackingFactors<'_> {
        VariableAndTheirTrackingFactors::new(
            &self.graph,
            &self.variable_indices[1..],
            &self.tracking_factor_indices,
        )
    }
}

// impl<'fg> std::ops::Index<FactorIndex> for FactorGraph<'fg> {
//     type Output = FactorNode<'fg>;

//     fn index(&self, index: FactorIndex) -> &'fg Self::Output {
//         let node: &'fg Node<'fg> = &self.graph[index.0];
//         node.as_factor()
//             .expect("a factor index points to a factor node in the graph")
//     }
// }

// impl std::ops::Index<VariableIndex> for FactorGraph {
//     type Output = VariableNode;

//     fn index(&self, index: VariableIndex) -> &Self::Output {
//         self.graph[index.0]
//             .as_variable()
//             .expect("a variable index points to a variable node in the
// graph")     }
// }

/// Iterator over the neighbours of a variable in the factorgraph
pub struct VariableNeighboursDyn<'fg> {
    graph:      &'fg Graph,
    neighbours: petgraph::stable_graph::Neighbors<'fg, (), IndexSize>,
}

impl<'fg> Iterator for VariableNeighboursDyn<'fg> {
    type Item = &'fg dyn Factor;

    fn next(&mut self) -> Option<Self::Item> {
        self.neighbours.next().map(|index| {
            &self.graph[index]
                .as_factor()
                .expect("a variable only has factors as neighbours")
                .kind as &dyn Factor
        })
    }
}

impl FactorGraph {
    /// Returns an iterator over the factor neighbours of a variable
    /// If the variable does not exist in the factorgraph, returns None
    pub fn variable_neighbours_dyn(
        &self,
        variable_index: VariableIndex,
    ) -> Option<VariableNeighboursDyn<'_>> {
        let node_ix = variable_index.0;
        self.graph.node_weight(node_ix)?;

        let neighbours = self.graph.neighbors(node_ix);

        Some(VariableNeighboursDyn {
            graph: &self.graph,
            neighbours,
        })
    }
}

pub struct VariableNeighbours<'fg> {
    graph:      &'fg Graph,
    neighbours: petgraph::stable_graph::Neighbors<'fg, (), IndexSize>,
}

impl<'fg> Iterator for VariableNeighbours<'fg> {
    type Item = &'fg FactorNode;

    fn next(&mut self) -> Option<Self::Item> {
        self.neighbours.next().map(|index| {
            self.graph[index]
                .as_factor()
                .expect("a variable only has factors as neighbours")
        })
    }
}

impl FactorGraph {
    /// Returns an iterator over the factor neighbours of a variable
    /// If the variable does not exist in the factorgraph, returns None
    pub fn variable_neighbours(
        &self,
        variable_index: VariableIndex,
    ) -> Option<VariableNeighbours<'_>> {
        let node_ix = variable_index.0;
        self.graph.node_weight(node_ix)?;

        let neighbours = self.graph.neighbors(node_ix);

        Some(VariableNeighbours {
            graph: &self.graph,
            neighbours,
        })
    }
}

/// Iterator over the neighbours of a factor in the factorgraph
pub struct FactorNeighbours<'fg> {
    graph:      &'fg Graph,
    neighbours: petgraph::stable_graph::Neighbors<'fg, (), IndexSize>,
}

impl<'fg> Iterator for FactorNeighbours<'fg> {
    type Item = &'fg VariableNode;

    fn next(&mut self) -> Option<Self::Item> {
        self.neighbours
            .next()
            .map(|index| self.graph[index].as_variable().unwrap())
    }
}

impl FactorGraph {
    /// Returns an iterator over the variable neighbours of a factor
    /// If the factor does not exist in the factorgraph, returns None
    pub fn factor_neighbours(&self, factor_index: FactorIndex) -> Option<FactorNeighbours<'_>> {
        let node_ix = factor_index.0;
        self.graph.node_weight(node_ix)?;

        let neighbours = self.graph.neighbors(node_ix);

        Some(FactorNeighbours {
            graph: &self.graph,
            neighbours,
        })
    }
}

/// Iterator over the factors in the factorgraph
pub struct FactorsDyn<'fg> {
    graph: &'fg Graph,
    iter:  std::slice::Iter<'fg, NodeIndex>,
}

impl<'fg> Iterator for FactorsDyn<'fg> {
    type Item = &'fg dyn Factor;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|index| {
            &self.graph[*index]
                .as_factor()
                .expect("factor indices only point to factors")
                .kind as &dyn Factor
        })
    }
}

impl FactorGraph {
    /// Returns an iterator over the factors in the factorgraph
    pub fn factors_dyn(&self) -> FactorsDyn<'_> {
        FactorsDyn {
            graph: &self.graph,
            iter:  self.factor_indices.iter(),
        }
    }
}

impl FactorGraph {
    /// Modify the tracking factors in the factorgraph
    pub fn modify_tracking_factors(&mut self, mut f: impl FnMut(&mut TrackingFactor)) {
        for ix in &self.tracking_factor_indices {
            let node = &mut self.graph[*ix];
            let factor = node.factor_mut();
            let FactorKind::Tracking(ref mut inner) = factor.kind else {
                panic!("Expected a tracking factor");
            };
            f(inner);
        }
    }
}

use super::graphviz;

impl graphviz::ExportGraph for FactorGraph {
    fn export_graph(&self) -> (Vec<super::graphviz::Node>, Vec<super::graphviz::Edge>) {
        let nodes = self
            .graph
            .node_indices()
            .map(|node_index| {
                let node = &self.graph[node_index];
                graphviz::Node {
                    index: node_index.index(),
                    kind:  match &node.kind {
                        NodeKind::Factor(factor) => match factor.kind {
                            FactorKind::Dynamic(_) => graphviz::NodeKind::DynamicFactor,
                            FactorKind::Obstacle(_) => graphviz::NodeKind::ObstacleFactor,
                            FactorKind::InterRobot(ref inner) => {
                                graphviz::NodeKind::InterRobotFactor {
                                    active: true,
                                    external_variable_id: inner.external_variable,
                                }
                            }
                            FactorKind::Tracking(_) => graphviz::NodeKind::TrackingFactor,
                        },
                        NodeKind::Variable(variable) => {
                            let [x, y] = variable.estimated_position();
                            graphviz::NodeKind::Variable { x, y }
                        }
                    },
                }
            })
            .collect::<Vec<_>>();

        let edges = self
            .graph
            .edge_indices()
            .filter_map(|edge_index| {
                self.graph
                    .edge_endpoints(edge_index)
                    .map(|(from, to)| graphviz::Edge {
                        from: from.index(),
                        to:   to.index(),
                    })
            })
            .collect::<Vec<_>>();

        (nodes, edges)
    }
}

impl FactorGraph {
    pub fn change_factor_enabled(&mut self, settings: gbp_config::FactorsEnabledSection) {
        for &ix in self.factor_indices.iter() {
            let factor = self.graph[ix].factor_mut();
            factor.enabled = match factor.kind {
                FactorKind::Dynamic(_) => settings.dynamic,
                FactorKind::Obstacle(_) => settings.obstacle,
                FactorKind::InterRobot(_) => settings.interrobot,
                FactorKind::Tracking(_) => settings.tracking,
            };
        }
    }
}
