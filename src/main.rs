use eframe::egui;
use game_state::MyGameApp;
mod data;
mod modes;
mod game_state;
mod ui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([760.0, 720.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Facename",
        options,
        Box::new(|_cc| {
            egui_extras::install_image_loaders(&_cc.egui_ctx);
            Ok(Box::new(MyGameApp::default()))
        }),
    )
}
