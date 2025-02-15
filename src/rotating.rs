use egui::{emath::TSTransform, Pos2};
use ringbuffer::RingBuffer;

use crate::{collision::Collision, drawable::Drawable, shape::Shape};

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
    pub shape: Shape,
    pub center_of_rotation: egui::Pos2,
    pub angle: f32,
    pub angular_velocity: f32,
    pub moment_of_inertia: f32,
    pub friction_coefficient: f32,
    pub collisions_with_angles: ringbuffer::AllocRingBuffer<(Collision, f32)>,
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

    pub fn update(&mut self, input: RotatingInput, dt: f32) -> (f32, f32, f32) {
        let friction_torque = -self.friction_coefficient * self.angular_velocity;
        let brake_torque = if input.brake {
            -input.braking_torque * self.angular_velocity.signum()
        } else {
            0.0
        };

        let motor_torque = if input.motor { input.motor_torque } else { 0.0 };
        let boost_torque = if input.boost { input.boost_torque } else { 0.0 };

        if input.brake && self.angular_velocity.abs() < 0.001 {
            self.angular_velocity = 0.0;
        } else {
            let torque = friction_torque + brake_torque + motor_torque + boost_torque;
            let angular_acceleration = torque / self.moment_of_inertia;

            self.angular_velocity += angular_acceleration * dt;
        }

        let delta_angle = self.angular_velocity * dt;
        self.angle += delta_angle;

        let brake_work = brake_torque * delta_angle;
        let boost_work = boost_torque * delta_angle;
        let motor_work = motor_torque * delta_angle;

        (brake_work, boost_work, motor_work)
    }
}

impl Drawable for RotatingBody {
    fn draw(&self, ctx: &egui::Context, painter: &egui::Painter, transform: TSTransform) {
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
