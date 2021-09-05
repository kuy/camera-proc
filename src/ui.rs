use eframe::{egui, epi};

#[derive(Default)]
pub struct App {}

impl epi::App for App {
    fn name(&self) -> &str {
        "Camera Proc"
    }

    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Camera Proc");
        });
    }
}

pub fn enter_loop() {
    let app = App::default();
    let options = eframe::NativeOptions {
        transparent: true,
        ..Default::default()
    };
    eframe::run_native(Box::new(app), options);
}
