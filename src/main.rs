mod camera;
mod display;
mod vertex;

fn main() {
    let (tx, rx) = flume::unbounded();
    camera::capture(tx);
    display::render(rx);
}
