use std::collections::HashMap;

use super::factorgraph::Inbox;
use super::factorgraph::NodeIndex;
use super::message::Message;
use gbp_linalg::{Float, Vector};
use gbp_multivariate_normal::MultivariateNormal;

/// A variable in the factor graph.
#[derive(Debug, Clone)]
pub struct Variable {
    /// Unique identifier that associates the variable with a factorgraph/robot.
    /// TODO: use typestate pattern to always ensure a variable has a `node_index`
    pub node_index: Option<NodeIndex>,
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
    inbox: Inbox,
}

impl Variable {
    pub fn new(prior: MultivariateNormal, dofs: usize) -> Self {
        // if !prior.precision_matrix().iter().all(|x| x.is_finite()) {
        //     // if (!lam_prior_.allFinite()) lam_prior_.setZero();

        //     prior.precision_matrix.fill(0.0);
        // }
        Self {
            node_index: None,
            prior: prior.clone(),
            belief: prior,
            dofs,
            inbox: Inbox::new(),
        }
    }

    pub fn set_node_index(&mut self, node_index: NodeIndex) {
        match self.node_index {
            Some(_) => panic!("The node index is already set"),
            None => self.node_index = Some(node_index),
        }
    }

    pub fn get_node_index(&self) -> NodeIndex {
        match self.node_index {
            Some(node_index) => node_index,
            None => panic!("The node index has not been set"),
        }
    }

    pub fn send_message(&mut self, from: NodeIndex, message: Message) {
        let _ = self.inbox.insert(from, message);
    }

    // TODO: why never used?
    pub fn read_message_from(&mut self, from: NodeIndex) -> Option<&Message> {
        self.inbox.get(&from)
    }

    /// Change the prior of the variable.
    /// It updates the belief of the variable.
    pub fn change_prior(
        &mut self,
        mean: Vector<Float>,
        indices_of_adjacent_factors: Vec<NodeIndex>,
    ) -> HashMap<NodeIndex, Message> {
        self.prior
            .update_information_vector(&self.prior.precision_matrix().dot(&mean));
        // self.prior.information_vector = self.prior.precision_matrix.dot(&mean);
        // QUESTION: why cache mu?
        // mu_ = new_mu;
        // belief_ = Message {eta_, lam_, mu_};
        // FIXME: we probably never update the belief of the variable
        // dbg!(&self.belief);
        self.belief = self.prior.clone();

        indices_of_adjacent_factors
            .into_iter()
            // .map(|factor_index| (factor_index, Message(self.belief.clone())))
            .map(|factor_index| (factor_index, Message::from(self.belief.clone())))
            .collect()
    }

    // /***********************************************************************************************************/
    // // Variable belief update step:
    // // Aggregates all the messages from its connected factors (begins with the prior, as this is effectively a unary factor)
    // // The valid_ flag is useful for drawing the variable.
    // // Finally the outgoing messages to factors is created.
    // /***********************************************************************************************************/
    /// Variable Belief Update step (Step 1 in the GBP algorithm)
    ///
    pub fn update_belief_and_create_responses(&mut self) -> HashMap<NodeIndex, Message> {
        // Collect messages from all other factors, begin by "collecting message from pose factor prior"
        // TODO: wrap in unsafe block for perf:

        unsafe {
            self.belief
                .set_information_vector(self.prior.information_vector());
            self.belief
                .set_precision_matrix(self.prior.precision_matrix());
        }

        for (_, message) in self.inbox.iter() {
            if message.is_empty() {
                continue;
            }
            unsafe {
                self.belief
                    .add_assign_information_vector(&message.information_vector());
                self.belief
                    .add_assign_precision_matrix(&message.precision_matrix());
            }
        }

        // Update the internal invariant of the belief, needed after the previous call to {add_assign,set}_{information_vector,precision_matrix}
        // which violates that the mean, information vector and precision matrix are consistent, for performance reasons.
        self.belief.update();

        // TODO: update self.sigma_ with covariance
        // -> Seems to not be useful

        // Update belief
        // println!("precision matrix: {:?}", self.belief.precision_matrix);

        // if let Some(covariance) = self.belief.precision_matrix().inv() {
        // let valid = self.belief.covariance().iter().all(|x| x.is_finite());
        // if valid {
        // TODO: is this meaningful?
        // if (valid_) mu_ = sigma_ * eta_;
        // }
        // }

        self.inbox
            .iter()
            .map(|(&factor_index, received_message)| {
                let response = match received_message {
                    Message::Empty(_) => Message::from(self.belief.clone()),
                    Message::Content { gaussian } => Message::from(&self.belief - gaussian),
                };
                (factor_index, response)
            })
            .collect()
    }
}
