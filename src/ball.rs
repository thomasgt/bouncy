use egui::{emath::TSTransform, Pos2, Vec2};
use serde::{Deserialize, Serialize};

use crate::drawable::Drawable;

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Ball {
    pub center: Pos2,
    pub radius: f32,
    pub velocity: Vec2,
}

impl Default for Ball {
    fn default() -> Self {
        Self {
            center: Pos2::new(0.0, 0.0),
            radius: 0.05,
            velocity: Vec2::new(0.0, 0.0),
        }
    }
}

impl Ball {
    pub fn update(&mut self, dt: f32, gravity: f32) {
        self.velocity.y += gravity * dt;
        self.center += self.velocity * dt;
    }
}

impl Drawable for Ball {
    fn draw(&self, ctx: &egui::Context, painter: &egui::Painter, transform: TSTransform) {
        let center = transform.mul_pos(self.center);
        let radius = self.radius * transform.scaling;

        let fill = ctx.style().visuals.error_fg_color;
        painter.add(egui::Shape::circle_filled(center, radius, fill));
    }
}
