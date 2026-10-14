use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use clap::Parser;
use demos::{
    body::Body,
    graph_builder::PhysicalGraphBuilder,
    runner::{Line, Point, RenderingShapeSet, run},
};
use macroquad::window::Conf;
use petgraph::{Directed, stable_graph::StableGraph};
use walkdir::WalkDir;
use weave::updater::{QuadTreeUpdater, Updater};

#[derive(Parser, Debug)]
#[command(author, version, long_about = None)]
struct Args {
    path: String,
    #[arg(short, long, default_value_t = 0.02)]
    wheel: f64,
}

fn build_directory_graph<T>(
    root_path: &str,
    node_weight_func: impl Fn(&PathBuf) -> T,
) -> StableGraph<T, (), Directed> {
    let mut graph = StableGraph::<T, (), Directed>::new();
    let mut path_to_node = HashMap::new();

    for entry in WalkDir::new(root_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path().to_path_buf();

        if path.is_dir() {
            let current_node = graph.add_node(node_weight_func(&path));
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

fn main() {
    let args = Args::parse();

    let fixed_dt = 1.0 / 120.0;
    let updater = QuadTreeUpdater {
        neighbor_edge_elasticity: 40.0,
        ..QuadTreeUpdater::default_setting()
    };

    let graph_ref = Arc::new(RwLock::new(build_directory_graph(&args.path, |_| {
        Body::random_range(1.0)
    })));

    run(
        Conf {
            window_title: format!("File tree at \"{}\"", args.path).to_owned(),
            window_resizable: true,
            high_dpi: true,
            ..Default::default()
        },
        {
            let graph_ref = graph_ref.clone();
            move || {
                let updated_graph = {
                    let graph = graph_ref.read().unwrap();
                    updater.update(&graph.to_physical_graph(), fixed_dt)
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
                        .map(|index| Point::new(graph[index].position, 1.0))
                        .collect::<Vec<Point>>(),
                    lines: graph
                        .edge_indices()
                        .filter_map(|index| graph.edge_endpoints(index))
                        .map(|(source_index, target_index)| {
                            Line::new(graph[source_index].position, graph[target_index].position)
                        })
                        .collect::<Vec<Line>>(),
                }
            }
        },
    );
}
