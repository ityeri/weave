use super::Updater;
use crate::{
    PhysicalGraph,
    stored_graph::{StoredGraph, StoredNode},
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};
pub struct DefaultUpdater {
    pub elasticity: f64,
    pub velocity_damping: f64,
    pub dt_amplification: f64,
    pub edge_length_rate: f64,
    pub min_mass: f64,
    pub min_protect_radius: f64,
    pub protect_radius_gap: f64,
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

    fn node_update<K: Copy + Hash + Eq + Send + Sync>(
        &self,
        graph: &impl PhysicalGraph<K>,
        node_id: K,
        dt: f64,
    ) -> StoredNode<K> {
        let incomings = graph.get_incomings(node_id).collect::<Vec<_>>();
        let outgoings = graph.get_outgoings(node_id).collect::<Vec<_>>();
        let mut neighbors: HashSet<K> = HashSet::new();
        neighbors.extend(incomings.iter());
        neighbors.extend(outgoings.iter());

        let old_position = graph.get_position(node_id);
        let mut position = old_position;

        let mass = self.mass(graph, node_id);
        let inner_radius = self.inner_radius(graph, node_id);
        let outer_radius = self.outer_radius(graph, node_id);

        for other_node_id in graph.get_all_nodes() {
            if other_node_id == node_id {
                continue;
            }

            if neighbors.contains(&other_node_id) {
                // 1촌
                // println!("1촌");
                let relative_position = graph.get_position(other_node_id) - position;
                let goal_distance = inner_radius + self.inner_radius(graph, other_node_id);
                let actual_distance = relative_position.length();
                let distance_error = goal_distance - actual_distance;
                let other_mass = self.mass(graph, other_node_id);

                position -= (relative_position.normalize() * distance_error * self.elasticity * dt)
                    * (other_mass / (mass + other_mass))
                    * self.elasticity;
            } else {
                // 2촌 이상
                // println!("2촌");
                let relative_position = graph.get_position(other_node_id) - position;
                let min_distance = outer_radius + self.outer_radius(graph, other_node_id);
                let actual_distance = relative_position.length();
                let distance_error = min_distance - actual_distance;

                if 0.0 < distance_error {
                    let other_mass = self.mass(graph, other_node_id);
                    position -=
                        (relative_position.normalize() * distance_error * self.elasticity * dt)
                            * (other_mass / (mass + other_mass))
                            * self.elasticity;
                }
            }
        }
        // println!("===\n");

        let dt = dt * self.dt_amplification;
        // F = ma -> a = F / m
        let mut updated_velocity = (position - old_position) * dt;
        updated_velocity *= self.velocity_damping;
        let updated_position = position + updated_velocity * dt;

        StoredNode {
            position: updated_position,
            velocity: updated_velocity,
            incomings: incomings,
            outgoings: outgoings,
            in_degree: graph.get_in_degree(node_id),
            out_debree: graph.get_out_degree(node_id),
        }
    }

    fn mass<K: Copy + Hash + Eq>(&self, graph: &impl PhysicalGraph<K>, node_id: K) -> f64 {
        graph.get_in_degree(node_id) as f64 + self.min_mass
    }
    fn inner_radius<K: Copy + Hash + Eq>(&self, graph: &impl PhysicalGraph<K>, node_id: K) -> f64 {
        graph.get_in_degree(node_id) as f64 * self.edge_length_rate
    }
    fn outer_radius<K: Copy + Hash + Eq>(&self, graph: &impl PhysicalGraph<K>, node_id: K) -> f64 {
        self.inner_radius(graph, node_id) * (1.0 + self.protect_radius_gap)
            + self.min_protect_radius
    }
}

impl Updater for DefaultUpdater {
    fn update<K: Copy + Hash + Eq + Send + Sync>(
        &self,
        graph: impl PhysicalGraph<K>,
        dt: f64,
    ) -> StoredGraph<K> {
        let converted_graph = StoredGraph {
            nodes: graph
                .get_all_nodes()
                .map(|node_id| {
                    (
                        node_id,
                        StoredNode::<K> {
                            position: graph.get_position(node_id),
                            velocity: graph.get_velocity(node_id),
                            incomings: graph.get_incomings(node_id).collect::<Vec<_>>(),
                            outgoings: graph.get_outgoings(node_id).collect::<Vec<_>>(),
                            in_degree: graph.get_in_degree(node_id),
                            out_debree: graph.get_out_degree(node_id),
                        },
                    )
                })
                .collect::<HashMap<K, StoredNode<K>>>(),
        };

        StoredGraph::<K> {
            nodes: converted_graph
                .get_all_nodes()
                .collect::<Vec<_>>()
                .into_par_iter()
                .map(|node_id| (node_id, self.node_update(&converted_graph, node_id, dt)))
                .collect::<HashMap<K, StoredNode<K>>>(),
        }
    }
}
