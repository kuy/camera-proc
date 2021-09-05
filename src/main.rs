mod camera;
mod preview;
mod ui;
mod vertex;

fn main() {
    let (tx, rx) = flume::unbounded();

    let camera_thread = camera::capture(tx);
    let preview_thread = preview::run(rx);

    ui::enter_loop();

    camera_thread.join().unwrap();
    preview_thread.join().unwrap();
}
