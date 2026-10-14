use std::{
    cmp::min,
    collections::{HashMap, HashSet},
    f32::consts::TAU,
    fs,
    ops::{Deref, DerefMut},
    path::PathBuf,
    sync::{Arc, RwLock},
};

use clap::Parser;
use demos::{
    body::Body,
    into_physical_graph::IntoPhysicalGraph,
    runner::{Line, MouseConf, Point, RenderingShapeSet, Shape, run},
};
use glam::Vec2;
use macroquad::{color::Color, window::Conf};
use petgraph::{Directed, Direction, graph::NodeIndex, stable_graph::StableGraph, visit::EdgeRef};
use rand::Rng;
use walkdir::WalkDir;
use weave::{
    PhysicalGraph, PhysicalNode,
    updater::{QuadTreeUpdater, Updater},
};

#[derive(Parser)]
struct Args {
    path: String,
    #[arg(short, long, default_value_t = 0.3)]
    wheel_sensitivity: f64,
    #[arg(short, long)]
    leaf_file_displaying: bool,
    #[arg(short, long)]
    square_node: bool,
}

fn build_directory_graph<T>(
    root_path: &str,
    mut node_weight_func: impl FnMut(&PathBuf) -> T,
) -> StableGraph<T, (), Directed> {
    let mut graph = StableGraph::<T, (), Directed>::new();
    let mut path_to_node = HashMap::new();

    for entry in WalkDir::new(root_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path().to_path_buf();

        if path.is_dir() {
            let current_node = graph.add_node(node_weight_func(&path.clone()));
            path_to_node.insert(path.clone(), current_node);

            if let Some(parent_path) = path.parent() {
                if let Some(&parent_node) = path_to_node.get(parent_path) {
                    if parent_path != path {
                        graph.add_edge(current_node, parent_node, ());
                    }
                }
            }
        }
    }

    graph
}

struct DirectoryNode {
    position: Vec2,
    prev_position: Vec2,
    path: String,
    children: usize,
}
struct DirectoryGraph {
    graph: StableGraph<DirectoryNode, (), Directed>,
}
impl Deref for DirectoryGraph {
    type Target = StableGraph<DirectoryNode, (), Directed>;
    fn deref(&self) -> &Self::Target {
        &self.graph
    }
}
impl DerefMut for DirectoryGraph {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.graph
    }
}

impl IntoPhysicalGraph<NodeIndex> for DirectoryGraph {
    fn into_physical_graph(&self) -> weave::PhysicalGraph<NodeIndex> {
        PhysicalGraph::new(
            self.node_indices()
                .map(|node_index| {
                    let node = PhysicalNode::new(
                        node_index,
                        self.edges_directed(node_index, Direction::Incoming)
                            .map(|edge_ref| edge_ref.source())
                            .collect::<HashSet<NodeIndex>>(),
                        self.edges_directed(node_index, Direction::Outgoing)
                            .map(|edge_ref| edge_ref.target())
                            .collect::<HashSet<NodeIndex>>(),
                        self[node_index].position,
                        self[node_index].prev_position,
                    );

                    (node_index, node)
                })
                .collect::<HashMap<NodeIndex, PhysicalNode<NodeIndex>>>(),
        )
    }
}

fn get_circle_positions(circle_count: u32, radius: f32, ring_number: f32) -> Vec<Vec2> {
    if circle_count == 0 {
        vec![]
    } else {
        let diameter = radius * 2.0;
        let ring_radius = diameter * ring_number as f32;
        let perimeter_length = ring_radius * TAU;
        let circles_for_current_ring = if ring_number == 0.0 {
            1
        } else {
            min(circle_count, (perimeter_length / diameter).round() as u32)
        };

        (0..circles_for_current_ring)
            .map(|number| {
                let angle = TAU / circles_for_current_ring as f32 * number as f32;
                Vec2::from_angle(angle) * ring_radius
            })
            .chain(get_circle_positions(
                circle_count - circles_for_current_ring,
                radius,
                ring_number + 1.0,
            ))
            .collect()
    }
}

fn main() {
    let args = Args::parse();
    let shape = if args.square_node {
        Shape::Square
    } else {
        Shape::Circle
    };

    let fixed_dt = 1.0 / 120.0;
    let updater = QuadTreeUpdater {
        ..QuadTreeUpdater::default_setting()
    };

    let random_radius = 1000.0;
    let graph_ref = {
        let mut rng = rand::rng();

        Arc::new(RwLock::new(DirectoryGraph {
            graph: build_directory_graph(&args.path, |path| {
                let position = Vec2::new(
                    rng.random_range(-random_radius..random_radius),
                    rng.random_range(-random_radius..random_radius),
                );

                DirectoryNode {
                    position: position,
                    prev_position: position,
                    path: path.to_str().unwrap().to_owned(),
                    children: if let Ok(dir) = fs::read_dir(path) {
                        dir.filter_map(|entry| entry.ok())
                            .filter(|entry| entry.file_type().map(|t| t.is_file()).unwrap_or(false))
                            .count()
                    } else {
                        0
                    },
                }
            }),
        }))
    };

    run(
        Conf {
            window_title: format!("File tree at \"{}\"", args.path).to_owned(),
            window_resizable: true,
            high_dpi: true,
            ..Default::default()
        },
        MouseConf {
            wheel_sensitivity: args.wheel_sensitivity,
        },
        {
            let graph_ref = graph_ref.clone();
            move || {
                let updated_graph = {
                    let graph = graph_ref.read().unwrap();
                    updater.update(&graph.into_physical_graph(), fixed_dt)
                };

                {
                    let mut graph = graph_ref.write().unwrap();
                    updated_graph.nodes.iter().for_each(|(&key, node)| {
                        graph[key].position = node.position;
                        graph[key].prev_position = node.prev_position;
                    });
                }
            }
        },
        {
            let graph_ref = graph_ref.clone();

            move || {
                let graph = graph_ref.read().unwrap();

                RenderingShapeSet {
                    points: graph
                        .node_indices()
                        .flat_map(|index| {
                            std::iter::once(Point::new(
                                graph[index].position,
                                1.0,
                                Color::new(1.0, 1.0, 1.0, 0.8),
                                shape,
                            ))
                            .chain(if args.leaf_file_displaying {
                                get_circle_positions(graph[index].children as u32, 0.5, 1.5)
                                    .iter()
                                    .map(|&v| {
                                        Point::new(
                                            graph[index].position + v,
                                            0.5,
                                            Color::new(1.0, 1.0, 1.0, 0.4),
                                            shape,
                                        )
                                    })
                                    .collect::<Vec<Point>>()
                            } else {
                                vec![]
                            })
                        })
                        .collect::<Vec<Point>>(),
                    lines: graph
                        .edge_indices()
                        .filter_map(|index| graph.edge_endpoints(index))
                        .map(|(source_index, target_index)| {
                            Line::new(
                                graph[source_index].position,
                                graph[target_index].position,
                                Color::new(0.6, 0.6, 0.6, 0.2),
                            )
                        })
                        .collect::<Vec<Line>>(),
                }
            }
        },
    );
}
