use egui::Pos2;
use serde::{Deserialize, Serialize};

use crate::{
    ball::Ball,
    control::{Input, InputSet},
    rotating::Body,
    shape::Shape,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Level {
    pub id: uuid::Uuid,
    pub name: String,
    pub body: Body,
    pub ball: Ball,
    pub input: InputSet,
    pub gravity: f32,
    pub max_time: web_time::Duration,
    pub max_work: f32,
}

impl Level {
    pub fn simple_polygon(num_sides: usize) -> Self {
        let id = uuid::Uuid::new_v4();
        let name = format!("Simple Polygon {}", num_sides);
        let shape = Shape::regular_polygon(num_sides, 1.0, Pos2::ZERO);
        let body = Body {
            shape,
            ..Default::default()
        };
        let ball = Ball::default();
        let input = InputSet {
            brake: Input {
                torque: 3.0,
                active: false,
            },
            motor: Input {
                torque: 1.0,
                active: true,
            },
            boost: Input {
                torque: 2.0,
                active: false,
            },
        };
        let gravity = 9.81;
        let max_time = web_time::Duration::from_secs(45);
        let max_work = 50.0;

        Self {
            id,
            name,
            body,
            ball,
            input,
            gravity,
            max_time,
            max_work,
        }
    }

    pub fn funky_polygon() -> Self {
        let id = uuid::Uuid::new_v4();
        let name = "Funky Polygon".to_string();
        let shape = Shape::funky_polygon();
        let body = Body {
            shape,
            ..Default::default()
        };
        let ball = Ball::default();
        let input = InputSet {
            brake: Input {
                torque: 3.0,
                active: false,
            },
            motor: Input {
                torque: 1.0,
                active: true,
            },
            boost: Input {
                torque: 2.0,
                active: false,
            },
        };
        let gravity = 9.81;
        let max_time = web_time::Duration::from_secs(45);
        let max_work = 50.0;

        Self {
            id,
            name,
            body,
            ball,
            input,
            gravity,
            max_time,
            max_work,
        }
    }
}
