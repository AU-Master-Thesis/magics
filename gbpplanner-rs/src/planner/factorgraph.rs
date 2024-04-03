#![warn(missing_docs)]
use std::{borrow::BorrowMut, collections::HashMap, ops::Range};

use bevy::prelude::*;
use gbp_linalg::{Float, Vector};
// use itertools::Itertools;
use petgraph::{data::DataMap, Undirected};
use tap::{Tap, TapOptional};

use super::{
    factor::{Factor, FactorKind},
    message::Message,
    robot::RobotId,
    variable::Variable,
};
use crate::{
    escape_codes::*,
    planner::message::{Eta, Lam, Mu},
    pretty_print_message,
};

#[derive(Debug, derive_more::Display)]
#[display(fmt = "no connection to the given factorgraph")]
pub struct RemoveConnectionToError;

impl std::error::Error for RemoveConnectionToError {}

pub(super) trait FactorGraphNode {
    fn remove_connection_to(
        &mut self,
        factorgraph_id: FactorGraphId,
    ) -> Result<(), RemoveConnectionToError>;
}

// /// How the messages are passed between factors and variables in the
// connected /// factorgraphs.
// #[derive(Debug, Clone, Copy)]
// pub enum MessagePassingMode {
//     /// Messages are passed within a robot's own factorgraph.
//     Internal,
//     /// Messages are passed between a robot factorgraph and other robots
//     /// factorgraphs.
//     External,
// }

/// A newtype used to enforce type safety of the indices of the factors in the
/// factorgraph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, derive_more::From)]
pub struct FactorIndex(NodeIndex);
/// A newtype used to enforce type safety of the indices of the variables in the
/// factorgraph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, derive_more::From)]
pub struct VariableIndex(NodeIndex);

// impl From<NodeIndex> for FactorIndex {
//     fn from(index: NodeIndex) -> Self {
//         Self(index)
//     }
// }

// impl From<NodeIndex> for VariableIndex {
//     fn from(index: NodeIndex) -> Self {
//         Self(index)
//     }
// }

pub trait AsNodeIndex {
    fn as_node_index(&self) -> NodeIndex;
}

impl From<FactorIndex> for usize {
    #[inline]
    fn from(index: FactorIndex) -> Self {
        index.0.index()
    }
}

impl From<VariableIndex> for usize {
    #[inline]
    fn from(index: VariableIndex) -> Self {
        index.0.index()
    }
}

impl AsNodeIndex for FactorIndex {
    #[inline]
    fn as_node_index(&self) -> NodeIndex {
        self.0
    }
}

impl AsNodeIndex for VariableIndex {
    #[inline]
    fn as_node_index(&self) -> NodeIndex {
        self.0
    }
}

#[derive(Debug, Clone, Copy, Hash)]
pub struct FactorId {
    pub factorgraph_id: FactorGraphId,
    pub factor_index:   FactorIndex,
    pub color:          &'static str,
}

// implement PartialEq and Eq manually
impl std::cmp::PartialEq for FactorId {
    // No need to compare color
    fn eq(&self, other: &Self) -> bool {
        self.factorgraph_id == other.factorgraph_id && self.factor_index == other.factor_index
    }
}

impl std::cmp::Eq for FactorId {}

macro_rules! factor_id_constructor {
    ($name:ident, color = $color:expr) => {
        pub fn $name(factorgraph_id: FactorGraphId, factor_index: FactorIndex) -> Self {
            Self {
                factorgraph_id,
                factor_index,
                color: $color,
            }
        }
    };
}

impl FactorId {
    /// Construct `FactorId` for an unknown factor kind
    factor_id_constructor!(new_ambiguous, color = CYAN);

    /// Construct `FactorId` for an obstacle factor
    factor_id_constructor!(new_obstacle, color = RED);

    /// Construct `FactorId` for an interrobot factor
    factor_id_constructor!(new_interrobot, color = GREEN);

    /// Construct `FactorId` for a pose factor
    factor_id_constructor!(new_pose, color = MAGENTA);

    /// Construct `FactorId` for a dynamic factor
    factor_id_constructor!(new_dynamic, color = BLUE);

    pub fn global_id(&self) -> String {
        format!("{:?}-{}", self.factorgraph_id, self.factor_index.0.index())
    }

    #[inline]
    pub fn get_factor_graph_id(&self) -> FactorGraphId {
        self.factorgraph_id
    }
}

#[derive(Debug, Clone, Copy, Hash)]
pub struct VariableId {
    pub factorgraph_id: FactorGraphId,
    pub variable_index: VariableIndex,
    pub color:          &'static str,
}

// implement PartialEq and Eq manually
impl std::cmp::PartialEq for VariableId {
    fn eq(&self, other: &Self) -> bool {
        self.factorgraph_id == other.factorgraph_id && self.variable_index == other.variable_index
    }
}
impl std::cmp::Eq for VariableId {}

impl VariableId {
    pub fn new(factorgraph_id: FactorGraphId, variable_index: VariableIndex) -> Self {
        Self {
            factorgraph_id,
            variable_index,
            color: YELLOW,
        }
    }

    pub fn global_id(&self) -> String {
        format!(
            "{:?}-{}",
            self.factorgraph_id,
            self.variable_index.0.index()
        )
    }

    pub fn get_factor_graph_id(&self) -> FactorGraphId {
        self.factorgraph_id
    }
}

#[derive(Debug)]
pub struct VariableToFactorMessage {
    pub from:    VariableId,
    pub to:      FactorId,
    pub message: Message,
}

#[derive(Debug)]
pub struct FactorToVariableMessage {
    pub from:    FactorId,
    pub to:      VariableId,
    pub message: Message,
}

pub type MessagesFromVariables = HashMap<FactorId, Message>;
pub type MessagesFromFactors = HashMap<VariableId, Message>;

pub type MessagesToVariables = HashMap<VariableId, Message>;
pub type MessagesToFactors = HashMap<FactorId, Message>;

#[derive(Debug)]
pub enum NodeKind {
    Factor(Factor),
    // TODO: wrap in Box<>
    Variable(Variable),
}

#[derive(Debug)]
pub struct Node {
    // TODO: change to factorgraph_id
    robot_id: RobotId,
    kind:     NodeKind,
}

// #[derive(Debug, Clone, Copy)]
// pub enum EdgeConnection {
//     Inter,
//     Intra,
// }

impl Node {
    // pub fn set_node_index(&mut self, index: NodeIndex) {
    //     match self {
    //         Self::Factor(factor) => factor.set_node_index(index),
    //         Self::Variable(variable) => variable.set_node_index(index),
    //     }
    // }
    // pub fn get_node_index(&mut self) -> NodeIndex {
    //     match self {
    //         Self::Factor(factor) => factor.get_node_index(),
    //         Self::Variable(variable) => variable.get_node_index(),
    //     }
    // }

    /// Returns `true` if the node is [`Factor`].
    ///
    /// [`Factor`]: Node::Factor
    #[must_use]
    pub fn is_factor(&self) -> bool {
        matches!(self.kind, NodeKind::Factor(..))
    }

    /// Returns `Some(&Factor)` if the node]s variant is [`Factor`], otherwise
    /// `None`.
    pub fn as_factor(&self) -> Option<&Factor> {
        if let NodeKind::Factor(ref v) = self.kind {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `Some(&mut Factor)` if the node]s variant is [`Factor`],
    /// otherwise `None`.
    pub fn as_factor_mut(&mut self) -> Option<&mut Factor> {
        if let NodeKind::Factor(ref mut v) = self.kind {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the node is [`Variable`].
    ///
    /// [`Variable`]: Node::Variable
    #[must_use]
    pub fn is_variable(&self) -> bool {
        matches!(self.kind, NodeKind::Variable(..))
    }

    /// Returns `Some(&Variable)` if the node]s variant is [`Variable`],
    /// otherwise `None`.
    pub fn as_variable(&self) -> Option<&Variable> {
        if let NodeKind::Variable(ref v) = self.kind {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `Some(&mut Variable)` if the node]s variant is [`Variable`],
    /// otherwise `None`.
    pub fn as_variable_mut(&mut self) -> Option<&mut Variable> {
        if let NodeKind::Variable(ref mut v) = self.kind {
            Some(v)
        } else {
            None
        }
    }
}

impl FactorGraphNode for Node {
    fn remove_connection_to(
        &mut self,
        factorgraph_id: FactorGraphId,
    ) -> Result<(), RemoveConnectionToError> {
        match self.kind {
            NodeKind::Factor(ref mut factor) => factor.remove_connection_to(factorgraph_id),
            NodeKind::Variable(ref mut variable) => variable.remove_connection_to(factorgraph_id),
        }
    }
}

/// The type used to represent indices into the nodes of the factorgraph.
/// This is just a type alias for `petgraph::graph::NodeIndex`, but
/// we make an alias for it here, such that it is easier to use the same
/// index type across modules, as the various node index types `petgraph`
/// are not interchangeable.
pub type NodeIndex = petgraph::stable_graph::NodeIndex;
// pub type VariableIndex = NodeIndex;
// pub type FactorIndex = NodeIndex;
/// The type used to represent indices into the nodes of the factorgraph.
pub type EdgeIndex = petgraph::stable_graph::EdgeIndex;
/// A factorgraph is an undirected graph
// pub type Graph = petgraph::graph::Graph<Node, (), Undirected, u32>;
pub type Graph = petgraph::stable_graph::StableGraph<Node, (), Undirected, u32>;

/// Record type used to keep track of how many factors and variables
/// there are in the factorgraph. We keep track of these counts internally in
/// the factorgraph, such a query for the counts, is **O(1)**.
#[derive(Debug, Clone, Copy, Default)]
pub struct NodeCount {
    pub factors:   usize,
    pub variables: usize,
}

pub type FactorGraphId = Entity;

/// A factor graph is a bipartite graph consisting of two types of nodes:
/// factors and variables.
#[derive(Component, Debug)]
pub struct FactorGraph {
    /// The id of the factorgraph. We store a copy of it here, for convenience.
    /// **Invariants**:
    /// - The id of the factorgraph is unique among all factorgraphs in the
    ///   system.
    /// - The id does not change during the lifetime of the factorgraph.
    id:               FactorGraphId,
    /// The underlying graph data structure
    graph:            Graph,
    /// In **gbpplanner** the sequence in which variables are inserted/created
    /// in the graph is meaningful. `self.graph` does not capture this
    /// ordering, so we use an extra vector to manage the order in which
    /// variables are inserted/removed from the graph. **IMPORTANT** we have
    /// to manually ensure the invariant that `self.graph` and this field is
    /// consistent at all time.
    variable_indices: Vec<NodeIndex>,
    /// List of indices of the factors in the graph. Order is not important.
    /// Used to speed up iteration over factors.
    factor_indices:   Vec<NodeIndex>,
}

pub struct Factors<'a> {
    graph:          &'a Graph,
    factor_indices: std::slice::Iter<'a, NodeIndex>,
}

impl<'a> Factors<'a> {
    pub fn new(graph: &'a Graph, factor_indices: &'a [NodeIndex]) -> Self {
        Self {
            graph,
            factor_indices: factor_indices.iter(),
        }
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

pub struct Variables<'a> {
    graph:            &'a Graph,
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

// pub struct VariablesMut<'a> {
//     graph: &'a mut Graph,
//     variable_indices: std::slice::Iter<'a, NodeIndex>,
// }

// impl<'a> VariablesMut<'a> {
//     pub fn new(graph: &'a mut Graph, variable_indices: &'a [NodeIndex]) ->
// Self {         Self {
//             graph,
//             variable_indices: variable_indices.iter(),
//         }
//     }
// }

// impl<'a> Iterator for VariablesMut<'a> {
//     type Item = (VariableIndex, &'a mut Variable);

//     fn next(&mut self) -> Option<Self::Item> {
//         let &index = self.variable_indices.next()?;
//         let node = &mut self.graph[index];
//         let NodeKind::Variable(ref mut variable) = &mut node.kind else {
//             panic!("A variable index either does not exist or does not point
// to a variable node");         };
//         Some((VariableIndex(index), variable))
//     }
// }

// struct AdjacentVariables<'a> {
//     graph: &'a Graph,
//     adjacent_variables: petgraph::stable_graph::Neighbors<'a, ()>,
// }
//
// impl<'a> AdjacentVariables<'a> {
//     pub fn new(graph: &'a Graph, factor_index: FactorIndex) -> Self {
//         Self {
//             graph,
//             adjacent_variables:
// graph.neighbors(factor_index.as_node_index()),         }
//     }
// }
//
// impl<'a> Iterator for AdjacentVariables<'a> {
//     // type Item = (RobotId, &'a Variable);
//     type Item = VariableId;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         self.adjacent_variables.next().map(|node_index| {
//             let node = &self.graph[node_index];
//             if node.is_variable() {
//                 VariableId::new(node.robot_id, node_index.into())
//             } else {
//                 panic!("A factor can only have variables as neighbors");
//             }
//         })
//     }
// }

impl FactorGraph {
    /// Construct a new empty factorgraph with a given id
    #[must_use]
    pub fn new(id: FactorGraphId) -> Self {
        Self {
            id,
            // graph: Graph::new_undirected(),
            graph: Graph::with_capacity(0, 0),
            variable_indices: Vec::new(),
            factor_indices: Vec::new(),
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
        }
    }

    /// Returns the `FactorGraphId` of the factorgraph
    #[inline]
    pub fn id(&self) -> FactorGraphId {
        self.id
    }

    /// Returns an `Iterator` over the variable nodes in the factorgraph.
    #[inline]
    pub fn variables(&self) -> Variables<'_> {
        Variables::new(&self.graph, &self.variable_indices)
    }

    /// Returns an `Iterator` over the factor nodes in the factorgraph.
    #[inline]
    pub fn factors(&self) -> Factors<'_> {
        Factors::new(&self.graph, &self.factor_indices)
    }

    pub fn add_variable(&mut self, variable: Variable) -> VariableIndex {
        let node = Node {
            robot_id: self.id,
            kind:     NodeKind::Variable(variable),
        };
        let node_index = self.graph.add_node(node);
        self.variable_indices.push(node_index);
        self.graph[node_index]
            .as_variable_mut()
            .expect("just added the variable to the graph in the previous statement")
            .set_node_index(node_index);
        info!(
            "added a variable with node_index: {:?} to factorgraph: {:?}",
            node_index, self.id
        );
        node_index.into()
    }

    pub fn add_factor(&mut self, factor: Factor) -> FactorIndex {
        let node = Node {
            robot_id: self.id,
            kind:     NodeKind::Factor(factor),
        };
        let node_index = self.graph.add_node(node);
        self.graph[node_index]
            .as_factor_mut()
            .expect("just added the factor to the graph in the previous statement")
            .tap(|f| {
                info!(
                    "adding a '{}' factor with node_index: {:?} to factorgraph: {:?}",
                    f.variant(),
                    node_index,
                    self.id
                );
            })
            .set_node_index(node_index);
        self.factor_indices.push(node_index);

        node_index.into()
    }

    /// Add an edge between nodes `a` and `b` in the factorgraph.
    ///
    /// **invariants**:
    /// - Both `a` and `b` must already be in the factorgraph. Panics if any of
    ///   the nodes does not exist.
    pub fn add_internal_edge(
        &mut self,
        // variable_index: VariableIndex,
        // factor_index: FactorIndex,
        variable_id: VariableId,
        factor_id: FactorId,
    ) -> EdgeIndex {
        let dofs = 4;

        let message_to_factor = {
            let Some(variable) =
                self.graph[variable_id.variable_index.as_node_index()].as_variable_mut()
            else {
                panic!(
                    "the variable index either does not exist or does not point to a variable node"
                );
            };
            // TODO: explain why we send an empty message
            variable.receive_message_from(factor_id, Message::empty(dofs));
            // .receive_message_from(FactorId::new(self.id, factor_index),
            // Message::empty(dofs));

            Message::new(
                Eta(variable.belief.eta.clone()),
                Lam(variable.belief.lambda.clone()),
                Mu(variable.belief.mu.clone()),
            )
        };

        let node = &mut self.graph[factor_id.factor_index.as_node_index()];
        match node.kind {
            NodeKind::Factor(ref mut factor) => {
                // NOTE: If this message were not empty, half a variable iteration will have
                // happened manually in secret, which is not wanted
                factor.receive_message_from(variable_id, Message::empty(dofs))
                // factor.receive_message_from(variable_id, message_to_factor)
            }
            // .receive_message_from(VariableId::new(self.id, variable_index), message_to_factor),
            NodeKind::Variable(_) => {
                panic!("the factor index either does not exist or does not point to a factor node")
            }
        }

        info!(
            "adding internal edge from {:?} to {:?}",
            variable_id, factor_id
        );

        self.graph.add_edge(
            variable_id.variable_index.as_node_index(),
            factor_id.factor_index.as_node_index(),
            (),
        )
    }

    pub fn add_external_edge(&mut self, factor_id: FactorId, nth_variable_index: usize) {
        let variable_index = self
            .nth_variable_index(nth_variable_index)
            .expect("The variable index does not exist");
        let variable = self.graph[variable_index.as_node_index()]
            .as_variable_mut()
            .expect("The variable index does not point to a variable node");

        let dofs = 4;
        info!(
            "adding external edge from {:?} to {:?} in factorgraph {:?}",
            variable_index, factor_id, self.id
        );
        variable.receive_message_from(factor_id, Message::empty(dofs));
    }

    /// Number of nodes in the factorgraph
    ///
    /// **Computes in O(1) time**
    #[inline]
    pub fn len(&self) -> usize {
        self.graph.node_count()
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

    #[inline]
    pub fn nth_variable_index(&self, index: usize) -> Option<VariableIndex> {
        self.variable_indices.get(index).copied().map(VariableIndex)
    }

    pub fn nth_variable(&self, index: usize) -> Option<(VariableIndex, &Variable)> {
        let variable_index = self.nth_variable_index(index)?;
        let node = &self.graph[variable_index.as_node_index()];
        let variable = node.as_variable()?;
        Some((variable_index, variable))
    }

    pub fn nth_variable_mut(&mut self, index: usize) -> Option<(VariableIndex, &mut Variable)> {
        let variable_index = self.nth_variable_index(index)?;
        let node = &mut self.graph[variable_index.as_node_index()];
        let variable = node.as_variable_mut()?;
        Some((variable_index, variable))
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

    pub fn factor(&self, index: FactorIndex) -> &Factor {
        self.graph[index.as_node_index()]
            .as_factor()
            .expect("A factor index should point to a Factor in the graph")
    }

    pub fn factor_mut(&mut self, index: FactorIndex) -> &mut Factor {
        self.graph[index.as_node_index()]
            .as_factor_mut()
            .expect("A factor index should point to a Factor in the graph")
    }

    pub fn variable(&self, index: VariableIndex) -> &Variable {
        self.graph[index.as_node_index()]
            .as_variable()
            .expect("A variable index should point to a Variable in the graph")
    }

    pub fn get_factor(&self, index: FactorIndex) -> Option<&Factor> {
        self.graph
            .node_weight(index.as_node_index())
            .and_then(|node| node.as_factor())
    }

    pub fn get_factor_mut(&mut self, index: FactorIndex) -> Option<&mut Factor> {
        self.graph
            .node_weight_mut(index.as_node_index())
            .and_then(|node| node.as_factor_mut())
    }

    pub fn get_variable(&self, index: VariableIndex) -> Option<&Variable> {
        self.graph
            .node_weight(index.as_node_index())
            .and_then(|node| node.as_variable())
    }

    pub fn get_variable_mut(&mut self, index: VariableIndex) -> Option<&mut Variable> {
        self.graph
            .node_weight_mut(index.as_node_index())
            .and_then(|node| node.as_variable_mut())
    }

    pub fn variable_mut(&mut self, index: VariableIndex) -> &mut Variable {
        self.graph[index.as_node_index()]
            .as_variable_mut()
            .expect("A variable index should point to a Variable in the graph")
    }

    pub fn export_data(&self) -> (Vec<graphviz::Node>, Vec<graphviz::Edge>) {
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
                            FactorKind::InterRobot(inner) => {
                                graphviz::NodeKind::InterRobotFactor(inner.connection.clone())
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

    /// Aggregate and marginalise over all adjacent variables, and send.
    /// Aggregation: product of all incoming messages
    pub fn factor_iteration(&mut self) -> Vec<FactorToVariableMessage> {
        let mut messages_to_external_variables: Vec<FactorToVariableMessage> = Vec::new();

        for node_index in self.graph.node_indices().collect::<Vec<_>>() {
            let node = &mut self.graph[node_index];
            let Some(factor) = node.as_factor_mut() else {
                continue;
            };
            // let factor_variant = factor.variant();

            let variable_messages = factor.update();
            if variable_messages.is_empty() {
                panic!(
                    "The factorgraph {:?} with factor {:?} did not receive any messages from its \
                     connected variables",
                    self.id, node_index
                );
            }

            let factor_id = match factor.kind {
                FactorKind::Dynamic(_) => FactorId::new_dynamic(self.id, FactorIndex(node_index)),
                FactorKind::Obstacle(_) => FactorId::new_obstacle(self.id, FactorIndex(node_index)),
                FactorKind::Pose(_) => FactorId::new_pose(self.id, FactorIndex(node_index)),
                FactorKind::InterRobot(_) => {
                    FactorId::new_interrobot(self.id, FactorIndex(node_index))
                }
            };
            // let factor_id = FactorId::new(self.id, FactorIndex(node_index));
            for (variable_id, message) in variable_messages {
                let in_internal_graph = variable_id.factorgraph_id == self.id;
                if in_internal_graph {
                    let variable = self.graph[variable_id.variable_index.as_node_index()]
                        .as_variable_mut()
                        .expect("A factor can only have variables as neighbors");

                    // info!(
                    //     "{:?}\t->\tvariable {:?}\tfactor variant: {:?}",
                    //     factor_id, variable_id, factor_variant
                    // );

                    // let factor_global_identifier = factor_id.global_id();
                    // let variable_global_identifier = variable_id.global_id();
                    // pretty_print_message!(factor_global_identifier, variable_global_identifier);
                    // NOTE: pretty print the message
                    // pretty_print_message!(factor_id, variable_id, "factor iteration");

                    variable.receive_message_from(factor_id, message);
                } else {
                    // error!(
                    //     "message from factor_id: {:?} to variable_id: {:?} is external",
                    //     factor_id, variable_id
                    // );
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
            // dbg!(&factor_messages);
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
                    if !self
                        .factor_indices
                        .contains(&factor_id.factor_index.as_node_index())
                    {
                        info!(
                            "factor_id: {:?} does not exist in the factorgraph {:?}",
                            factor_id, self.id
                        );
                        continue;
                    }

                    self.graph[factor_id.factor_index.as_node_index()]
                        .as_factor_mut()
                        .expect("A factor can only have variables as neighbours")
                        .receive_message_from(variable_id, message);

                    let factor = self.graph[factor_id.factor_index.as_node_index()]
                        .as_factor()
                        .expect("A factor index should point to a Factor in the graph");
                    // info!(
                    //     "Message variable {:?}\t->\tfactor {:?}\tfactor
                    // variant: {:?}",     variable_id,
                    //     factor_id,
                    //     factor.variant()
                    // );
                    // NOTE: pretty print the message
                    // pretty_print_message!(variable_id, factor_id, "variable
                    // iteration");
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

        // std::process::exit(0);

        // Return the messages to be sent to the connected factors in other factorgraphs
        // The caller is responsible for sending these messages to the correct
        // factorgraphs
        messages_to_external_factors
    }

    pub fn change_prior_of_variable(
        &mut self,
        variable_index: VariableIndex,
        new_mean: Vector<Float>,
    ) -> Vec<VariableToFactorMessage> {
        let variable_id = VariableId::new(self.id, variable_index);
        let variable = self.variable_mut(variable_id.variable_index);

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

    pub(crate) fn delete_interrobot_factors_connected_to(
        &mut self,
        other: RobotId,
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

            if interrobot.connection.id_of_robot_connected_with == other {
                info!("deleting interrobot factor {:?}", node_index);
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
                    .remove(&FactorId::new_ambiguous(self.id, *factor_index));
            }
        }

        Ok(())
    }

    pub(crate) fn delete_messages_from_interrobot_factor_at(&mut self, other: RobotId) {
        for node_index in self.graph.node_indices().collect::<Vec<_>>() {
            let node = &mut self.graph[node_index];
            let Some(variable) = node.as_variable_mut() else {
                continue;
            };
            // println!("robot id {:?}", other);
            // println!("before {:?}", variable.inbox.keys().collect::<Vec<_>>());
            variable
                .inbox
                .retain(|factor_id, _| factor_id.factorgraph_id != other);
            // println!("after  {:?}",
            // variable.inbox.keys().collect::<Vec<_>>());
        }
    }

    pub fn remove_connection_to(
        &mut self,
        factorgraph_id: FactorGraphId,
    ) -> Result<(), RemoveConnectionToError> {
        // go through all nodes, and remove their individual connection to the other
        // factorgraph if none of the nodes has a connection to the other
        // factorgraph, then return and Error.

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
}

pub mod graphviz {
    use crate::planner::factor::InterRobotConnection;

    pub struct Node {
        pub index: usize,
        pub kind:  NodeKind,
    }

    impl Node {
        pub fn color(&self) -> &'static str {
            self.kind.color()
        }

        pub fn shape(&self) -> &'static str {
            self.kind.shape()
        }

        pub fn width(&self) -> f64 {
            self.kind.width()
        }
    }

    pub enum NodeKind {
        Variable { x: f32, y: f32 },
        InterRobotFactor(InterRobotConnection),
        // InterRobotFactor {
        //     /// The id of the robot the interrobot factor is connected to
        //     other_robot_id: RobotId,
        //     /// The index of the variable in the other robots factorgraph, that the interrobot
        // factor is connected with     variable_index_in_other_robot: usize,
        // },
        DynamicFactor,
        ObstacleFactor,
        PoseFactor,
    }

    impl NodeKind {
        pub fn color(&self) -> &'static str {
            match self {
                Self::Variable { .. } => "#eff1f5",         // latte base (white)
                Self::InterRobotFactor { .. } => "#a6da95", // green
                Self::DynamicFactor => "#8aadf4",           // blue
                // Self::ObstacleFactor => "#c6a0f6",          // mauve (purple)
                Self::ObstacleFactor => "#ee99a0", // mauve (purple)
                Self::PoseFactor => "#c6aof6",     // maroon (red)
            }
        }

        pub fn shape(&self) -> &'static str {
            match self {
                Self::Variable { .. } => "circle",
                _ => "square",
            }
        }

        pub fn width(&self) -> f64 {
            match self {
                Self::Variable { .. } => 0.8,
                _ => 0.2,
            }
        }
    }

    pub struct Edge {
        pub from: usize,
        pub to:   usize,
    }
}
