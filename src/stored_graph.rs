use crate::PhysicalGraph;
use glam::DVec2;
use std::collections::HashMap;
use std::hash::Hash;

pub struct StoredGraph<K: Copy + Hash + Eq + Send + Sync> {
    pub nodes: HashMap<K, StoredNode<K>>,
}
impl<K: Copy + Hash + Eq + Send + Sync> PhysicalGraph<K> for StoredGraph<K> {
    fn get_all_nodes(&self) -> impl Iterator<Item = K> {
        self.nodes.keys().map(|node_id| *node_id)
    }
    fn get_position(&self, node_id: K) -> DVec2 {
        self.nodes[&node_id].position
    }
    fn get_velocity(&self, node_id: K) -> DVec2 {
        self.nodes[&node_id].velocity
    }

    fn get_incomings(&self, node_id: K) -> impl Iterator<Item = K> {
        self.nodes[&node_id].incomings.iter().copied()
    }
    fn get_outgoings(&self, node_id: K) -> impl Iterator<Item = K> {
        self.nodes[&node_id].outgoings.iter().copied()
    }

    fn get_in_degree(&self, node_id: K) -> u32 {
        self.nodes[&node_id].in_degree
    }
    fn get_out_degree(&self, node_id: K) -> u32 {
        self.nodes[&node_id].out_debree
    }
}
impl<K: Copy + Hash + Eq + Send + Sync> Clone for StoredGraph<K> {
    fn clone(&self) -> Self {
        Self {
            nodes: self.nodes.clone(),
        }
    }
}

pub struct StoredNode<K: Copy + Hash + Eq + Send + Sync> {
    pub position: DVec2,
    pub velocity: DVec2,
    pub incomings: Vec<K>,
    pub outgoings: Vec<K>,
    pub in_degree: u32,
    pub out_debree: u32,
}
impl<K: Copy + Hash + Eq + Send + Sync> Clone for StoredNode<K> {
    fn clone(&self) -> Self {
        Self {
            position: self.position.clone(),
            velocity: self.velocity.clone(),
            incomings: self.incomings.clone(),
            outgoings: self.outgoings.clone(),
            in_degree: self.in_degree,
            out_debree: self.out_debree,
        }
    }
}
