use std::sync::mpsc;

mod bevy_renderer;

fn main() {
    // create channel to send input commands (unused by default)
    // run the bevy renderer in this process (blocking)
    bevy_renderer::start_bevy();
}
