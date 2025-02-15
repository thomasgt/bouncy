use egui::Pos2;

use crate::{ball::Ball, shape::Shape};

#[derive(Debug)]
pub struct Level {
    pub name: String,
    pub shape: Shape,
    pub center_of_rotation: Pos2,
    pub initial_ball: Ball,
}
