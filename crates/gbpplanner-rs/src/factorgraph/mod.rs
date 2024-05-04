#![warn(missing_docs)]
//! ...

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

#[derive(Debug, Clone, Copy, derive_more::Add)]
pub struct MessageCount {
    pub sent:     usize,
    pub received: usize,
}

impl MessageCount {
    pub fn reset(&mut self) {
        self.sent = 0;
        self.received = 0;
    }

    pub fn new() -> Self {
        Self {
            sent:     0,
            received: 0,
        }
    }
}

impl Default for MessageCount {
    fn default() -> Self {
        Self::new()
    }
}
