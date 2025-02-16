use egui::{Pos2, Vec2};

use crate::{ball::Ball, shape::Segment};

#[derive(Clone, Copy, Debug)]
pub struct Collision {
    pub point: Pos2,
    pub normal: Vec2,
}

impl Collision {
    pub fn new(point: Pos2, normal: Vec2) -> Self {
        Self { point, normal }
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

        Self { point, normal }
    }
}

pub fn detect_collision(segment: Segment, ball: Ball) -> Option<Collision> {
    let p1 = segment.0;
    let p2 = segment.1;

    let v = p2 - p1;
    let v_length = v.length();
    let n1 = egui::vec2(-v.y, v.x).normalized();

    let d = (ball.center - p1).dot(n1);

    if d.abs() < ball.radius {
        let p = ball.center - d * n1;
        let t = (p - p1).dot(v) / v_length;

        if t >= -ball.radius && t < 0.0 {
            // Collision with edge at p1
            let n2 = ball.center - p1;
            let n2 = if n1.dot(n2) > 0. { n2 } else { -n2 };
            Some(Collision::new(p1, n2.normalized()))
        } else if t > v_length && t <= v_length + ball.radius {
            // Collision with edge at p2
            let n2 = ball.center - p2;
            let n2 = if n1.dot(n2) > 0. { n2 } else { -n2 };
            Some(Collision::new(p2, n2.normalized()))
        } else if t >= 0.0 && t <= v_length {
            Some(Collision::new(p, n1))
        } else {
            None
        }
    } else {
        None
    }
}
