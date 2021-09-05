use crate::ui::{CameraIndex, Command, Response};
use image::RgbImage;
use nokhwa::{Camera, CaptureAPIBackend, FrameFormat};
use std::thread::{self, JoinHandle};

pub fn capture(
    tx: flume::Sender<RgbImage>,
    from_ui: flume::Receiver<Command>,
    to_ui: flume::Sender<Response>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let devices = match nokhwa::query_devices(CaptureAPIBackend::Auto) {
            Ok(devices) => devices,
            Err(e) => panic!("Failed to enumerate camera devices: {:?}", e),
        };

        to_ui.send(Response::Devices(devices)).unwrap();

        loop {
            let index = match from_ui.recv() {
                Ok(Command::CameraChanged(CameraIndex(index))) => index,
                _ => {
                    log::error!("Expected to receive Command::CameraChanged, but not");
                    break;
                }
            };

            let fps = 10;
            let mut camera = match Camera::new_with(
                index,
                640,
                480,
                fps,
                FrameFormat::MJPEG,
                CaptureAPIBackend::Auto,
            ) {
                Ok(camera) => camera,
                Err(e) => {
                    log::info!("Camera not available: {:?}", e);
                    continue; // wait for change camera
                }
            };

            camera.open_stream().unwrap();

            loop {
                let frame = camera.frame().unwrap();
                log::debug!(
                    "Captured frame {}x{} @ {}FPS size {}",
                    frame.width(),
                    frame.height(),
                    fps,
                    frame.len()
                );
                tx.send(frame).unwrap()
            }
        }
    })
}
