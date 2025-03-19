//! Node abstractions for the factor graph
//!
//! This module contains the node abstractions used in the factorgraph.

use super::{
    factorgraph::FactorGraphId,
    MessageCount, MessagesReceived, MessagesSent,
};

/// Error that occurs when removing a connection to a node fails
#[derive(Debug)]
pub struct RemoveConnectionToError;

/// Common trait for all nodes in the factor graph
pub trait FactorGraphNode {
    /// Remove all connections to a specific factorgraph
    fn remove_connection_to(
        &mut self,
        factorgraph_id: FactorGraphId,
    ) -> Result<(), RemoveConnectionToError>;

    /// Return the number of messages sent
    fn messages_sent(&self) -> MessagesSent;

    /// Return the number of messages received
    fn messages_received(&self) -> MessagesReceived;

    /// Reset the message count
    fn reset_message_count(&mut self);

    /// Get the message count
    fn message_count(&self) -> MessageCount {
        MessageCount {
            sent: self.messages_sent(),
            received: self.messages_received(),
        }
    }
}
