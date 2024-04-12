#![deny(missing_docs)]
//! ...

use bevy::ecs::entity::Entity;
use petgraph::Undirected;

mod factor;
mod factorgraph;
mod graphviz;
mod id;
mod message;
mod node;
mod variable;

/// Degrees of Freedom of the ground robot.
/// The robot has 4 degrees, of freedom:
/// 1. position.x
/// 2. position.y
/// 3. velocity.x
/// 4. velocity.y
/// [x, y, x', y']
pub const DOFS: u32 = 4;

/// prelude module bringing entire public API into score
pub mod prelude {
    pub use super::{factorgraph::FactorGraph, message::Message, DOFS};
}

#[derive(Debug, Default, Clone, Copy)]
pub(super) struct MessageCount {
    pub sent:     usize,
    pub received: usize,
}

impl MessageCount {
    pub fn reset(&mut self) {
        self.sent = 0;
        self.received = 0;
    }
}

impl std::ops::Add for MessageCount {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        MessageCount {
            sent:     self.sent + rhs.sent,
            received: self.received + rhs.received,
        }
    }
}
