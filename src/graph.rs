use glam::Vec2;
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

pub trait NodeKey: Copy + Eq + Hash + Send + Sync {}
impl<T: Copy + Hash + Eq + Send + Sync> NodeKey for T {}

pub struct PhysicalGraph<K: NodeKey> {
    pub nodes: HashMap<K, PhysicalNode<K>>,
}

impl<K: NodeKey> PhysicalGraph<K> {
    pub fn new(nodes: HashMap<K, PhysicalNode<K>>) -> Self {
        PhysicalGraph { nodes }
    }
}

pub struct PhysicalNode<K: NodeKey> {
    pub key: K,
    pub incomings: HashSet<K>,
    pub outgoings: HashSet<K>,
    pub position: Vec2,
    pub prev_position: Vec2,
}

impl<K: NodeKey> PhysicalNode<K> {
    pub fn new(
        key: K,
        incomings: HashSet<K>,
        outgoings: HashSet<K>,
        position: Vec2,
        prev_position: Vec2,
    ) -> Self {
        PhysicalNode {
            key,
            incomings,
            outgoings,
            position,
            prev_position,
        }
    }
}
