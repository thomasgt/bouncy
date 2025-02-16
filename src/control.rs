use std::ops::AddAssign;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Input {
    pub torque: f32,
    pub active: bool,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct InputSet {
    pub brake: Input,
    pub motor: Input,
    pub boost: Input,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct InputSetWork {
    pub brake: f32,
    pub motor: f32,
    pub boost: f32,
}

impl AddAssign<InputSetWork> for InputSetWork {
    fn add_assign(&mut self, rhs: InputSetWork) {
        self.brake += rhs.brake;
        self.motor += rhs.motor;
        self.boost += rhs.boost;
    }
}
