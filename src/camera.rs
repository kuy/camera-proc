use image::RgbImage;
use nokhwa::Camera;
use std::thread::{self, JoinHandle};

pub fn capture(tx: flume::Sender<RgbImage>) -> JoinHandle<()> {
    thread::spawn(move || {
        let fps = 10;
        let mut camera = Camera::new_with(
            4,
            640,
            480,
            fps,
            nokhwa::FrameFormat::MJPEG,
            nokhwa::CaptureAPIBackend::Auto,
        )
        .unwrap();
        camera.open_stream().unwrap();
        loop {
            let frame = camera.frame().unwrap();
            println!(
                "Captured frame {}x{} @ {}FPS size {}",
                frame.width(),
                frame.height(),
                fps,
                frame.len()
            );
            tx.send(frame).unwrap()
        }
    })
}
