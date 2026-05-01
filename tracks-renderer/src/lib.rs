// #![cfg_attr(target_arch = "wasm32", feature(async_closure))]
pub mod bevy_renderer;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// #[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn init(canvas_id: &str) -> Result<JsValue, JsValue> {
    console_error_panic_hook::set_once();
    web_sys::console::log_1(&format!("tracks-renderer wasm init: {}", canvas_id).into());

    #[cfg(feature = "wasm")]
    {
        // Call the async start function in bevy_renderer
        crate::bevy_renderer::start_bevy_wasm(canvas_id).await;
        Ok(JsValue::from_str("ok"))
    }

    #[cfg(not(feature = "wasm"))]
    {
        web_sys::console::warn_1(&"tracks-renderer compiled without `wasm` feature".into());
        Ok(JsValue::from_str("no-wasm-feature"))
    }
}

// When not wasm, nothing to export here.
#[cfg(not(target_arch = "wasm32"))]
pub fn init_native() {
    // no-op
}
