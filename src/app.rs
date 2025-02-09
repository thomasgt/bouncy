use std::time::Duration;

use egui::{emath::TSTransform, Color32, Pos2, Vec2};
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

pub struct Polygon {
    pub num_sides: usize,
    pub center: egui::Pos2,
    pub radius: f32,
    pub angle: f32,
    pub angular_velocity: f32,
    collisions: ringbuffer::AllocRingBuffer<Collision>,
}

impl Default for Polygon {
    fn default() -> Self {
        Self {
            num_sides: 6,
            center: egui::Pos2::new(0.0, 0.0),
            radius: 1.0,
            angle: 0.,
            angular_velocity: 1.0,
            collisions: ringbuffer::AllocRingBuffer::new(100),
        }
    }
}

impl Polygon {
    pub fn update(&mut self, dt: f32) {
        let angle_delta = self.angular_velocity * dt;
        self.angle += angle_delta;

        // Rotate the collisions so they are fixed to the hexagon
        self.collisions.iter_mut().for_each(|collision| {
            let p = collision.point - self.center;
            let p = egui::vec2(
                p.x * angle_delta.cos() - p.y * angle_delta.sin(),
                p.x * angle_delta.sin() + p.y * angle_delta.cos(),
            );
            collision.point = self.center + p;
        });
    }

    pub fn get_points(&self) -> Vec<Pos2> {
        (0..self.num_sides)
            .map(|i| {
                let angle =
                    self.angle + i as f32 * 2. * std::f32::consts::PI / self.num_sides as f32;
                self.center + self.radius * egui::vec2(angle.cos(), angle.sin())
            })
            .collect()
    }

    pub fn get_line_segments(&self) -> Vec<(Pos2, Pos2)> {
        let points = self.get_points();
        let mut segments = Vec::with_capacity(6);

        for i in 0..self.num_sides {
            segments.push((points[i], points[(i + 1) % self.num_sides]));
        }

        segments
    }

    pub fn draw(&self, ctx: &egui::Context, painter: &egui::Painter, transform: TSTransform) {
        let points = self
            .get_points()
            .into_iter()
            .map(|p| transform.mul_pos(p))
            .collect::<Vec<Pos2>>();

        let stroke = egui::Stroke::new(1.0, ctx.style().visuals.text_color());
        painter.add(egui::Shape::closed_line(points, stroke));

        self.collisions.iter().for_each(|collision| {
            collision.draw(ctx, painter, transform);
        });
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
    time_rate: f32,
    gravity: f32,
    target_frame_rate: f64,
    previous_frame_times: ringbuffer::AllocRingBuffer<web_time::Instant>,
    polygon: Polygon,
    ball: Ball,
}

impl Default for App {
    fn default() -> Self {
        Self {
            time_rate: 1.,
            gravity: 9.81,
            target_frame_rate: 60.0,
            previous_frame_times: ringbuffer::AllocRingBuffer::new(100),
            polygon: Polygon::default(),
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

    fn update_frame_time(&mut self) -> std::time::Duration {
        let now = web_time::Instant::now();

        let dt = if let Some(prev) = self.previous_frame_times.back() {
            now - *prev
        } else {
            web_time::Duration::from_secs_f64(1.0 / self.target_frame_rate)
        };

        self.previous_frame_times.push(now);
        dt
    }

    fn compute_fps(&self) -> f64 {
        if self.previous_frame_times.len() < 2 {
            return self.target_frame_rate;
        }

        let first = self.previous_frame_times.front().unwrap();
        let last = self.previous_frame_times.back().unwrap();
        let elapsed = *last - *first;
        let elapsed_secs = elapsed.as_secs_f64();

        (self.previous_frame_times.len() as f64 - 1.0) / elapsed_secs
    }

    fn handle_collisions(&mut self) {
        let ball = &mut self.ball;
        let hexagon = &mut self.polygon;

        let line_segments = hexagon.get_line_segments();

        // Determine which, if any, line segments the ball is colliding with
        let collisions = line_segments
            .into_iter()
            .filter_map(|(p1, p2)| {
                let v = p2 - p1;
                let n = egui::vec2(-v.y, v.x).normalized();

                let d = (ball.center - p1).dot(n);

                if d.abs() < ball.radius {
                    let p = ball.center - d * n;
                    let t = (p - p1).dot(v) / v.length_sq();

                    if t >= 0.0 && t <= 1.0 {
                        Some(Collision::new(p, n))
                    } else {
                        None
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

        hexagon.collisions.extend(collisions);
    }

    fn update_physics(&mut self, dt: f32) {
        self.polygon.update(dt);
        self.ball.update(dt, self.gravity);
        self.handle_collisions();
    }
}

impl eframe::App for App {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let dt = self.update_frame_time();
        let fps = self.compute_fps();

        self.update_physics(dt.as_secs_f32() * self.time_rate);

        // Schedule a repaint at the next frame
        ctx.request_repaint_after(Duration::from_secs_f64(1.0 / self.target_frame_rate));

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
                        .add(egui::Slider::new(&mut self.polygon.num_sides, 3..=12).text("ea"))
                        .changed()
                    {
                        self.ball.reset();
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Angular velocity");
                    ui.add(
                        egui::Slider::new(&mut self.polygon.angular_velocity, -5.0..=5.0)
                            .text("(rad/s)"),
                    );
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

            ui.vertical_centered(|ui| {
                let available_size = ui.available_size();

                // Allocate a painting region that takes up the remaining space
                let (response, painter) = ui.allocate_painter(available_size, egui::Sense::hover());

                let canvas_rect = response.rect;

                // Define scaling factor so hexagon takes up 80% of the available space
                let scale = 0.8 * canvas_rect.size().min_elem() / (2. * self.polygon.radius);

                let transform = TSTransform {
                    scaling: scale,
                    translation: canvas_rect.center().to_vec2(),
                };

                self.polygon.draw(ctx, &painter, transform);
                self.ball.draw(ctx, &painter, transform);
            });
        });
    }
}
