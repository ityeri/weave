use std::sync::{Arc, RwLock};

use clap::Parser;
use demos::{
    body::Body,
    into_physical_graph::IntoPhysicalGraph,
    runner::{Line, MouseConf, Point, RenderingShapeSet, Shape, run},
};
use macroquad::{color, window::Conf};
use petgraph::{Directed, stable_graph::StableGraph};
use weave::updater::{QuadTreeUpdater, Updater};

#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value_t = 0.3)]
    wheel_sensitivity: f64,
    #[arg(short, long, default_value_t = 1200)]
    nodes: u32,
}

fn main() {
    let args = Args::parse();

    let mut graph: StableGraph<Body, (), Directed> = StableGraph::new();
    let random_radius = 10000.0;
    let center_node1 = graph.add_node(Body::random_range(random_radius));
    let center_node2 = graph.add_node(Body::random_range(random_radius));

    for _ in 0..args.nodes {
        let sub_node = graph.add_node(Body::random_range(random_radius));
        graph.add_edge(sub_node, center_node1, ());
    }

    for _ in 0..args.nodes {
        let sub_node = graph.add_node(Body::random_range(random_radius));
        graph.add_edge(sub_node, center_node2, ());
    }

    graph.add_edge(center_node1, center_node2, ());
    graph.add_edge(center_node2, center_node1, ());

    let fixed_dt = 1.0 / 60.0;
    let updater = QuadTreeUpdater {
        neighbor_edge_elasticity: 0.7,
        non_neighbor_repulsive_force: 1000.0,
        neighbor_radius: 0.2,
        ..QuadTreeUpdater::default_setting()
    };

    let graph_ref = Arc::new(RwLock::new(graph));

    run(
        Conf {
            window_title: format!("Hello weave!").to_owned(),
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
                    updated_graph.nodes.values().for_each(|node| {
                        graph[node.key].position = node.position;
                        graph[node.key].prev_position = node.prev_position;
                    });
                };
            }
        },
        {
            let graph_ref = graph_ref.clone();
            move || {
                let graph = graph_ref.read().unwrap();
                RenderingShapeSet {
                    points: graph
                        .node_indices()
                        .map(|index| {
                            Point::new(graph[index].position, 1.0, color::WHITE, Shape::Circle)
                        })
                        .collect(),
                    lines: graph
                        .edge_indices()
                        .filter_map(|index| graph.edge_endpoints(index))
                        .map(|(source_index, target_index)| {
                            Line::new(
                                graph[source_index].position,
                                graph[target_index].position,
                                color::GRAY,
                            )
                        })
                        .collect(),
                }
            }
        },
    )
}
