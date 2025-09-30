use clock::Clock;
use glam::DVec2;
use petgraph::{
    Directed, Direction, Undirected,
    stable_graph::{NodeIndex, StableGraph},
};
use rand::Rng;
use scrollrs::{self, Projector};
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Keycode,
    mouse::MouseWheelDirection,
    pixels::Color,
    rect::Rect,
};
use weave::{
    PhysicalGraph,
    stored_graph::{StoredGraph, StoredNode},
    updater::{MultiThreadUpdater, Updater},
};

struct PhsicalGraphData {
    position: DVec2,
    velocity: DVec2,
}
impl PhsicalGraphData {
    fn zero() -> Self {
        Self {
            position: DVec2::new(0.0, 0.0),
            velocity: DVec2::new(0.0, 0.0),
        }
    }

    fn random(radius: f64) -> Self {
        let mut rng = rand::rng();
        Self {
            position: DVec2::new(
                rng.random_range(-radius..radius),
                rng.random_range(-radius..radius),
            ),
            velocity: DVec2::new(
                rng.random_range(-radius..radius),
                rng.random_range(-radius..radius),
            ),
        }
    }
}

struct GraphWrapper<'a> {
    graph: &'a mut StableGraph<PhsicalGraphData, (), Directed>,
}
impl<'a> PhysicalGraph<NodeIndex> for GraphWrapper<'a> {
    fn get_all_nodes(&self) -> impl Iterator<Item = NodeIndex> {
        self.graph.node_indices()
    }
    fn get_position(&self, node_id: NodeIndex) -> DVec2 {
        self.graph[node_id].position
    }
    fn get_velocity(&self, node_id: NodeIndex) -> DVec2 {
        self.graph[node_id].velocity
    }

    fn get_incomings(&self, node_id: NodeIndex) -> impl Iterator<Item = NodeIndex> {
        self.graph.neighbors_directed(node_id, Direction::Incoming)
    }
    fn get_outgoings(&self, node_id: NodeIndex) -> impl Iterator<Item = NodeIndex> {
        self.graph.neighbors_directed(node_id, Direction::Outgoing)
    }

    fn get_in_degree(&self, node_id: NodeIndex) -> u32 {
        self.graph
            .neighbors_directed(node_id, Direction::Incoming)
            .count() as u32
    }

    fn get_out_degree(&self, node_id: NodeIndex) -> u32 {
        self.graph
            .neighbors_directed(node_id, Direction::Outgoing)
            .count() as u32
    }
}

fn main() {
    let mut screen_width: u32 = 500;
    let mut screen_height: u32 = 500;

    let fps: f64 = 100.0;
    let mut clock = Clock::new(1.0 / fps);
    let mut adaptor =
        scrollrs::ScrollAdaptor::new(screen_width as f64, screen_height as f64, None, None, None);
    let wheel_sensitivity = 0.1;

    let mut graph: StableGraph<PhsicalGraphData, (), Directed> = StableGraph::new();

    let graph_updater = MultiThreadUpdater::default_setting();

    let random_radius: f64 = 0.00001;
    let center_node1 = graph.add_node(PhsicalGraphData::random(random_radius));
    let center_node2 = graph.add_node(PhsicalGraphData::random(random_radius));

    graph.add_edge(center_node1, center_node2, ()); // graph.add_edge(center_node2, center_node1, ());

    for i in 0..3000 {
        let sub_node = graph.add_node(PhsicalGraphData::random(random_radius));
        graph.add_edge(sub_node, center_node1, ());
        // graph.add_edge(center_node1, sub_node, ());
    }

    for i in 0..3000 {
        let sub_node = graph.add_node(PhsicalGraphData::random(random_radius));
        graph.add_edge(sub_node, center_node2, ());
        // graph.add_edge(center_node2, sub_node, ());
    }

    let mut graph_running = false;
    let mut average_tick_per_time = 1.0;

    let sdl = sdl2::init().unwrap();
    let mut event_pump = sdl.event_pump().unwrap();
    let video = sdl.video().unwrap();
    let window = video
        .window("wasans", screen_width, screen_height)
        .position_centered()
        .resizable()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();

    'running: loop {
        let dt = clock.last_dt;

        let mouse_state = event_pump.mouse_state();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::Window { win_event, .. } => {
                    if let WindowEvent::Resized(width, height) = win_event {
                        screen_width = width as u32;
                        screen_height = height as u32;
                        adaptor =
                            adaptor.resized(screen_width as f64, screen_height as f64, None, None);
                    }
                }
                Event::MouseMotion {
                    x, y, xrel, yrel, ..
                } => {
                    if mouse_state.left() {
                        let camera_delta =
                            adaptor.unproject_scalev(DVec2::new(xrel as f64, yrel as f64));
                        adaptor = adaptor.moved(-camera_delta);
                    }
                }
                Event::MouseWheel {
                    x, y, direction, ..
                } => {
                    let amount = match direction {
                        MouseWheelDirection::Normal => y,
                        MouseWheelDirection::Flipped => -y,
                        _ => 0,
                    };
                    adaptor = adaptor.zoom(amount as f64 * wheel_sensitivity);
                }
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    if key == Keycode::Space {
                        graph_running = !graph_running
                    }
                }

                _ => {}
            }
        }

        if graph_running {
            let update_clock = Clock::new(1.0);

            let update_clock = update_clock.tick(100000000000000000000000000000.0);
            let updated_graph =
                graph_updater.update(GraphWrapper { graph: &mut graph }, 1.0 / 100.0);
            let updated_clock = update_clock.tick(100000000000000000000000000000.0);

            average_tick_per_time += (updated_clock.last_dt - average_tick_per_time) / 10.0;
            println!(
                "average tick per time ms: {}",
                (average_tick_per_time * 1000.0) as i32
            );

            for index in updated_graph.get_all_nodes() {
                graph[index].position = updated_graph.get_position(index);
            }
        }

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        for index in graph.node_indices() {
            let position = graph[index].position;
            let node_radius = (graph.neighbors_directed(index, Direction::Incoming).count() as f64
                * 0.003
                + 0.05)
                .sqrt();

            let projected_rect_position =
                adaptor.project(position + DVec2::new(-node_radius, node_radius));
            let projected_rect_size = adaptor.project_scale(node_radius * 2.0);

            let rect = Rect::new(
                projected_rect_position.x as i32,
                projected_rect_position.y as i32,
                projected_rect_size as u32,
                projected_rect_size as u32,
            );

            canvas.set_draw_color(Color::RGB(255, 0, 255));
            canvas.fill_rect(rect).unwrap();
        }

        adaptor = adaptor.updated(dt);

        canvas.present();
        clock = clock.tick(fps);
    }
}
