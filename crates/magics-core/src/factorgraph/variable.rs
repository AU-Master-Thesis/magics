//! Variable node implementation for the GBP algorithm
//!
//! This module contains the implementation of variable nodes used in the
//! Gaussian Belief Propagation algorithm.

use std::collections::HashMap;
use gbp_linalg::prelude::*;
use ndarray_inverse::Inverse;

use super::{
    factorgraph::{FactorGraphId, NodeIndex},
    id::FactorId, 
    message::Message,
    node::{FactorGraphNode, RemoveConnectionToError},
    MessageCount, MessagesReceived, MessagesSent, DOFS,
};

/// Messages sent to factors
pub type MessagesToFactors = HashMap<FactorId, Message>;

/// Variable node in the factorgraph
#[derive(Debug)]
pub struct VariableNode {
    /// The factorgraph this variable belongs to
    factorgraph_id: FactorGraphId,
    /// Unique identifier within the factorgraph
    pub node_index: Option<NodeIndex>,
    /// The current mean of the variable
    pub mean: Vector<Float>,
    /// The current covariance of the variable
    pub covariance: Matrix<Float>,
    /// The messages received from factors
    pub inbox: MessagesToFactors,
    /// The current information vector of the variable
    pub information_vector: Vector<Float>,
    /// The current precision matrix of the variable
    pub precision_matrix: Matrix<Float>,
    /// Count of messages sent/received
    message_count: MessageCount,
    /// Whether this variable is enabled
    pub enabled: bool,
}

impl VariableNode {
    /// Create a new variable node
    pub fn new(factorgraph_id: FactorGraphId, enabled: bool) -> Self {
        Self {
            factorgraph_id,
            node_index: None,
            mean: Vector::zeros(DOFS),
            covariance: Matrix::eye(DOFS), 
            inbox: MessagesToFactors::new(),
            information_vector: Vector::zeros(DOFS),
            precision_matrix: Matrix::zeros((DOFS, DOFS)),
            message_count: MessageCount::default(),
            enabled,
        }
    }
    
    /// Returns the factorgraph ID that the variable belongs to
    #[inline]
    pub fn factorgraph_id(&self) -> FactorGraphId {
        self.factorgraph_id
    }

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

    /// Sets the node index of the variable
    ///
    /// # Panics
    ///
    /// Panics if the node index has already been set.
    pub fn set_node_index(&mut self, node_index: NodeIndex) {
        assert!(self.node_index.is_none(), "The node index is already set");
        self.node_index = Some(node_index);
    }
    
    /// Receive a message from a factor
    pub fn receive_message_from(&mut self, from: FactorId, message: Message) {
        if !self.enabled {
            return;
        }
        let _ = self.inbox.insert(from, message);
        if from.factorgraph_id == self.factorgraph_id {
            self.message_count.received.internal += 1;
        } else {
            self.message_count.received.external += 1;
        }
    }
    
    /// Update the variable using the GBP algorithm
    #[must_use]
    pub fn update(&mut self) -> MessagesToFactors {
        // Reset the information vector and precision matrix
        self.information_vector = Vector::zeros(DOFS);
        self.precision_matrix = Matrix::zeros((DOFS, DOFS));
        
        // Aggregate all messages to calculate the variable's belief
        for message in self.inbox.values() {
            if let Some(info_vec) = message.information_vector() {
                self.information_vector += info_vec;
            }
            
            if let Some(precision) = message.precision_matrix() {
                self.precision_matrix += precision;
            }
        }
        
        // Calculate covariance (inverse of precision matrix)
        // Only update if the precision matrix is invertible
        if let Some(cov) = self.precision_matrix.clone().inv() {
            self.covariance = cov;
            self.mean = self.covariance.dot(&self.information_vector);
        }
        
        // Create messages to send to all factors
        let mut messages = MessagesToFactors::new();
        let mut messages_sent = MessagesSent::new();
        
        for (factor_id, incoming_message) in &self.inbox {
            if incoming_message.is_empty() {
                messages.insert(*factor_id, Message::empty());
                continue;
            }
            
            // For each factor, create outgoing message by removing that factor's 
            // contribution from the belief
            let mut outgoing_info_vec = self.information_vector.clone();
            let mut outgoing_precision = self.precision_matrix.clone();
            
            if let Some(incoming_info) = incoming_message.information_vector() {
                outgoing_info_vec -= incoming_info;
            }
            
            if let Some(incoming_precision) = incoming_message.precision_matrix() {
                outgoing_precision -= incoming_precision;
            }
            
            // If the resulting precision matrix is not positive definite, send empty message
            if let Some(outgoing_covariance) = outgoing_precision.clone().inv() {
                let outgoing_mean = outgoing_covariance.dot(&outgoing_info_vec);
                
                let message = Message::new(
                    super::message::InformationVec(outgoing_info_vec),
                    super::message::PrecisionMatrix(outgoing_precision),
                    super::message::Mean(outgoing_mean),
                );
                
                messages.insert(*factor_id, message);
            } else {
                messages.insert(*factor_id, Message::empty());
            }
            
            // Update message count
            if factor_id.factorgraph_id == self.factorgraph_id {
                messages_sent.internal += 1;
            } else {
                messages_sent.external += 1;
            }
        }
        
        self.message_count.sent += messages_sent;
        messages
    }
    
    /// Empty the inbox of the variable
    pub fn empty_inbox(&mut self) {
        self.inbox.values_mut().for_each(|m| *m = Message::empty());
    }
}

impl FactorGraphNode for VariableNode {
    fn remove_connection_to(
        &mut self,
        factorgraph_id: FactorGraphId,
    ) -> Result<(), RemoveConnectionToError> {
        let connections_before = self.inbox.len();
        self.inbox.retain(|factor_id, _| factor_id.factorgraph_id != factorgraph_id);
        let connections_after = self.inbox.len();

        let no_connections_removed = connections_before == connections_after;
        if no_connections_removed {
            Err(RemoveConnectionToError)
        } else {
            Ok(())
        }
    }

    #[inline(always)]
    fn messages_sent(&self) -> MessagesSent {
        self.message_count.sent
    }

    #[inline(always)]
    fn messages_received(&self) -> MessagesReceived {
        self.message_count.received
    }

    #[inline(always)]
    fn reset_message_count(&mut self) {
        self.message_count.reset();
    }
}

impl std::fmt::Display for VariableNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "mean: {}", self.mean)?;
        writeln!(f, "covariance:\n{}", self.covariance.pretty_format())?;
        writeln!(f, "information_vector: {}", self.information_vector)?;
        writeln!(f, "precision_matrix:\n{}", self.precision_matrix.pretty_format())?;
        writeln!(f, "enabled: {}", self.enabled)?;
        writeln!(f, "messages sent: {}", self.messages_sent())?;
        writeln!(f, "messages received: {}", self.messages_received())
    }
}
