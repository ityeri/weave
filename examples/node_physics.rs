use std::collections::{HashMap, HashSet};

use glam::{DVec2, Vec2};
use macroquad::{
    color,
    input::{
        KeyCode, MouseButton, is_key_down, is_key_pressed, is_mouse_button_down, mouse_position,
        mouse_wheel,
    },
    shapes::draw_rectangle,
    text::draw_text,
    time::{get_fps, get_frame_time},
    window::{Conf, clear_background, next_frame, screen_height, screen_width},
};
use petgraph::{
    Directed, Direction,
    data::{Element, FromElements},
    stable_graph::{NodeIndex, StableGraph},
    visit::EdgeRef,
};
use rand::Rng;
use scrollrs::Projector;
use weave::{
    NodeKey, PhysicalGraph, PhysicalNode,
    updater::{DefaultUpdater, Updater},
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

#[derive(Hash, PartialEq, Eq)]
struct HasheableEdge {
    source: NodeIndex,
    target: NodeIndex,
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut graph: StableGraph<Body, (), Directed> = StableGraph::new();

    let random_radius = 100.0;

    let center_node1 = graph.add_node(Body::random(random_radius));
    let center_node2 = graph.add_node(Body::random(random_radius));

    for _ in 0..100 {
        let sub_node = graph.add_node(Body::random(random_radius));
        graph.add_edge(sub_node, center_node1, ());
    }

    for _ in 0..100 {
        let sub_node = graph.add_node(Body::random(random_radius));
        graph.add_edge(sub_node, center_node2, ());
    }

    graph.add_edge(center_node1, center_node2, ());
    graph.add_edge(center_node2, center_node1, ());

    let fixed_dt = 1.0 / 100.0;
    let updater = DefaultUpdater::default_setting();
    let mut update_running = false;

    let mut adaptor = scrollrs::ScrollAdaptor::new(0.0, 0.0, None, None, None);
    adaptor = adaptor.set_zoom(0.1);

    let mut last_mouse_pos = mouse_position();
    let wheel_sensitivity = 0.01;

    loop {
        let dt = get_frame_time();

        let screen_width = screen_width();
        let screen_height = screen_height();

        adaptor = adaptor.resized(screen_width as f64, screen_height as f64, None, None);

        if is_mouse_button_down(MouseButton::Left) {
            let current_mouse_pos = mouse_position();

            let xrel = current_mouse_pos.0 - last_mouse_pos.0;
            let yrel = current_mouse_pos.1 - last_mouse_pos.1;

            let camera_delta = adaptor.unproject_scalev(DVec2::new(xrel as f64, yrel as f64));
            adaptor = adaptor.moved(-camera_delta);
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

        if update_running {
            let updated_graph = updater.update(physical_graph, fixed_dt);

            for (node_index, node) in updated_graph.nodes {
                graph[node_index].position = node.position;
                graph[node_index].prev_position = node.prev_position;
            }
        }

        clear_background(color::BLACK);

        graph.node_indices().for_each(|node_index| {
            let position = adaptor.project(DVec2::new(
                graph[node_index].position.x as f64,
                graph[node_index].position.y as f64,
            ));
            let degrees = graph
                .edges_directed(node_index, Direction::Incoming)
                .count() as f64;
            let radius = 1.0 + degrees * 0.03;
            let projcted_radius = adaptor.project_scale(radius) as f32;

            draw_rectangle(
                position.x as f32 - projcted_radius,
                position.y as f32 - projcted_radius,
                projcted_radius * 2.0,
                projcted_radius * 2.0,
                color::WHITE,
            );
        });

        draw_text(
            &format!("FPS: {}", get_fps()),
            20.0,
            20.0,
            20.0,
            color::DARKGRAY,
        );
        draw_text(&format!("DT: {:.4}", dt), 20.0, 40.0, 20.0, color::DARKGRAY);

        last_mouse_pos = mouse_position();
        next_frame().await
    }
}
