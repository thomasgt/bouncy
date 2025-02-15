use egui::{emath::TSTransform, Color32, Pos2, Vec2};

use crate::drawable::Drawable;

#[derive(Clone, Copy, Debug)]
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
            
            egui::vec2(
                n.x * angle.cos() - n.y * angle.sin(),
                n.x * angle.sin() + n.y * angle.cos(),
            )
        };

        Self {
            point,
            normal,
            time: self.time,
        }
    }
}

impl Drawable for Collision {
    fn draw(&self, ctx: &egui::Context, painter: &egui::Painter, transform: TSTransform) {
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
