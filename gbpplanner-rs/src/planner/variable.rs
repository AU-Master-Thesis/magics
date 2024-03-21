use bevy::log::warn;
use gbp_linalg::{Float, Matrix, Vector};
use ndarray_inverse::Inverse;

use super::{
    factorgraph::{FactorId, MessagesFromVariables, MessagesToFactors},
    message::{Eta, Lam, Message, Mu},
};

/// A variable in the factor graph.
#[derive(Debug)]
pub struct Variable {
    /// In **gbpplanner** the `prior` is stored in 2 separate variables:
    /// 1. `eta_prior_` Information vector of prior on variable (essentially
    ///    like a unary factor)
    /// 2. `lam_prior_` Precision matrix of prior on variable (essentially like
    ///    a unary factor)
    // pub prior: MultivariateNormal,
    // pub belief: MultivariateNormal,
    /// Degrees of freedom. For 2D case n_dofs_ = 4 ([x,y,xdot,ydot])
    pub dofs: usize,

    pub eta_prior: Vector<Float>,
    pub lam_prior: Matrix<Float>,
    pub eta:       Vector<Float>,
    pub lam:       Matrix<Float>,
    pub mu:        Vector<Float>,
    pub sigma:     Matrix<Float>,

    /// Flag to indicate if the variable's covariance is finite, i.e. it does
    /// not contain NaNs or Infs In gbpplanner it is used to control if a
    /// variable can be rendered.
    pub valid: bool,
    /// Mailbox for incoming message storage
    pub inbox: MessagesToFactors,
}

impl Variable {
    // pub fn new(prior: MultivariateNormal, dofs: usize) -> UninsertedVariable {
    //     UninsertedVariable { prior, dofs }
    //     // Self {
    //     //     node_index: None,
    //     //     prior: prior.clone(),
    //     //     belief: prior,
    //     //     dofs,
    //     //     inbox: Inbox::new(),
    //     // }
    // }

    #[must_use]
    pub fn new(mu_prior: Vector<Float>, mut lam_prior: Matrix<Float>, dofs: usize) -> Self {
        // if (!lam_prior_.allFinite()) lam_prior_.setZero();
        // if !prior.precision_matrix().iter().all(|x| x.is_finite()) {
        //     prior.precision_matrix().fill(0.0);
        // }
        if !lam_prior.iter().all(|x| x.is_finite()) {
            lam_prior.fill(0.0);
        }

        // pretty_print_matrix!(&lam_prior);

        let eta_prior = lam_prior.dot(&mu_prior);
        let sigma = lam_prior
            .inv()
            .unwrap_or_else(|| Matrix::<Float>::zeros((dofs, dofs)));
        let eta = eta_prior.clone();
        let lam = lam_prior.clone();

        Self {
            dofs,
            eta_prior,
            lam_prior,
            eta,
            lam,
            mu: mu_prior,
            sigma,
            valid: false,
            inbox: MessagesToFactors::new(),
        }

        // Self {
        //     prior: prior.clone(),
        //     belief: prior,
        //     dofs,
        //     inbox: Inbox::new(),
        // }
    }

    // pub fn new(mut mu_prior: Vector<Float>, mut

    // pub fn set_node_index(&mut self, node_index: NodeIndex) {
    //     match self.node_index {
    //         Some(_) => panic!("The node index is already set"),
    //         None => self.node_index = Some(node_index),
    //     }
    // }
    //
    // pub fn get_node_index(&self) -> NodeIndex {
    //     match self.node_index {
    //         Some(node_index) => node_index,
    //         None => panic!("The node index has not been set"),
    //     }
    // }

    pub fn send_message(&mut self, from: FactorId, message: Message) {
        if message.is_empty() {
            warn!("Empty message received from factor {:?}", from);
        }
        let _ = self.inbox.insert(from, message);
    }

    // TODO: why never used?
    pub fn read_message_from(&mut self, from: FactorId) -> Option<&Message> {
        self.inbox.get(&from)
    }

    /// Change the prior of the variable.
    /// It updates the belief of the variable.
    /// The prior acts as the pose factor
    /// Called `Variable::change_variable_prior` in **gbpplanner**
    pub fn change_prior(&mut self, mean: Vector<Float>) -> MessagesFromVariables {
        self.eta_prior = self.lam_prior.dot(&mean);
        self.mu = mean;
        // dbg!(&self.mu);

        // FIXME: forgot this line in the original code
        // this->belief_ = Message {this->eta_, this->lam_, this->mu_};

        let messages: MessagesFromVariables = self
            .inbox
            .keys()
            .map(|factor_id| (*factor_id, self.prepare_message()))
            .collect();

        for message in self.inbox.values_mut() {
            *message = Message::empty(self.dofs);
        }

        messages
    }

    pub fn prepare_message(&self) -> Message {
        Message::new(
            Eta(self.eta.clone()),
            Lam(self.lam.clone()),
            Mu(self.mu.clone()),
        )
    }

    // /****************************************************************************
    // *******************************/ // Variable belief update step:
    // // Aggregates all the messages from its connected factors (begins with the
    // prior, as this is effectively a unary factor) // The valid_ flag is
    // useful for drawing the variable. // Finally the outgoing messages to
    // factors is created. /****************************************************
    // *******************************************************/
    /// Variable Belief Update step (Step 1 in the GBP algorithm)
    /// called `Variable::update_belief` in **gbpplanner**
    pub fn update_belief_and_create_factor_responses(&mut self) -> MessagesFromVariables {
        // Collect messages from all other factors, begin by "collecting message from
        // pose factor prior"
        self.eta = self.eta_prior.clone();
        self.lam = self.lam_prior.clone();

        // Go through received messages and update belief
        for (_, message) in self.inbox.iter() {
            let Some(payload) = message.payload() else {
                // empty message
                // info!("skipping empty message");
                continue;
            };
            self.eta = &self.eta + &payload.eta;
            self.lam = &self.lam + &payload.lam;
        }

        // Update belief
        if let Some(sigma) = self.lam.inv() {
            self.sigma = sigma;
            self.valid = self.sigma.iter().all(|x| x.is_finite());
            if self.valid {
                self.mu = self.sigma.dot(&self.eta);
            }
        }

        // self.sigma = self
        //     .lam
        //     .inv()
        //     .unwrap_or_else(|| Matrix::<Float>::zeros((self.dofs, self.dofs)));
        // let valid = self.sigma.iter().all(|x| x.is_finite());
        //
        // self.valid = valid;
        //
        // if self.valid {
        //     self.mu = self.sigma.dot(&self.eta);
        //     // dbg!(&self.mu);
        // }

        self.inbox
            .iter()
            .map(|(&factor_id, received_message)| {
                let response = received_message.payload().map_or_else(
                    || {
                        Message::new(
                            Eta(self.eta.clone()),
                            Lam(self.lam.clone()),
                            Mu(self.mu.clone()),
                        )
                    },
                    |gaussian| {
                        Message::new(
                            Eta(&self.eta - &gaussian.eta),
                            Lam(&self.lam - &gaussian.lam),
                            Mu(&self.mu - &gaussian.mu),
                        )
                    },
                );
                (factor_id, response)
            })
            .collect()
    }

    pub fn finite_covariance(&self) -> bool {
        self.valid
    }
}
