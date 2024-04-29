use super::{factor::FactorNode, factorgraph::FactorGraphId, variable::VariableNode};

#[derive(Debug, derive_more::Display)]
#[display(fmt = "no connection to the given factorgraph")]
pub struct RemoveConnectionToError;

impl std::error::Error for RemoveConnectionToError {}

pub(in crate::factorgraph) trait FactorGraphNode {
    fn remove_connection_to(&mut self, factorgraph_id: FactorGraphId) -> Result<(), RemoveConnectionToError>;

    fn messages_sent(&self) -> usize;
    fn messages_received(&self) -> usize;

    fn reset_message_count(&mut self);
}

#[derive(Debug, derive_more::IsVariant)]
pub enum NodeKind {
    /// The node is a factor
    Factor(FactorNode),
    // TODO: wrap in Box<>
    /// The node is a variable
    Variable(VariableNode),
}

#[derive(Debug)]
pub struct Node {
    factorgraph_id: FactorGraphId,
    pub kind:       NodeKind,
}

impl Node {
    /// Construct a new node
    pub const fn new(factorgraph_id: FactorGraphId, kind: NodeKind) -> Self {
        Self { factorgraph_id, kind }
    }

    /// Returns `true` if the node is [`Factor`].
    ///
    /// [`Factor`]: Node::Factor
    #[must_use]
    #[inline]
    pub fn is_factor(&self) -> bool {
        self.kind.is_factor()
    }

    /// Returns a reference to the inner factor node
    ///
    /// # Panics
    ///
    /// Panics if the node is not a factor
    #[inline]
    pub fn factor(&self) -> &FactorNode {
        self.as_factor().expect("The node should be a Factor")
    }

    /// Returns a mutable reference to the inner factor node
    ///
    /// # Panics
    ///
    /// Panics if the node is not a factor
    #[inline]
    pub fn factor_mut(&mut self) -> &mut FactorNode {
        self.as_factor_mut().expect("The node should be a Factor")
    }

    /// Returns `Some(&Factor)` if the node]s variant is [`Factor`], otherwise
    /// `None`.
    pub const fn as_factor(&self) -> Option<&FactorNode> {
        if let NodeKind::Factor(ref v) = self.kind {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `Some(&mut Factor)` if the node]s variant is [`Factor`],
    /// otherwise `None`.
    pub fn as_factor_mut(&mut self) -> Option<&mut FactorNode> {
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
    pub const fn as_variable(&self) -> Option<&VariableNode> {
        if let NodeKind::Variable(ref v) = self.kind {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `Some(&mut Variable)` if the node]s variant is [`Variable`],
    /// otherwise `None`.
    pub fn as_variable_mut(&mut self) -> Option<&mut VariableNode> {
        if let NodeKind::Variable(ref mut v) = self.kind {
            Some(v)
        } else {
            None
        }
    }

    /// Returns a reference to the inner variable node
    ///
    /// # Panics
    ///
    /// Panics if the node is not a variable
    #[inline]
    pub fn variable(&self) -> &VariableNode {
        self.as_variable().expect("The node should be a Variable")
    }

    /// Returns a mutable reference to the inner variable node
    ///
    /// # Panics
    ///
    /// Panics if the node is not a variable
    #[inline]
    pub fn variable_mut(&mut self) -> &mut VariableNode {
        self.as_variable_mut().expect("The node should be a Variable")
    }
}

impl FactorGraphNode for Node {
    fn remove_connection_to(&mut self, factorgraph_id: FactorGraphId) -> Result<(), RemoveConnectionToError> {
        match self.kind {
            NodeKind::Factor(ref mut factor) => factor.remove_connection_to(factorgraph_id),
            NodeKind::Variable(ref mut variable) => variable.remove_connection_to(factorgraph_id),
        }
    }

    fn messages_sent(&self) -> usize {
        match self.kind {
            NodeKind::Factor(ref factor) => factor.messages_sent(),
            NodeKind::Variable(ref variable) => variable.messages_sent(),
        }
    }

    fn messages_received(&self) -> usize {
        match self.kind {
            NodeKind::Factor(ref factor) => factor.messages_received(),
            NodeKind::Variable(ref variable) => variable.messages_received(),
        }
    }

    fn reset_message_count(&mut self) {
        match self.kind {
            NodeKind::Factor(ref mut factor) => factor.reset_message_count(),
            NodeKind::Variable(ref mut variable) => variable.reset_message_count(),
        };
    }
}
