use std::{collections::BTreeMap, rc::Rc};

use nalgebra::DVector;

use crate::{
    factor::Factor, multivariate_normal::MultivariateNormal, Key, Mailbox, Message,
};

#[derive(Debug)]
struct VariableBelief {}

#[derive(Debug)]
pub struct Variable {
    /// Unique identifier that associates the variable with a factorgraph/robot.
    pub key: Key,
    /// Called `factors_` in **gbpplanner**.
    /// **gbpplanner** uses `std::map<Key, std::shared_ptr<Factor>>`
    /// So we use `BTreeMap` as it provides iteration sorted by the `Key` similar to `std::map` in C++.
    pub adjacent_factors: BTreeMap<Key, Rc<Factor>>,
    /// In **gbpplanner** the `prior` is stored in 2 separate variables:
    /// 1. `eta_prior_` Information vector of prior on variable (essentially like a unary factor)
    /// 2. `lam_prior_` Precision matrix of prior on variable (essentially like a unary factor)
    pub prior: MultivariateNormal,
    pub belief: MultivariateNormal,
    /// Degrees of freedom. For 2D case n_dofs_ = 4 ([x,y,xdot,ydot])
    pub dofs: usize,
    /// Flag to indicate if the variable's covariance is finite, i.e. it does not contain NaNs or Infs
    /// In gbpplanner it is used to control if a variable can be rendered.
    pub valid: bool,
    /// Mailboxes for message storage
    pub inbox: Mailbox,
    pub outbox: Mailbox,
}

impl Variable {
    // Variable Belief Update step (Step 1 in the GBP algorithm)
    // Aggregates all the messages from its adjacent factors (begins with the prior, as this is effectively a unary factor)
    // Finally the outgoing messages to factors is created.
    // pub fn update_belief(&mut self, adjacent_factors: &[]) {}

    pub fn add_factor(&mut self, factor: Rc<Factor>) {
        self.adjacent_factors.insert(factor.key, factor);
        self.inbox.insert(factor.key, Message::with_dofs(self.dofs));
        self.outbox.insert(factor.key, self.belief.clone());
    }

    /// Delete a factor from this variable's list of factors. Remove it from its inbox too.
    pub fn delete_factor(&mut self, factor_key: Key) {
        let Some(_) = self.adjacent_factors.remove(&factor_key) else {
            eprintln!("Factor with key {} not found in the adjacent factors of the variable with key {}", factor_key, self.key);
            return;
        };

        let Some(_) = self.inbox.remove(&factor_key) else {
            eprintln!("Factor with key {} not found in the inbox of the variable with key {}", factor_key, self.key);
            return;
        };
    }

    // void Variable::change_variable_prior(const Eigen::VectorXd& new_mu){
    //     eta_prior_ = lam_prior_ * new_mu;
    //     mu_ = new_mu;
    //     belief_ = Message {eta_, lam_, mu_};
    //     for (auto [fkey, fac] : factors_){
    //         outbox_[fkey] = belief_;
    //         inbox_[fkey].setZero();
    //     }
    // };
    

    pub fn change_prior(&mut self, mean: DVector<f32>) {
        self.prior.information_vector = self.prior.precision_matrix * mean;
        // QUESTION: why cache mu?
        // mu_ = new_mu;
        // belief_ = Message {eta_, lam_, mu_};
    
        for (f_key, factor) in self.adjacent_factors.iter() {
            if let Some(message) = self.outbox.get_mut(f_key) {
                // outbox_[fkey] = belief_;
                *message = self.belief.clone();
            }
            if let Some(message) = self.inbox.get_mut(f_key) {
                // inbox_[fkey].setZero();
                message.zeroize();
            }
        }
    }

    pub fn update_belief(&mut self) {
        unimplemented!()
    }

}

// fn update_variable_belief(var: &mut Variable, messages_of_adjacent_factors: &[MultivariateNormal]) {
//     // let information_vector = adjacent_factors.iter().map(|f| f.state.)
//     // let (updated_precision_matrix, )
//     // let updated_belief = messages_of_adjacent_factors
//     // .iter()
//     // .sum();

//     // var.belief = updated_belief;

//     // var.belief = adjacent_factors.iter().product()
// }
