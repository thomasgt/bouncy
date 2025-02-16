use egui::{emath::TSTransform, Pos2};
use ringbuffer::RingBuffer;

use crate::{
    collision,
    control::{Input, InputSet, InputSetWork},
    drawable::Drawable,
    level::Level,
    rotating::{self, CollisionList},
};

#[derive(Debug)]
pub enum State {
    Playing,
    Won,
    GameOver,
}

#[derive(Debug)]
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
            return State::GameOver;
        }

        let target_ticks = (elapsed.as_secs_f32() * self.tick_rate).round() as u64;
        while self.tick_counter < target_ticks {
            self.tick_counter += 1;
            self.update_physics();
        }

        self.frame_counter += 1;
        State::Playing
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

        // Pick the collision that is closest to the ball's previous position
        let closest_collision = collisions
            .iter()
            .min_by(|a, b| {
                let dist_a = (a.point - ball_previous_position).length();
                let dist_b = (b.point - ball_previous_position).length();
                dist_a.partial_cmp(&dist_b).unwrap()
            })
            .unwrap();

        self.level.ball.velocity = self.level.ball.velocity
            - 2.0
                * self.level.ball.velocity.dot(closest_collision.normal)
                * closest_collision.normal;

        self.level.ball.center =
            closest_collision.point + closest_collision.normal * self.level.ball.radius;

        let rotating_collision =
            rotating::Collision::new(*closest_collision, self.level.body.center_of_rotation);

        self.collision_list.push(rotating_collision);
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
