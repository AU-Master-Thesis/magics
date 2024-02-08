use std::rc::Rc;

use crate::{factor::Factor, multivariate_normal::MultivariateNormal};
// use crate::NodeId;

// use std::sync::mpsc::channel;

#[derive(Debug)]
pub struct Variable {
    adjacent_factors: Vec<Rc<Factor>>,
    prior: MultivariateNormal,
    belief: MultivariateNormal,
    dofs: usize,
    valid: bool,
}

impl Variable {
    /// Variable Belief Update step (Step 1 in the GBP algorithm)
    /// Aggregates all the messages from its adjacent factors (begins with the prior, as this is effectively a unary factor)
    /// Finally the outgoing messages to factors is created.
    pub fn update_belief(&mut self) {}
}
