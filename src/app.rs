use egui::{emath::TSTransform, Color32, Pos2, Rect, Vec2};
use ringbuffer::RingBuffer;

pub struct Collision {
    pub point: Pos2,
    pub normal: Vec2,
    pub time: web_time::Instant,
}

impl Collision {
    pub fn new(point: Pos2, normal: Vec2) -> Self {
        Self {
            point,
            normal,
            time: web_time::Instant::now(),
        }
    }

    pub fn rotate(&self, angle: f32, center_of_rotation: Pos2) -> Self {
        let point = {
            let p = self.point - center_of_rotation;
            let p = egui::vec2(
                p.x * angle.cos() - p.y * angle.sin(),
                p.x * angle.sin() + p.y * angle.cos(),
            );
            center_of_rotation + p
        };

        let normal = {
            let n = self.normal;
            let n = egui::vec2(
                n.x * angle.cos() - n.y * angle.sin(),
                n.x * angle.sin() + n.y * angle.cos(),
            );
            n
        };

        Self {
            point,
            normal,
            time: self.time,
        }
    }

    pub fn draw(&self, ctx: &egui::Context, painter: &egui::Painter, transform: TSTransform) {
        let age = (web_time::Instant::now() - self.time).as_secs_f32();
        let size = 10. * age;
        let opacity = 1.0 - age / 2.;

        if size <= 0.0 || opacity <= 0.0 {
            return;
        }

        let point = transform.mul_pos(self.point);
        let warn_colour = ctx.style().visuals.warn_fg_color;

        let fill_colour = Color32::from_rgba_unmultiplied(
            warn_colour.r(),
            warn_colour.g(),
            warn_colour.b(),
            (255. * opacity) as u8,
        );

        painter.add(egui::Shape::circle_filled(point, size, fill_colour));
    }
}

pub type Segment = (Pos2, Pos2);
pub type Line = Vec<Pos2>;

pub struct Shape {
    pub lines: Vec<Line>,
}

impl Shape {
    pub fn regular_polygon(num_sides: usize, radius: f32, center: Pos2) -> Self {
        let angle = 2. * std::f32::consts::PI / num_sides as f32;
        let lines = (0..num_sides + 1)
            .map(|i| {
                let angle = i as f32 * angle;
                center + radius * egui::vec2(angle.cos(), angle.sin())
            })
            .collect();

        Self { lines: vec![lines] }
    }

    pub fn all_segments(&self) -> Vec<Segment> {
        self.lines
            .iter()
            .flat_map(|line| line.windows(2).map(|w| (w[0], w[1])))
            .collect()
    }

    pub fn all_points(&self) -> Vec<Pos2> {
        self.lines
            .iter()
            .flat_map(|line| line.iter().copied())
            .collect()
    }

    pub fn max_extent(&self, center_of_rotation: Pos2) -> Rect {
        let radiuses = self
            .all_points()
            .iter()
            .map(|p| (*p - center_of_rotation).length())
            .collect::<Vec<f32>>();

        let max_radius = radiuses.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

        Rect::from_center_size(center_of_rotation, Vec2::splat(2. * max_radius))
    }

    pub fn draw(&self, ctx: &egui::Context, painter: &egui::Painter, transform: TSTransform) {
        let lines = self
            .lines
            .iter()
            .map(|line| {
                line.iter()
                    .map(|p| transform.mul_pos(*p))
                    .collect::<Vec<Pos2>>()
            })
            .collect::<Vec<Line>>();

        let stroke = egui::Stroke::new(1.0, ctx.style().visuals.text_color());
        for line in lines {
            painter.add(egui::Shape::line(line, stroke));
        }
    }

    pub fn rotate(&self, angle: f32, center_of_rotation: Pos2) -> Self {
        let lines = self
            .lines
            .iter()
            .map(|line| {
                line.iter()
                    .map(|p| {
                        let p = *p - center_of_rotation;
                        let p = egui::vec2(
                            p.x * angle.cos() - p.y * angle.sin(),
                            p.x * angle.sin() + p.y * angle.cos(),
                        );
                        center_of_rotation + p
                    })
                    .collect()
            })
            .collect();

        Self { lines }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RotatingInput {
    pub braking_torque: f32,
    pub motor_torque: f32,
    pub boost_torque: f32,
    pub brake: bool,
    pub motor: bool,
    pub boost: bool,
}

impl Default for RotatingInput {
    fn default() -> Self {
        Self {
            braking_torque: 2.0,
            motor_torque: 1.0,
            boost_torque: 1.0,
            brake: false,
            motor: true,
            boost: false,
        }
    }
}

pub struct RotatingBody {
    shape: Shape,
    pub center_of_rotation: egui::Pos2,
    pub angle: f32,
    pub angular_velocity: f32,
    pub moment_of_inertia: f32,
    pub friction_coefficient: f32,
    collisions_with_angles: ringbuffer::AllocRingBuffer<(Collision, f32)>,
}

impl Default for RotatingBody {
    fn default() -> Self {
        Self {
            shape: Shape::regular_polygon(6, 1., Pos2::new(0.0, 0.0)),
            center_of_rotation: egui::Pos2::new(0.0, 0.0),
            angle: 0.,
            angular_velocity: 1.0,
            moment_of_inertia: 1.0,
            friction_coefficient: 0.7,
            collisions_with_angles: ringbuffer::AllocRingBuffer::new(100),
        }
    }
}

impl RotatingBody {
    pub fn shape_with_rotation_applied(&self) -> Shape {
        self.shape.rotate(self.angle, self.center_of_rotation)
    }

    pub fn record_collisions(&mut self, collisions: Vec<Collision>) {
        for collision in collisions {
            self.collisions_with_angles.push((collision, self.angle));
        }
    }

    pub fn update(&mut self, input: RotatingInput, dt: f32) {
        let friction_torque = -self.friction_coefficient * self.angular_velocity;
        let brake_torque = if input.brake {
            -input.braking_torque * self.angular_velocity.signum()
        } else {
            0.0
        };

        let motor_torque = if input.motor { input.motor_torque } else { 0.0 };
        let boost_torque = if input.boost { input.boost_torque } else { 0.0 };

        let torque = friction_torque + brake_torque + motor_torque + boost_torque;
        let angular_acceleration = torque / self.moment_of_inertia;

        self.angular_velocity += angular_acceleration * dt;
        self.angle += self.angular_velocity * dt;
    }

    pub fn draw(&self, ctx: &egui::Context, painter: &egui::Painter, transform: TSTransform) {
        let shape = self.shape.rotate(self.angle, self.center_of_rotation);
        shape.draw(ctx, painter, transform);

        let collisions = self
            .collisions_with_angles
            .iter()
            .map(|(collision, angle)| {
                collision.rotate(self.angle - *angle, self.center_of_rotation)
            })
            .collect::<Vec<Collision>>();
        for collision in collisions {
            collision.draw(ctx, painter, transform);
        }
    }
}

pub struct Ball {
    pub center: egui::Pos2,
    pub radius: f32,
    pub velocity: egui::Vec2,
}

impl Default for Ball {
    fn default() -> Self {
        Self {
            center: egui::Pos2::new(0.0, 0.0),
            radius: 0.05,
            velocity: egui::Vec2::new(0.0, 0.0),
        }
    }
}

impl Ball {
    pub fn reset(&mut self) {
        self.center = egui::Pos2::new(0.0, 0.0);
        self.velocity = egui::Vec2::new(0.0, 0.0);
    }

    pub fn update(&mut self, dt: f32, gravity: f32) {
        self.velocity.y += gravity * dt;
        self.center += self.velocity * dt;
    }

    pub fn draw(&self, ctx: &egui::Context, painter: &egui::Painter, transform: TSTransform) {
        let center = transform.mul_pos(self.center);
        let radius = self.radius * transform.scaling;

        let fill = ctx.style().visuals.error_fg_color;
        painter.add(egui::Shape::circle_filled(center, radius, fill));
    }
}

pub struct App {
    gravity: f32,
    target_frame_rate: f32,
    target_simulation_rate: f32,
    n_sides: usize,
    _n_holes: usize,
    previous_frame_times: ringbuffer::AllocRingBuffer<web_time::Instant>,
    rotating_input: RotatingInput,
    rotating_body: RotatingBody,
    ball: Ball,
}

impl Default for App {
    fn default() -> Self {
        Self {
            gravity: 9.81,
            target_frame_rate: 60.0,
            target_simulation_rate: 1000.0,
            n_sides: 6,
            _n_holes: 0,
            previous_frame_times: ringbuffer::AllocRingBuffer::new(100),
            rotating_input: RotatingInput::default(),
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

    fn handle_collisions(&mut self) {
        let ball = &mut self.ball;
        let rotating_body = &mut self.rotating_body;

        let shape = rotating_body.shape_with_rotation_applied();

        let line_segments = shape.all_segments();

        // Determine which, if any, line segments the ball is colliding with
        let collisions = line_segments
            .into_iter()
            .filter_map(|(p1, p2)| {
                let v = p2 - p1;
                let n1 = egui::vec2(-v.y, v.x).normalized();

                let d = (ball.center - p1).dot(n1);

                if d.abs() < ball.radius {
                    let p = ball.center - d * n1;
                    let t = (p - p1).dot(v) / v.length_sq();

                    if t < 0.0 {
                        // Collision with edge at p1
                        let n2 = (ball.center - p1).normalized();
                        let n2 = if n1.dot(n2) > 0. { n2 } else { -n2 };
                        Some(Collision::new(p1, n2))
                    } else if t > 1.0 {
                        // Collision with edge at p2
                        let n2 = (ball.center - p2).normalized();
                        let n2 = if n1.dot(n2) > 0. { n2 } else { -n2 };
                        Some(Collision::new(p2, n2))
                    } else {
                        Some(Collision::new(p, n1))
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<Collision>>();

        // Resolve collisions by reflecting the ball's velocity about the normal of the line segments. If there
        // are multiple collisions, average the final velocity.
        if !collisions.is_empty() {
            let average_normal = collisions
                .iter()
                .fold(Vec2::ZERO, |acc, collision| acc + collision.normal);
            let average_normal = average_normal.normalized();

            ball.velocity =
                ball.velocity - 2.0 * ball.velocity.dot(average_normal) * average_normal;

            // Correct the ball's position so it is not intersecting with the hexagon
            let p = collisions
                .iter()
                .fold(Pos2::ZERO, |acc, collision| acc + collision.point.to_vec2());

            let average_position = p / collisions.len() as f32;

            ball.center = average_position + ball.radius * average_normal;
        }

        rotating_body.record_collisions(collisions);
    }

    fn update_physics(&mut self, dt: f32) {
        self.rotating_body.update(self.rotating_input, dt);
        self.ball.update(dt, self.gravity);
        self.handle_collisions();
    }
}

impl eframe::App for App {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let dt = self.update_frame_time().as_secs_f32();
        let fps = self.compute_fps();

        let n_sub_steps = (dt * self.target_simulation_rate).round();
        let dt_sim = dt / n_sub_steps;

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

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("Bouncy");

            ui.collapsing("Shape Settings", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Sides");
                    if ui
                        .add(egui::Slider::new(&mut self.n_sides, 3..=12).text("ea"))
                        .changed()
                    {
                        self.rotating_body = RotatingBody {
                            shape: Shape::regular_polygon(self.n_sides, 1., Pos2::new(0.0, 0.0)),
                            ..Default::default()
                        };
                        self.ball.reset();
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
                .button("Reset Ball")
                .on_hover_text("Reset the ball's position and velocity.")
                .clicked()
            {
                self.ball.reset();
            }

            ui.separator();

            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.horizontal(|ui| {
                    let brake_button =
                        ui.add(egui::Button::new("Brake").min_size(Vec2::new(100., 50.)));
                    if brake_button.is_pointer_button_down_on() {
                        self.rotating_input.brake = true;
                    } else {
                        self.rotating_input.brake = false;
                    }

                    let boost_button =
                        ui.add(egui::Button::new("Boost").min_size(Vec2::new(100., 50.)));
                    if boost_button.is_pointer_button_down_on() {
                        self.rotating_input.boost = true;
                    } else {
                        self.rotating_input.boost = false;
                    }
                });

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
        });
    }
}
