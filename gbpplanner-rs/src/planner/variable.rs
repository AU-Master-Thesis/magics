use super::factor::Factor;
use super::factorgraph::{Inbox, Message};
use super::multivariate_normal::MultivariateNormal;
use nalgebra::DVector;
use petgraph::prelude::NodeIndex;

/// A variable in the factor graph.
#[derive(Debug)]
pub struct Variable {
    /// Unique identifier that associates the variable with a factorgraph/robot.
    pub node_index: Option<NodeIndex>,
    /// Called `factors_` in **gbpplanner**.
    /// **gbpplanner** uses `std::map<Key, std::shared_ptr<Factor>>`
    /// So we use `BTreeMap` as it provides iteration sorted by the `Key` similar to `std::map` in C++.
    // pub adjacent_factors: BTreeMap<Key, Rc<Factor>>,
    /// In **gbpplanner** the `prior` is stored in 2 separate variables:
    /// 1. `eta_prior_` Information vector of prior on variable (essentially like a unary factor)
    /// 2. `lam_prior_` Precision matrix of prior on variable (essentially like a unary factor)
    pub prior: MultivariateNormal,
    pub belief: MultivariateNormal,
    /// Degrees of freedom. For 2D case n_dofs_ = 4 ([x,y,xdot,ydot])
    pub dofs: usize,
    /// Flag to indicate if the variable's covariance is finite, i.e. it does not contain NaNs or Infs
    /// In gbpplanner it is used to control if a variable can be rendered.
    // pub valid: bool,
    /// Mailbox for incoming message storage
    pub inbox: Inbox,
}

impl Variable {
    pub fn new(mut prior: MultivariateNormal, dofs: usize) -> Self {
        if !prior.precision_matrix.iter().all(|x| x.is_finite()) {
            // if (!lam_prior_.allFinite()) lam_prior_.setZero();
            prior.precision_matrix.fill(0.0);
        }
        Self {
            node_index: None,
            // key,
            // adjacent_factors: BTreeMap::new(),
            prior: prior.clone(),
            belief: prior,
            dofs,
            // valid: false,
            inbox: Inbox::new(),
        }
    }

    pub fn set_node_index(&mut self, node_index: NodeIndex) {
        if self.node_index.is_some() {
            panic!("The node index is already set");
        }
        self.node_index = Some(node_index);
    }

    pub fn get_node_index(&self) -> NodeIndex {
        if self.node_index.is_none() {
            panic!("The node index has not been set");
        }
        self.node_index.expect("I checked it was there 3 lines ago")
    }

    /// Change the prior of the variable.
    /// It updates the belief of the variable.
    pub fn change_prior(
        &mut self,
        mean: DVector<f32>,
        adjacent_factors: &mut [(NodeIndex, Factor)],
    ) {
        self.prior.information_vector = self.prior.precision_matrix.clone() * mean;
        // QUESTION: why cache mu?
        // mu_ = new_mu;
        // belief_ = Message {eta_, lam_, mu_};

        for (factor_node_index, factor) in adjacent_factors.iter_mut() {
            factor
                .inbox
                .insert(self.get_node_index(), Message(self.belief.clone()));
            self.inbox.insert(
                *factor_node_index,
                Message(MultivariateNormal::zeros(self.dofs)),
            );
        }
    }

    // /***********************************************************************************************************/
    // // Variable belief update step:
    // // Aggregates all the messages from its connected factors (begins with the prior, as this is effectively a unary factor)
    // // The valid_ flag is useful for drawing the variable.
    // // Finally the outgoing messages to factors is created.
    // /***********************************************************************************************************/
    /// Variable Belief Update step (Step 1 in the GBP algorithm)
    ///
    pub fn update_belief(&mut self, adjacent_factors: &mut [(NodeIndex, Factor)]) {
        // Collect messages from all other factors, begin by "collecting message from pose factor prior"
        self.belief.information_vector = self.prior.information_vector.clone();
        self.belief.precision_matrix = self.prior.precision_matrix.clone();

        for (_, message) in self.inbox.iter() {
            self.belief.information_vector += message.0.information_vector.clone();
            self.belief.precision_matrix += message.0.precision_matrix.clone();
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
        // Message is the aggregate of all OTHER factor messages (belief - last sent msg of that factor)
        for (factor_node_index, factor) in adjacent_factors.iter_mut() {
            factor.inbox.insert(
                self.get_node_index(),
                Message(self.belief.clone()) - self.inbox[factor_node_index].clone(),
            );
            // factor.send_message(self.belief.clone() - factor.);
            // if let Some(message) = self.outbox.get_mut(f_key) {
            //     *message = Message(
            //         self.belief
            //             - self
            //                 .inbox
            //                 .get(f_key)
            //                 .expect("The message should exist in the inbox")
            //                 .0,
            //     );
            // }
        }
    }
}
