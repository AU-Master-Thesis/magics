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

use std::rc::Rc;

pub use factor::Factor;
pub use variable::Variable;

pub type NodeId = usize;
pub type FactorGraphId = usize;

#[derive(Debug)]
pub enum MessagePassingMode {
    Internal,
    External,
}

/// A factor graph is a bipartite graph consisting of two types of nodes: factors and variables.
#[derive(Debug)]
pub struct FactorGraph {
    // pub id: FactorGraphId,
    pub factors: Vec<Rc<Factor>>,
    pub variables: Vec<Rc<Variable>>,
    // TODO: implmement such that the communication flag is per robot factorgraph connection
    // i.e. if gA is connected to gB, and gA not connected to gC, then gA should still be able to communicate with gB
    interrobot_comms_active: bool,
}

impl FactorGraph {
    pub fn new() -> Self {
        Self {
            // id,
            factors: Vec::new(),
            variables: Vec::new(),
            interrobot_comms_active: false,
        }
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

    // TODO: come up with a method name
    /// Aggregate over all adjacent factors, and send.
    /// Aggregation: product of all incoming messages
    pub fn variable_iteration(&mut self, mode: MessagePassingMode) {
        for (i, variable) in self.variables.iter().enumerate() {}
    }
}
