pub mod bbox;
pub mod factor;
pub mod message;
pub mod multivariate_normal;
pub mod robot;
pub mod variable;
pub mod factors;

pub mod prelude {
    pub use super::Factor;
    pub use super::FactorGraph;
    pub use super::MessagePassingMode;
    pub use super::Variable;
}

use std::{
    collections::{BTreeMap, HashMap},
    rc::Rc,
};

pub use factor::Factor;
use multivariate_normal::MultivariateNormal;
use robot::RobotId;
pub use variable::Variable;

pub type NodeId = usize;
pub type FactorGraphId = usize;

use rayon::prelude::*;

/// Datastructure used to identify both variables and factors
/// It includes the id of the robot that the variable/factor belongs to, as well as its own id.
#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Clone)]
pub struct Key {
    pub robot_id: RobotId,
    pub node_id: NodeId,
    // valid: bool, // Set in gbpplanner as `valid_` but not used
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(robot_id: {}, node_id: {})", self.robot_id, self.node_id)
    }
}


impl Key {
    pub fn new(robot_id: RobotId, node_id: NodeId) -> Self {
        Self {
            robot_id,
            node_id,
            // valid: true,
        }
    }
}


#[derive(Debug)]
pub struct Message(MultivariateNormal);

impl Message {
    pub fn with_dofs(dofs: usize) -> Self {
        Self(MultivariateNormal::zeros(dofs))
    }
}


pub type Mailbox = HashMap<Key, Message>;
/*
impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.robot_id == other.robot_id && self.node_id == other.node_id
    }
}

impl Eq for Key {}*/

#[derive(Debug)]
pub enum MessagePassingMode {
    Internal,
    External,
}

/// A factor graph is a bipartite graph consisting of two types of nodes: factors and variables.
#[derive(Debug)]
pub struct FactorGraph {
    // pub factors: Vec<Rc<Factor>>,
    /// Called `factors_` in **gbpplanner**.
    /// **gbpplanner** uses `std::map<Key, std::shared_ptr<Factor>>`
    /// So we use `BTreeMap` as it provides iteration sorted by the `Key` similar to `std::map` in C++.
    pub factors: BTreeMap<Key, Rc<Factor>>,
    // pub variables: Vec<Rc<Variable>>,
    /// Called `variables_` in **gbpplanner**.
    /// **gbpplanner** uses `std::map<Key, std::shared_ptr<Variable>>`
    /// So we use `BTreeMap` as it provides iteration sorted by the `Key` similar to `std::map` in C++.
    pub variables: BTreeMap<Key, Rc<Variable>>,
    // TODO: implmement such that the communication flag is per robot factorgraph connection
    // i.e. if gA is connected to gB, and gA not connected to gC, then gA should still be able to communicate with gB
    /// Flag for whether this factorgraph/robot communicates with other robots
    interrobot_comms_active: bool,
}

impl FactorGraph {
    pub fn new() -> Self {
        Self {
            // id,
            factors: BTreeMap::new(),
            variables: BTreeMap::new(),
            interrobot_comms_active: false,
        }
    }

    /// Access the i'th variable within the factorgraph
    pub fn get_variable_by_index(&self, index: usize) -> Option<Rc<Variable>> {
        if self.variables.is_empty() {
            return None;
        }

        Some(self.variables[index % self.variables.len()])
    }

    /// Access the variable by a specific key
    /// Called `getVar()` in **gbpplanner**
    pub fn get_variable_by_key(&self, key: &Key) -> Option<Rc<Variable>> {
        self.variables.get(key)
    }

    // TODO: come up with a method name
    /// Aggregate and marginalise over all adjacent variables, and send.
    /// Aggregation: product of all incoming messages
    pub fn factor_iteration(&mut self, mode: MessagePassingMode) {
        for (i, factor) in self.factors.iter().enumerate() {
            for variable in factor.adjacent_variables.iter() {
                // Check if the factor needs to be skipped
                // let variable_in_internal_graph = variable.graph_id == self.id;
                let variable_in_internal_graph = false;

                match mode {
                    MessagePassingMode::Internal if !variable_in_internal_graph => continue,
                    MessagePassingMode::External
                        if variable_in_internal_graph || self.interrobot_comms_active =>
                    {
                        continue
                    }
                    _ => {} // Remove this empty match arm
                }

                // Read message from each connected variable
            }

            // Calculate factor potential and create outgoing messages
            // *factor.update();
            // factor.up
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
    pub fn variable_iteration(&mut self, mode: MessagePassingMode) {
        // TODO: use rayon .par_iter()
        for (i, (v_key, variable)) in self.variables.iter().enumerate() {
            for (f_key, factor) in variable.adjacent_factors.iter() {
                let factor_in_internal_graph = v_key.robot_id == f_key.robot_id;

                if (mode == MessagePassingMode::Internal && factor_in_internal_graph)
                    || !self.interrobot_comms_active
                        && factor_in_internal_graph
                        && mode == MessagePassingMode::External
                {
                    continue;
                }
            }
        }
    }
}
