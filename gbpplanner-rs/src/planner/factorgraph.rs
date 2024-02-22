use bevy::prelude::*;
use petgraph::Undirected;
use petgraph::{
    dot::{Config, Dot},
    graph::Graph,
};

pub enum Node {
    Factor(Factor),
    Variable(Variable),
}

/// A factor graph is a bipartite graph consisting of two types of nodes: factors and variables.
/// Factors and variables are stored in separate btree maps, that are indexed by a unique tuple of (robot_id, node_id).
#[derive(Component, Debug)]
pub struct FactorGraph {
    // /// Called `factors_` in **gbpplanner**.
    // /// **gbpplanner** uses `std::map<Key, std::shared_ptr<Factor>>`
    // /// So we use `BTreeMap` as it provides iteration sorted by the `Key` similar to `std::map` in C++.
    // pub factors: BTreeMap<Key, Rc<Factor>>,
    // /// Called `variables_` in **gbpplanner**.
    // /// **gbpplanner** uses `std::map<Key, std::shared_ptr<Variable>>`
    // /// So we use `BTreeMap` as it provides iteration sorted by the `Key` similar to `std::map` in C++.
    // pub variables: BTreeMap<Key, Rc<Variable>>,
    graph: Graph<Node, (), Undirected>,
    /// Flag for whether this factorgraph/robot communicates with other robots
    interrobot_comms_active: bool,
    /// Node id counter
    node_id_counter: usize,
}

impl FactorGraph {
    pub fn new() -> Self {
        Self {
            graph: Graph::new_undirected(),
            interrobot_comms_active: true,
        }
    }

    pub fn next_node_id(&mut self) -> usize {
        self.node_id_counter += 1;
        self.node_id_counter
    }

    pub fn add_variable(&mut self, variable: Variable) -> NodeIndex {
        let node_index = self.graph.add_node(Node::Variable(variable));
        self.graph[node_index].set_node_index(node_index);
        node_index
    }

    pub fn add_factor(&mut self, factor: Factor) -> NodeIndex {
        let node_index = self.graph.add_node(Node::Factor(factor));
        self.graph[node_index].set_node_index(node_index);
        node_index
    }

    pub fn add_edge(&mut self, a: NodeIndex, b: NodeIndex) -> EdgeIndex {
        self.graph.add_edge(a, b, ())
    }

    // TODO: Implement our own export to `DOT` format, which can be much more specific with styling.
    /// Exports tree to `graphviz` `DOT` format
    pub fn export() -> String {
        format!(
            "{:?}",
            Dot::with_config(&self.graph, &[Config::EdgeNoLabel])
        )
    }
}
