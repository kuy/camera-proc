mod camera;
mod preview;
mod ui;
mod vertex;

fn main() {
    if cfg!(debug_assertions) {
        std::env::set_var("RUST_LOG", "debug");
    }
    env_logger::init();

    let (cmd_tx, cmd_rx) = flume::unbounded();
    let (res_tx, res_rx) = flume::unbounded();
    let (cap_tx, cap_rx) = flume::unbounded();
    let (cfg_tx, cfg_rx) = flume::unbounded();

    let camera_thread = camera::capture(cap_tx, cmd_rx, res_tx);
    let preview_thread = preview::run(cap_rx, cfg_rx);

    ui::enter_loop(cmd_tx, res_rx, cfg_tx);

    camera_thread.join().unwrap();
    preview_thread.join().unwrap();
}
