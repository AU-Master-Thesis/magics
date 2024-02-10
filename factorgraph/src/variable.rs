use std::rc::Rc;

use crate::{factor::Factor, multivariate_normal::MultivariateNormal};

#[derive(Debug)]
struct VariableBelief {

}

#[derive(Debug)]
pub struct Variable {
    // TODO: how do other nodes access the variable?
    adjacent_factors: Vec<Rc<Factor>>,
    prior: MultivariateNormal,
    belief: MultivariateNormal,
    dofs: usize,
    /// Flag to indicate if the variable's covariance is finite, i.e. it does not contain NaNs or Infs
    valid: bool,
}

impl Variable {
    /// Variable Belief Update step (Step 1 in the GBP algorithm)
    /// Aggregates all the messages from its adjacent factors (begins with the prior, as this is effectively a unary factor)
    /// Finally the outgoing messages to factors is created.
    pub fn update_belief(&mut self) {}

    // fn 
}
