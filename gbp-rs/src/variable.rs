use std::{collections::BTreeMap, rc::Rc};

use nalgebra::DVector;

use crate::{factor::Factor, multivariate_normal::MultivariateNormal, Key, Mailbox, Message};

/// A variable in the factor graph.
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

    pub fn new(key: Key, mut prior: MultivariateNormal, dofs: usize) -> Self {
        if !prior.precision_matrix.iter().all(|x| x.is_finite()) {
            // if (!lam_prior_.allFinite()) lam_prior_.setZero();
            prior.precision_matrix.fill(0.0);
        }
        Self {
            key,
            adjacent_factors: BTreeMap::new(),
            prior,
            belief: prior.clone(),
            dofs,
            valid: false,
            inbox: Mailbox::new(),
            outbox: Mailbox::new(),
        }
    }

    // Variable Belief Update step (Step 1 in the GBP algorithm)
    // Aggregates all the messages from its adjacent factors (begins with the prior, as this is effectively a unary factor)
    // Finally the outgoing messages to factors is created.
    // pub fn update_belief(&mut self, adjacent_factors: &[]) {}

    pub fn add_factor(&mut self, factor: Rc<Factor>) {
        self.adjacent_factors.insert(factor.key, factor);
        self.inbox.insert(factor.key, Message::with_dofs(self.dofs));
        self.outbox.insert(factor.key, Message(self.belief.clone()));
    }

    /// Delete a factor from this variable's list of factors. Remove it from its inbox too.
    pub fn delete_factor(&mut self, factor_key: Key) {
        let Some(_) = self.adjacent_factors.remove(&factor_key) else {
            eprintln!(
                "Factor with key {} not found in the adjacent factors of the variable with key {}",
                factor_key, self.key
            );
            return;
        };

        let Some(_) = self.inbox.remove(&factor_key) else {
            eprintln!(
                "Factor with key {} not found in the inbox of the variable with key {}",
                factor_key, self.key
            );
            return;
        };
    }

    pub fn change_prior(&mut self, mean: DVector<f32>) {
        self.prior.information_vector = self.prior.precision_matrix * mean;
        // QUESTION: why cache mu?
        // mu_ = new_mu;
        // belief_ = Message {eta_, lam_, mu_};

        for (f_key, factor) in self.adjacent_factors.iter() {
            if let Some(message) = self.outbox.get_mut(f_key) {
                // outbox_[fkey] = belief_;
                *message = Message(self.belief.clone());
            }
            if let Some(message) = self.inbox.get_mut(f_key) {
                // inbox_[fkey].setZero();
                message.zeroize();
            }
        }
    }

    // /***********************************************************************************************************/
    // // Variable belief update step:
    // // Aggregates all the messages from its connected factors (begins with the prior, as this is effectively a unary factor)
    // // The valid_ flag is useful for drawing the variable.
    // // Finally the outgoing messages to factors is created.
    // /***********************************************************************************************************/
    // void Variable::update_belief(){
    //     // Collect messages from all other factors, begin by "collecting message from pose factor prior"
    //     eta_ = eta_prior_;
    //     lam_ = lam_prior_;

    //     for (auto& [f_key, msg] : inbox_) {
    //         auto [eta_msg, lam_msg, _] = msg;
    //         eta_ += eta_msg;
    //         lam_ += lam_msg;
    //     }
    //     // Update belief
    //     sigma_ = lam_.inverse();

    //     valid_ = sigma_.allFinite();
    //     if (valid_) mu_ = sigma_ * eta_;
    //     belief_ = Message {eta_, lam_, mu_};

    //     // Create message to send to each factor that sent it stuff
    //     // msg is the aggregate of all OTHER factor messages (belief - last sent msg of that factor)
    //     for (auto [f_key, fac] : factors_) {
    //         outbox_[f_key] = belief_ - inbox_.at(f_key);
    //     }
    // }

    /// Variable Belief Update step (Step 1 in the GBP algorithm)
    ///
    pub fn update_belief(&mut self) {
        // Collect messages from all other factors, begin by "collecting message from pose factor prior"
        self.belief.information_vector = self.prior.information_vector.clone();
        self.belief.precision_matrix = self.prior.precision_matrix.clone();

        for (f_key, msg) in self.inbox.iter() {
            self.belief.information_vector += msg.0.information_vector.clone();
            self.belief.precision_matrix += msg.0.precision_matrix.clone();
        }

        for (_, message) in self.inbox.iter() {
            self.belief.information_vector += &message.0.information_vector;
            self.belief.precision_matrix += &message.0.precision_matrix;
        }

        // Update belief
        let covariance = self
            .belief
            .precision_matrix
            .clone()
            .try_inverse()
            .expect("Precision matrix should be nonsingular");

        let valid = covariance.iter().all(|x| x.is_finite());
        if valid {
            // TODO: is this meaningful?
            // if (valid_) mu_ = sigma_ * eta_;
        }

        // belief_ = Message {eta_, lam_, mu_};

        // Create message to send to each factor
        // Message is teh aggregate of all OTHER factor messages (belief - last sent msg of that factor)
        for (f_key, factor) in self.adjacent_factors.iter() {
            if let Some(message) = self.outbox.get_mut(f_key) {
                *message = Message(self.belief
                    - self
                        .inbox
                        .get(f_key)
                        .expect("The message should exist in the inbox").0);
            }
        }
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
