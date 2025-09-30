use glam::DVec2;
use std::hash::Hash;

pub trait PhysicalGraph<K: Copy + Hash + Eq> {
    // CHES
    fn get_all_nodes(&self) -> impl Iterator<Item = K>;

    fn get_position(&self, node_id: K) -> DVec2;
    fn get_velocity(&self, node_id: K) -> DVec2;

    fn get_incomings(&self, node_id: K) -> impl Iterator<Item = K>;
    fn get_outgoings(&self, node_id: K) -> impl Iterator<Item = K>;

    fn get_in_degree(&self, node_id: K) -> u32;
    fn get_out_degree(&self, node_id: K) -> u32;
}
