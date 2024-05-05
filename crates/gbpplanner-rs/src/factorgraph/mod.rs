#![warn(missing_docs)]
//! ...
use derive_more::{Add, AddAssign};

pub mod factor;
#[allow(clippy::module_inception)]
pub mod factorgraph;
pub mod graphviz;
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

#[derive(Debug, Clone, Copy, Add, AddAssign)]
pub struct MessagesSent {
    pub internal: usize,
    pub external: usize,
}

impl std::fmt::Display for MessagesSent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[internal: {}, external: {}]", self.internal, self.external)
    }
}

impl std::iter::Sum for MessagesSent {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::new(), |a, b| a + b)
    }
}

impl MessagesSent {
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

#[derive(Debug, Clone, Copy, Add, AddAssign)]
pub struct MessagesReceived {
    pub internal: usize,
    pub external: usize,
}

impl MessagesReceived {
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
        write!(f, "[internal: {}, external: {}]", self.internal, self.external)
    }
}

#[derive(Debug, Clone, Copy, Add, AddAssign)]
pub struct MessageCount {
    // pub sent:     usize,
    // pub received: usize,
    pub sent:     MessagesSent,
    pub received: MessagesReceived,
}

impl MessageCount {
    pub fn reset(&mut self) {
        self.sent = MessagesSent::new();
        self.received = MessagesReceived::new();
    }

    pub fn new() -> Self {
        Self {
            sent:     MessagesSent::new(),
            received: MessagesReceived::new(),
            // sent:     0,
            // received: 0,
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
