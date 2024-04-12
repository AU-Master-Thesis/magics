#![deny(missing_docs)]

use bevy::log::{debug, error};
use gbp_linalg::{pretty_print_matrix, pretty_print_vector, Float, Matrix, Vector};
use ndarray_inverse::Inverse;
use tap::Tap;

use super::{
    factorgraph::{
        FactorGraphNode, FactorId, MessageCount, MessagesFromVariables, MessagesToFactors,
        VariableId,
    },
    message::{InformationVec, Mean, Message, PrecisionMatrix},
};
use crate::{
    escape_codes::*,
    planner::{factorgraph::VariableIndex, NodeIndex},
    pretty_print_line, pretty_print_message, pretty_print_subtitle, pretty_print_title,
};

#[derive(Debug, Clone)]
pub struct VariablePrior {
    information_vector: Vector<Float>,
    precision_matrix:   Matrix<Float>,
}

impl VariablePrior {
    #[must_use]
    fn new(information_vector: Vector<Float>, precision_matrix: Matrix<Float>) -> Self {
        Self {
            information_vector,
            precision_matrix,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VariableBelief {
    pub information_vector: Vector<Float>,
    pub precision_matrix: Matrix<Float>,
    pub mean: Vector<Float>,
    pub covariance_matrix: Matrix<Float>,
    /// Flag to indicate if the variable's covariance is finite, i.e. it does
    /// not contain NaNs or Infs In gbpplanner it is used to control if a
    /// variable can be rendered.
    valid: bool,
}

impl VariableBelief {
    fn new(
        information_vector: Vector<Float>,
        precision_matrix: Matrix<Float>,
        mean: Vector<Float>,
        covariance_matrix: Matrix<Float>,
    ) -> Self {
        let valid = covariance_matrix.iter().all(|x| x.is_finite());
        Self {
            information_vector,
            precision_matrix,
            mean,
            covariance_matrix,
            valid,
        }
    }
}

impl From<VariableBelief> for Message {
    fn from(value: VariableBelief) -> Self {
        Message::new(
            InformationVec(value.information_vector),
            PrecisionMatrix(value.precision_matrix),
            Mean(value.mean),
        )
    }
}

/// A variable in the factor graph.
#[derive(Debug)]
pub struct Variable {
    pub prior:  VariablePrior,
    pub belief: VariableBelief,

    // / Flag to indicate if the variable's covariance is finite, i.e. it does
    // / not contain NaNs or Infs In gbpplanner it is used to control if a
    // / variable can be rendered.
    // pub valid: bool,
    /// Mailbox for incoming message storage
    pub inbox: MessagesToFactors,

    /// index
    node_index: Option<NodeIndex>,

    message_count: MessageCount,
}

impl Variable {
    /// Returns the node index of the variable
    ///
    /// # Panics
    ///
    /// Panics if the node index has not been set, which should not happen.
    #[inline]
    pub fn node_index(&self) -> NodeIndex {
        if self.node_index.is_none() {
            panic!("The node index has not been set");
        }
        self.node_index.unwrap()
    }

    /// Returns the variables belief about its position
    #[inline]
    pub fn estimated_position(&self) -> [Float; 2] {
        [self.belief.mean[0], self.belief.mean[1]]
    }

    /// Returns the variables belief about its velocity
    #[inline]
    pub fn estimated_velocity(&self) -> [Float; 2] {
        [self.belief.mean[2], self.belief.mean[3]]
    }

    #[must_use]
    pub fn new(
        prior_mean: Vector<Float>,
        mut prior_precision_matrix: Matrix<Float>,
        dofs: usize,
    ) -> Self {
        if !prior_precision_matrix.iter().all(|x| x.is_finite()) {
            prior_precision_matrix.fill(0.0);
        }

        let eta_prior = prior_precision_matrix.dot(&prior_mean);

        let sigma = prior_precision_matrix
            .inv()
            .unwrap_or_else(|| Matrix::<Float>::zeros((dofs, dofs)));
        let eta = eta_prior.clone();
        let lam = prior_precision_matrix.clone();

        Self {
            prior: VariablePrior::new(eta_prior, prior_precision_matrix),
            belief: VariableBelief::new(eta, lam, prior_mean, sigma),
            inbox: MessagesToFactors::new(),
            node_index: None,
            message_count: MessageCount::default(),
        }
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
        self.message_count.received += 1;
    }

    // TODO: why never used?
    #[inline]
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
        // pretty_print_matrix!(&self.prior.lambda);
        // pretty_print_vector!(&mean);
        self.prior.information_vector = self.prior.precision_matrix.dot(&mean);
        // pretty_print_vector!(&self.prior.eta);
        // pretty_print_line!();
        // self.eta_prior = self.lam_prior.dot(&mean);
        self.belief.mean = mean;
        // dbg!(&self.mu);

        // FIXME: forgot this line in the original code
        // this->belief_ = Message {this->eta_, this->lam_, this->mu_};

        let messages: MessagesFromVariables = self
            .inbox
            .keys()
            .map(|factor_id| (*factor_id, self.belief.clone().into()))
            .collect();

        for message in self.inbox.values_mut() {
            *message = Message::empty(self.dofs);
        }

        messages
    }

    pub fn prepare_message(&self) -> Message {
        Message::new(
            InformationVec(self.belief.information_vector.clone()),
            PrecisionMatrix(self.belief.precision_matrix.clone()),
            Mean(self.belief.mean.clone()),
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
        self.belief.information_vector = self.prior.information_vector.clone();
        self.belief.precision_matrix = self.prior.precision_matrix.clone();

        // let mut title = format!("{}{}{}", YELLOW, "Variable belief BEFORE update:",
        // RESET); pretty_print_subtitle!(title);
        // pretty_print_vector!(&self.belief.eta);
        // pretty_print_matrix!(&self.belief.lambda);
        // pretty_print_vector!(&self.belief.mu);

        // Go through received messages and update belief
        for (_, message) in self.inbox.iter() {
            let Some(payload) = message.payload() else {
                // empty message
                // info!("skipping empty message");
                continue;
            };
            self.belief.information_vector = &self.belief.information_vector + &payload.eta;
            self.belief.precision_matrix = &self.belief.precision_matrix + &payload.lam;
        }

        // Update belief
        // NOTE: This might not be correct, but it seems the `.inv()` method doesn't
        // catch and all-zero matrix
        let lam_not_zero = self.belief.precision_matrix.iter().any(|x| *x - 1e-6 > 0.0);
        // println!("lam_not_zero: {}", lam_not_zero);
        if lam_not_zero {
            if let Some(sigma) = self.belief.precision_matrix.inv() {
                // pretty_print_matrix!(&sigma);
                self.belief.covariance_matrix = sigma;
                self.belief.valid = self.belief.covariance_matrix.iter().all(|x| x.is_finite());
                if self.belief.valid {
                    self.belief.mean = self
                        .belief
                        .covariance_matrix
                        .dot(&self.belief.information_vector);
                } else {
                    println!(
                        "{}:{},Variable covariance is not finite",
                        file!().split('/').last().unwrap(),
                        line!()
                    );
                }
            }
        }

        // let title = format!("{}{}{}", YELLOW, "Variable belief update:", RESET);
        // pretty_print_subtitle!(title);
        // pretty_print_vector!(&self.belief.eta);
        // pretty_print_matrix!(&self.belief.lambda);
        // pretty_print_vector!(&self.belief.mu);
        // pretty_print_line!();

        let messages = self
            .inbox
            .iter()
            .map(|(&factor_id, received_message)| {
                let response = received_message.payload().map_or_else(
                    || self.prepare_message(),
                    |message_from_factor| {
                        // pretty_print_subtitle!("BEFORE FACTOR SUBSTRACTION");
                        // pretty_print_vector!(&self.belief.eta);
                        // pretty_print_matrix!(&self.belief.lam);
                        // pretty_print_vector!(&self.belief.mu);
                        // pretty_print_line!();
                        let msg = Message::new(
                            InformationVec(
                                &self.belief.information_vector - &message_from_factor.eta,
                            ),
                            PrecisionMatrix(
                                &self.belief.precision_matrix - &message_from_factor.lam,
                            ),
                            Mean(&self.belief.mean - &message_from_factor.mu),
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

        // messages.iter().for_each(|(factor_id, message)| {
        //     pretty_print_message!(
        //         VariableId::new(
        //             factor_id.get_factor_graph_id(),
        //             self.node_index.unwrap().into()
        //         ),
        //         factor_id,
        //         ""
        //     );
        //     pretty_print_vector!(message.information_vector().unwrap());
        //     pretty_print_matrix!(message.precision_matrix().unwrap());
        //     pretty_print_vector!(message.mean().unwrap());
        // });

        self.message_count.sent += messages.len();

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

    #[inline(always)]
    fn messages_sent(&self) -> usize {
        self.message_count.sent
    }

    #[inline(always)]
    fn messages_received(&self) -> usize {
        self.message_count.received
    }

    #[inline(always)]
    fn reset_message_count(&mut self) {
        self.message_count.reset();
    }
}
