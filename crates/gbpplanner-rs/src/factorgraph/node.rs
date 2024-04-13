use super::{factor::Factor, factorgraph::FactorGraphId, variable::Variable};

#[derive(Debug, derive_more::Display)]
#[display(fmt = "no connection to the given factorgraph")]
pub(in crate::factorgraph) struct RemoveConnectionToError;

impl std::error::Error for RemoveConnectionToError {}

pub(in crate::factorgraph) trait FactorGraphNode {
    #[must_use]
    fn remove_connection_to(
        &mut self,
        factorgraph_id: FactorGraphId,
    ) -> Result<(), RemoveConnectionToError>;

    fn messages_sent(&self) -> usize;
    fn messages_received(&self) -> usize;

    fn reset_message_count(&mut self);
}

#[derive(Debug, derive_more::IsVariant)]
pub enum NodeKind {
    Factor(Factor),
    // TODO: wrap in Box<>
    Variable(Variable),
}

#[derive(Debug)]
pub struct Node {
    factorgraph_id: FactorGraphId,
    kind: NodeKind,
}

impl Node {
    /// Construct a new node
    pub fn new(factorgraph_id: FactorGraphId, kind: NodeKind) -> Self {
        Self {
            factorgraph_id,
            kind,
        }
    }

    /// Returns `true` if the node is [`Factor`].
    ///
    /// [`Factor`]: Node::Factor
    #[must_use]
    #[inline]
    pub fn is_factor(&self) -> bool {
        self.kind.is_factor()
    }

    /// Returns `Some(&Factor)` if the node]s variant is [`Factor`], otherwise
    /// `None`.
    pub fn as_factor(&self) -> Option<&Factor> {
        if let NodeKind::Factor(ref v) = self.kind {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `Some(&mut Factor)` if the node]s variant is [`Factor`],
    /// otherwise `None`.
    pub fn as_factor_mut(&mut self) -> Option<&mut Factor> {
        if let NodeKind::Factor(ref mut v) = self.kind {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the node is [`Variable`].
    ///
    /// [`Variable`]: Node::Variable
    #[must_use]
    #[inline]
    pub fn is_variable(&self) -> bool {
        self.kind.is_variable()
    }

    /// Returns `Some(&Variable)` if the node]s variant is [`Variable`],
    /// otherwise `None`.
    pub fn as_variable(&self) -> Option<&Variable> {
        if let NodeKind::Variable(ref v) = self.kind {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `Some(&mut Variable)` if the node]s variant is [`Variable`],
    /// otherwise `None`.
    pub fn as_variable_mut(&mut self) -> Option<&mut Variable> {
        if let NodeKind::Variable(ref mut v) = self.kind {
            Some(v)
        } else {
            None
        }
    }
}
