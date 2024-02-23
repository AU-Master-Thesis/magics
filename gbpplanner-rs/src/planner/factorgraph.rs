use std::collections::HashMap;

use bevy::prelude::*;
use nalgebra::DVector;
use petgraph::dot::{Config, Dot};
use petgraph::prelude::{EdgeIndex, NodeIndex};
use petgraph::Undirected;

use crate::utils;

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
            let factor_index = node_index;
            let factor = self.graph[factor_index]
                .as_factor_mut()
                .expect("factor_index should point to a Factor in the graph");

            // Update factor and receive messages to send to its connected variables
            // let variable_messages = factor.update(factor_index, & self.graph);
            let adjacent_variables =
                self.graph.neighbors(factor_index).collect::<Vec<_>>();
            let _ = factor.update(factor_index, &adjacent_variables, &self.graph);

            // TODO: propagate the variable_messages to the factors neighbour variables
            // for variable_index in self.graph.neighbors(factor_index) {
            //     let variable = self.graph[variable_index]
            //         .as_variable()
            //         .expect("A factor can only have variables as neighbors");

            //     let message = variable_messages
            //         .get(variable_index)
            //         .expect("There should be a message from the factor to the variable");
            //     variable.send_message(factor_index, message);
            // }
        }
    }

    fn update_factor(
        &mut self,
        factor_index: NodeIndex,
        // &mut factor: Factor,
        // adjacent_variables: impl Iterator<Item = NodeIndex>,
    ) -> HashMap<NodeIndex, Message> {
        let factor = self.graph[factor_index]
            .as_factor_mut()
            .expect("factor_index should point to a Factor in the graph");
        let adjacent_variables = self.graph.neighbors(factor_index);

        let mut idx = 0;
        for variable_index in adjacent_variables.clone() {
            let variable = self.graph[variable_index]
                .as_variable()
                .expect("A factor can only have variables as neighbors");

            idx += variable.dofs;
            let message = factor
                .read_message_from(variable_index)
                .expect("There should be a message from the variable");

            utils::nalgebra::insert_subvector(
                &mut factor.state.linearisation_point,
                idx..idx + variable.dofs,
                &message.mean(),
            );
        }

        // *Depending on the problem*, we may need to skip computation of this factor.␍
        // eg. to avoid extra computation, factor may not be required if two connected variables are too far apart.␍
        // in which case send out a Zero Message.␍
        if factor.skip() {
            for variable_index in adjacent_variables {
                let variable = self.graph[variable_index]
                    .as_variable_mut()
                    .expect("A factor can only have variables as neighbors");

                let message = Message::with_dofs(idx);
                variable.send_message(factor_index, message);
            }
            return false;
        }

        let measurement = factor.measurement(factor.state.linearisation_point);
        let jacobian = factor.jacobian(factor.state.linearisation_point);

        todo!()
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
            self.update_variable(node_index);
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

    fn update_variable(&mut self, variable_index: NodeIndex) {
        // let adjacent_factors = self.graph.neighbors(node_index).collect::<Vec<_>>();
        let adjacent_factors = self.graph.neighbors(variable_index);
        // // Update variable belief and create outgoing messages
        // variable.update_belief(&adjacent_factors, &mut self.graph);

        todo!()
    }
}
