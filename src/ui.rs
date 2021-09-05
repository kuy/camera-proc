use crate::preview::Config;
use asdf_pixel_sort::PColor;
use eframe::{egui, epi};
use nokhwa::CameraInfo;
use once_cell::sync::Lazy;

#[derive(Debug)]
pub enum Command {
    CameraChanged(CameraIndex),
}

#[derive(Debug)]
pub enum Response {
    Devices(Vec<CameraInfo>),
}

#[derive(Clone, PartialEq)]
pub struct CameraIndex(pub usize);

impl std::fmt::Debug for CameraIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "device {}", &self.0)
    }
}

pub struct App {
    // Context
    to_camera: flume::Sender<Command>,
    from_camera: flume::Receiver<Response>,
    to_preview: flume::Sender<Config>,

    // State
    devices: Vec<CameraInfo>,
    selected_camera: CameraIndex,
    black_threshold: PColor,
}

impl epi::App for App {
    fn name(&self) -> &str {
        "ctrl | ENDNAUT"
    }

    fn setup(&mut self, _ctx: &egui::CtxRef, _: &mut epi::Frame<'_>, _: Option<&dyn epi::Storage>) {
        self.devices = match self.from_camera.recv() {
            Ok(Response::Devices(devices)) => devices,
            _ => panic!("Failed to receive available devices"),
        };
    }

    fn update(&mut self, ctx: &egui::CtxRef, _: &mut epi::Frame<'_>) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Grid::new("grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    let Self {
                        to_camera,
                        to_preview,
                        selected_camera,
                        devices,
                        black_threshold,
                        ..
                    } = self;

                    let before = selected_camera.clone();
                    ui.label("Camera");
                    egui::ComboBox::from_label("Choose camera")
                        .selected_text(format!("{:?}", selected_camera))
                        .show_ui(ui, |ui| {
                            for d in devices {
                                let caption = format!("{} {}", d.human_name(), d.description());
                                ui.selectable_value(
                                    selected_camera,
                                    CameraIndex(*d.index()),
                                    caption,
                                );
                            }
                        });
                    ui.end_row();

                    if before != *selected_camera {
                        to_camera
                            .send(Command::CameraChanged(selected_camera.clone()))
                            .unwrap();
                    }

                    let mut value = (black_threshold.red as u32
                        + black_threshold.green as u32
                        + black_threshold.blue as u32)
                        / 3;
                    let before = value;
                    ui.label("Black threshold");
                    ui.add(egui::Slider::new(&mut value, 0..=255));
                    ui.end_row();

                    if before != value {
                        *black_threshold = PColor::new(value as u8, value as u8, value as u8);
                        to_preview
                            .send(Config::Threshold(black_threshold.clone()))
                            .unwrap();
                    }
                });
        });
    }
}

pub static BLACK: Lazy<PColor> = Lazy::new(|| PColor::new(0, 0, 0));

pub fn enter_loop(
    to_camera: flume::Sender<Command>,
    from_camera: flume::Receiver<Response>,
    to_preview: flume::Sender<Config>,
) {
    let app = App {
        to_camera,
        from_camera,
        to_preview,
        devices: vec![],
        selected_camera: CameraIndex(0),
        black_threshold: BLACK.clone(),
    };
    let options = eframe::NativeOptions {
        transparent: true,
        initial_window_size: Some(egui::vec2(400.0, 400.0)),
        ..Default::default()
    };
    eframe::run_native(Box::new(app), options);
}
