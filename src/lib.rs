#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::App;

mod ball;
mod collision;
mod control;
mod drawable;
mod game;
mod level;
mod rotating;
mod shape;
