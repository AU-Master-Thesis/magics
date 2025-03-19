//! Message implementation for the GBP algorithm
//!
//! This module contains the implementation of messages passed between
//! factor nodes and variable nodes in the Gaussian Belief Propagation algorithm.

use std::collections::HashMap;

use gbp_linalg::prelude::*;
use ndarray::array;

use super::id::VariableId;

/// Messages sent to variables
pub type MessagesToVariables = HashMap<VariableId, Message>;

/// A message payload in the Gaussian Belief Propagation algorithm
#[derive(Debug, Clone)]
pub struct MessageContent {
    /// The information vector of the message (η)
    pub information_vector: Vector<Float>,
    /// The precision matrix of the message (Λ)
    pub precision_matrix: Matrix<Float>,
    /// The mean vector of the message (μ)
    pub mean: Vector<Float>,
}

/// A message in the Gaussian Belief Propagation algorithm
#[derive(Debug, Clone)]
pub struct Message {
    /// The content of the message, if any
    payload: Option<MessageContent>,
}

impl Message {
    /// Create a new message with the given content
    pub fn new(
        information_vector: InformationVec,
        precision_matrix: PrecisionMatrix,
        mean: Mean,
    ) -> Self {
        Self {
            payload: Some(MessageContent {
                information_vector: information_vector.0,
                precision_matrix: precision_matrix.0,
                mean: mean.0,
            }),
        }
    }

    /// Create an empty message
    pub fn empty() -> Self {
        Self { payload: None }
    }

    /// Check if the message is empty
    pub fn is_empty(&self) -> bool {
        self.payload.is_none()
    }

    /// Get the information vector of the message
    pub fn information_vector(&self) -> Option<&Vector<Float>> {
        self.payload.as_ref().map(|p| &p.information_vector)
    }

    /// Get the precision matrix of the message
    pub fn precision_matrix(&self) -> Option<&Matrix<Float>> {
        self.payload.as_ref().map(|p| &p.precision_matrix)
    }

    /// Get the mean vector of the message
    pub fn mean(&self) -> Option<&Vector<Float>> {
        self.payload.as_ref().map(|p| &p.mean)
    }

    /// Take the payload, leaving an empty message
    pub fn take(&mut self) -> Option<MessageContent> {
        self.payload.take()
    }
}

/// Wrapper for an information vector
pub struct InformationVec(pub Vector<Float>);

/// Wrapper for a precision matrix
pub struct PrecisionMatrix(pub Matrix<Float>);

/// Wrapper for a mean vector
pub struct Mean(pub Vector<Float>);

impl Default for Message {
    fn default() -> Self {
        Self::empty()
    }
}
