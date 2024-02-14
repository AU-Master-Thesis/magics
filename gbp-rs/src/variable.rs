use std::rc::Rc;

use crate::{factor::Factor, multivariate_normal::MultivariateNormal, message::Message};

#[derive(Debug)]
struct VariableBelief {}

#[derive(Debug)]
pub struct Variable {
    // TODO: how do other nodes access the variable?
    // TODO: are these only factors that belongs to the same factorgraph/robot as self?
    adjacent_factors: Vec<Rc<Factor>>,
    prior: MultivariateNormal,
    belief: MultivariateNormal,
    pub dofs: usize,
    /// Flag to indicate if the variable's covariance is finite, i.e. it does not contain NaNs or Infs
    /// In gbpplanner it is used to control if a variable can be rendered.
    valid: bool,
}

impl Variable {
    /// Variable Belief Update step (Step 1 in the GBP algorithm)
    /// Aggregates all the messages from its adjacent factors (begins with the prior, as this is effectively a unary factor)
    /// Finally the outgoing messages to factors is created.
    // pub fn update_belief(&mut self, adjacent_factors: &[]) {}

    // fn
}


fn update_variable_belief(var: &mut Variable, messages_of_adjacent_factors: &[MultivariateNormal]) {
    // let information_vector = adjacent_factors.iter().map(|f| f.state.)
    // let (updated_precision_matrix, )
    let updated_belief = messages_of_adjacent_factors
    .iter()
    .sum();

    var.belief = updated_belief;


    // var.belief = adjacent_factors.iter().product()
}
