use bevy::log::debug;
use gbp_linalg::{Float, Matrix, Vector};
use ndarray_inverse::Inverse;

use super::{
    factorgraph::NodeIndex,
    id::FactorId,
    message::{InformationVec, Mean, Message, MessagesToFactors, PrecisionMatrix},
    node::{FactorGraphNode, RemoveConnectionToError},
    MessageCount,
};

#[derive(Debug, Clone)]
pub struct VariablePrior {
    information_vector: Vector<Float>,
    precision_matrix:   Matrix<Float>,
}

impl VariablePrior {
    #[must_use]
    const fn new(information_vector: Vector<Float>, precision_matrix: Matrix<Float>) -> Self {
        Self {
            information_vector,
            precision_matrix,
        }
    }
}

// TODO: use pretty_print_matrix!
// impl std::fmt::Display for VariablePrior {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         todo!()
//     }
// }

/// PERF: use fixed size vectors and matrices, either bevy Vec4, or nalgebra
/// Vec4
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
        Self::new(
            InformationVec(value.information_vector),
            PrecisionMatrix(value.precision_matrix),
            Mean(value.mean),
        )
    }
}

/// A variable in the factor graph.
#[derive(Debug)]
pub struct VariableNode {
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

impl VariableNode {
    /// Returns the node index of the variable
    ///
    /// # Panics
    ///
    /// Panics if the node index has not been set, which should not happen.
    #[inline]
    #[allow(clippy::unwrap_used)]
    pub fn node_index(&self) -> NodeIndex {
        assert!(self.node_index.is_some(), "The node index has not been set");
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
        assert!(self.node_index.is_none(), "The node index is already set");
        self.node_index = Some(index);
    }

    pub fn receive_message_from(&mut self, from: FactorId, message: Message) {
        debug!("variable ? received message from {:?}", from);
        if message.is_empty() {
            // warn!("Empty message received from factor {:?}", from);
        }
        let _ = self.inbox.insert(from, message);
        self.message_count.received += 1;
    }

    // // TODO: why never used?
    // #[inline]
    // pub fn read_message_from(&mut self, from: FactorId) -> Option<&Message> {
    //     self.inbox.get(&from)
    // }

    /// Change the prior of the variable.
    /// It updates the belief of the variable.
    /// The prior acts as the pose factor
    /// Called `Variable::change_variable_prior` in **gbpplanner**
    pub fn change_prior(&mut self, mean: Vector<Float>) -> MessagesToFactors {
        self.prior.information_vector = self.prior.precision_matrix.dot(&mean);
        self.belief.mean = mean;

        // FIXME: forgot this line in the original code
        // this->belief_ = Message {this->eta_, this->lam_, this->mu_};

        let messages: MessagesToFactors = self
            .inbox
            .keys()
            .map(|factor_id| (*factor_id, self.belief.clone().into()))
            .collect();

        for message in self.inbox.values_mut() {
            *message = Message::empty();
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
    pub fn update_belief_and_create_factor_responses(&mut self) -> MessagesToFactors {
        // Collect messages from all other factors, begin by "collecting message from
        // pose factor prior"
        // self.belief.information_vector = self.prior.information_vector.clone();
        // self.belief.precision_matrix = self.prior.precision_matrix.clone();
        self.belief
            .information_vector
            .clone_from(&self.prior.information_vector);
        self.belief
            .precision_matrix
            .clone_from(&self.prior.precision_matrix);

        // Go through received messages and update belief
        for message in self.inbox.values() {
            let Some(payload) = message.payload() else {
                continue;
            };
            self.belief.information_vector =
                &self.belief.information_vector + &payload.information_factor;
            self.belief.precision_matrix =
                &self.belief.precision_matrix + &payload.precision_matrix;
        }

        // Update belief
        // NOTE: This might not be correct, but it seems the `.inv()` method doesn't
        // catch and all-zero matrix
        let lam_not_zero = self.belief.precision_matrix.iter().any(|x| *x - 1e-6 > 0.0);
        if lam_not_zero {
            if let Some(sigma) = self.belief.precision_matrix.inv() {
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
                        file!()
                            .split('/')
                            .last()
                            .expect("the basename of the filename always exist"),
                        line!()
                    );
                }
            }
        }

        let messages: MessagesToFactors = self
            .inbox
            .iter()
            .map(|(&factor_id, received_message)| {
                let response = received_message.payload().map_or_else(
                    || self.prepare_message(),
                    |message_from_factor| {
                        Message::new(
                            InformationVec(
                                &self.belief.information_vector
                                    - &message_from_factor.information_factor,
                            ),
                            PrecisionMatrix(
                                &self.belief.precision_matrix
                                    - &message_from_factor.precision_matrix,
                            ),
                            Mean(&self.belief.mean - &message_from_factor.mean),
                        )
                    },
                );
                (factor_id, response)
            })
            .collect();

        self.message_count.sent += messages.len();

        messages
    }

    /// Returns `true` if the covariance matrix is finite, `false` otherwise.
    #[inline]
    pub const fn finite_covariance(&self) -> bool {
        self.belief.valid
    }
}

impl FactorGraphNode for VariableNode {
    fn remove_connection_to(
        &mut self,
        factorgraph_id: super::factorgraph::FactorGraphId,
    ) -> Result<(), RemoveConnectionToError> {
        let connections_before = self.inbox.len();
        self.inbox
            .retain(|factor_id, _| factor_id.factorgraph_id != factorgraph_id);
        let connections_after = self.inbox.len();

        let no_connections_removed = connections_before == connections_after;
        if no_connections_removed {
            Err(RemoveConnectionToError)
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
