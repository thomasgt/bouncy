use serde::{Deserialize, Serialize};

use crate::{ball::Ball, control::InputSet, rotating::Body};

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
