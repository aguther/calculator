use eframe::{App, egui};
struct Smoke {
    app: calculator::CalculatorApp,
    frames: u8,
}
impl App for Smoke {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.app.update(ctx, frame);
        self.frames += 1;
        if self.frames >= 3 {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        } else {
            ctx.request_repaint();
        }
    }
}
fn main() -> eframe::Result {
    eframe::run_native(
        "Calculator smoke",
        eframe::NativeOptions::default(),
        Box::new(|cc| {
            Ok(Box::new(Smoke {
                app: calculator::CalculatorApp::new(cc),
                frames: 0,
            }))
        }),
    )
}
