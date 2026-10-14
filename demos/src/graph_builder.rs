use std::collections::{HashMap, HashSet};

use petgraph::{Directed, Direction, graph::NodeIndex, stable_graph::StableGraph, visit::EdgeRef};
use weave::{NodeKey, PhysicalGraph, PhysicalNode};

use crate::body::Body;

pub trait PhysicalGraphBuilder<K: NodeKey> {
    fn to_physical_graph(&self) -> PhysicalGraph<K>;
}

impl PhysicalGraphBuilder<NodeIndex> for StableGraph<Body, (), Directed> {
    fn to_physical_graph(&self) -> PhysicalGraph<NodeIndex> {
        PhysicalGraph::new(
            self.node_indices()
                .map(|node_index| {
                    let node = PhysicalNode::new(
                        node_index,
                        self.edges_directed(node_index, Direction::Incoming)
                            .map(|edge_ref| edge_ref.source())
                            .collect::<HashSet<NodeIndex>>(),
                        self.edges_directed(node_index, Direction::Outgoing)
                            .map(|edge_ref| edge_ref.target())
                            .collect::<HashSet<NodeIndex>>(),
                        self[node_index].position,
                        self[node_index].prev_position,
                    );

                    (node_index, node)
                })
                .collect::<HashMap<NodeIndex, PhysicalNode<NodeIndex>>>(),
        )
    }
}
