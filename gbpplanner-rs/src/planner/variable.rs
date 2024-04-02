use bevy::log::{debug, error};
use gbp_linalg::{pretty_print_matrix, pretty_print_vector, Float, Matrix, Vector};
use ndarray_inverse::Inverse;
use tap::Tap;

use super::{
    factorgraph::{
        FactorGraphNode, FactorId, MessagesFromVariables, MessagesToFactors, VariableId,
    },
    message::{Eta, Lam, Message, Mu},
};
use crate::{
    escape_codes::*,
    planner::{factorgraph::VariableIndex, NodeIndex},
    pretty_print_line, pretty_print_message, pretty_print_subtitle, pretty_print_title,
};

#[derive(Debug, Clone)]
pub struct VariablePrior {
    eta:    Vector<Float>,
    lambda: Matrix<Float>,
}

impl VariablePrior {
    fn new(eta: Vector<Float>, lam: Matrix<Float>) -> Self {
        Self { eta, lambda: lam }
    }
}

#[derive(Debug, Clone)]
pub struct VariableBelief {
    pub eta:    Vector<Float>,
    pub lambda: Matrix<Float>,
    pub mu:     Vector<Float>,
    pub sigma:  Matrix<Float>,
    /// Flag to indicate if the variable's covariance is finite, i.e. it does
    /// not contain NaNs or Infs In gbpplanner it is used to control if a
    /// variable can be rendered.
    valid:      bool,
}

impl VariableBelief {
    fn new(
        eta: Vector<Float>,
        lam: Matrix<Float>,
        mu: Vector<Float>,
        sigma: Matrix<Float>,
        valid: bool,
    ) -> Self {
        Self {
            eta,
            lambda: lam,
            mu,
            sigma,
            valid,
        }
    }
}

impl From<VariableBelief> for Message {
    fn from(value: VariableBelief) -> Self {
        Message::new(Eta(value.eta), Lam(value.lambda), Mu(value.mu))
    }
}

/// A variable in the factor graph.
#[derive(Debug)]
pub struct Variable {
    /// Degrees of freedom. For 2D case n_dofs_ = 4 ([x,y,xdot,ydot])
    pub dofs:   usize,
    pub prior:  VariablePrior,
    pub belief: VariableBelief,

    // pub eta_prior: Vector<Float>,
    // pub lam_prior: Matrix<Float>,
    // pub eta: Vector<Float>,
    // pub lam: Matrix<Float>,
    // pub mu: Vector<Float>,
    // pub sigma: Matrix<Float>,

    // / Flag to indicate if the variable's covariance is finite, i.e. it does
    // / not contain NaNs or Infs In gbpplanner it is used to control if a
    // / variable can be rendered.
    // pub valid: bool,
    /// Mailbox for incoming message storage
    pub inbox: MessagesToFactors,

    /// index
    pub node_index: Option<NodeIndex>,
}

impl Variable {
    /// Returns the variables belief about its position
    #[inline]
    pub fn estimated_position(&self) -> [Float; 2] {
        [self.belief.mu[0], self.belief.mu[1]]
    }

    /// Returns the variables belief about its velocity
    #[inline]
    pub fn estimated_velocity(&self) -> [Float; 2] {
        [self.belief.mu[2], self.belief.mu[3]]
    }

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

        let subtitle = format!("{}{}{}", RED, "Creating variable", RESET);
        pretty_print_subtitle!(subtitle);
        pretty_print_vector!(&mu_prior);
        pretty_print_matrix!(&lam_prior);
        pretty_print_vector!(&eta_prior);

        let sigma = lam_prior
            .inv()
            .unwrap_or_else(|| Matrix::<Float>::zeros((dofs, dofs)));
        let eta = eta_prior.clone();
        let lam = lam_prior.clone();

        Self {
            dofs,
            prior: VariablePrior::new(eta_prior, lam_prior),
            belief: VariableBelief::new(eta, lam, mu_prior, sigma, false),
            // eta_prior,
            // lam_prior,
            // eta,
            // lam,
            // mu: mu_prior,
            // sigma,
            // valid: false,
            inbox: MessagesToFactors::new(),
            node_index: None,
        }

        // Self {
        //     prior: prior.clone(),
        //     belief: prior,
        //     dofs,
        //     inbox: Inbox::new(),
        // }
    }

    pub fn set_node_index(&mut self, index: NodeIndex) {
        if self.node_index.is_some() {
            panic!("The node index is already set");
        }
        self.node_index = Some(index);
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

    pub fn receive_message_from(&mut self, from: FactorId, message: Message) {
        debug!("variable ? received message from {:?}", from);
        if message.is_empty() {
            // warn!("Empty message received from factor {:?}", from);
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
        // let subtitle = format!("{}{}{}", RED, "Changing prior", RESET);
        // pretty_print_subtitle!(subtitle);
        pretty_print_matrix!(&self.prior.lambda);
        pretty_print_vector!(&mean);
        self.prior.eta = self.prior.lambda.dot(&mean);
        // pretty_print_vector!(&self.prior.eta);
        // pretty_print_line!();
        // self.eta_prior = self.lam_prior.dot(&mean);
        self.belief.mu = mean;
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
            Eta(self.belief.eta.clone()),
            Lam(self.belief.lambda.clone()),
            Mu(self.belief.mu.clone()),
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
        self.belief.eta = self.prior.eta.clone();
        self.belief.lambda = self.prior.lambda.clone();

        let mut title = format!("{}{}{}", YELLOW, "Variable belief BEFORE update:", RESET);
        pretty_print_subtitle!(title);
        pretty_print_vector!(&self.belief.eta);
        pretty_print_matrix!(&self.belief.lambda);
        pretty_print_vector!(&self.belief.mu);

        // Go through received messages and update belief
        for (_, message) in self.inbox.iter() {
            let Some(payload) = message.payload() else {
                // empty message
                // info!("skipping empty message");
                continue;
            };
            self.belief.eta = &self.belief.eta + &payload.eta;
            self.belief.lambda = &self.belief.lambda + &payload.lam;
        }

        // Update belief
        if let Some(sigma) = self.belief.lambda.inv() {
            pretty_print_matrix!(&sigma);
            self.belief.sigma = sigma;
            self.belief.valid = self.belief.sigma.iter().all(|x| x.is_finite());
            if self.belief.valid {
                self.belief.mu = self.belief.sigma.dot(&self.belief.eta);
            } else {
                println!(
                    "{}:{},Variable covariance is not finite",
                    file!().split('/').last().unwrap(),
                    line!()
                );
            }
        }

        title = format!("{}{}{}", YELLOW, "Variable belief AFTER update:", RESET);
        pretty_print_subtitle!(title);
        pretty_print_vector!(&self.belief.eta);
        pretty_print_matrix!(&self.belief.lambda);
        pretty_print_vector!(&self.belief.mu);
        pretty_print_line!();

        // pretty_print_matrix!(&self.prior.lam);
        // pretty_print_vector!(&self.prior.eta);
        // pretty_print_matrix!(&self.belief.lam);
        // pretty_print_vector!(&self.belief.eta);
        // pretty_print_vector!(&self.belief.mu);

        let messages = self
            .inbox
            .iter()
            .map(|(&factor_id, received_message)| {
                let response = received_message.payload().map_or_else(
                    || {
                        Message::new(
                            Eta(self.belief.eta.clone()),
                            Lam(self.belief.lambda.clone()),
                            Mu(self.belief.mu.clone()),
                        )
                    },
                    |message_from_factor| {
                        // pretty_print_subtitle!("BEFORE FACTOR SUBSTRACTION");
                        // pretty_print_vector!(&self.belief.eta);
                        // pretty_print_matrix!(&self.belief.lam);
                        // pretty_print_vector!(&self.belief.mu);
                        // pretty_print_line!();
                        let msg = Message::new(
                            Eta(&self.belief.eta - &message_from_factor.eta),
                            Lam(&self.belief.lambda - &message_from_factor.lam),
                            Mu(&self.belief.mu - &message_from_factor.mu),
                        );
                        // pretty_print_subtitle!("AFTER FACTOR SUBSTRACTION");
                        // pretty_print_vector!(&self.belief.eta);
                        // pretty_print_matrix!(&self.belief.lam);
                        // pretty_print_vector!(&self.belief.mu);
                        msg
                    },
                );
                (factor_id, response)
            })
            .collect::<MessagesFromVariables>();

        messages.iter().for_each(|(factor_id, message)| {
            pretty_print_message!(
                VariableId::new(
                    factor_id.get_factor_graph_id(),
                    self.node_index.unwrap().into()
                ),
                factor_id,
                ""
            );
            pretty_print_vector!(message.information_vector().unwrap());
            pretty_print_matrix!(message.precision_matrix().unwrap());
            pretty_print_vector!(message.mean().unwrap());
        });

        messages

        // self.inbox
        //     .iter()
        //     .map(|(&factor_id, received_message)| {
        //         let response = Message::new(
        //             Eta(&self.eta - &received_message.eta),
        //             Lam(&self.lam - &received_message.lam),
        //             Mu(&self.mu - &received_message.mu),
        //         );
        //         (factor_id, response)
        //     })
        //     .collect()
    }

    /// Returns `true` if the covariance matrix is finite, `false` otherwise.
    #[inline]
    pub fn finite_covariance(&self) -> bool {
        self.belief.valid
    }
}

impl FactorGraphNode for Variable {
    fn remove_connection_to(
        &mut self,
        factorgraph_id: super::factorgraph::FactorGraphId,
    ) -> Result<(), super::factorgraph::RemoveConnectionToError> {
        let connections_before = self.inbox.len();
        self.inbox
            .retain(|factor_id, v| factor_id.factorgraph_id != factorgraph_id);
        let connections_after = self.inbox.len();

        let no_connections_removed = connections_before == connections_after;
        if no_connections_removed {
            Err(super::factorgraph::RemoveConnectionToError)
        } else {
            Ok(())
        }
    }
}
