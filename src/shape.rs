use egui::{emath::TSTransform, Pos2, Rect, Vec2};
use serde::{Deserialize, Serialize};

use crate::drawable::Drawable;

pub type Segment = (Pos2, Pos2);
pub type Line = Vec<Pos2>;

#[derive(Clone, Debug, Deserialize, Serialize)]
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

    pub fn funky_polygon() -> Self {
        let lines = vec![
            vec![
                Pos2::new(1.0, 0.0),
                Pos2::new(0.9, 0.4),
                Pos2::new(0.8, 0.6),
                Pos2::new(0.2, 1.0),
                Pos2::new(0.0, 1.0),
                Pos2::new(0.4, 0.7),
                Pos2::new(-0.6, 0.5),
                Pos2::new(-1.0, 0.0),
            ],
            vec![
                Pos2::new(-1.0, -0.2),
                Pos2::new(-0.5, -0.5),
                Pos2::new(0.0, -1.0),
                Pos2::new(0.3, -0.8),
                Pos2::new(0.6, -0.4),
                Pos2::new(1.0, 0.0),
            ],
        ];

        Self { lines }
    }

    pub fn funky_polygon2() -> Self {
        Self {
            lines: vec![vec![
                Pos2::new(1.000, 0.000),
                Pos2::new(0.809, 0.588),
                Pos2::new(0.309, 0.951),
                Pos2::new(-0.309, 0.951),
                Pos2::new(-0.809, 0.588),
                Pos2::new(-0.700, 0.000),
                Pos2::new(-0.809, -0.588),
                Pos2::new(-0.309, -0.951),
                Pos2::new(0.309, -0.951),
                Pos2::new(0.809, -0.588),
                Pos2::new(1.000, 0.000),
            ]],
        }
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

impl Drawable for Shape {
    fn draw(&self, ctx: &egui::Context, painter: &egui::Painter, transform: TSTransform) {
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
}
