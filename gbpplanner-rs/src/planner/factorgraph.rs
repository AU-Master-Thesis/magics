use std::cell::Cell;
use std::collections::HashMap;
use std::ops::AddAssign;

use bevy::prelude::*;
use ndarray::s;
// use nalgebra::{Matrix, Vector};
use petgraph::dot::{Config, Dot};
// use petgraph::prelude::{EdgeIndex, NodeIndex};
use petgraph::Undirected;

use super::factor::Factor;
use super::multivariate_normal::MultivariateNormal;
use super::variable::Variable;
use super::{Matrix, Vector};

/// How the messages are passed between factors and variables in the connected factorgraphs.
#[derive(Debug)]
pub enum MessagePassingMode {
    /// Messages are passed within a robot's own factorgraph.
    Internal,
    /// Messages are passed between a robot factorgraph and other robots factorgraphs.
    External,
}

#[derive(Debug, Clone)]
pub struct Message(pub MultivariateNormal<f32>);

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

    pub fn mean(&self) -> Vector<f32> {
        self.0.mean()
    }

    pub fn new(information_vector: Vector<f32>, precision_matrix: Matrix<f32>) -> Self {
        Self(MultivariateNormal::new(
            information_vector,
            precision_matrix,
        ))
    }

    pub fn zeros(dims: usize) -> Self {
        Self(MultivariateNormal::zeros(dims))
    }

    pub fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

pub type Inbox = HashMap<NodeIndex, Message>;

#[derive(Debug, Clone)]
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

pub type NodeIndex = petgraph::graph::NodeIndex;
pub type EdgeIndex = petgraph::graph::EdgeIndex;
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
    // graph: Cell<Graph>,
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
            let adjacent_variables =
                self.graph.neighbors(factor_index).collect::<Vec<_>>();
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
                // self.send_message_to_variable(variable_index, message.clone());
                variable.send_message(factor_index, message.clone());
            }
        }
    }

    // fn send_message_to_variable(&mut self, variable_index: NodeIndex, message: Message) {
    //     let variable = self.graph[variable_index]
    //         .as_variable_mut()
    //         .expect("A factor can only have variables as neighbors");
    //     variable.send_message(variable_index, message);
    // }

    // /*****************************************************************************************************/
    // Main section: Factor update:
    // Messages from connected variables are aggregated. The beliefs are used to create the linearisation point X_.
    // The Factor potential is calculated using h_func_ and J_func_
    // The factor precision and information is created, and then marginalised to create outgoing messages to its connected variables.
    // /*****************************************************************************************************/
    // bool Factor::update_factor(){

    //     // Messages from connected variables are aggregated.
    //     // The beliefs are used to create the linearisation point X_.
    //     int idx = 0; int n_dofs;
    //     for (int v=0; v<variables_.size(); v++){
    //         n_dofs = variables_[v]->n_dofs_;
    //         auto& [_, __, mu_belief] = this->inbox_[variables_[v]->key_];
    //         X_(seqN(idx, n_dofs)) = mu_belief;
    //         idx += n_dofs;
    //     }

    //     // *Depending on the problem*, we may need to skip computation of this factor.
    //     // eg. to avoid extra computation, factor may not be required if two connected variables are too far apart.
    //     // in which case send out a Zero Message.
    //     if (this->skip_factor()){
    //         for (auto var : variables_){
    //             this->outbox_[var->key_] = Message(var->n_dofs_);
    //         }
    //         return false;
    //     }

    //     // The Factor potential and linearised Factor Precision and Information is calculated using h_func_ and J_func_
    //     // residual() is by default (z - h_func_(X))
    //     // Skip calculation of Jacobian if the factor is linear and Jacobian has already been computed once
    //     h_ = h_func_(X_);
    //     J_ = (this->linear_ && this->initialised_)? J_ : this->J_func_(X_);
    //     Eigen::MatrixXd factor_lam_potential = J_.transpose() * meas_model_lambda_ * J_;
    //     Eigen::VectorXd factor_eta_potential = (J_.transpose() * meas_model_lambda_) * (J_ * X_ + residual());
    //     this->initialised_ = true;

    //     //  Update factor precision and information with incoming messages from connected variables.
    //     int marginalisation_idx = 0;
    //     for (int v_out_idx=0; v_out_idx<variables_.size(); v_out_idx++){
    //         auto var_out = variables_[v_out_idx];
    //         // Initialise with factor values
    //         Eigen::VectorXd factor_eta = factor_eta_potential;
    //         Eigen::MatrixXd factor_lam = factor_lam_potential;

    //         // Combine the factor with the belief from other variables apart from the receiving variable
    //         int idx_v = 0;
    //         for (int v_idx=0; v_idx<variables_.size(); v_idx++){
    //             int n_dofs = variables_[v_idx]->n_dofs_;
    //             if (variables_[v_idx]->key_ != var_out->key_) {
    //                 auto [eta_belief, lam_belief, _] = inbox_[variables_[v_idx]->key_];
    //                 factor_eta(seqN(idx_v, n_dofs)) += eta_belief;
    //                 factor_lam(seqN(idx_v, n_dofs), seqN(idx_v, n_dofs)) += lam_belief;
    //             }
    //             idx_v += n_dofs;
    //         }

    //         // Marginalise the Factor Precision and Information to send to the relevant variable
    //         outbox_[var_out->key_] = marginalise_factor_dist(factor_eta, factor_lam, v_out_idx, marginalisation_idx);
    //         marginalisation_idx += var_out->n_dofs_;
    //     }

    //     return true;
    // };

    fn update_factor(
        &mut self,
        factor_index: NodeIndex,
        // &mut factor: Factor,
        adjacent_variables: Vec<NodeIndex>,
    ) -> HashMap<NodeIndex, Message> {
        // let graph_clone = self.graph.clone();
        // let adjacent_variables = self.graph.neighbors(factor_index).collect::<Vec<_>>();
        let factor = self.graph[factor_index]
            .as_factor_mut()
            .expect("factor_index should point to a Factor in the graph");

        let mut idx = 0;
        let dofs = 4;
        for &variable_index in adjacent_variables.iter() {
            // let dofs = {
            //     self.graph[variable_index]
            //         .as_variable()
            //         .expect("A factor can only have variables as neighbors")
            //         .dofs
            // };
            idx += dofs;
            let message = factor
                .read_message_from(variable_index)
                .expect("There should be a message from the variable");

            let message_mean = message.mean();

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
                        .add_assign(&message.0.precision_matrix);
                }
                idx_v += dofs;
            }

            let message = factor.marginalise_factor_distance(
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
