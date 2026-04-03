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
        let node = graph.nodes.get(&node_key).unwrap();

        let mass = self.min_mass + node.incomings.len() as f32;

        let force = graph
            .nodes
            .values()
            .filter(|&node| node.key != node_key)
            .map(|other_node| -> Vec2 {
                let other_node_mass = self.min_mass + other_node.incomings.len() as f32;

                if node.incomings.contains(&other_node.key)
                    || node.outgoings.contains(&other_node.key)
                {
                    let diff = other_node.position - node.position;

                    let target_distance =
                        (node.incomings.len() + other_node.incomings.len()) as f32 * 0.3 + 1.0;
                    let distance_diff = diff.length() - target_distance;

                    diff.normalize_or_zero() * distance_diff * 20.0 * mass * other_node_mass
                } else {
                    let diff = other_node.position - node.position;

                    let min_distance =
                        (node.incomings.len() + other_node.incomings.len()) as f32 * 0.2 + 2.0;

                    if diff.length() < min_distance {
                        let distance_diff = min_distance - diff.length();

                        -diff.normalize_or_zero() * distance_diff * mass * other_node_mass * 100.0
                    } else {
                        Vec2::ZERO
                    }
                }
            })
            .sum::<Vec2>();

        let velocity = node.position - node.prev_position;
        let acc = force / mass - velocity * 800.0;

        PhysicalNode {
            key: node_key,
            incomings: node.incomings.iter().copied().collect::<HashSet<K>>(),
            outgoings: node.outgoings.iter().copied().collect::<HashSet<K>>(),
            position: 2.0 * node.position - node.prev_position + acc * dt.powi(2),
            prev_position: node.position,
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
