#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod app;
mod foundation;
fn main() -> eframe::Result {
    if std::env::args().any(|a| a == "--foundation" || a == "--opencode-gate") {
        return foundation::run();
    }
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Terminal Workspace")
            .with_inner_size([1280., 800.])
            .with_min_inner_size([800., 520.]),
        ..Default::default()
    };
    eframe::run_native(
        "Terminal Workspace",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)?))),
    )
}
