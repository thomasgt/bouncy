use egui::emath::TSTransform;

pub trait Drawable {
    fn draw(&self, ctx: &egui::Context, painter: &egui::Painter, transform: TSTransform);
}
