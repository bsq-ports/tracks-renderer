pub mod bevy_renderer;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
#[wasm_bindgen(start)] // Automatically runs this setup when the WASM module loads
pub fn main_setup() {
    console_error_panic_hook::set_once();
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub async fn init(canvas_id: &str) -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    web_sys::console::log_1(&format!("tracks-renderer wasm init on canvas: {}", canvas_id).into());

    // Simply pass the canvas ID string down to Bevy
    crate::bevy_renderer::start_bevy_wasm(canvas_id).await;
    
    Ok(())
}

#[cfg(not(feature = "wasm"))]
pub fn init_native() {
    // Optional: Call native runner here if testing locally
    crate::bevy_renderer::start_bevy();
}