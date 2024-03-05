use std::collections::{HashMap, VecDeque};
use std::ops::{AddAssign, Range};

use bevy::prelude::*;
use ndarray::s;
use petgraph::Undirected;

use crate::planner::factor;

use super::factor::{Factor, FactorKind};
// use super::multivariate_normal::MultivariateNormal;
use super::robot::RobotId;
use super::variable::Variable;
use super::{marginalise_factor_distance, Matrix, Vector};
use gbp_multivariate_normal::MultivariateNormal;

pub mod graphviz {
    use crate::planner::factor::InterRobotConnection;

    pub struct Node {
        pub index: usize,
        pub kind: NodeKind,
    }

    impl Node {
        pub fn color(&self) -> &'static str {
            self.kind.color()
        }

        pub fn shape(&self) -> &'static str {
            self.kind.shape()
        }

        pub fn width(&self) -> &'static str {
            self.kind.width()
        }
    }

    pub enum NodeKind {
        Variable { x: f32, y: f32 },
        InterRobotFactor(InterRobotConnection),
        // InterRobotFactor {
        //     /// The id of the robot the interrobot factor is connected to
        //     other_robot_id: RobotId,
        //     /// The index of the variable in the other robots factorgraph, that the interrobot factor is connected with
        //     variable_index_in_other_robot: usize,
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
                Self::ObstacleFactor => "#c6a0f6",          // mauve (purple)
                Self::PoseFactor => "#ee99a0",              // maroon (red)
            }
        }

        pub fn shape(&self) -> &'static str {
            match self {
                Self::Variable { .. } => "circle",
                _ => "square",
            }
        }

        pub fn width(&self) -> &'static str {
            match self {
                Self::Variable { .. } => "0.8",
                _ => "0.2",
            }
        }
    }

    pub struct Edge {
        pub from: usize,
        pub to: usize,
    }
}

/// How the messages are passed between factors and variables in the connected factorgraphs.
// #[derive(Debug)]
pub enum MessagePassingMode {
    /// Messages are passed within a robot's own factorgraph.
    Internal,
    /// Messages are passed between a robot factorgraph and other robots factorgraphs.
    External,
}

#[derive(Debug, Clone)]
// pub struct Message(pub MultivariateNormal<f32>);
pub enum Message {
    Content { gaussian: MultivariateNormal<f32> },
    Empty(usize), // dofs
}

impl Message {
    pub fn mean(&self) -> Vector<f32> {
        match self {
            Self::Content { gaussian } => *gaussian.mean(),
            Self::Empty(dofs) => Vector::<f32>::zeros(*dofs),
        }
    }

    pub fn precision_matrix(&self) -> Matrix<f32> {
        match self {
            Self::Content { gaussian } => *gaussian.precision_matrix(),
            Self::Empty(dofs) => Matrix::<f32>::zeros((*dofs, *dofs)),
        }
    }

    pub fn information_vector(&self) -> Vector<f32> {
        match self {
            Self::Content { gaussian } => *gaussian.information_vector(),
            Self::Empty(dofs) => Vector::<f32>::zeros(*dofs),
        }
    }
}

// /// Overload subtraction for `Message`
// impl std::ops::Sub for Message {
//     type Output = Self;
//     fn sub(self, rhs: Self) -> Self::Output {
//         Message(self.0 - rhs.0)
//     }
// }

// impl std::ops::Sub<&Message> for Message {
//     type Output = Self;
//     fn sub(self, rhs: &Self) -> Self::Output {
//         Message(self.0 - &rhs.0)
//     }
// }

impl Message {
    pub fn with_dofs(dofs: usize) -> Self {
        let information_vector = Vector::<f32>::from_elem(dofs, 1.0 / dofs as f32);
        let precision_matrix = Matrix::<f32>::eye(dofs);
        MultivariateNormal::from_information_and_precision(information_vector, precision_matrix)
            .map(|gaussian| Self::Content { gaussian })
            .expect("An identity matrix and uniform vector is a valid multivariate normal")
        // MultivariateNormal::empty(dofs)
        //     .map(|gaussian| Self { gaussian })
        //     .expect("Empty `MultiVarianteNormal` is always valid")
    }

    // pub fn mean(&self) -> Vector<f32> {
    //     self.0.mean()
    // }

    pub fn new(
        information_vector: Vector<f32>,
        precision_matrix: Matrix<f32>,
    ) -> gbp_multivariate_normal::Result<Self> {
        MultivariateNormal::from_mean_and_covariance(information_vector, precision_matrix)
            .map(|gaussian| Self::Content { gaussian })
    }

    // pub fn zeros(dims: usize) -> Self {
    //     Self(MultivariateNormal::zeros(dims))
    // }

    // pub fn zeroize(&mut self) {
    //     self.0.zeroize();
    // }

    // pub fn empty(dofs: usize) -> Self {
    //     let information_vector = Vector::<f32>::from_elem(dofs, 0.0);
    //     let precision_matrix = Matrix::<f32>::zeros((dofs, dofs));
    //     MultivariateNormal::from_information_and_precision(information_vector, precision_matrix)
    //         .map(|gaussian| Self { gaussian })
    //         .expect("An identity matrix and uniform vector is a valid multivariate normal")
    // }
}

impl From<MultivariateNormal<f32>> for Message {
    fn from(value: MultivariateNormal<f32>) -> Self {
        Self::Content { gaussian: value }
    }
}

// impl TryFrom<MultivariateNormal<f32>> for Message {
//     type Error: gbp_multivariate_normal::Error;

//     fn try_from(value: MultivariateNormal<f32>) -> Result<Self, Self::Error> {
//         todo!()
//     }
// }

pub type Inbox = HashMap<NodeIndex, Message>;

#[derive(Debug, Clone)]
pub enum Node {
    Factor(Factor),
    // TODO: wrap in Box<>
    Variable(Variable),
}

impl Node {
    pub fn set_node_index(&mut self, index: NodeIndex) {
        match self {
            Self::Factor(factor) => factor.set_node_index(index),
            Self::Variable(variable) => variable.set_node_index(index),
        }
    }
    pub fn get_node_index(&mut self) -> NodeIndex {
        match self {
            Self::Factor(factor) => factor.get_node_index(),
            Self::Variable(variable) => variable.get_node_index(),
        }
    }

    /// Returns `true` if the node is [`Factor`].
    ///
    /// [`Factor`]: Node::Factor
    #[must_use]
    pub fn is_factor(&self) -> bool {
        matches!(self, Self::Factor(..))
    }

    pub fn as_factor(&self) -> Option<&Factor> {
        if let Self::Factor(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_factor_mut(&mut self) -> Option<&mut Factor> {
        if let Self::Factor(v) = self {
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
        matches!(self, Self::Variable(..))
    }

    pub fn as_variable(&self) -> Option<&Variable> {
        if let Self::Variable(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_variable_mut(&mut self) -> Option<&mut Variable> {
        if let Self::Variable(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

/// The type used to represent indices into the nodes of the factorgraph.
/// This is just a type alias for `petgraph::graph::NodeIndex`, but
/// we make an alias for it here, such that it is easier to use the same
/// index type across modules, as the various node index types `petgraph`
/// are not interchangeable.
pub type NodeIndex = petgraph::graph::NodeIndex;
/// The type used to represent indices into the nodes of the factorgraph.
pub type EdgeIndex = petgraph::graph::EdgeIndex;
/// A factorgraph is an undirected graph
pub type Graph = petgraph::graph::Graph<Node, (), Undirected>;

/// Record type used to keep track of how many factors and variables
/// there are in the factorgraph. We keep track of these counts internally in the
/// factorgraph, such a query for the counts, is **O(1)**.
#[derive(Debug, Clone, Copy)]
pub struct NodeCount {
    pub factors: usize,
    pub variables: usize,
}

/// A factor graph is a bipartite graph consisting of two types of nodes: factors and variables.
/// Factors and variables are stored in separate btree maps, that are indexed by a unique tuple of (robot_id, node_id).
#[derive(Component, Debug)]
pub struct FactorGraph {
    graph: Graph,
    /// tracks how many variable and factor nodes there are in the graph.
    node_count: NodeCount,
    /// In **gbpplanner** the sequence in which variables are inserted/created in the graph
    /// is meaningful. `self.graph` does not capture this ordering, so we use an extra queue
    /// to manage the order in which variables are inserted/removed from the graph.
    /// **IMPORTANT** we have to manually ensure the invariant that `self.graph` and this field
    /// is consistent at all time.
    variable_indices_ordered: VecDeque<NodeIndex>,
}

impl FactorGraph {
    /// Construct a new empty factorgraph
    pub fn new() -> Self {
        Self {
            graph: Graph::new_undirected(),
            node_count: NodeCount {
                factors: 0usize,
                variables: 0usize,
            },
            variable_indices_ordered: VecDeque::new(),
        }
    }

    pub fn add_variable(&mut self, variable: Variable) -> NodeIndex {
        let node_index = self.graph.add_node(Node::Variable(variable));
        self.graph[node_index].set_node_index(node_index);
        self.variable_indices_ordered.push_back(node_index);
        self.node_count.variables += 1;
        node_index
    }

    pub fn add_factor(&mut self, factor: Factor) -> NodeIndex {
        let node_index = self.graph.add_node(Node::Factor(factor));
        self.graph[node_index].set_node_index(node_index);
        self.node_count.factors += 1;
        node_index
    }

    /// Add an edge between nodes `a` and `b` in the factorgraph.
    ///
    /// **invariants**:
    /// - Both `a` and `b` must already be in the factorgraph. Panics if any of the nodes does not exist.
    pub fn add_edge(&mut self, a: NodeIndex, b: NodeIndex) -> EdgeIndex {
        let dofs = 4;
        match self.graph[a] {
            Node::Factor(ref mut factor) => factor.send_message(b, Message::Empty(dofs)),
            Node::Variable(ref mut variable) => variable.send_message(b, Message::Empty(dofs)),
        }
        match self.graph[b] {
            Node::Factor(ref mut factor) => factor.send_message(a, Message::Empty(dofs)),
            Node::Variable(ref mut variable) => variable.send_message(a, Message::Empty(dofs)),
        }
        self.graph.add_edge(a, b, ())
    }

    /// Number of nodes in the factorgraph
    ///
    /// **Computes in O(1) time**
    pub fn len(&self) -> usize {
        self.graph.node_count()
    }

    /// Return an ordered interval of variables indices.
    /// The indices are ordered by the order in which they are inserted into the factorgraph.
    /// Returns `None`, if the end of the  **range** exceeds the number of variables in the factorgraph.
    pub fn variable_indices_ordered_by_creation(
        &self,
        range: Range<usize>,
    ) -> Option<Vec<NodeIndex>> {
        let within_range = range.end <= self.variable_indices_ordered.len();
        if within_range {
            Some(
                self.variable_indices_ordered
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

    // pub fn factors(&self) -> impl Iterator<Item = Node> {}

    /// A count over the number of variables and factors in the factorgraph
    ///
    /// **Computes in O(1) time**
    pub fn node_count(&self) -> NodeCount {
        self.node_count
    }

    pub fn nth_variable_index(&self, index: usize) -> Option<NodeIndex> {
        self.variable_indices_ordered.get(index).copied()
    }

    pub fn nth_variable(&self, index: usize) -> Option<&Variable> {
        let variable_index = self.nth_variable_index(index)?;
        let node = &self.graph[variable_index];
        node.as_variable()
    }

    pub fn nth_variable_mut(&mut self, index: usize) -> Option<&mut Variable> {
        let variable_index = self.nth_variable_index(index)?;
        let node = &mut self.graph[variable_index];
        node.as_variable_mut()
    }

    pub fn first_variable(&self) -> Option<&Variable> {
        self.nth_variable(0usize)
    }

    pub fn last_variable(&self) -> Option<&Variable> {
        self.nth_variable(self.variable_indices_ordered.len())
    }

    pub fn last_variable_mut(&mut self) -> Option<&mut Variable> {
        self.nth_variable_mut(self.variable_indices_ordered.len())
    }

    // TODO: Implement our own export to `DOT` format, which can be much more specific with styling.
    /// Exports tree to `graphviz` `DOT` format
    // pub fn export(&self) -> String {
    //     // println!("graph {{");
    //     // for node_index in self.graph.node_indices() {
    //     //     for neighbour_index in self.graph.neighbors(node_index) {
    //     //         println!("    {} -- {}", node_index.index(), neighbour_index.index());
    //     //     }
    //     // }
    //     // println!("}}");
    //     let mut output = String::new();
    //     output.push_str("subgraph {\n");
    //     output.push_str("    node [style=filled]\n");

    //     let mut connections_made = HashMap::<NodeIndex, NodeIndex>::new();

    //     for node_index in self.graph.node_indices() {
    //         let node = &self.graph[node_index];

    //         let shape = match node {
    //             Node::Factor(_) => "square",
    //             Node::Variable(_) => "circle",
    //         };

    //         let color = match node {
    //             Node::Factor(factor) => match factor.kind {
    //                 super::factor::FactorKind::InterRobot(_) => "\"#a6da95\"", // green
    //                 super::factor::FactorKind::Dynamic(_) => "\"#8aadf4\"",    // blue
    //                 super::factor::FactorKind::Obstacle(_) => "\"#c6a0f6\"", // mauve (purple)
    //                 super::factor::FactorKind::Pose(_) => "\"#ee99a0\"", // maroon (red)
    //             },
    //             Node::Variable(_) => "\"#eff1f5\"", // latte base (white)
    //         };

    //         let width = match node {
    //             Node::Factor(_) => "0.2",
    //             Node::Variable(_) => "0.8",
    //         };

    //         output.push_str(&format!(
    //             "    {} [shape={}, fillcolor={}, width={}]\n",
    //             node_index.index(),
    //             shape,
    //             color,
    //             width
    //         ));

    //         for neighbour_index in self.graph.neighbors(node_index) {
    //             if let Some(existing_neighbour) = connections_made.get(&neighbour_index) {
    //                 if *existing_neighbour == node_index {
    //                     continue;
    //                 }
    //             } else {
    //                 connections_made.insert(neighbour_index, node_index);
    //             }
    //         }
    //     }

    //     for (neighbour_index, node_index) in connections_made {
    //         output.push_str(&format!(
    //             "    {} -- {}\n",
    //             node_index.index(),
    //             neighbour_index.index()
    //         ));
    //     }

    //     output.push_str("}\n");
    //     output
    // }

    pub fn export_data(&self) -> (Vec<graphviz::Node>, Vec<graphviz::Edge>) {
        // let mut nodes = Vec::<graphviz::Node>::with_capacity(self.graph.node_count());
        // let mut edges = Vec::<graphviz::Edge>::with_capacity(self.graph.edge_count());

        let nodes = self
            .graph
            .node_indices()
            .map(|node_index| {
                let node = &self.graph[node_index];
                graphviz::Node {
                    index: node_index.index(),
                    kind: match node {
                        Node::Factor(factor) => match factor.kind {
                            FactorKind::Dynamic(_) => graphviz::NodeKind::DynamicFactor,
                            FactorKind::Obstacle(_) => graphviz::NodeKind::ObstacleFactor,
                            FactorKind::Pose(_) => graphviz::NodeKind::PoseFactor,
                            FactorKind::InterRobot(inner) => {
                                graphviz::NodeKind::InterRobotFactor(inner.connection.clone())
                                // graphviz::NodeKind::InterRobotFactor {
                                //     other_robot_id: inner.connection.id_of_robot_connected_with,
                                //     variable_index_in_other_robot: self.graph.neighbors(node_index).nth(0).expect("interrobot factor have exactly 1 variable node as neighbour").index()
                                // }
                            }
                        },
                        Node::Variable(variable) => {
                            let mean = variable.belief.mean();
                            graphviz::NodeKind::Variable {
                                x: mean[0],
                                y: mean[1],
                            }
                            // graphviz::NodeKind::Variable { x: 0.0, y: 0.0 }
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
                        to: to.index(),
                    })
            })
            .collect::<Vec<_>>();

        (nodes, edges)
    }

    /// Aggregate and marginalise over all adjacent variables, and send.
    /// Aggregation: product of all incoming messages
    pub fn factor_iteration(&mut self, robot_id: Entity, mode: MessagePassingMode) {
        // TODO: use rayon .par_iter()
        for node_index in self.graph.node_indices() {
            let node = &mut self.graph[node_index];
            if node.is_variable() {
                continue;
            }
            let factor_index = node_index;
            let adjacent_variables = self.graph.neighbors(factor_index).collect::<Vec<_>>();
            // let factor = self.graph[factor_index]
            //     .as_factor_mut()
            //     .expect("factor_index should point to a Factor in the graph");

            // Update factor and receive messages to send to its connected variables
            // let variable_messages =
            //     factor.update(factor_index, &adjacent_variables, &self.graph);
            // let graph_clone = self.graph.clone();
            // let _ = factor.update(factor_index, &adjacent_variables, &self.graph);
            let variable_messages = self.update_factor(node_index, adjacent_variables);

            let variable_indices = self.graph.neighbors(factor_index).collect::<Vec<_>>();
            // TODO: propagate the variable_messages to the factors neighbour variables
            for variable_index in variable_indices {
                let variable = self.graph[variable_index]
                    .as_variable_mut()
                    .expect("A factor can only have variables as neighbors");

                let message = variable_messages
                    .get(&variable_index)
                    .expect("There should be a message from the factor to the variable");
                variable.send_message(factor_index, message.clone());
            }
        }
    }

    fn update_factor(
        &mut self,
        factor_index: NodeIndex,
        adjacent_variables: Vec<NodeIndex>,
    ) -> HashMap<NodeIndex, Message> {
        let factor = self.graph[factor_index]
            .as_factor_mut()
            .expect("factor_index should point to a Factor in the graph");

        let mut idx = 0;
        // TODO: do not hardcode
        let dofs = 4;
        for &variable_index in adjacent_variables.iter() {
            idx += dofs;
            // let message = factor
            //     .read_message_from(variable_index)
            //     .expect("There should be a message from the variable");
            let message_mean = factor
                .read_message_from(variable_index)
                .map(|message| message.mean())
                .expect("There should be a message from the variable");

            factor
                .state
                .linearisation_point
                .slice_mut(s![idx..idx + dofs])
                .assign(&message_mean);
        }

        // *Depending on the problem*, we may need to skip computation of this factor.␍
        // eg. to avoid extra computation, factor may not be required if two connected variables are too far apart.␍
        // in which case send out a Zero Message.␍
        if factor.skip() {
            // for variable_index in adjacent_variables {
            //     let variable = self.graph[variable_index]
            //         .as_variable_mut()
            //         .expect("A factor can only have variables as neighbors");

            //     let message = Message::with_dofs(idx);
            //     variable.send_message(factor_index, message);
            // }
            // return false;

            let messages = adjacent_variables
                .iter()
                .map(|&variable_index| {
                    // let variable = self.graph[variable_index]
                    //     .as_variable_mut()
                    //     .expect("A factor can only have variables as neighbors");

                    let message = Message::with_dofs(idx);
                    // variable.send_message(factor_index, message);
                    (variable_index, message)
                })
                .collect::<HashMap<_, _>>();

            return messages;
        }

        let measurement = factor.measure(&factor.state.linearisation_point.clone());
        let jacobian = factor.jacobian(&factor.state.linearisation_point.clone());

        let factor_lam_potential = jacobian
            .t()
            .dot(&factor.state.measurement_precision)
            .dot(&jacobian);
        let factor_eta_potential = jacobian
            .t()
            .dot(&factor.state.measurement_precision)
            .dot(&(jacobian.dot(&factor.state.linearisation_point) - measurement));

        factor.mark_initialized();

        // update factor precision and information with incoming messages from connected variables.
        let mut marginalisation_idx = 0;
        let mut messages = HashMap::new();

        // let adjacent_variables_clone = adjacent_variables.clone();

        let dofs = 4;
        for &variable_index in adjacent_variables.iter() {
            // let variable = self.graph[variable_index]
            //     .as_variable()
            //     .expect("A factor can only have variables as neighbors");

            let mut factor_eta = factor_eta_potential.clone();
            let mut factor_lam = factor_lam_potential.clone();

            let mut idx_v = 0;
            for &v_idx in adjacent_variables.iter() {
                // let variable = self.graph[v_idx]
                //     .as_variable()
                //     .expect("A factor can only have variables as neighbors");

                if v_idx != variable_index {
                    let message = factor
                        .read_message_from(v_idx)
                        .expect("There should be a message from the variable");

                    let message_mean = message.mean();

                    // factor_eta += message_mean;
                    // factor_eta.add_assign(&message_mean);
                    factor_eta
                        .slice_mut(s![idx_v..idx_v + dofs])
                        .add_assign(&message_mean);
                    // factor_lam += message.0.precision_matrix;
                    // factor_lam.add_assign(&message.0.precision_matrix);
                    factor_lam
                        .slice_mut(s![idx_v..idx_v + dofs, idx_v..idx_v + dofs])
                        .add_assign(&message.precision_matrix());
                }
                idx_v += dofs;
            }

            // Marginalise the Factor Precision and Information to send to the relevant variable
            let message = marginalise_factor_distance::marginalise_factor_distance(
                factor_eta,
                factor_lam,
                variable_index.index(),
                marginalisation_idx,
            );
            messages.insert(variable_index, message);
            marginalisation_idx += dofs;
        }

        messages
    }

    /// Variable Iteration in Gaussian Belief Propagation (GBP).
    /// For each variable in the factorgraph:
    ///  - Messages are collected from the outboxes of each of the connected factors
    ///  - Variable belief is updated and outgoing message in the variable's outbox is created.
    ///
    ///  * Note: we deal with cases where the variable/factor iteration may need to be skipped:
    ///      - communications failure modes:
    ///          if interrobot_comms_active_ is false, variables and factors connected to
    ///          other robots should not take part in GBP iterations,
    ///      - message passing modes (INTERNAL within a robot's own factorgraph or EXTERNAL between a robot and other robots):
    ///          in which case the variable or factor may or may not need to take part in GBP depending on if it's connected to another robot
    pub fn variable_iteration(&mut self, robot_id: Entity, mode: MessagePassingMode) {
        // for (i, (v_key, variable)) in self.variables.iter().enumerate() {
        for variable_index in self.graph.node_indices() {
            let node = &mut self.graph[variable_index];
            if node.is_factor() {
                continue;
            }
            let variable = node
                .as_variable_mut()
                .expect("variable_index should point to a Variable in the graph");
            let factor_messages = variable.update_belief();

            // KEEP: for future debugging
            // info!(
            //     "{} messages received by variable: {:?}",
            //     factor_messages.len(),
            //     variable_index
            // );
            // factor_messages.iter().for_each(|(factor_index, message)| {
            //     let node = &self.graph[*factor_index];
            //     if node.is_factor() {
            //         info!("Message from factor: {:#?}", message);
            //     } else if node.is_variable() {
            //         info!("Message from variable: {:#?}", message);
            //     }
            // });

            // println!("factor_messages: {:#?}", factor_messages);

            for (factor_index, message) in factor_messages {
                let factor = self.graph[factor_index]
                    .as_factor_mut()
                    .expect("The variable can only send messages to factors");
                factor.send_message(variable_index, message);
            }

            // let variable = node.as_variable().expect("Node is a variable");
            // for (f_key, factor) in variable.adjacent_factors.iter() {
            //     // QUESTION(kpbaks): is this not always true? Given that only factors can be interract with other robots factorgraphs?
            //     let variable_in_robots_factorgraph = v_key.robot_id == robot_id;

            //     // if (mode == MessagePassingMode::Internal && variable_in_robots_factorgraph)
            //     //     || !self.interrobot_comms_active
            //     //         && variable_in_robots_factorgraph
            //     //         && mode == MessagePassingMode::External
            //     // {
            //     //     continue;
            //     // }

            //     match mode {
            //         MessagePassingMode::Internal if !variable_in_robots_factorgraph => {
            //             continue
            //         }
            //         MessagePassingMode::External
            //             if !variable_in_robots_factorgraph
            //                 && self.interrobot_comms_active =>
            //         {
            //             continue
            //         }
            //         _ => {}
            //     }

            //     // Read message from each connected factor
            //     // var->inbox_[f_key] = fac->outbox_.at(v_key);
            //     let message =
            //         factor.outbox.get(f_key).expect("f_key is in factor.outbox");
            //     variable.inbox.insert(*f_key, message.clone());
            // }

            // let adjacent_factors = self.graph.neighbors(node_index).collect::<Vec<_>>();

            // // Update variable belief and create outgoing messages
            // variable.update_belief(&adjacent_factors, &mut self.graph);
        }
    }

    pub(crate) fn delete_interrobot_factor_connected_to(
        &mut self,
        other: RobotId,
    ) -> Result<(), &'static str> {
        let node_idx = self
        .graph
        .node_indices()
        // Use `find_map` for a more concise filter-and-map operation
        .find_map(|node_idx| {
            let node = &self.graph[node_idx];
            node.as_factor()
                .and_then(|factor| factor.kind.as_inter_robot())
                // Extract `id_of_robot_connected_with` directly
                .filter(|interrobot| interrobot.connection.id_of_robot_connected_with == other)
                .map(|_interrobot| node_idx)
        })
        .ok_or("not found")?;

        // Directly remove the node using the fallible method
        self.graph
            .remove_node(node_idx)
            .ok_or("the interrobot factor does not exist in the graph")?;

        Ok(())
    }

    // /// TODO: should probably not be a method on the graph, but on the robot, but whatever
    // pub(crate) fn delete_interrobot_factor_connected_to(
    //     &mut self,
    //     other: RobotId,
    // ) -> Result<(), &'static str> {
    //     let node_idx = self
    //         .graph
    //         .node_indices()
    //         .filter_map(|node_idx| {
    //             let node = &self.graph[node_idx];
    //             let Some(factor) = node.as_factor() else {
    //                 return None;
    //             };

    //             let Some(interrobot) = factor.kind.as_inter_robot() else {
    //                 return None;
    //             };

    //             Some((node_idx, interrobot))
    //         })
    //         .find(|(_, interrobot)| interrobot.id_of_robot_connected_with == other)
    //         .map(|(node_idx, _)| node_idx);

    //     let Some(node_idx) = node_idx else {
    //         return Err("not found");
    //     };

    //     self.graph.remove_node(node_idx).expect(
    //         "The node index was retrieved from the graph in the previous statement",
    //     );

    //     Ok(())

    //     // let node_idx = self
    //     //     .graph
    //     //     .raw_nodes()
    //     //     .iter()
    //     //     .filter_map(|node| node.weight.as_factor())
    //     //     .filter_map(|factor| factor.kind.as_inter_robot())
    //     //     .find(|&interrobot| interrobot.id_of_robot_connected_with == other)
    //     //     .ok_or("not found")?;
    // }

    // fn update_variable(
    //     &mut self,
    //     variable_index: NodeIndex,
    //     indices_of_adjacent_factors: Vec<NodeIndex>,
    // ) -> HashMap<NodeIndex, Message> {
    //     let adjacent_factors = self.graph.neighbors(variable_index);
    //     // // Update variable belief and create outgoing messages
    //     // variable.update_belief(&adjacent_factors, &mut self.graph);

    //     todo!()
    // }
}
