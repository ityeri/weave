use chrono::Utc;
use clap::Parser;
use glam::{DVec2, Vec2};
use macroquad::prelude::get_fps;
use macroquad::shapes::{draw_circle, draw_line};
use macroquad::{
    color,
    input::{
        KeyCode, MouseButton, is_key_pressed, is_mouse_button_down, mouse_position, mouse_wheel,
    },
    text::draw_text,
    time::get_frame_time,
    window::{Conf, clear_background, next_frame, screen_height, screen_width},
};
use petgraph::visit::EdgeRef;
use petgraph::visit::IntoEdgeReferences;
use petgraph::{
    Directed, Direction,
    stable_graph::{NodeIndex, StableGraph},
};
use rand::Rng;
use scrollrs::Projector;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use walkdir::WalkDir;
use weave::updater::DefaultUpdater;
use weave::{
    PhysicalGraph, PhysicalNode,
    updater::{QuadTreeUpdater, Updater},
};

fn window_conf() -> Conf {
    Conf {
        window_title: "Hello weave!".to_owned(),
        window_width: 500,
        window_height: 500,
        fullscreen: false,
        window_resizable: true,
        ..Default::default()
    }
}

#[derive(Parser, Debug)]
#[command(author, version, long_about = None)]
struct Args {
    path: String,
    #[arg(short, long, default_value_t = 0.02)]
    wheel: f64,
}

struct Body {
    position: Vec2,
    prev_position: Vec2,
}

impl Body {
    fn random(radius: f32) -> Self {
        let mut rng = rand::rng();
        let position = Vec2::new(
            rng.random_range(-radius..radius),
            rng.random_range(-radius..radius),
        );
        Body {
            position: position,
            prev_position: position,
        }
    }
}

fn build_directory_graph<T>(
    root_path: &str,
    node_weight_func: impl Fn(&PathBuf) -> T,
) -> StableGraph<T, (), Directed> {
    let mut graph = StableGraph::<T, (), Directed>::new();
    let mut path_to_node = HashMap::new();

    for entry in WalkDir::new(root_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path().to_path_buf();

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

    graph
}

#[macroquad::main(window_conf)]
async fn main() {
    let args = Args::parse();

    let mut graph = build_directory_graph(&args.path, |_| Body::random(1.0));

    let fixed_dt = 1.0 / 120.0;
    let updater = DefaultUpdater {
        ..DefaultUpdater::default_setting()
    };
    let mut update_running = false;

    let mut adaptor = scrollrs::ScrollAdaptor::new(0.0, 0.0, None, None, None);
    adaptor = adaptor.set_zoom(0.1);

    let mut selected_node_index: Option<NodeIndex> = None;

    let mut last_mouse_pos = mouse_position();
    let wheel_sensitivity = args.wheel;

    loop {
        let dt = get_frame_time();

        let screen_width = screen_width();
        let screen_height = screen_height();

        adaptor = adaptor.resized(screen_width as f64, screen_height as f64, None, None);

        let (mouse_x, mouse_y) = mouse_position();
        let projected_mouse_pos = adaptor.unproject(DVec2::new(mouse_x as f64, mouse_y as f64));
        let projected_mouse_pos =
            Vec2::new(projected_mouse_pos.x as f32, projected_mouse_pos.y as f32);

        if is_mouse_button_down(MouseButton::Left) {
            if let Some(index) = selected_node_index {
                graph[index].prev_position = graph[index].position;
                graph[index].position = projected_mouse_pos;
            } else {
                selected_node_index = graph.node_indices().find(|node_index| {
                    let degrees = graph
                        .edges_directed(*node_index, Direction::Incoming)
                        .count();
                    let diameter = 1.0 + degrees as f32 * 0.1;

                    projected_mouse_pos.distance(graph[*node_index].position) < diameter * 0.5
                });
            }

            if selected_node_index == None {
                let current_mouse_pos = mouse_position();

                let xrel = current_mouse_pos.0 - last_mouse_pos.0;
                let yrel = current_mouse_pos.1 - last_mouse_pos.1;

                let camera_delta = adaptor.unproject_scalev(DVec2::new(xrel as f64, yrel as f64));
                adaptor = adaptor.moved(-camera_delta);
            }
        } else {
            selected_node_index = None;
        }

        let (_wheel_x, wheel_y) = mouse_wheel();
        let zoom_amount = wheel_y as f64 * wheel_sensitivity;
        adaptor = adaptor.zoom(zoom_amount);

        adaptor = adaptor.updated(dt as f64);

        let physical_graph = PhysicalGraph::new(
            graph
                .node_indices()
                .map(|node_index| {
                    let node = PhysicalNode::new(
                        node_index,
                        graph
                            .edges_directed(node_index, Direction::Incoming)
                            .map(|edge_ref| edge_ref.source())
                            .collect::<HashSet<NodeIndex>>(),
                        graph
                            .edges_directed(node_index, Direction::Outgoing)
                            .map(|edge_ref| edge_ref.target())
                            .collect::<HashSet<NodeIndex>>(),
                        graph[node_index].position,
                        graph[node_index].prev_position,
                    );

                    (node_index, node)
                })
                .collect::<HashMap<NodeIndex, PhysicalNode<NodeIndex>>>(),
        );

        if is_key_pressed(KeyCode::Space) {
            update_running = !update_running;
        }

        if is_key_pressed(KeyCode::Period) {
            let updated_graph = updater.update(&physical_graph, fixed_dt);

            for (node_index, node) in updated_graph.nodes {
                graph[node_index].position = node.position;
                graph[node_index].prev_position = node.prev_position;
            }
        }

        let update_dt = if update_running {
            let now = Utc::now();
            let started_at = now.timestamp() as f64 + now.timestamp_subsec_micros() as f64 / 1e+6;

            let updated_graph = updater.update(&physical_graph, fixed_dt);

            let now = Utc::now();
            let done_at = now.timestamp() as f64 + now.timestamp_subsec_micros() as f64 / 1e+6;

            for (node_index, node) in updated_graph.nodes {
                graph[node_index].position = node.position;
                graph[node_index].prev_position = node.prev_position;
            }

            done_at - started_at
        } else {
            -1.0
        };

        clear_background(color::BLACK);

        graph.node_indices().for_each(|node_index| {
            let position = adaptor.project(DVec2::new(
                graph[node_index].position.x as f64,
                graph[node_index].position.y as f64,
            ));
            let degrees = graph
                .edges_directed(node_index, Direction::Incoming)
                .count();
            let diameter = 1.0 + degrees as f64 * 0.1;

            let projcted_radius = if 1.0 < adaptor.project_scale(diameter * 0.5) {
                adaptor.project_scale(diameter * 0.5)
            } else {
                1.0
            };

            draw_circle(
                position.x as f32,
                position.y as f32,
                projcted_radius as f32,
                color::WHITE,
            )
        });

        draw_text(
            &format!("FPS: {}", get_fps()),
            20.0,
            20.0,
            20.0,
            color::DARKGRAY,
        );
        draw_text(
            &format!("Total frame DT: {:.4}", dt),
            20.0,
            40.0,
            20.0,
            color::DARKGRAY,
        );
        draw_text(
            &format!("Pure update DT: {:.4}", update_dt),
            20.0,
            60.0,
            20.0,
            color::DARKGRAY,
        );

        last_mouse_pos = mouse_position();
        next_frame().await
    }
}
