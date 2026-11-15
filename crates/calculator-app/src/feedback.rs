#[cfg(not(target_arch = "wasm32"))]
pub fn click() {
    use rodio::Source;
    use std::sync::{Mutex, OnceLock};
    use std::time::Duration;
    static OUTPUT: OnceLock<Mutex<Option<rodio::OutputStream>>> = OnceLock::new();
    let output =
        OUTPUT.get_or_init(|| Mutex::new(rodio::OutputStreamBuilder::open_default_stream().ok()));
    if let Ok(output) = output.lock()
        && let Some(stream) = output.as_ref()
    {
        stream.mixer().add(
            rodio::source::SineWave::new(900.0)
                .take_duration(Duration::from_millis(12))
                .amplify(0.08),
        );
    }
}
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(module = "/src/audio.js")]
extern "C" {
    #[wasm_bindgen::prelude::wasm_bindgen(js_name = configureSound)]
    pub fn configure_sound(enabled: bool);
    #[wasm_bindgen::prelude::wasm_bindgen(js_name = playClick)]
    fn play_click();
}
#[cfg(target_arch = "wasm32")]
pub fn click() {
    play_click();
}
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(inline_js = "
export function hapticsSupported() {
    try { return typeof navigator.vibrate === 'function'; } catch (_) { return false; }
}
export function vibrateSafely() {
    try { if (typeof navigator.vibrate === 'function') navigator.vibrate(8); } catch (_) {}
}
")]
extern "C" {
    #[wasm_bindgen::prelude::wasm_bindgen(js_name = hapticsSupported)]
    fn haptics_supported() -> bool;
    #[wasm_bindgen::prelude::wasm_bindgen(js_name = vibrateSafely)]
    fn vibrate_safely();
}

pub fn supports_haptics() -> bool {
    #[cfg(target_arch = "wasm32")]
    return haptics_supported();
    #[cfg(not(target_arch = "wasm32"))]
    false
}

pub fn vibrate() {
    #[cfg(target_arch = "wasm32")]
    vibrate_safely();
}
