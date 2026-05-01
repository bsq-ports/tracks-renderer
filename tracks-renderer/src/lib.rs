// #![cfg_attr(target_arch = "wasm32", feature(async_closure))]
pub mod bevy_renderer;
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
use wasm_bindgen::JsValue;

#[cfg(feature = "wasm")]
use web_sys;

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub async fn init(canvas_id: &str) -> Result<JsValue, JsValue> {
    console_error_panic_hook::set_once();
    web_sys::console::log_1(&format!("tracks-renderer wasm init: {}", canvas_id).into());

    // Call the async start function in bevy_renderer
    crate::bevy_renderer::start_bevy_wasm(canvas_id).await;
    Ok(JsValue::from_str("ok"))
}

// Native/no-wasm build: provide a no-op initializer
#[cfg(not(feature = "wasm"))]
pub fn init_native() {
    // no-op
}
