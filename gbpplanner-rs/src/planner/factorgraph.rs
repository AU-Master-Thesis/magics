use std::collections::HashMap;

use bevy::prelude::*;
use nalgebra::DVector;
use petgraph::dot::{Config, Dot};
use petgraph::prelude::{EdgeIndex, NodeIndex};
use petgraph::Undirected;

use super::factor::Factor;
use super::multivariate_normal::MultivariateNormal;
use super::variable::Variable;

/// How the messages are passed between factors and variables in the connected factorgraphs.
#[derive(Debug)]
pub enum MessagePassingMode {
    /// Messages are passed within a robot's own factorgraph.
    Internal,
    /// Messages are passed between a robot factorgraph and other robots factorgraphs.
    External,
}

#[derive(Debug, Clone)]
pub struct Message(pub MultivariateNormal);

/// Overload subtraction for `Message`
impl std::ops::Sub for Message {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Message(self.0 - rhs.0)
    }
}

impl std::ops::Sub<&Message> for Message {
    type Output = Self;
    fn sub(self, rhs: &Self) -> Self::Output {
        Message(self.0 - &rhs.0)
    }
}

impl Message {
    pub fn with_dofs(dofs: usize) -> Self {
        Self(MultivariateNormal::zeros(dofs))
    }

    pub fn mean(&self) -> DVector<f32> {
        self.0.mean()
    }
}

pub type Inbox = HashMap<NodeIndex, Message>;

#[derive(Debug)]
pub enum Node {
    Factor(Factor),
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

pub type Graph = petgraph::graph::Graph<Node, (), Undirected>;

/// A factor graph is a bipartite graph consisting of two types of nodes: factors and variables.
/// Factors and variables are stored in separate btree maps, that are indexed by a unique tuple of (robot_id, node_id).
#[derive(Component, Debug)]
pub struct FactorGraph {
    // /// Called `factors_` in **gbpplanner**.
    // /// **gbpplanner** uses `std::map<Key, std::shared_ptr<Factor>>`
    // /// So we use `BTreeMap` as it provides iteration sorted by the `Key` similar to `std::map` in C++.
    // pub factors: BTreeMap<Key, Rc<Factor>>,
    // /// Called `variables_` in **gbpplanner**.
    // /// **gbpplanner** uses `std::map<Key, std::shared_ptr<Variable>>`
    // /// So we use `BTreeMap` as it provides iteration sorted by the `Key` similar to `std::map` in C++.
    // pub variables: BTreeMap<Key, Rc<Variable>>,
    graph: Graph,
    /// Flag for whether this factorgraph/robot communicates with other robots
    interrobot_comms_active: bool,
    /// Node id counter
    node_id_counter: usize,
}

impl FactorGraph {
    pub fn new() -> Self {
        Self {
            graph: Graph::new_undirected(),
            interrobot_comms_active: true,
            node_id_counter: 0usize,
        }
    }

    pub fn next_node_id(&mut self) -> usize {
        self.node_id_counter += 1;
        self.node_id_counter
    }

    pub fn add_variable(&mut self, variable: Variable) -> NodeIndex {
        let node_index = self.graph.add_node(Node::Variable(variable));
        self.graph[node_index].set_node_index(node_index);
        node_index
    }

    pub fn add_factor(&mut self, factor: Factor) -> NodeIndex {
        let node_index = self.graph.add_node(Node::Factor(factor));
        self.graph[node_index].set_node_index(node_index);
        node_index
    }

    pub fn add_edge(&mut self, a: NodeIndex, b: NodeIndex) -> EdgeIndex {
        self.graph.add_edge(a, b, ())
    }

    // TODO: Implement our own export to `DOT` format, which can be much more specific with styling.
    /// Exports tree to `graphviz` `DOT` format
    pub fn export(&self) -> String {
        format!(
            "{:?}",
            Dot::with_config(&self.graph, &[Config::EdgeNoLabel])
        )
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
            let factor = node.as_factor().expect("Node is a factor");

            // for node_index in self.graph.neighbors(node_index) {
            //     let node = &mut self.graph[node_index];
            //     let variable = node.as_variable().expect("Node is a variable, since a factor cannot have another factor as a neighbour");

            //     // Check if the factor needs to be skipped
            //     // let v_key = variable.key;
            //     // let variable_in_robots_factorgraph = v_key.robot_id == robot_id;

            //     // if matches!(factor.kind, Factor::InterRobot) {
            //     //     let variable_in_robots_factorgraph =
            //     //         factor.id_of_robot_connected_with == robot_id;
            //     // }

            //     let variable_in_robots_factorgraph = match factor.kind {
            //         FactorKind::InterRobot(f) => f.id_of_robot_connected_with == robot_id,
            //         _ => true,
            //     };

            //     // Check if the factor need to be skipped [see note in description]
            //     // if (((msg_passing_mode==INTERNAL) == (var->key_.robot_id_!=this->robot_id_) ||
            //     // (!interrobot_comms_active_ && (var->key_.robot_id_!=this->robot_id_) && (msg_passing_mode==EXTERNAL)))) continue;

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

            //     // // Read message from each connected variable
            //     // let message = variable
            //     //     .outbox
            //     //     .get(f_key)
            //     //     .expect("f_key is in variable.outbox");
            //     // factor.inbox.insert(node_index, message.clone());
            // }

            let adjacent_variables = self.graph.neighbors(node_index).collect::<Vec<_>>();

            // self.graph.node_weights_mut().filter

            // Calculate factor potential and create outgoing messages
            factor.update(&adjacent_variables, &mut self.graph);
        }
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
        // TODO: use rayon .par_iter()
        // for (i, (v_key, variable)) in self.variables.iter().enumerate() {
        for node_index in self.graph.node_indices() {
            let node = &mut self.graph[node_index];
            if node.is_factor() {
                continue;
            }
            let variable = node.as_variable().expect("Node is a variable");
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

            let adjacent_factors = self.graph.neighbors(node_index).collect::<Vec<_>>();

            // Update variable belief and create outgoing messages
            variable.update_belief(&adjacent_factors, &mut self.graph);
        }
    }
}
