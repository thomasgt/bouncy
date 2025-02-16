use egui::{emath::TSTransform, Color32, Pos2};
use serde::{Deserialize, Serialize};

use crate::{
    collision,
    control::{InputSet, InputSetWork},
    drawable::Drawable,
    shape::Shape,
};

#[derive(Debug, Clone)]
pub struct Collision {
    pub collision: collision::Collision,
    pub center_of_rotation: Pos2,
    pub angle: f32,
    pub time: web_time::Instant,
}

pub type CollisionList = ringbuffer::AllocRingBuffer<Collision>;

impl Collision {
    pub fn new(collision: collision::Collision, center_of_rotation: Pos2) -> Self {
        Self {
            collision,
            center_of_rotation,
            angle: 0.0,
            time: web_time::Instant::now(),
        }
    }

    pub fn update(&mut self, delta_angle: f32) {
        self.angle += delta_angle;
    }
}

impl Drawable for Collision {
    fn draw(&self, ctx: &egui::Context, painter: &egui::Painter, transform: TSTransform) {
        let collision = self.collision.rotate(self.angle, self.center_of_rotation);
        let age = (web_time::Instant::now() - self.time).as_secs_f32();
        let size = 10. * age;
        let opacity = 1.0 - age / 2.;

        if size <= 0.0 || opacity <= 0.0 {
            return;
        }

        let point = transform.mul_pos(collision.point);
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

pub struct BodyUpdateResult {
    pub work: InputSetWork,
    pub delta_angle: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Body {
    pub shape: Shape,
    pub center_of_rotation: egui::Pos2,
    pub angle: f32,
    pub angular_velocity: f32,
    pub moment_of_inertia: f32,
    pub friction_coefficient: f32,
}

impl Default for Body {
    fn default() -> Self {
        Self {
            shape: Shape::regular_polygon(6, 1., Pos2::new(0.0, 0.0)),
            center_of_rotation: egui::Pos2::new(0.0, 0.0),
            angle: 0.,
            angular_velocity: 1.0,
            moment_of_inertia: 1.0,
            friction_coefficient: 0.7,
        }
    }
}

impl Body {
    pub fn shape_with_rotation_applied(&self) -> Shape {
        self.shape.rotate(self.angle, self.center_of_rotation)
    }

    pub fn update(&mut self, input: InputSet, dt: f32) -> BodyUpdateResult {
        let friction_torque = -self.friction_coefficient * self.angular_velocity;
        let brake_torque = if input.brake.active {
            -input.brake.torque * self.angular_velocity.signum()
        } else {
            0.0
        };

        let motor_torque = if input.motor.active {
            input.motor.torque
        } else {
            0.0
        };

        let boost_torque = if input.boost.active {
            input.boost.torque
        } else {
            0.0
        };

        if input.brake.active && self.angular_velocity.abs() < 0.001 {
            self.angular_velocity = 0.0;
        } else {
            let torque = friction_torque + brake_torque + motor_torque + boost_torque;
            let angular_acceleration = torque / self.moment_of_inertia;

            self.angular_velocity += angular_acceleration * dt;
        }

        let delta_angle = self.angular_velocity * dt;
        self.angle += delta_angle;

        let brake_work = -brake_torque * delta_angle;
        let boost_work = boost_torque * delta_angle;
        let motor_work = motor_torque * delta_angle;

        BodyUpdateResult {
            work: InputSetWork {
                brake: brake_work,
                motor: motor_work,
                boost: boost_work,
            },
            delta_angle,
        }
    }
}

impl Drawable for Body {
    fn draw(&self, ctx: &egui::Context, painter: &egui::Painter, transform: TSTransform) {
        let shape = self.shape.rotate(self.angle, self.center_of_rotation);
        shape.draw(ctx, painter, transform);
    }
}
