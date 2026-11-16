//! Native window startup and asynchronous browser canvas initialization.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../../../web/icon-512.png"))
        .expect("embedded icon");
    eframe::run_native(
        "Calculator",
        eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default()
                .with_inner_size([400.0, 760.0])
                .with_min_inner_size([300.0, 400.0])
                .with_icon(icon),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(calculator::CalculatorApp::new(cc)))),
    )
}
#[cfg(target_arch = "wasm32")]
fn main() {
    use wasm_bindgen::JsCast;
    wasm_bindgen_futures::spawn_local(async {
        let canvas = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("calculator-canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        let result = eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|cc| Ok(Box::new(calculator::CalculatorApp::new(cc)))),
            )
            .await;
        if let Some(doc) = web_sys::window().and_then(|w| w.document())
            && let Some(el) = doc.get_element_by_id("loading")
        {
            if result.is_ok() {
                el.remove();
            } else {
                let message = match calculator::Language::system() {
                    calculator::Language::German => {
                        "Calculator konnte nicht starten. Bitte WebGL aktivieren und neu laden."
                    }
                    calculator::Language::English => {
                        "Calculator could not start. Enable WebGL and reload."
                    }
                };
                el.set_text_content(Some(message));
            }
        }
    });
}
