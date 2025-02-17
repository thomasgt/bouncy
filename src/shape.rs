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

    pub fn all_segments(&self) -> Vec<Segment> {
        self.lines
            .iter()
            .flat_map(|line| line.windows(2).map(|w| (w[0], w[1])))
            .collect()
    }

    pub fn all_segments_including_openings(&self) -> Vec<Segment> {
        let points: Vec<Pos2> = self.lines.iter().flatten().copied().collect();
        let mut segments: Vec<Segment> = points.windows(2).map(|w| (w[0], w[1])).collect();

        if points.first() != points.last() {
            segments.push((*points.last().unwrap(), *points.first().unwrap()));
        }

        segments
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

pub fn compute_winding_number(point: Pos2, shape: &Shape) -> i32 {
    let mut winding_number = 0;

    // Use Sunday's algorithm to compute the winding number without trigonometry
    // Use points vs segments because there may be openings in the shape and we want to treat the polygon
    // as closed for this calculation
    for segment in shape.all_segments_including_openings() {
        let (start, end) = segment;
        // Check if horizontal ray from point intersects with segment
        if (start.y > point.y) != (end.y > point.y) {
            // Check if ray intersects with segment
            if point.x < (end.x - start.x) * (point.y - start.y) / (end.y - start.y) + start.x {
                // If the ray intersects with the segment, check if it intersects from the left
                if end.y > start.y {
                    winding_number += 1;
                } else {
                    winding_number -= 1;
                }
            }
        }
    }

    winding_number
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_winding_number() {
        let shape = Shape::regular_polygon(4, 1.0, Pos2::ZERO);

        assert_eq!(compute_winding_number(Pos2::ZERO, &shape), 1);
        assert_eq!(compute_winding_number(Pos2::new(1., 1.), &shape), 0);
        assert_eq!(compute_winding_number(Pos2::new(0., 0.5), &shape), 1);
        assert_eq!(compute_winding_number(Pos2::new(-1., 0.5), &shape), 0);
        assert_eq!(compute_winding_number(Pos2::new(-4., 0.0), &shape), 0);
        assert_eq!(compute_winding_number(Pos2::new(-1., 1.), &shape), 0);
    }
}
