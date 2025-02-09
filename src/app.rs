use std::time::Duration;

use ringbuffer::RingBuffer;

pub struct Hexagon {
    pub center: egui::Pos2,
    pub radius: f32,
    pub angle: f32,
}

impl Default for Hexagon {
    fn default() -> Self {
        Self {
            center: egui::Pos2::new(0.0, 0.0),
            radius: 1.0,
            angle: 0.0,
        }
    }
}

pub struct App {
    target_frame_rate: f64,
    previous_frame_times: ringbuffer::AllocRingBuffer<web_time::Instant>,
    hexagon: Hexagon,
}

impl Default for App {
    fn default() -> Self {
        Self {
            target_frame_rate: 60.0,
            previous_frame_times: ringbuffer::AllocRingBuffer::new(100),
            hexagon: Hexagon::default(),
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

    fn draw_hexagon(&self, ctx: &egui::Context, painter: &egui::Painter, canvas_rect: egui::Rect) {
        let Hexagon {
            center,
            radius,
            angle,
        } = self.hexagon;

        // Offset hexagon center relative to the middle of the canvas
        let center = center + canvas_rect.center().to_vec2();

        let points = (0..6)
            .map(|i| {
                let angle = angle + i as f32 * std::f32::consts::PI / 3.0;
                center + radius * egui::vec2(angle.cos(), angle.sin())
            })
            .collect::<Vec<_>>();

        let stroke = egui::Stroke::new(1.0, ctx.style().visuals.text_color());
        painter.add(egui::Shape::closed_line(points, stroke));
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
}

impl eframe::App for App {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let _dt = self.update_frame_time();
        let fps = self.compute_fps();

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
                powered_by_egui_and_eframe(ui);
                ui.add(egui::github_link_file!(
                    "https://github.com/emilk/eframe_template/blob/main/",
                    "Source code."
                ));
                ui.label(format!("FPS: {:.1}", fps));
                egui::warn_if_debug_build(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("Bouncy");

            ui.horizontal(|ui| {
                ui.label("Hexagon:");
                ui.add(egui::Slider::new(&mut self.hexagon.radius, 0.0..=100.0).text("radius"));
                ui.add(
                    egui::Slider::new(&mut self.hexagon.angle, 0.0..=std::f32::consts::PI)
                        .text("angle"),
                );
            });

            ui.separator();

            let available_size = ui.available_size_before_wrap();

            ui.vertical_centered(|ui| {
                // Allocate a painting region that takes up the remaining space
                let (response, painter) = ui.allocate_painter(available_size, egui::Sense::hover());

                let canvas_rect = response.rect; // The actual rectangle of the canvas
                self.draw_hexagon(ctx, &painter, canvas_rect);
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
