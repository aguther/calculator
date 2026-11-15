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
pub fn click() {
    use web_sys::AudioContext;
    thread_local! {static AUDIO:Option<AudioContext>=AudioContext::new().ok();}
    AUDIO.with(|audio| {
        if let Some(audio) = audio {
            let _ = audio.resume();
            if let (Ok(osc), Ok(gain)) = (audio.create_oscillator(), audio.create_gain()) {
                osc.frequency().set_value(900.0);
                gain.gain().set_value(0.04);
                let _ = osc.connect_with_audio_node(&gain);
                let _ = gain.connect_with_audio_node(&audio.destination());
                let _ = osc.start();
                let _ = osc.stop_with_when(audio.current_time() + 0.012);
            }
        }
    });
}
pub fn vibrate() {
    #[cfg(target_arch = "wasm32")]
    if let Some(window) = web_sys::window() {
        window.navigator().vibrate_with_duration(8);
    }
}
