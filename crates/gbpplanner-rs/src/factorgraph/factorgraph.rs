use std::ops::Range;

use bevy::{
    ecs::{component::Component, entity::Entity},
    log::{debug, info},
};
// use gbp_linalg::Float;
use gbp_linalg::prelude::*;
use petgraph::Undirected;

use super::{
    factor::{interrobot::InterRobotFactor, Factor, FactorKind},
    id::{FactorId, VariableId},
    message::{
        FactorToVariableMessage, InformationVec, Mean, PrecisionMatrix, VariableToFactorMessage,
    },
    node::{FactorGraphNode, Node, NodeKind, RemoveConnectionToError},
    prelude::Message,
    variable::Variable,
};

/// type alias used to represent the id of the factorgraph
/// Since we use **Bevy** we can use the `Entity` id of the whatever entity the
/// the factorgraph is attached to as a Component, as its unique identifier.
pub type FactorGraphId = Entity;

/// The type used to represent indices into the nodes of the factorgraph.
/// This is just a type alias for `petgraph::graph::NodeIndex`, but
/// we make an alias for it here, such that it is easier to use the same
/// index type across modules, as the various node index types `petgraph`
/// are not interchangeable.
pub(crate) type NodeIndex = petgraph::stable_graph::NodeIndex;
/// The type used to represent indices into the nodes of the factorgraph.
pub type EdgeIndex = petgraph::stable_graph::EdgeIndex;
/// A factorgraph is an undirected graph
pub type Graph = petgraph::stable_graph::StableGraph<Node, (), Undirected, u32>;

/// A newtype used to enforce type safety of the indices of the factors in the
/// factorgraph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, derive_more::From)]
pub struct FactorIndex(pub NodeIndex);

impl std::ops::Deref for FactorIndex {
    type Target = NodeIndex;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<FactorIndex> for usize {
    fn from(index: FactorIndex) -> Self {
        index.0.index()
    }
}

/// A newtype used to enforce type safety of the indices of the variables in the
/// factorgraph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, derive_more::From)]
pub struct VariableIndex(pub NodeIndex);

impl std::ops::Deref for VariableIndex {
    type Target = NodeIndex;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

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
    id: FactorGraphId,
    /// The underlying graph data structure
    graph: Graph,
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
    factor_indices: Vec<NodeIndex>,

    /// List of indices of the interrobot factors in the graph. Order is not
    /// important. Used to speed up iteration over interrobot factors.
    /// When querying for number of external messages sent
    interrobot_factor_indices: Vec<NodeIndex>,
}

impl FactorGraph {
    /// Construct a new empty factorgraph with a given id
    #[must_use]
    pub fn new(id: FactorGraphId) -> Self {
        Self {
            id,
            graph: Graph::with_capacity(0, 0),
            variable_indices: Vec::new(),
            factor_indices: Vec::new(),
            interrobot_factor_indices: Vec::new(),
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
            interrobot_factor_indices: Vec::new(),
        }
    }

    /// Returns the `FactorGraphId` of the factorgraph
    #[inline(always)]
    pub fn id(&self) -> FactorGraphId {
        self.id
    }

    /// Adds a variable to the factorgraph
    /// Returns the index of the variable in the factorgraph
    pub fn add_variable(&mut self, variable: Variable) -> VariableIndex {
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

    pub fn add_factor(&mut self, factor: Factor) -> FactorIndex {
        let is_interrobot = factor.is_inter_robot();
        let node = Node::new(self.id, NodeKind::Factor(factor));
        let node_index = self.graph.add_node(node);
        self.graph[node_index]
            .as_factor_mut()
            .expect("just added the factor to the graph in the previous statement")
            // .tap(|f| {
            //     info!(
            //         "adding a '{}' factor with node_index: {:?} to factorgraph: {:?}",
            //         f.variant(),
            //         node_index,
            //         self.id
            //     );
            // })
            .set_node_index(node_index);
        self.factor_indices.push(node_index);
        if is_interrobot {
            self.interrobot_factor_indices.push(node_index);
        }

        node_index.into()
    }

    /// Number of nodes in the factorgraph
    ///
    /// **Computes in O(1) time**
    #[inline]
    pub fn len(&self) -> usize {
        self.graph.node_count()
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
        let message_to_factor = {
            let Some(variable) = self.graph[variable_id.variable_index.0].as_variable_mut() else {
                panic!(
                    "the variable index either does not exist or does not point to a variable node"
                );
            };
            // TODO: explain why we send an empty message
            variable.receive_message_from(factor_id, Message::empty());

            Message::new(
                InformationVec(variable.belief.information_vector.clone()),
                PrecisionMatrix(variable.belief.precision_matrix.clone()),
                Mean(variable.belief.mean.clone()),
            )
        };

        let node = &mut self.graph[factor_id.factor_index.0];
        match node.kind {
            NodeKind::Factor(ref mut factor) => {
                // NOTE: If this message were not empty, half a variable iteration will have
                // happened manually in secret, which is not wanted
                factor.receive_message_from(variable_id, Message::empty())
            }
            NodeKind::Variable(_) => {
                panic!("the factor index either does not exist or does not point to a factor node")
            }
        }

        self.graph
            .add_edge(variable_id.variable_index.0, factor_id.factor_index.0, ())
    }

    pub fn add_external_edge(&mut self, factor_id: FactorId, nth_variable_index: usize) {
        let variable_index = self
            .nth_variable_index(nth_variable_index)
            .expect("The variable index does not exist");
        let variable = self.graph[variable_index.0]
            .as_variable_mut()
            .expect("The variable index does not point to a variable node");

        let dofs = 4;
        debug!(
            "adding external edge from {:?} to {:?} in factorgraph {:?}",
            variable_index, factor_id, self.id
        );
        variable.receive_message_from(factor_id, Message::empty());
    }

    #[inline]
    pub fn nth_variable_index(&self, index: usize) -> Option<VariableIndex> {
        self.variable_indices.get(index).copied().map(VariableIndex)
    }

    pub fn nth_variable(&self, index: usize) -> Option<(VariableIndex, &Variable)> {
        let variable_index = self.nth_variable_index(index)?;
        let node = &self.graph[variable_index.0];
        let variable = node.as_variable()?;
        Some((variable_index, variable))
    }

    pub fn nth_variable_mut(&mut self, index: usize) -> Option<(VariableIndex, &mut Variable)> {
        let variable_index = self.nth_variable_index(index)?;
        let node = &mut self.graph[variable_index.0];
        let variable = node.as_variable_mut()?;
        Some((variable_index, variable))
    }

    pub(crate) fn delete_interrobot_factors_connected_to(
        &mut self,
        other: FactorGraphId,
    ) -> Result<(), &'static str> {
        // 1. Find all interrobot factors connected to the robot with id `other`
        // and remove them from the graph

        let mut factor_indices_to_remove = Vec::new();

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
            let Some(interrobot) = factor.kind.as_inter_robot() else {
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

        Ok(())
    }

    pub(crate) fn delete_messages_from_interrobot_factor_at(&mut self, other: FactorGraphId) {
        // PERF: avoid allocation
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

    /// Return an ordered interval of variables indices.
    /// The indices are ordered by the order in which they are inserted into the
    /// factorgraph. Returns `None`, if the end of the  **range** exceeds
    /// the number of variables in the factorgraph.
    pub fn variable_indices_ordered_by_creation(
        &self,
        range: Range<usize>,
    ) -> Option<Vec<NodeIndex>> {
        let within_range = range.end <= self.variable_indices.len();
        if within_range {
            Some(
                self.variable_indices
                    .iter()
                    .skip(range.start)
                    .take(range.end - range.start)
                    .copied()
                    .collect::<Vec<_>>(),
            )
        } else {
            None
        }
    }

    pub fn change_prior_of_variable(
        &mut self,
        variable_index: VariableIndex,
        new_mean: Vector<Float>,
    ) -> Vec<VariableToFactorMessage> {
        let variable_id = VariableId::new(self.id, variable_index);
        let Some(variable) = self.get_variable_mut(variable_id.variable_index) else {
            panic!("the variable index either does not exist or does not point to a variable node");
        };
        // let variable = self.variable_mut(variable_id.variable_index);

        let factor_messages = variable.change_prior(new_mean);
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

        messages_to_external_factors
    }

    pub fn get_factor(&self, index: FactorIndex) -> Option<&Factor> {
        self.graph
            .node_weight(index.0)
            .and_then(|node| node.as_factor())
    }

    pub fn get_factor_mut(&mut self, index: FactorIndex) -> Option<&mut Factor> {
        self.graph
            .node_weight_mut(*index)
            .and_then(|node| node.as_factor_mut())
    }

    pub fn get_variable(&self, index: VariableIndex) -> Option<&Variable> {
        self.graph
            .node_weight(*index)
            .and_then(|node| node.as_variable())
    }

    pub fn get_variable_mut(&mut self, index: VariableIndex) -> Option<&mut Variable> {
        self.graph
            .node_weight_mut(*index)
            .and_then(|node| node.as_variable_mut())
    }

    #[inline(always)]
    pub fn first_variable(&self) -> Option<(VariableIndex, &Variable)> {
        self.nth_variable(0usize)
    }

    pub fn last_variable(&self) -> Option<(VariableIndex, &Variable)> {
        if self.variable_indices.is_empty() {
            None
        } else {
            self.nth_variable(self.variable_indices.len() - 1)
        }
    }

    pub fn last_variable_mut(&mut self) -> Option<(VariableIndex, &mut Variable)> {
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
    pub fn variable_iteration(&mut self) -> Vec<VariableToFactorMessage> {
        let mut messages_to_external_factors: Vec<VariableToFactorMessage> = Vec::new();

        for &node_index in self.variable_indices.iter() {
            let node = &mut self.graph[node_index];
            let variable = node.as_variable_mut().expect(
                "self.variable_indices should only contain indices that point to Variables in the \
                 graph",
            );
            let variable_index = VariableIndex(node_index);

            let factor_messages = variable.update_belief_and_create_factor_responses();
            if factor_messages.is_empty() {
                panic!(
                    "The factorgraph {:?} with variable {:?} did not receive any messages from \
                     its connected factors",
                    self.id, variable_index
                );
            }

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

                    let factor = self.graph[factor_id.factor_index.0]
                        .as_factor()
                        .expect("A factor index should point to a Factor in the graph");
                } else {
                    // error!(
                    //     "message from factor_id: {:?} to variable_id: {:?} is external",
                    //     factor_id, variable_id
                    // );
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

    /// Aggregate and marginalise over all adjacent variables, and send.
    /// Aggregation: product of all incoming messages
    pub fn factor_iteration(&mut self) -> Vec<FactorToVariableMessage> {
        let mut messages_to_external_variables: Vec<FactorToVariableMessage> = Vec::new();

        for node_index in self.graph.node_indices().collect::<Vec<_>>() {
            let node = &mut self.graph[node_index];
            let Some(factor) = node.as_factor_mut() else {
                continue;
            };

            let variable_messages = factor.update();
            if variable_messages.is_empty() {
                panic!(
                    "The factorgraph {:?} with factor {:?} did not receive any messages from its \
                     connected variables",
                    self.id, node_index
                );
            }

            let factor_id = FactorId::new(self.id, FactorIndex(node_index));

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

    #[must_use]
    pub fn messages_sent(&mut self) -> usize {
        self.graph
            .node_weights_mut()
            .map(|node| {
                let messages_sent = node.messages_sent();

                node.reset_message_count();
                messages_sent
            })
            .sum()
    }
}

/// Record type used to keep track of how many factors and variables
/// there are in the factorgraph. We keep track of these counts internally in
/// the factorgraph, such a query for the counts, is **O(1)**.
#[derive(Debug, Clone, Copy, Default)]
pub struct NodeCount {
    pub factors:   usize,
    pub variables: usize,
}

/// Iterator over the factors in the factorgraph.
///
/// Iterator element type is `(FactorIndex, &'a Factor)`.
///
/// Created with [`.factors()`][1]
///
/// [1]: struct.FactorGraph.html#method.factors
pub struct Factors<'a> {
    graph: &'a Graph,
    factor_indices: std::slice::Iter<'a, NodeIndex>,
}

impl<'a> Factors<'a> {
    #[must_use]
    pub fn new(graph: &'a Graph, factor_indices: &'a [NodeIndex]) -> Self {
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

impl<'a> Iterator for Factors<'a> {
    type Item = (NodeIndex, &'a Factor);

    fn next(&mut self) -> Option<Self::Item> {
        let &index = self.factor_indices.next()?;
        let node = &self.graph[index];
        node.as_factor().map(|factor| (index, factor))
    }
}

/// Iterator over the variables in the factorgraph.
///
/// Iterator element type is `(VariableIndex, &'a Variable)`.
///
/// Created with [`.variables()`][1]
///
/// [1]: struct.FactorGraph.html#method.variables
pub struct Variables<'a> {
    graph: &'a Graph,
    variable_indices: std::slice::Iter<'a, NodeIndex>,
}

impl<'a> Variables<'a> {
    pub fn new(graph: &'a Graph, variable_indices: &'a [NodeIndex]) -> Self {
        Self {
            graph,
            variable_indices: variable_indices.iter(),
        }
    }
}

impl<'a> Iterator for Variables<'a> {
    type Item = (VariableIndex, &'a Variable);

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
pub struct InterRobotFactors<'a> {
    graph: &'a Graph,
    factor_indices: std::slice::Iter<'a, NodeIndex>,
}

impl<'a> InterRobotFactors<'a> {
    pub fn new(graph: &'a Graph, factor_indices: &'a [NodeIndex]) -> Self {
        Self {
            graph,
            factor_indices: factor_indices.iter(),
        }
    }
}

impl<'a> Iterator for InterRobotFactors<'a> {
    type Item = (NodeIndex, &'a InterRobotFactor);

    fn next(&mut self) -> Option<Self::Item> {
        let &index = self.factor_indices.next()?;
        let node = &self.graph[index];
        node.as_factor()
            .and_then(|factor| factor.kind.as_inter_robot())
            .map(|interrobot| (index, interrobot))
    }
}

impl FactorGraph {
    #[inline]
    #[must_use]
    pub fn inter_robot_factors(&self) -> InterRobotFactors<'_> {
        InterRobotFactors::new(&self.graph, &self.interrobot_factor_indices)
    }
}

impl std::ops::Index<FactorIndex> for FactorGraph {
    type Output = Factor;

    // type Output = Option<Factor>;

    fn index(&self, index: FactorIndex) -> &Self::Output {
        let node = &self.graph[index.0];
        node.as_factor()
            .expect("a factor index points to a factor node in the graph")
    }
}

impl std::ops::Index<VariableIndex> for FactorGraph {
    type Output = Variable;

    fn index(&self, index: VariableIndex) -> &Self::Output {
        self.graph[index.0]
            .as_variable()
            .expect("a variable index points to a variable node in the graph")
    }
}

use super::graphviz;

impl graphviz::Graph for FactorGraph {
    fn export_data(&self) -> (Vec<super::graphviz::Node>, Vec<super::graphviz::Edge>) {
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
                            FactorKind::Pose(_) => graphviz::NodeKind::PoseFactor,
                            FactorKind::InterRobot(ref inner) => {
                                graphviz::NodeKind::InterRobotFactor(inner.external_variable)
                            }
                        },
                        NodeKind::Variable(variable) => {
                            // let mean = variable.belief.mean();
                            // let mean = &variable.mu;
                            let [x, y] = variable.estimated_position();
                            graphviz::NodeKind::Variable {
                                x: x as f32,
                                y: y as f32,
                            }
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
