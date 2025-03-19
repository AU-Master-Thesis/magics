//! Factor graph implementation
//!
//! This module contains the implementation of the FactorGraph struct
//! which is the main data structure for the GBP algorithm.

use std::num::NonZeroUsize;
use gbp_linalg::prelude::*;
use super::{MessageCount, MessagesSent, MessagesReceived};

/// Index of a factor in a factorgraph
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FactorIndex(pub NonZeroUsize);

/// Index of a variable in a factorgraph
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VariableIndex(pub NonZeroUsize);

/// Node index in the factorgraph
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeIndex(pub NonZeroUsize);

/// Unique identifier for a factorgraph
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FactorGraphId(pub NonZeroUsize);

impl FactorGraphId {
    /// Create a new factor graph ID from a non-zero usize
    pub const fn new(id: NonZeroUsize) -> Self {
        Self(id)
    }

    /// Get the index value
    pub const fn index(&self) -> usize {
        self.0.get()
    }
}

/// Factor graph implementation
#[derive(Debug)]
pub struct FactorGraph {
    /// Unique identifier for this factorgraph
    pub id: FactorGraphId,
    /// Number of nodes in the graph
    pub node_count: usize,
    /// Message statistics for this factorgraph
    pub message_count: MessageCount,
}

impl FactorGraph {
    /// Create a new factorgraph with the given ID
    pub fn new(id: FactorGraphId) -> Self {
        Self {
            id,
            node_count: 0,
            message_count: MessageCount::default(),
        }
    }
    
    /// Reset the message counters
    pub fn reset_message_counters(&mut self) {
        self.message_count.reset();
    }
}

impl Default for FactorGraph {
    fn default() -> Self {
        Self::new(FactorGraphId::new(NonZeroUsize::new(1).unwrap()))
    }
}
