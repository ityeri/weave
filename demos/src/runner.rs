use chrono::Utc;
use glam::{DVec2, Vec2};
use macroquad::{
    color,
    input::{
        KeyCode, MouseButton, is_key_pressed, is_mouse_button_down, mouse_position, mouse_wheel,
    },
    shapes::{draw_circle, draw_line},
    text::draw_text,
    time::{get_fps, get_frame_time},
    window::{Conf, next_frame, screen_height, screen_width},
};
use scrollrs::Projector;
use std::sync::{Arc, RwLock};

pub struct RenderingShapeSet {
    pub points: Vec<Point>,
    pub lines: Vec<Line>,
}
pub struct Point {
    pub position: Vec2,
    pub radius: f32,
}
impl Point {
    pub fn new(position: Vec2, radius: f32) -> Self {
        Point { position, radius }
    }
}
pub struct Line {
    pub a: Vec2,
    pub b: Vec2,
}
impl Line {
    pub fn new(a: Vec2, b: Vec2) -> Self {
        Line { a, b }
    }
}

pub fn run<Tf, Rf>(window_conf: Conf, mut ticking_func: Tf, mut rendering_func: Rf)
where
    Tf: FnMut() -> () + std::marker::Send + 'static,
    Rf: FnMut() -> RenderingShapeSet + 'static,
{
    macroquad::Window::from_config(
        Conf {
            window_title: window_conf.window_title.clone(),
            window_width: window_conf.window_width,
            window_height: window_conf.window_height,
            high_dpi: window_conf.high_dpi,
            fullscreen: window_conf.fullscreen,
            sample_count: window_conf.sample_count,
            window_resizable: window_conf.window_resizable,
            icon: window_conf.icon.clone(),
            platform: window_conf.platform,
        },
        async move {
            let mut adaptor = scrollrs::ScrollAdaptor::new(0.0, 0.0, None, None, None);
            adaptor = adaptor.set_zoom(0.1);
            let wheel_sensitivity = 0.03;

            let mut last_mouse_pos = mouse_position();

            let update_running_ref = Arc::new(RwLock::new(false));
            let ticking_dt_ref: Arc<RwLock<Option<f32>>> = Arc::new(RwLock::new(None));

            let ticking_thread = {
                let update_running_ref = update_running_ref.clone();
                let ticking_dt_ref = ticking_dt_ref.clone();

                std::thread::spawn(move || {
                    loop {
                        let dt = if *update_running_ref.read().unwrap() {
                            let now = Utc::now();

                            let started_at = now.timestamp() as f64
                                + now.timestamp_subsec_micros() as f64 / 1e+6;

                            ticking_func();

                            let now = Utc::now();
                            let done_at = now.timestamp() as f64
                                + now.timestamp_subsec_micros() as f64 / 1e+6;

                            Some((done_at - started_at) as f32)
                        } else {
                            None
                        };
                        *ticking_dt_ref.write().unwrap() = dt;
                    }
                })
            };
            loop {
                let dt = get_frame_time();
                let screen_width = screen_width();
                let screen_height = screen_height();

                adaptor = adaptor.resized(screen_width as f64, screen_height as f64, None, None);

                // let (mouse_x, mouse_y) = mouse_position();
                // let projected_mouse_pos =
                //     adaptor.unproject(DVec2::new(mouse_x as f64, mouse_y as f64));
                // let projected_mouse_pos =
                //     Vec2::new(projected_mouse_pos.x as f32, projected_mouse_pos.y as f32);

                if is_mouse_button_down(MouseButton::Left) {
                    let current_mouse_pos = mouse_position();

                    let xrel = current_mouse_pos.0 - last_mouse_pos.0;
                    let yrel = current_mouse_pos.1 - last_mouse_pos.1;

                    let camera_delta =
                        adaptor.unproject_scalev(DVec2::new(xrel as f64, yrel as f64));
                    adaptor = adaptor.moved(-camera_delta);
                }

                let (_wheel_x, wheel_y) = mouse_wheel();
                let zoom_amount = wheel_y as f64 * wheel_sensitivity;
                adaptor = adaptor.zoom(zoom_amount);

                adaptor = adaptor.updated(dt as f64);

                let update_running = { *update_running_ref.read().unwrap() };

                if is_key_pressed(KeyCode::Space) {
                    *update_running_ref.write().unwrap() = !update_running;
                }

                let shape_set = rendering_func();
                // let shape_set = RenderingShapeSet {
                //     points: vec![],
                //     lines: vec![],
                // };

                shape_set.lines.iter().for_each(|line| {
                    let projected_a = adaptor.project(line.a.as_dvec2()).as_vec2();
                    let projected_b = adaptor.project(line.b.as_dvec2()).as_vec2();

                    let projected_tickness = adaptor.project_scale(0.5) as f32;

                    draw_line(
                        projected_a.x,
                        projected_a.y,
                        projected_b.x,
                        projected_b.y,
                        projected_tickness,
                        color::GRAY,
                    );
                });
                shape_set.points.iter().for_each(|point| {
                    let projected_pos = adaptor.project(point.position.as_dvec2()).as_vec2();
                    let projected_radius = adaptor.project_scale(point.radius as f64) as f32;

                    draw_circle(
                        projected_pos.x,
                        projected_pos.y,
                        projected_radius,
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
                    &format!("Rendering DT: {:.4}", dt),
                    20.0,
                    40.0,
                    20.0,
                    color::DARKGRAY,
                );
                draw_text(
                    &format!(
                        "Pure ticking DT: {:.40}",
                        { ticking_dt_ref.read().unwrap() }
                            .map(|v| v.to_string())
                            .unwrap_or("Simulation paused".to_owned())
                    ),
                    20.0,
                    60.0,
                    20.0,
                    color::DARKGRAY,
                );

                last_mouse_pos = mouse_position();
                next_frame().await
            }
        },
    )
}
