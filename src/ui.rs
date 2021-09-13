use crate::preview::Config;
use asdf_pixel_sort::{
    Direction, Mode, Options, PColor, DEFAULT_BLACK, DEFAULT_BRIGHTNESS, DEFAULT_WHITE,
};
use eframe::{egui, epi};
use nokhwa::CameraInfo;

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

    // Data
    devices: Vec<CameraInfo>,
    // modes: Vec<Mode>,

    // State
    selected_camera: CameraIndex,
    options: Options,
    black: u16,
    brightness: u8,
    white: u16,
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
                        options,
                        black,
                        brightness,
                        white,
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

                    ui.label("Mode");
                    ui.label(format!("{:?}", options.mode));
                    ui.end_row();

                    let before = *black;
                    ui.label("Black threshold");
                    ui.add(egui::Slider::new(black, 0..=1000));
                    ui.end_row();

                    if &before != black {
                        let map = map_fn((0, 1000), (-16777216, -1));
                        let color = PColor::from_raw(map(*black));
                        options.mode = Mode::Black(color);
                        to_preview.send(Config::Mode(options.mode.clone())).unwrap();
                    }

                    let before = *brightness;
                    ui.label("Brightness threshold");
                    ui.add(egui::Slider::new(brightness, 0..=255));
                    ui.end_row();

                    if &before != brightness {
                        options.mode = Mode::Brightness(*brightness);
                        to_preview.send(Config::Mode(options.mode.clone())).unwrap();
                    }

                    let before = *white;
                    ui.label("White threshold");
                    ui.add(egui::Slider::new(white, 0..=1000));
                    ui.end_row();

                    if &before != white {
                        let map = map_fn((0, 1000), (-16777216, -1));
                        let color = PColor::from_raw(map(*white));
                        options.mode = Mode::White(color);
                        to_preview.send(Config::Mode(options.mode.clone())).unwrap();
                    }

                    let before = options.direction.clone();
                    ui.label("Direction");
                    egui::ComboBox::from_label("Choose direction")
                        .selected_text(format!("{:?}", options.direction))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut options.direction, Direction::Both, "Both");
                            ui.selectable_value(
                                &mut options.direction,
                                Direction::Column,
                                "Column",
                            );
                            ui.selectable_value(&mut options.direction, Direction::Row, "Row");
                        });
                    ui.end_row();

                    if before != options.direction {
                        to_preview
                            .send(Config::Direction(options.direction.clone()))
                            .unwrap();
                    }
                });
        });
    }
}

pub fn enter_loop(
    to_camera: flume::Sender<Command>,
    from_camera: flume::Receiver<Response>,
    to_preview: flume::Sender<Config>,
    options: &Options,
) {
    let scale = map_fn_r((-16777216, -1), (0, 1000));
    let app = App {
        to_camera,
        from_camera,
        to_preview,
        devices: vec![],
        selected_camera: CameraIndex(0),
        options: options.clone(),
        black: scale(DEFAULT_BLACK.as_raw()),
        brightness: DEFAULT_BRIGHTNESS,
        white: scale(DEFAULT_WHITE.as_raw()),
    };
    let options = eframe::NativeOptions {
        transparent: true,
        initial_window_size: Some(egui::vec2(400.0, 400.0)),
        ..Default::default()
    };
    eframe::run_native(Box::new(app), options);
}

fn map_fn(domain: (u16, u16), codomain: (i32, i32)) -> impl Fn(u16) -> i32 {
    // TODO: check pre-conditions

    move |input| {
        let (input_min, input_max) = domain;
        let (output_min, output_max) = codomain;

        if input <= input_min {
            return output_min;
        }

        if input_max <= input {
            return output_max;
        }

        let ratio = (input - input_min) as f64 / (input_max - input_min) as f64;
        ((output_max - output_min) as f64 * ratio + output_min as f64) as i32
    }
}

fn map_fn_r(domain: (i32, i32), codomain: (u16, u16)) -> impl Fn(i32) -> u16 {
    // TODO: check pre-conditions

    move |input| {
        let (input_min, input_max) = domain;
        let (output_min, output_max) = codomain;

        if input <= input_min {
            return output_min;
        }

        if input_max <= input {
            return output_max;
        }

        let ratio = (input - input_min) as f64 / (input_max - input_min) as f64;
        ((output_max - output_min) as f64 * ratio + output_min as f64) as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_fn() {
        let map = map_fn((10, 110), (-127, 127));
        assert_eq!(-127, map(9));
        assert_eq!(-127, map(10));
        assert_eq!(0, map(60));
        assert_eq!(-50, map(40));
        assert_eq!(50, map(80));
        assert_eq!(127, map(110));
        assert_eq!(127, map(111));

        let map = map_fn((0, 10000), (-16777216, -1));
        assert_eq!(-16777216, map(0));
        assert_eq!(-16000430, map(463));
        assert_eq!(-13000664, map(2251));
        assert_eq!(-1, map(10000));
        assert_eq!(-1, map(10001));
    }
}
