use super::factorgraph::{FactorGraphId, FactorIndex, VariableIndex};

/// Unique identifier of a factor in the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::Display)]
#[display(fmt = "{:?}-{}", factorgraph_id, "factor_index.0.index()")]
pub struct FactorId {
    /// The id of the factorgraph that the factor belongs to.
    pub factorgraph_id: FactorGraphId,
    /// The index of the factor in the factorgraph.
    pub factor_index: FactorIndex,
}

impl FactorId {
    /// Create a new `FactorId`.
    #[must_use]
    pub const fn new(factorgraph_id: FactorGraphId, factor_index: FactorIndex) -> Self {
        Self {
            factorgraph_id,
            factor_index,
        }
    }
}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl std::cmp::PartialOrd for FactorId {
    /// Returns `Some(std::cmp::ordering::Equal)` if `self` and `other` are
    /// equal, `Some(std::cmp::ordering::Less)` if `other.factorgraph_id` is
    /// greater than `self.factorgraph_id`, or if the factors belong to the
    /// same factorgraph and `other.factor_index` is greater than
    /// `self.factor_index`. Returns `Some(std::cmp::ordering::Greater)`
    /// otherwise.
    ///
    /// # NOTE
    ///
    /// This ordering is really important for the **gbpplanner** algorithm, to
    /// work correctly.
    #[allow(clippy::if_same_then_else)]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.cmp(&other).into()
    }
}


impl std::cmp::Ord for FactorId {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.factorgraph_id
            .cmp(&other.factorgraph_id)
            .then_with(|| self.factor_index.cmp(&other.factor_index))
    }
}

// implement PartialEq and Eq manually
// impl std::cmp::PartialEq for FactorId {
//     fn eq(&self, other: &Self) -> bool {
//         self.factorgraph_id == other.factorgraph_id && self.factor_index ==
// other.factor_index     }
// }
//
// impl std::cmp::Eq for FactorId {}

/// Unique identifier of a variable in the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::Display)]
#[display(fmt = "{:?}-{}", factorgraph_id, "variable_index.0.index()")]
pub struct VariableId {
    /// The id of the factorgraph that the variable belongs to.
    pub factorgraph_id: FactorGraphId,
    /// The index of the variable in the factorgraph.
    pub variable_index: VariableIndex,
}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl std::cmp::PartialOrd for VariableId {
    /// Returns `Some(std::cmp::ordering::Equal)` if `self` and `other` are
    /// equal, `Some(std::cmp::ordering::Less)` if `other.factorgraph_id` is
    /// greater than `self.factorgraph_id`, or if the factors belong to the
    /// same factorgraph and `other.variable_index` is greater than
    /// `self.variable_index`. Returns `Some(std::cmp::ordering::Greater)`
    /// otherwise.
    ///
    /// # NOTE
    ///
    /// This ordering is really important for the **gbpplanner** algorithm, to
    /// work correctly.
    #[allow(clippy::if_same_then_else)]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.cmp(&other).into()
    }
}

impl std::cmp::Ord for VariableId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
         self.factorgraph_id
            .cmp(&other.factorgraph_id)
            .then_with(|| self.variable_index.cmp(&other.variable_index))
    }
}

// implement PartialEq and Eq manually
// impl std::cmp::PartialEq for VariableId {
//     fn eq(&self, other: &Self) -> bool {
//         self.factorgraph_id == other.factorgraph_id && self.variable_index ==
// other.variable_index     }
// }
// impl std::cmp::Eq for VariableId {}

impl VariableId {
    /// Create a new `VariableId`.
    #[must_use]
    pub const fn new(factorgraph_id: FactorGraphId, variable_index: VariableIndex) -> Self {
        Self {
            factorgraph_id,
            variable_index,
        }
    }
}
