use crate::factor::Factor;
use crate::robot::RobotId;
use crate::variable::Variable;
use crate::Key;

use std::collections::BTreeMap;
use std::rc::Rc;

// use rayon::prelude::*;

/// How the messages are passed between factors and variables in the connected factorgraphs.
#[derive(Debug)]
pub enum MessagePassingMode {
    /// Messages are passed within a robot's own factorgraph.
    Internal,
    /// Messages are passed between a robot factorgraph and other robots factorgraphs.
    External,
}

/// A factor graph is a bipartite graph consisting of two types of nodes: factors and variables.
/// Factors and variables are stored in separate btree maps, that are indexed by a unique tuple of (robot_id, node_id).
#[derive(Debug)]
pub struct FactorGraph {
    /// Called `factors_` in **gbpplanner**.
    /// **gbpplanner** uses `std::map<Key, std::shared_ptr<Factor>>`
    /// So we use `BTreeMap` as it provides iteration sorted by the `Key` similar to `std::map` in C++.
    pub factors: BTreeMap<Key, Rc<Factor>>,
    /// Called `variables_` in **gbpplanner**.
    /// **gbpplanner** uses `std::map<Key, std::shared_ptr<Variable>>`
    /// So we use `BTreeMap` as it provides iteration sorted by the `Key` similar to `std::map` in C++.
    pub variables: BTreeMap<Key, Rc<Variable>>,
    /// Flag for whether this factorgraph/robot communicates with other robots
    interrobot_comms_active: bool,
}

impl FactorGraph {
    pub fn new() -> Self {
        Self {
            factors: BTreeMap::new(),
            variables: BTreeMap::new(),
            interrobot_comms_active: true,
        }
    }

    /// Access the i'th variable within the factorgraph
    /// Called `getVar(const int& i)` in **gbpplanner**
    pub fn get_variable_by_index(&self, index: usize) -> Option<Rc<Variable>> {
        if self.variables.is_empty() {
            return None;
        }
        let index = index % self.variables.len();
        // TODO: is there a more rust-idiomatic way to do this? i.e. not cloning the Rc
        Some(
            self.variables
                .values()
                .nth(index)
                .expect("index is within [0, len)")
                .clone(),
        )
    }

    /// Access the variable by a specific key
    /// Called `getVar(const Key& v_key)` in **gbpplanner**
    pub fn get_variable_by_key(&self, key: &Key) -> Option<Rc<Variable>> {
        self.variables.get(key).cloned()
    }

    /// Aggregate and marginalise over all adjacent variables, and send.
    /// Aggregation: product of all incoming messages
    pub fn factor_iteration(&mut self, robot_id: RobotId, mode: MessagePassingMode) {
        // TODO: use rayon .par_iter()
        for (i, (f_key, &factor)) in self.factors.iter().enumerate() {
            for (v_key, &variable) in factor.adjacent_variables.iter() {
                // Check if the factor needs to be skipped
                let variable_in_robots_factorgraph = v_key.robot_id == robot_id;

                // Check if the factor need to be skipped [see note in description]
                // if (((msg_passing_mode==INTERNAL) == (var->key_.robot_id_!=this->robot_id_) ||
                // (!interrobot_comms_active_ && (var->key_.robot_id_!=this->robot_id_) && (msg_passing_mode==EXTERNAL)))) continue;

                match mode {
                    MessagePassingMode::Internal if !variable_in_robots_factorgraph => continue,
                    MessagePassingMode::External
                        if !variable_in_robots_factorgraph && self.interrobot_comms_active =>
                    {
                        continue
                    }
                    _ => {}
                }

                // Read message from each connected variable
                let message = variable
                    .outbox
                    .get(f_key)
                    .expect("f_key is in variable.outbox");
                factor.inbox.insert(v_key, message.clone());
            }

            // Calculate factor potential and create outgoing messages
            factor.update();
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
    pub fn variable_iteration(&mut self, robot_id: RobotId, mode: MessagePassingMode) {
        // TODO: use rayon .par_iter()
        for (i, (v_key, &variable)) in self.variables.iter().enumerate() {
            for (f_key, factor) in variable.adjacent_factors.iter() {
                // QUESTION(kpbaks): is this not always true? Given that only factors can be interract with other robots factorgraphs?
                let variable_in_robots_factorgraph = v_key.robot_id == robot_id;

                // if (mode == MessagePassingMode::Internal && variable_in_robots_factorgraph)
                //     || !self.interrobot_comms_active
                //         && variable_in_robots_factorgraph
                //         && mode == MessagePassingMode::External
                // {
                //     continue;
                // }

                match mode {
                    MessagePassingMode::Internal if !variable_in_robots_factorgraph => continue,
                    MessagePassingMode::External
                        if !variable_in_robots_factorgraph && self.interrobot_comms_active =>
                    {
                        continue
                    }
                    _ => {}
                }

                // Read message from each connected factor
                // var->inbox_[f_key] = fac->outbox_.at(v_key);
                let message = factor.outbox.get(f_key).expect("f_key is in factor.outbox");
                variable.inbox.insert(f_key, message.clone());
            }

            // Update variable belief and create outgoing messages
            variable.update_belief();
        }
    }
}
