#![deny(missing_docs)]
//! Message module.
//!
//! Contains the message struct that variables and factors send between each
//! other in the factorgraph.

use std::collections::BTreeMap;

use gbp_linalg::prelude::*;

use super::{
    id::{FactorId, VariableId},
    DOFS,
};

/// Payload of a message
#[derive(Debug, Clone)]
pub struct Payload {
    /// Information vector of a multivariate gaussian
    pub information_factor: Vector<Float>,
    /// Precision matrix of a multivariate gaussian
    pub precision_matrix: Matrix<Float>,
    /// Mean vector of a multivariate gaussian
    /// The mean can be computed from the information vector and the precision
    /// matrix but the mean vector is stored here to trade some memory, for
    /// having to compute it multiple times
    pub mean: Vector<Float>,
}

/// Newtype used to make prevent the caller of `Message::new()` from mixing up
/// the information vector and mean vector argument.
pub struct InformationVec(pub Vector<Float>);
/// Newtype used to make make it clear for the caller that the matrix argument
/// for `Message::new()` has to be a precision matrix, and not a covariance.
pub struct PrecisionMatrix(pub Matrix<Float>);
/// Newtype used to make prevent the caller of `Message::new()` from mixing up
/// the information vector and mean vector argument.
pub struct Mean(pub Vector<Float>);

#[derive(Debug, Clone)]
pub struct Message {
    payload: Option<Payload>,
}

impl Message {
    /// Returns a reference to the mean
    /// or `None` if the message is empty.
    #[inline]
    pub fn mean(&self) -> Option<&Vector<Float>> {
        self.payload.as_ref().map(|payload| &payload.mean)
    }

    /// Returns a reference to the precision matrix
    /// or `None` if the message is empty.
    #[inline]
    pub fn precision_matrix(&self) -> Option<&Matrix<Float>> {
        self.payload
            .as_ref()
            .map(|payload| &payload.precision_matrix)
    }

    /// Returns a reference to the information vector
    /// or `None` if the message is empty.
    #[inline]
    pub fn information_vector(&self) -> Option<&Vector<Float>> {
        self.payload
            .as_ref()
            .map(|payload| &payload.information_factor)
    }

    /// Returns `true` if the message is [`Empty`].
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.payload.is_none()
    }

    /// Take the inner `MultivariateNormal` from the message.
    /// Leaving the message in an empty state.
    #[inline]
    pub fn take(&mut self) -> Option<Payload> {
        self.payload.take()
    }

    /// Access the payload of the message.
    /// Returns `None` if the message is empty.
    #[inline]
    pub fn payload(&self) -> Option<&Payload> {
        self.payload.as_ref()
    }

    /// Create an empty message
    #[must_use]
    pub fn empty() -> Self {
        Self {
            payload: Some(Payload {
                information_factor: Vector::<Float>::zeros(DOFS),
                precision_matrix: Matrix::<Float>::zeros((DOFS, DOFS)),
                mean: Vector::<Float>::zeros(DOFS),
            }),
        }
    }

    /// Create a new message
    ///
    /// # Panics
    ///
    /// - if `eta.0.len() != DOFS`
    /// - if `lam.0.nrows() != DOFS`
    /// - if `lam.0.ncols() != DOFS`
    /// - if `mu.0.len() != DOFS`
    #[must_use]
    pub fn new(eta: InformationVec, lam: PrecisionMatrix, mu: Mean) -> Self {
        debug_assert_eq!(eta.0.len(), DOFS);
        debug_assert_eq!(lam.0.nrows(), DOFS);
        debug_assert_eq!(lam.0.ncols(), DOFS);
        debug_assert_eq!(mu.0.len(), DOFS);

        Self {
            payload: Some(Payload {
                information_factor: eta.0,
                precision_matrix: lam.0,
                mean: mu.0,
            }),
        }
    }
}

/// A message from a factor to a variable
#[derive(Debug)]
pub struct VariableToFactorMessage {
    /// The factor that sends the message
    pub from:    VariableId,
    /// The variable that receives the message
    pub to:      FactorId,
    /// The message
    pub message: Message,
}

/// A message from a variable to a factor
#[derive(Debug)]
pub struct FactorToVariableMessage {
    /// The variable that sends the message
    pub from:    FactorId,
    /// The factor that receives the message
    pub to:      VariableId,
    /// The message
    pub message: Message,
}

// pub type MessagesFromVariables = BTreeMap<FactorId, Message>;
// pub type MessagesFromFactors = BTreeMap<VariableId, Message>;

/// Type alias for a map of messages from factors connected to a variable
/// A (`BTreeMap`)[std::collections::BTreeMap] is used, instead of a
/// (`HashMap`)[std::collections::HashMap] to ensure that the messages are
/// stored in a consistent order This is necessary for the **gbpplanner**
/// algorithm to work correctly.
pub type MessagesToFactors = BTreeMap<FactorId, Message>;

/// Type alias for a map of messages from variables connected to a factor
/// A (`BTreeMap`)[std::collections::BTreeMap] is used, instead of a
/// (`HashMap`)[std::collections::HashMap] to ensure that the messages are
/// stored in a consistent order This is necessary for the **gbpplanner**
/// algorithm to work correctly.
pub type MessagesToVariables = BTreeMap<VariableId, Message>;
