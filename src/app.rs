use egui::{emath::TSTransform, Color32, RichText};
use ringbuffer::RingBuffer;

use crate::{
    drawable::Drawable,
    game::{self, Game},
    level::Level,
};

#[derive(Debug)]
pub enum State {
    Menu,
    Playing(Game),
    Won,
    Lost,
}

#[derive(Debug)]
pub struct App {
    target_frame_rate: f32,
    previous_frame_times: ringbuffer::AllocRingBuffer<web_time::Instant>,
    state: State,
    levels: Vec<Level>,
    current_level: uuid::Uuid,
}

impl App {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        target_frame_rate: f32,
        levels: Vec<Level>,
    ) -> Self {
        if levels.is_empty() {
            panic!("No levels provided");
        }

        let current_level_from_storage: Option<uuid::Uuid> = if let Some(storage) = cc.storage {
            eframe::get_value::<Option<uuid::Uuid>>(storage, "current_level").unwrap_or_default()
        } else {
            None
        };

        let current_level = if current_level_from_storage.is_none()
            || !levels
                .iter()
                .any(|level| level.id == current_level_from_storage.unwrap())
        {
            levels[0].id
        } else {
            current_level_from_storage.unwrap()
        };

        Self {
            target_frame_rate,
            previous_frame_times: ringbuffer::AllocRingBuffer::new(128),
            state: State::Menu,
            levels,
            current_level,
        }
    }

    pub fn simple_polygons(cc: &eframe::CreationContext<'_>) -> Self {
        let mut levels: Vec<Level> = (3..=6).map(Level::simple_polygon).collect();
        levels.push(Level::funky_polygon());

        Self::new(cc, 60.0, levels)
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

    fn draw_chrome(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame, fps: f32) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("SpinScape");
                if ui.button("Menu").clicked() {
                    self.state = State::Menu;
                }
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.add(egui::github_link_file!(
                    "https://github.com/thomasgt/bouncy/blob/main/",
                    "Source code."
                ));
                ui.label(format!("FPS: {:.0}", fps.round()));
                egui::warn_if_debug_build(ui);
            });
        });
    }

    fn draw_menu(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label("Select a level to play:");

                for level in &self.levels {
                    if ui.button(&level.name).clicked() {
                        self.state = State::Playing(Game::new(level.clone(), 1024.));
                    }
                }
            });
        });
    }

    fn draw_game(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let game = if let State::Playing(game) = &mut self.state {
            game
        } else {
            panic!("Invalid game state");
        };

        egui::TopBottomPanel::top("countdown")
            .show_separator_line(false)
            .show(ctx, |ui| {
                let elapsed = (web_time::Instant::now() - game.start_time).as_secs_f32();
                let limit = game.level.max_time.as_secs_f32();
                let remaining = limit - elapsed;
                let time_progress = remaining / limit;

                ui.add(
                    egui::ProgressBar::new(time_progress)
                        .text(format!("Time remaining: {:.1} s", remaining)),
                );

                let work_remaining = game.work_remaining();
                let work_progress = work_remaining / game.level.max_work;
                ui.add(egui::ProgressBar::new(work_progress).text(format!(
                    "Power remaining: {:.0} %",
                    (work_progress * 100.).round()
                )));
            });

        egui::TopBottomPanel::bottom("controls")
            .show_separator_line(false)
            .show(ctx, |ui| {
                ui.add_enabled_ui(game.inputs_enabled(), |ui| {
                    ui.columns(4, |ui| {
                        let brake_button = ui[1].add_sized(
                            egui::vec2(50.0, 50.0),
                            egui::Button::new(
                                RichText::new("Brake")
                                    .strong()
                                    .heading()
                                    .color(Color32::BLACK),
                            )
                            .fill(Color32::LIGHT_RED),
                        );
                        game.level.input.brake.active = brake_button.is_pointer_button_down_on();

                        let boost_button = ui[2].add_sized(
                            egui::vec2(50.0, 50.0),
                            egui::Button::new(
                                RichText::new("Boost")
                                    .strong()
                                    .heading()
                                    .color(Color32::BLACK),
                            )
                            .fill(Color32::LIGHT_GREEN),
                        );
                        game.level.input.boost.active = boost_button.is_pointer_button_down_on();
                    });
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let available_size = ui.available_size();

            // Allocate a painting region that takes up the remaining space
            let (response, painter) = ui.allocate_painter(available_size, egui::Sense::hover());

            let canvas_rect = response.rect;

            // Define scaling factor so hexagon takes up 80% of the available space
            let max_extent = game
                .level
                .body
                .shape
                .max_extent(game.level.body.center_of_rotation);

            let left_top_radius = max_extent.min.to_vec2().length();
            let bottom_right_radius = max_extent.max.to_vec2().length();
            let radius = left_top_radius.max(bottom_right_radius);

            let scale = 0.8 * canvas_rect.size().min_elem() / (2. * radius);

            let transform = TSTransform {
                scaling: scale,
                translation: canvas_rect.center().to_vec2(),
            };

            game.level.body.draw(ctx, &painter, transform);
            game.level.ball.draw(ctx, &painter, transform);
            game.collision_list.iter().for_each(|collision| {
                collision.draw(ctx, &painter, transform);
            });
        });
    }
}

impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let State::Playing(game) = &self.state {
            eframe::set_value(storage, "current_level", &game.level.id);
        } else {
            eframe::set_value(storage, "current_level", &self.current_level);
        }
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.previous_frame_times.push(web_time::Instant::now());
        let fps = self.compute_fps();

        self.draw_chrome(ctx, _frame, fps);

        match &mut self.state {
            State::Menu => self.draw_menu(ctx, _frame),
            State::Playing(g) => {
                let game_state = g.update();
                self.draw_game(ctx, _frame);

                match game_state {
                    game::State::Won => {
                        self.state = State::Menu;
                    }
                    game::State::GameOver => {
                        self.state = State::Menu;
                    }
                    game::State::Playing => {
                        // Schedule a repaint at the next frame
                        ctx.request_repaint_after(web_time::Duration::from_secs_f32(
                            1.0 / self.target_frame_rate,
                        ));
                    }
                }
            }
            State::Won => {}
            State::Lost => {}
        }
    }
}
