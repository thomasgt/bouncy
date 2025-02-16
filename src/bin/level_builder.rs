// struct App {}

// impl eframe::App for App {
//     fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
//         todo!()
//     }
// }

// fn main() -> eframe::Result {
//     env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

//     let native_options = eframe::NativeOptions {
//         viewport: egui::ViewportBuilder::default()
//             .with_inner_size([400.0, 300.0])
//             .with_min_inner_size([300.0, 220.0])
//             .with_icon(
//                 // NOTE: Adding an icon is optional
//                 eframe::icon_data::from_png_bytes(&include_bytes!("../../assets/icon-256.png")[..])
//                     .expect("Failed to load icon"),
//             ),
//         ..Default::default()
//     };
//     eframe::run_native(
//         "Level Builder",
//         native_options,
//         Box::new(|cc| Ok(Box::new(App {}))),
//     )
// }

use bouncy::level::Level;

fn main() {
    let mut levels: Vec<Level> = (3..=6).map(Level::simple_polygon).collect();
    levels.push(Level::funky_polygon());

    serde_json::to_writer_pretty(std::io::stdout(), &levels).unwrap();
}
