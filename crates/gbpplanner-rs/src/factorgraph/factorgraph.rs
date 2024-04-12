use bevy::{
    ecs::{component::Component, entity::Entity},
    log::debug,
};
use petgraph::Undirected;

use super::{
    factor::Factor,
    node::{Node, NodeKind},
    variable::Variable,
};

/// type alias used to represent the id of the factorgraph
/// Since we use **Bevy** we can use the `Entity` id of the whatever entity the
/// the factorgraph is attached to as a Component, as its unique identifier.
pub type FactorGraphId = Entity;

/// The type used to represent indices into the nodes of the factorgraph.
/// This is just a type alias for `petgraph::graph::NodeIndex`, but
/// we make an alias for it here, such that it is easier to use the same
/// index type across modules, as the various node index types `petgraph`
/// are not interchangeable.
pub type NodeIndex = petgraph::stable_graph::NodeIndex;
/// The type used to represent indices into the nodes of the factorgraph.
pub type EdgeIndex = petgraph::stable_graph::EdgeIndex;
/// A factorgraph is an undirected graph
pub type Graph = petgraph::stable_graph::StableGraph<Node, (), Undirected, u32>;

/// A newtype used to enforce type safety of the indices of the factors in the
/// factorgraph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, derive_more::From)]
pub struct FactorIndex(pub NodeIndex);
/// A newtype used to enforce type safety of the indices of the variables in the
/// factorgraph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, derive_more::From)]
pub struct VariableIndex(pub NodeIndex);

/// A factor graph is a bipartite graph consisting of two types of nodes:
/// factors and variables.
#[derive(Component, Debug)]
// #[cfg_attr(feature = "bevy", derive(Component))]
pub struct FactorGraph {
    /// The id of the factorgraph. We store a copy of it here, for convenience.
    /// **Invariants**:
    /// - The id of the factorgraph is unique among all factorgraphs in the
    ///   system.
    /// - The id does not change during the lifetime of the factorgraph.
    id: FactorGraphId,
    /// The underlying graph data structure
    graph: Graph,
    /// In **gbpplanner** the sequence in which variables are inserted/created
    /// in the graph is meaningful. `self.graph` does not capture this
    /// ordering, so we use an extra vector to manage the order in which
    /// variables are inserted/removed from the graph.
    ///
    /// **IMPORTANT** we have  to manually ensure the invariant that
    /// `self.graph` and this field is consistent at all time.
    variable_indices: Vec<NodeIndex>,
    /// List of indices of the factors in the graph. Order is not important.
    /// Used to speed up iteration over factors.
    factor_indices: Vec<NodeIndex>,

    /// List of indices of the interrobot factors in the graph. Order is not
    /// important. Used to speed up iteration over interrobot factors.
    /// When querying for number of external messages sent
    interrobot_factor_indices: Vec<NodeIndex>,
}

impl FactorGraph {
    /// Construct a new empty factorgraph with a given id
    #[must_use]
    pub fn new(id: FactorGraphId) -> Self {
        Self {
            id,
            graph: Graph::with_capacity(0, 0),
            variable_indices: Vec::new(),
            factor_indices: Vec::new(),
            interrobot_factor_indices: Vec::new(),
        }
    }

    /// Construct a new empty factorgraph with the specified capacity
    /// for nodes and edges.
    #[must_use]
    pub fn with_capacity(id: FactorGraphId, nodes: usize, edges: usize) -> Self {
        Self {
            id,
            graph: Graph::with_capacity(nodes, edges),
            variable_indices: Vec::with_capacity(nodes),
            factor_indices: Vec::with_capacity(edges),
            interrobot_factor_indices: Vec::new(),
        }
    }

    /// Returns the `FactorGraphId` of the factorgraph
    #[inline(always)]
    pub fn id(&self) -> FactorGraphId {
        self.id
    }

    /// Adds a variable to the factorgraph
    /// Returns the index of the variable in the factorgraph
    pub fn add_variable(&mut self, variable: Variable) -> VariableIndex {
        let node = Node::new(self.id, NodeKind::Variable(variable));
        let node_index = self.graph.add_node(node);
        self.variable_indices.push(node_index);
        self.graph[node_index]
            .as_variable_mut()
            .expect("just added the variable to the graph in the previous statement")
            .set_node_index(node_index);
        debug!(
            "added a variable with node_index: {:?} to factorgraph: {:?}",
            node_index, self.id
        );
        node_index.into()
    }

    pub fn add_factor(&mut self, factor: Factor) -> FactorIndex {
        let is_interrobot = factor.is_inter_robot();
        let node = Node::new(self.id, NodeKind::Factor(factor));
        let node_index = self.graph.add_node(node);
        self.graph[node_index]
            .as_factor_mut()
            .expect("just added the factor to the graph in the previous statement")
            // .tap(|f| {
            //     info!(
            //         "adding a '{}' factor with node_index: {:?} to factorgraph: {:?}",
            //         f.variant(),
            //         node_index,
            //         self.id
            //     );
            // })
            .set_node_index(node_index);
        self.factor_indices.push(node_index);
        if is_interrobot {
            self.interrobot_factor_indices.push(node_index);
        }

        node_index.into()
    }

    /// Number of nodes in the factorgraph
    ///
    /// **Computes in O(1) time**
    #[inline]
    pub fn len(&self) -> usize {
        self.graph.node_count()
    }

    /// A count over the number of variables and factors in the factorgraph
    ///
    /// **Computes in O(1) time**
    #[must_use]
    pub fn node_count(&self) -> NodeCount {
        NodeCount {
            factors:   self.factor_indices.len(),
            variables: self.variable_indices.len(),
        }
    }
}

/// Record type used to keep track of how many factors and variables
/// there are in the factorgraph. We keep track of these counts internally in
/// the factorgraph, such a query for the counts, is **O(1)**.
#[derive(Debug, Clone, Copy, Default)]
pub struct NodeCount {
    pub factors:   usize,
    pub variables: usize,
}

/// Iterator over the factors in the factorgraph.
///
/// Iterator element type is `(FactorIndex, &'a Factor)`.
///
/// Created with [`.factors()`][1]
///
/// [1]: struct.FactorGraph.html#method.factors
pub struct Factors<'a> {
    graph: &'a Graph,
    factor_indices: std::slice::Iter<'a, NodeIndex>,
}

impl<'a> Factors<'a> {
    #[must_use]
    pub fn new(graph: &'a Graph, factor_indices: &'a [NodeIndex]) -> Self {
        Self {
            graph,
            factor_indices: factor_indices.iter(),
        }
    }
}

impl FactorGraph {
    /// Returns an iterator over the factors in the factorgraph.
    #[inline]
    #[must_use]
    pub fn factors(&self) -> Factors<'_> {
        Factors::new(&self.graph, &self.factor_indices)
    }
}

impl<'a> Iterator for Factors<'a> {
    type Item = (NodeIndex, &'a Factor);

    fn next(&mut self) -> Option<Self::Item> {
        let &index = self.factor_indices.next()?;
        let node = &self.graph[index];
        node.as_factor().map(|factor| (index, factor))
    }
}

/// Iterator over the variables in the factorgraph.
///
/// Iterator element type is `(VariableIndex, &'a Variable)`.
///
/// Created with [`.variables()`][1]
///
/// [1]: struct.FactorGraph.html#method.variables
pub struct Variables<'a> {
    graph: &'a Graph,
    variable_indices: std::slice::Iter<'a, NodeIndex>,
}

impl<'a> Variables<'a> {
    pub fn new(graph: &'a Graph, variable_indices: &'a [NodeIndex]) -> Self {
        Self {
            graph,
            variable_indices: variable_indices.iter(),
        }
    }
}

impl<'a> Iterator for Variables<'a> {
    type Item = (VariableIndex, &'a Variable);

    fn next(&mut self) -> Option<Self::Item> {
        let &index = self.variable_indices.next()?;
        let node = &self.graph[index];
        node.as_variable()
            .map(|variable| (VariableIndex(index), variable))
    }
}

impl FactorGraph {
    /// Returns an iterator over the variables in the factorgraph.
    #[inline]
    #[must_use]
    pub fn variables(&self) -> Variables<'_> {
        Variables::new(&self.graph, &self.variable_indices)
    }
}

/// Iterator over the interrobot factors in the factorgraph.
///
/// Iterator element type is `(FactorIndex, &'a InterRobotFactor)`.
///
/// Created with [`.inter_robot_factors()`][1]
pub struct InterRobotFactors<'a> {
    graph: &'a Graph,
    factor_indices: std::slice::Iter<'a, NodeIndex>,
}

impl<'a> InterRobotFactors<'a> {
    pub fn new(graph: &'a Graph, factor_indices: &'a [NodeIndex]) -> Self {
        Self {
            graph,
            factor_indices: factor_indices.iter(),
        }
    }
}

impl<'a> Iterator for InterRobotFactors<'a> {
    type Item = (NodeIndex, &'a InterRobotFactor);

    fn next(&mut self) -> Option<Self::Item> {
        let &index = self.factor_indices.next()?;
        let node = &self.graph[index];
        node.as_factor().map(|factor| (index, factor))
    }
}

impl FactorGraph {
    #[inline]
    #[must_use]
    pub fn inter_robot_factors(&self) -> InterRobotFactors<'_> {
        InterRobotFactors::new(&self.graph, &self.interrobot_factor_indices)
    }
}

impl std::ops::Index<FactorIndex> for FactorGraph {
    type Output = Factor;

    // type Output = Option<Factor>;

    fn index(&self, index: FactorIndex) -> &Self::Output {
        self.graph[index.into()].as_factor()
    }
}

impl std::ops::Index<VariableIndex> for FactorGraph {
    type Output = Variable;

    fn index(&self, index: VariableIndex) -> &Self::Output {
        self.graph[index.into()]
            .as_variable()
            .expect("a variable index points to a variable node in the graph")
    }
}
