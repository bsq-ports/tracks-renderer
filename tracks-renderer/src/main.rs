use std::sync::mpsc;

mod bevy_renderer;

fn main() {
    // create channel to send input commands (unused by default)
    let (_tx, rx) = mpsc::channel();
    // run the bevy renderer in this process (blocking)
    bevy_renderer::start_bevy(rx);
}
