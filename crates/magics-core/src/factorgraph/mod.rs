#![warn(missing_docs)]
//! Factor graph implementation for the GBP algorithm
//!
//! This module contains the implementation of the factor graph used for
//! Gaussian Belief Propagation (GBP) in the planning algorithm.

use derive_more::{Add, AddAssign};

pub mod factor;
#[allow(clippy::module_inception)]
pub mod factorgraph;
pub mod id;
pub mod message;
pub mod node;
pub mod variable;

/// Degrees of Freedom of the ground robot.
/// The robot has 4 degrees, of freedom:
/// 1. position.x
/// 2. position.y
/// 3. velocity.x
/// 4. velocity.y
/// [x, y, x', y']
pub const DOFS: usize = 4;

/// prelude module bringing entire public API into score
#[allow(unused_imports)]
pub mod prelude {
    pub use super::{factorgraph::FactorGraph, message::Message, DOFS};
}

#[derive(Debug, Clone, Copy, Add, AddAssign, serde::Serialize)]
pub struct MessagesSent {
    /// Number of internal messages sent
    pub internal: usize,
    /// Number of external messages sent
    pub external: usize,
}

impl std::fmt::Display for MessagesSent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[internal: {}, external: {}]",
            self.internal, self.external
        )
    }
}

impl std::iter::Sum for MessagesSent {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::new(), |a, b| a + b)
    }
}

impl MessagesSent {
    /// Create a new MessagesSent counter
    pub fn new() -> Self {
        Self {
            internal: 0,
            external: 0,
        }
    }
}

impl Default for MessagesSent {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, Add, AddAssign, serde::Serialize)]
pub struct MessagesReceived {
    /// Number of internal messages received
    pub internal: usize,
    /// Number of external messages received
    pub external: usize,
}

impl MessagesReceived {
    /// Create a new MessagesReceived counter
    pub fn new() -> Self {
        Self {
            internal: 0,
            external: 0,
        }
    }
}

impl Default for MessagesReceived {
    fn default() -> Self {
        Self::new()
    }
}

impl std::iter::Sum for MessagesReceived {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::new(), |a, b| a + b)
    }
}

impl std::fmt::Display for MessagesReceived {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[internal: {}, external: {}]",
            self.internal, self.external
        )
    }
}

#[derive(Debug, Clone, Copy, Add, AddAssign)]
pub struct MessageCount {
    /// Messages sent
    pub sent: MessagesSent,
    /// Messages received
    pub received: MessagesReceived,
}

impl MessageCount {
    /// Reset the message counters
    pub fn reset(&mut self) {
        self.sent = MessagesSent::new();
        self.received = MessagesReceived::new();
    }

    /// Create a new MessageCount
    pub fn new() -> Self {
        Self {
            sent: MessagesSent::new(),
            received: MessagesReceived::new(),
        }
    }
}

impl Default for MessageCount {
    fn default() -> Self {
        Self::new()
    }
}

impl std::iter::Sum for MessageCount {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::new(), |a, b| a + b)
    }
}
