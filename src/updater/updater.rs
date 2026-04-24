use crate::{NodeKey, PhysicalGraph};

pub trait Updater {
    fn update<K: NodeKey>(&self, graph: &PhysicalGraph<K>, dt: f32) -> PhysicalGraph<K>;
}
