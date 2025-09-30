use crate::{PhysicalGraph, stored_graph::StoredGraph};
use std::hash::Hash;

pub trait Updater {
    fn update<K: Copy + Hash + Eq + Send + Sync>(
        &self,
        graph: impl PhysicalGraph<K>,
        dt: f64,
    ) -> StoredGraph<K>;
}
