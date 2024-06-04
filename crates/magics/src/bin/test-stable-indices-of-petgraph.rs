// use petgraph::

use petgraph::prelude::*;

fn main() {
    let mut graph = Graph::<usize, ()>::new();
    let node_ix_a = graph.add_node(0);
    let node_ix_b = graph.add_node(1);
    let node_ix_c = graph.add_node(2);
    let node_ix_d = graph.add_node(3);

    graph.add_edge(node_ix_a, node_ix_b, ());
    graph.add_edge(node_ix_a, node_ix_c, ());
    graph.add_edge(node_ix_a, node_ix_d, ());
    graph.add_edge(node_ix_b, node_ix_c, ());
    graph.add_edge(node_ix_b, node_ix_d, ());
    graph.add_edge(node_ix_c, node_ix_d, ());

    // let mut graph = Graph::<usize, ()>::from_edges(&[(0, 1), (0, 2), (0, 3), (1,
    // 2), (1, 3), (2, 3)]);
    dbg!(&graph);

    for node_ix in graph.node_indices() {
        let node = graph.node_weight(node_ix).unwrap();
        println!("node_ix: {:?}, node: {:?}", node_ix, node);
    }
    let node_ix = graph.node_indices().nth(1).unwrap();
    let node_before = graph.node_weight(node_ix).copied().unwrap();

    // remove the first node
    graph.remove_node(graph.node_indices().nth(0).unwrap());
    dbg!(&graph);
    let node_after = graph.node_weight(node_ix).unwrap();

    dbg!(node_before);
    dbg!(node_after);

    // let mut graphmap = UnGraphMap::<Node, ()>::new();
}

struct Node {
    belief: Vec<usize>,
}
