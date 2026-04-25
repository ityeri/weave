use super::Updater;
use crate::{NodeKey, PhysicalGraph, PhysicalNode};
use glam::Vec2;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

#[derive(Clone, Copy)]
struct Rect {
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
}

impl Rect {
    fn new(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Rect {
        Rect {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    fn contain(&self, v: Vec2) -> bool {
        self.min_x <= v.x && v.x <= self.max_x && self.min_y <= v.y && v.y <= self.max_y
    }
}

struct Body {
    position: Vec2,
    mass: f32,
}

struct TreeNode {
    // Surprisingly, region field is not need to read..
    // only center field is required
    // region: Rect,
    center: Vec2,
    radius: f32,
    mass: f32,
    children: Option<Vec<Arc<TreeNode>>>,
}

impl TreeNode {
    fn create_node(bodies: Vec<&Body>, region: Rect, depth: usize, max_depth: usize) -> TreeNode {
        if bodies.is_empty() {
            return TreeNode {
                center: Vec2::new(
                    (region.min_x + region.max_x) / 2.0,
                    (region.min_y + region.max_y) / 2.0,
                ),
                radius: 0.0,
                mass: 0.0,
                children: None,
            };
        }

        let position_sum = bodies
            .iter()
            .map(|&body| body.position * body.mass)
            .sum::<Vec2>();
        let mass = bodies.iter().map(|&body| body.mass).sum::<f32>();

        let center = if bodies.is_empty() {
            Vec2::new(
                (region.min_x + region.max_x) / 2.0,
                (region.min_y + region.max_y) / 2.0,
            )
        } else {
            position_sum / mass
        };

        let radius = bodies
            .iter()
            .map(|body| center.distance(body.position))
            .reduce(f32::max)
            .unwrap_or(0.0);

        let is_depth_over = if max_depth == 0 {
            false
        } else {
            max_depth <= depth
        };

        let children = if bodies.len() <= 1 || is_depth_over {
            None
        } else {
            let center_x = (region.min_x + region.max_x) / 2.0;
            let center_y = (region.min_y + region.max_y) / 2.0;

            Some(
                vec![
                    Rect::new(region.min_x, region.min_y, center_x, center_y),
                    Rect::new(center_x, region.min_y, region.max_x, center_y),
                    Rect::new(region.min_x, center_y, center_x, region.max_y),
                    Rect::new(center_x, center_y, region.max_x, region.max_y),
                ]
                .into_par_iter()
                .map(|region| {
                    Arc::new(TreeNode::create_node(
                        bodies
                            .clone()
                            .into_iter()
                            .filter(|&body| region.contain(body.position))
                            .collect::<Vec<&Body>>(),
                        region,
                        depth + 1,
                        max_depth,
                    ))
                })
                .collect::<Vec<Arc<TreeNode>>>(),
            )
        };

        TreeNode {
            center,
            radius,
            mass,
            children,
        }
    }
}

pub struct QuadTreeUpdater {
    pub base_neighbor_radius: f32,
    pub neighbor_radius: f32,
    pub neighbor_space_ratio: f32,
    pub mass_amplification: f32,
    pub min_node_mass: f32,
    pub neighbor_edge_elasticity: f32,
    pub non_neighbor_repulsive_force: f32,
    pub non_neighbor_distance_softning: f32,
    pub attenuation_rate: f32,
    pub max_tree_node_radius_ratio: f32,
}

impl QuadTreeUpdater {
    pub fn default_setting() -> Self {
        Self {
            base_neighbor_radius: 3.0,
            neighbor_radius: 0.5,
            neighbor_space_ratio: 0.2,
            mass_amplification: 0.25,
            min_node_mass: 1.0,
            neighbor_edge_elasticity: 20.0,
            non_neighbor_repulsive_force: 2000.0,
            non_neighbor_distance_softning: 0.1,
            attenuation_rate: 800.0,
            max_tree_node_radius_ratio: 0.5,
        }
    }

    fn create_root_node<K: NodeKey>(&self, graph: &PhysicalGraph<K>) -> TreeNode {
        let min_x = graph
            .nodes
            .values()
            .map(|node| node.position.x)
            .reduce(f32::min)
            .unwrap();
        let min_y = graph
            .nodes
            .values()
            .map(|node| node.position.y)
            .reduce(f32::min)
            .unwrap();
        let max_x = graph
            .nodes
            .values()
            .map(|node| node.position.x)
            .reduce(f32::max)
            .unwrap();
        let max_y = graph
            .nodes
            .values()
            .map(|node| node.position.y)
            .reduce(f32::max)
            .unwrap();

        let bodies = graph
            .nodes
            .values()
            .map(|node| Body {
                position: node.position,
                mass: self.get_mass(node),
            })
            .collect::<Vec<Body>>();

        TreeNode::create_node(
            bodies.iter().collect::<Vec<&Body>>(),
            Rect::new(min_x, min_y, max_x, max_y),
            0,
            127,
        )
    }

    fn update_node<K: NodeKey>(
        &self,
        graph: &PhysicalGraph<K>,
        node: &PhysicalNode<K>,
        tree_node: Arc<TreeNode>,
        dt: f32,
    ) -> PhysicalNode<K> {
        let mass = self.get_mass(node);
        let this_radius = self.get_neighbor_radius(node);

        let body = Body {
            position: node.position,
            mass: self.get_mass(node),
        };
        let global_force = self.calculate_tree_node_force(&body, tree_node);

        let neighbor_force = graph
            .nodes
            .values()
            .filter(|&other_node| {
                other_node.key != node.key
                    && (node.incomings.contains(&other_node.key)
                        || node.outgoings.contains(&other_node.key))
            })
            .map(|other_node| -> Vec2 {
                let other_node_mass = self.get_mass(&other_node);

                let edge_length = this_radius + self.get_neighbor_radius(&other_node);

                let min_distance = edge_length * (1.0 - self.neighbor_space_ratio / 2.0)
                    + self.base_neighbor_radius;
                let max_distance = edge_length * (1.0 + self.neighbor_space_ratio / 2.0)
                    + self.base_neighbor_radius;

                let diff = other_node.position - node.position;

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
            })
            .sum::<Vec2>();

        let velocity = node.position - node.prev_position;
        let acc = (neighbor_force + global_force) / mass - velocity * self.attenuation_rate;

        PhysicalNode {
            key: node.key,
            incomings: node.incomings.iter().copied().collect::<HashSet<K>>(),
            outgoings: node.outgoings.iter().copied().collect::<HashSet<K>>(),
            position: 2.0 * node.position - node.prev_position + acc * dt.powi(2),
            prev_position: node.position,
        }
    }

    /// Calculates force from tree node.
    fn calculate_tree_node_force(&self, body: &Body, tree_node: Arc<TreeNode>) -> Vec2 {
        let distance = body.position.distance(tree_node.center);

        // It's not enough as f32 std epsilon. Just using distance softening value is more appropriate.
        if distance <= self.non_neighbor_distance_softning {
            return Vec2::ZERO;
        }

        if tree_node.radius < distance * self.max_tree_node_radius_ratio
            || tree_node.children.is_none()
        {
            let diff = tree_node.center - body.position;

            -diff.normalize_or_zero()
                * self.non_neighbor_repulsive_force
                * (body.mass * tree_node.mass)
                / (distance.powi(2) + self.non_neighbor_distance_softning)
        } else {
            tree_node
                .children
                .clone()
                .unwrap()
                .into_iter()
                .map(|child| self.calculate_tree_node_force(body, child))
                .sum::<Vec2>()
        }
    }

    fn get_mass<K: NodeKey>(&self, node: &PhysicalNode<K>) -> f32 {
        self.min_node_mass + node.incomings.len() as f32 * self.mass_amplification
    }
    fn get_neighbor_radius<K: NodeKey>(&self, node: &PhysicalNode<K>) -> f32 {
        node.incomings.len() as f32 * self.neighbor_radius
    }
}

impl Updater for QuadTreeUpdater {
    fn update<K: NodeKey>(&self, graph: &PhysicalGraph<K>, dt: f32) -> PhysicalGraph<K> {
        let tree_node_ref = Arc::new(self.create_root_node(&graph));

        PhysicalGraph::<K> {
            nodes: graph
                .nodes
                .keys()
                .copied()
                .collect::<Vec<K>>()
                .into_par_iter()
                .map(|node_key| {
                    (
                        node_key,
                        self.update_node(
                            &graph,
                            graph.nodes.get(&node_key).unwrap(),
                            Arc::clone(&tree_node_ref),
                            dt,
                        ),
                    )
                })
                .collect::<HashMap<K, PhysicalNode<K>>>(),
        }
    }
}
