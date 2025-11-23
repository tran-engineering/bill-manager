mod app;
mod db;
mod pdf;
mod types;
mod ui;

use app::BillManagerApp;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Bill Manager",
        options,
        Box::new(|cc| Ok(Box::new(BillManagerApp::new(cc)))),
    )
}
