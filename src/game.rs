use egui::{emath::TSTransform, Pos2, Vec2};
use ringbuffer::RingBuffer;

use crate::{
    collision,
    control::{Input, InputSet, InputSetWork},
    drawable::Drawable,
    level::Level,
    rotating::{self, CollisionList},
    shape::compute_winding_number,
};

#[derive(Debug)]
pub enum State {
    Playing,
    Victory,
    Defeat,
}

#[derive(Debug, Clone)]
pub struct Game {
    pub start_time: web_time::Instant,
    pub tick_rate: f32,
    pub tick_dt: f32,
    pub tick_counter: u64,
    pub frame_counter: u64,
    pub level: Level,
    pub input_work: InputSetWork,
    pub collision_list: CollisionList,
}

impl Game {
    pub fn new(level: Level, tick_rate: f32) -> Self {
        Self {
            start_time: web_time::Instant::now(),
            tick_rate,
            tick_dt: 1.0 / tick_rate,
            tick_counter: 0,
            frame_counter: 0,
            level,
            input_work: InputSetWork::default(),
            collision_list: CollisionList::new(1024),
        }
    }

    pub fn update(&mut self) -> State {
        let now = web_time::Instant::now();
        let elapsed = now - self.start_time;

        // TODO Implement this in terms of ticks to allow buzzer beaters
        if elapsed > self.level.max_time {
            return State::Defeat;
        }

        let target_ticks = (elapsed.as_secs_f32() * self.tick_rate).round() as u64;
        while self.tick_counter < target_ticks {
            self.tick_counter += 1;
            self.update_physics();
            if self.has_escaped() {
                return State::Victory;
            }
        }

        self.frame_counter += 1;
        State::Playing
    }

    pub fn has_escaped(&self) -> bool {
        let winding_number = compute_winding_number(
            self.level.ball.center,
            &self.level.body.shape_with_rotation_applied(),
        );

        winding_number == 0
    }

    pub fn work_remaining(&self) -> f32 {
        let work_spent = self.input_work.brake + self.input_work.boost;
        (self.level.max_work - work_spent).max(0.0)
    }

    pub fn inputs_enabled(&self) -> bool {
        self.work_remaining() > 0.0
    }

    fn input(&self) -> InputSet {
        if self.inputs_enabled() {
            self.level.input
        } else {
            InputSet {
                brake: Input {
                    torque: 0.0,
                    active: false,
                },
                boost: Input {
                    torque: 0.0,
                    active: false,
                },
                ..self.level.input
            }
        }
    }

    fn update_physics(&mut self) {
        let update_result = self.level.body.update(self.input(), self.tick_dt);
        self.input_work += update_result.work;
        self.collision_list.iter_mut().for_each(|collision| {
            collision.update(update_result.delta_angle);
        });

        let ball_previous_position = self.level.ball.center;
        self.level.ball.update(self.tick_dt, self.level.gravity);

        self.handle_collisions(ball_previous_position);
    }

    fn detect_collisions(&self) -> Vec<collision::Collision> {
        let ball = &self.level.ball;
        let body = &self.level.body;

        let shape = body.shape_with_rotation_applied();

        let line_segments = shape.all_segments();

        // Determine which, if any, line segments the ball is colliding with
        line_segments
            .into_iter()
            .filter_map(|segment| collision::detect_collision(segment, *ball))
            .collect()
    }

    fn handle_collisions(&mut self, ball_previous_position: Pos2) {
        let collisions = self.detect_collisions();

        if collisions.is_empty() {
            return;
        }

        let aggregate_normal = collisions
            .iter()
            .map(|collision| collision.normal)
            .fold(Vec2::ZERO, |acc, n| acc + n)
            .normalized();

        self.level.ball.velocity = self.level.ball.velocity
            - 2.0 * self.level.ball.velocity.dot(aggregate_normal) * aggregate_normal;

        let delta_angle = -self.level.body.angular_velocity * self.tick_dt;

        // Pick the collision that is closest for the shape and ball's previous position
        let closest_collision = collisions
            .iter()
            .map(|collision| {
                let rotated = collision.rotate(delta_angle, self.level.body.center_of_rotation);
                (collision, rotated.point)
            })
            .min_by(|a, b| {
                let dist_a = (a.1 - ball_previous_position).length();
                let dist_b = (b.1 - ball_previous_position).length();

                dist_a.partial_cmp(&dist_b).unwrap()
            })
            .unwrap()
            .0;

        self.level.ball.center =
            closest_collision.point + closest_collision.normal * self.level.ball.radius;

        let rotating_collisions = collisions.into_iter().map(|collision| {
            rotating::Collision::new(collision, self.level.body.center_of_rotation)
        });

        self.collision_list.extend(rotating_collisions);
    }
}

impl Drawable for Game {
    fn draw(&self, ctx: &egui::Context, painter: &egui::Painter, transform: TSTransform) {
        self.level.body.draw(ctx, painter, transform);
        self.collision_list.iter().for_each(|collision| {
            collision.draw(ctx, painter, transform);
        });
        self.level.ball.draw(ctx, painter, transform);
    }
}
