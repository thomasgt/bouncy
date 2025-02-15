use egui::{emath::TSTransform, Pos2, Vec2};
use ringbuffer::RingBuffer;

use crate::{
    ball::{self, Ball},
    collision::{self, Collision},
    drawable::Drawable,
    rotating::{RotatingBody, RotatingInput},
    shape::Shape,
};

pub struct App {
    gravity: f32,
    target_frame_rate: f32,
    simulation_rate: f32,
    n_sides: usize,
    _n_holes: usize,
    previous_frame_times: ringbuffer::AllocRingBuffer<web_time::Instant>,
    rotating_input: RotatingInput,
    brake_work: f32,
    boost_work: f32,
    motor_work: f32,
    rotating_body: RotatingBody,
    ball: Ball,
}

impl Default for App {
    fn default() -> Self {
        Self {
            gravity: 9.81,
            target_frame_rate: 60.,
            simulation_rate: 1024.,
            n_sides: 6,
            _n_holes: 0,
            previous_frame_times: ringbuffer::AllocRingBuffer::new(100),
            rotating_input: RotatingInput::default(),
            brake_work: 0.0,
            boost_work: 0.0,
            motor_work: 0.0,
            rotating_body: RotatingBody::default(),
            ball: Ball::default(),
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        Default::default()
    }

    fn reset(&mut self) {
        let shape = if self.n_sides >= 3 {
            Shape::regular_polygon(self.n_sides, 1., Pos2::new(0.0, 0.0))
        } else {
            Shape::funky_polygon()
        };

        self.rotating_body = RotatingBody {
            shape: shape,
            ..Default::default()
        };
        self.ball = Ball::default();

        self.brake_work = 0.0;
        self.boost_work = 0.0;
        self.motor_work = 0.0;
    }

    fn update_frame_time(&mut self) -> web_time::Duration {
        let now = web_time::Instant::now();

        let dt = if let Some(prev) = self.previous_frame_times.back() {
            now - *prev
        } else {
            web_time::Duration::from_secs_f32(1.0 / self.target_frame_rate)
        };

        self.previous_frame_times.push(now);
        dt
    }

    fn compute_fps(&self) -> f32 {
        if self.previous_frame_times.len() < 2 {
            return self.target_frame_rate;
        }

        let first = self.previous_frame_times.front().unwrap();
        let last = self.previous_frame_times.back().unwrap();
        let elapsed = *last - *first;
        let elapsed_secs = elapsed.as_secs_f32();

        (self.previous_frame_times.len() as f32 - 1.0) / elapsed_secs
    }

    fn detect_collisions(&self) -> Vec<Collision> {
        let ball = &self.ball;
        let rotating_body = &self.rotating_body;

        let shape = rotating_body.shape_with_rotation_applied();

        let line_segments = shape.all_segments();

        // Determine which, if any, line segments the ball is colliding with
        line_segments
            .into_iter()
            .filter_map(|(p1, p2)| {
                let v = p2 - p1;
                let n1 = egui::vec2(-v.y, v.x).normalized();

                let d = (ball.center - p1).dot(n1);

                if d.abs() < ball.radius {
                    let p = ball.center - d * n1;
                    let t = (p - p1).dot(v) / v.length_sq();

                    if t < 0.0 {
                        // Possible collision with edge at p1
                        let n2 = ball.center - p1;
                        if n2.length() > ball.radius {
                            return None;
                        }

                        let n2 = if n1.dot(n2) > 0. { n2 } else { -n2 };
                        Some(Collision::new(p1, n2.normalized()))
                    } else if t > 1.0 {
                        // Possible collision with edge at p2
                        let n2 = ball.center - p2;
                        if n2.length() > ball.radius {
                            return None;
                        }
                        let n2 = if n1.dot(n2) > 0. { n2 } else { -n2 };
                        Some(Collision::new(p2, n2.normalized()))
                    } else {
                        Some(Collision::new(p, n1))
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    fn handle_collisions(&mut self, ball_previous_position: Pos2) {
        let collisions = self.detect_collisions();

        if collisions.is_empty() {
            return;
        }

        // Pick the collision that is closest to the ball's previous position
        let closest_collision = collisions
            .iter()
            .min_by(|a, b| {
                let dist_a = (a.point - ball_previous_position).length();
                let dist_b = (b.point - ball_previous_position).length();
                dist_a.partial_cmp(&dist_b).unwrap()
            })
            .unwrap();

        self.ball.velocity = self.ball.velocity
            - 2.0 * self.ball.velocity.dot(closest_collision.normal) * closest_collision.normal;

        self.ball.center = closest_collision.point + closest_collision.normal * self.ball.radius;

        self.rotating_body
            .record_collisions(vec![closest_collision.clone()])
    }

    fn update_physics(&mut self, dt: f32) {
        let (brake_work, boost_work, motor_work) =
            self.rotating_body.update(self.rotating_input, dt);
        self.brake_work += brake_work;
        self.boost_work += boost_work;
        self.motor_work += motor_work;
        let ball_previous_position = self.ball.center;
        self.ball.update(dt, self.gravity);
        self.handle_collisions(ball_previous_position);
    }
}

impl eframe::App for App {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let dt = self.update_frame_time().as_secs_f32();
        let fps = self.compute_fps();

        let n_sub_steps = (dt * self.simulation_rate).round();
        let dt_sim = 1. / self.simulation_rate;

        for _ in 0..n_sub_steps as usize {
            self.update_physics(dt_sim);
        }

        // Schedule a repaint at the next frame
        ctx.request_repaint_after(web_time::Duration::from_secs_f32(
            1.0 / self.target_frame_rate,
        ));

        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.add(egui::github_link_file!(
                    "https://github.com/thomasgt/bouncy/blob/main/",
                    "Source code."
                ));
                ui.label(format!("FPS: {:.1}", fps));
                egui::warn_if_debug_build(ui);
            });
        });

        egui::TopBottomPanel::bottom("controls")
            .show_separator_line(false)
            .show(ctx, |ui| {
                ui.columns(4, |ui| {
                    let brake_button =
                        ui[1].add_sized(egui::vec2(50.0, 50.0), egui::Button::new("Brake"));
                    self.rotating_input.brake = brake_button.is_pointer_button_down_on();

                    let boost_button =
                        ui[2].add_sized(egui::vec2(50.0, 50.0), egui::Button::new("Boost"));
                    self.rotating_input.boost = boost_button.is_pointer_button_down_on();
                });

                ui.label(format!("Brake work: {:.2} J", self.brake_work));
                ui.label(format!("Boost work: {:.2} J", self.boost_work));
                ui.label(format!("Motor work: {:.2} J", self.motor_work));
                ui.label(format!(
                    "Angular velocity: {:.5} rad/s",
                    self.rotating_body.angular_velocity
                ));
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("Bouncy");

            ui.collapsing("Shape Settings", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Sides");
                    if ui
                        .add(egui::Slider::new(&mut self.n_sides, 2..=12).text("ea"))
                        .changed()
                    {
                        self.reset();
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Moment of inertia");
                    ui.add(
                        egui::Slider::new(&mut self.rotating_body.moment_of_inertia, 0.1..=100.)
                            .text("(kg m²)"),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Friction coefficient");
                    ui.add(
                        egui::Slider::new(&mut self.rotating_body.friction_coefficient, 0.01..=1.0)
                            .text("(N m s)"),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Braking torque");
                    ui.add(
                        egui::Slider::new(&mut self.rotating_input.braking_torque, 0.0..=10.0)
                            .text("(N m)"),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Motor torque");
                    ui.add(
                        egui::Slider::new(&mut self.rotating_input.motor_torque, 0.0..=10.0)
                            .text("(N m)"),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Boost torque");
                    ui.add(
                        egui::Slider::new(&mut self.rotating_input.boost_torque, 0.0..=10.0)
                            .text("(N m)"),
                    );
                });

                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.rotating_input.motor, "Motor");
                    ui.checkbox(&mut self.rotating_input.brake, "Brake");
                    ui.checkbox(&mut self.rotating_input.boost, "Boost");
                });
            });

            ui.collapsing("World Settings", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Gravity");
                    ui.add(egui::Slider::new(&mut self.gravity, 0.0..=20.0).text("(m/s²)"));
                })
            });

            if ui
                .button("Reset")
                .on_hover_text("Reset the simulation")
                .clicked()
            {
                self.reset();
            }

            ui.separator();

            let available_size = ui.available_size();

            // Allocate a painting region that takes up the remaining space
            let (response, painter) = ui.allocate_painter(available_size, egui::Sense::hover());

            let canvas_rect = response.rect;

            // Define scaling factor so hexagon takes up 80% of the available space
            let max_extent = self
                .rotating_body
                .shape
                .max_extent(self.rotating_body.center_of_rotation);

            let left_top_radius = max_extent.min.to_vec2().length();
            let bottom_right_radius = max_extent.max.to_vec2().length();
            let radius = left_top_radius.max(bottom_right_radius);

            let scale = 0.8 * canvas_rect.size().min_elem() / (2. * radius);

            let transform = TSTransform {
                scaling: scale,
                translation: canvas_rect.center().to_vec2(),
            };

            self.rotating_body.draw(ctx, &painter, transform);
            self.ball.draw(ctx, &painter, transform);
        });
    }
}
