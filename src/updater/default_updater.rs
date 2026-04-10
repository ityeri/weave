use super::Updater;
use crate::{NodeKey, PhysicalGraph, PhysicalNode};
use glam::Vec2;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::collections::{HashMap, HashSet};

pub struct DefaultUpdater {
    pub base_neighbor_radius: f32,
    pub neighbor_radius: f32,
    pub neighbor_space_ratio: f32,
    pub mass_amplification: f32,
    pub min_node_mass: f32,
    pub neighbor_edge_elasticity: f32,
    pub non_neighbor_repulsive_force: f32,
    pub non_neighbor_distance_softning: f32,
    pub attenuation_rate: f32,
}

impl DefaultUpdater {
    pub fn default_setting() -> Self {
        Self {
            base_neighbor_radius: 3.0,
            neighbor_radius: 0.5,
            neighbor_space_ratio: 0.2,
            mass_amplification: 0.25,
            min_node_mass: 1.0,
            neighbor_edge_elasticity: 20.0,
            non_neighbor_repulsive_force: 2000.0,
            non_neighbor_distance_softning: 0.01,
            attenuation_rate: 800.0,
        }
    }

    fn update_node<K: NodeKey>(
        &self,
        graph: &PhysicalGraph<K>,
        node_key: K,
        dt: f32,
    ) -> PhysicalNode<K> {
        let node = graph.nodes.get(&node_key).unwrap();

        let mass = self.get_mass(&node);
        let this_radius = self.get_neighbor_radius(&node);

        let force = graph
            .nodes
            .values()
            .filter(|&node| node.key != node_key)
            .map(|other_node| -> Vec2 {
                let other_node_mass = self.get_mass(&other_node);

                let edge_length = this_radius + self.get_neighbor_radius(&other_node);

                let min_distance = edge_length * (1.0 - self.neighbor_space_ratio / 2.0)
                    + self.base_neighbor_radius;
                let max_distance = edge_length * (1.0 + self.neighbor_space_ratio / 2.0)
                    + self.base_neighbor_radius;

                let diff = other_node.position - node.position;

                if node.incomings.contains(&other_node.key)
                    || node.outgoings.contains(&other_node.key)
                {
                    let distance_diff = if diff.length() < min_distance {
                        diff.length() - min_distance
                    } else if max_distance < diff.length() {
                        diff.length() - max_distance
                    } else {
                        0.0f32
                    };

                    self.neighbor_edge_elasticity
                        * diff.normalize_or_zero()
                        * distance_diff
                        * mass
                        * other_node_mass
                } else {
                    self.non_neighbor_repulsive_force
                        * -diff.normalize_or_zero()
                        * (mass * other_node_mass)
                        / (diff.length().powi(2) + self.non_neighbor_distance_softning)
                }
            })
            .sum::<Vec2>();

        let velocity = node.position - node.prev_position;
        let acc = force / mass - velocity * self.attenuation_rate;

        PhysicalNode {
            key: node_key,
            incomings: node.incomings.iter().copied().collect::<HashSet<K>>(),
            outgoings: node.outgoings.iter().copied().collect::<HashSet<K>>(),
            position: 2.0 * node.position - node.prev_position + acc * dt.powi(2),
            prev_position: node.position,
        }
    }

    fn get_mass<K: NodeKey>(&self, node: &PhysicalNode<K>) -> f32 {
        self.min_node_mass + node.incomings.len() as f32 * self.mass_amplification
    }
    fn get_neighbor_radius<K: NodeKey>(&self, node: &PhysicalNode<K>) -> f32 {
        node.incomings.len() as f32 * self.neighbor_radius
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
