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

pub struct PhysicalNode<K: NodeKey> {
    pub incomings: HashSet<K>,
    pub outgoings: HashSet<K>,
    pub position: Vec2,
    pub prev_position: Vec2,
}
