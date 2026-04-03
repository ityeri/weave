use super::Updater;
use crate::{NodeKey, PhysicalGraph, PhysicalNode};
use glam::Vec2;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::collections::{HashMap, HashSet};

pub struct DefaultUpdater {
    pub elasticity: f32,
    pub velocity_damping: f32,
    pub dt_amplification: f32,
    pub edge_length_rate: f32,
    pub min_mass: f32,
    pub min_protect_radius: f32,
    pub protect_radius_gap: f32,
}

impl DefaultUpdater {
    pub fn default_setting() -> Self {
        Self {
            elasticity: 2.0,
            velocity_damping: 1.0,
            dt_amplification: 1.0,
            edge_length_rate: 0.012,
            min_mass: 1.0,
            min_protect_radius: 1.0,
            protect_radius_gap: 0.2,
        }
    }

    fn update_node<K: NodeKey>(
        &self,
        graph: &PhysicalGraph<K>,
        node_key: K,
        dt: f32,
    ) -> PhysicalNode<K> {
        let self_node = graph.nodes.get(&node_key).unwrap();

        let mass = self.min_mass + self_node.incomings.len() as f32;

        let force = graph
            .nodes
            .values()
            .filter(|&node| node.key != node_key)
            .map(|node| -> Vec2 { Vec2::new(0.1, 0.0) })
            .sum::<Vec2>();

        let acc = force / mass;

        println!("{}", acc);

        PhysicalNode {
            key: node_key,
            incomings: self_node.incomings.iter().copied().collect::<HashSet<K>>(),
            outgoings: self_node.outgoings.iter().copied().collect::<HashSet<K>>(),
            position: 2.0 * self_node.position - self_node.prev_position + acc * dt.powi(2),
            prev_position: self_node.position,
        }
    }
}

impl Updater for DefaultUpdater {
    fn update<K: NodeKey>(&self, graph: PhysicalGraph<K>, dt: f32) -> PhysicalGraph<K> {
        PhysicalGraph::<K> {
            nodes: graph
                .nodes
                .keys()
                .copied()
                .collect::<Vec<K>>()
                .into_par_iter()
                .map(|node_key| (node_key, self.update_node(&graph, node_key, dt)))
                .collect::<HashMap<K, PhysicalNode<K>>>(),
        }
    }
}
