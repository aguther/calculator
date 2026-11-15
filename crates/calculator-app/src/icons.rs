//! Original Lucide SVG geometry, rendered identically on desktop and WASM.
use eframe::egui::{self, Color32, Vec2};
pub enum Icon {
    Undo,
    Delete,
}
pub fn paint(ui: &egui::Ui, center: egui::Pos2, size: f32, icon: Icon, color: Color32) {
    let (uri, svg) = match icon {
        Icon::Undo => (
            "bytes://lucide-undo-2.svg",
            include_str!("../../../assets/icons/undo-2.svg"),
        ),
        Icon::Delete => (
            "bytes://lucide-delete.svg",
            include_str!("../../../assets/icons/delete.svg"),
        ),
    };
    // A white base allows egui tinting while preserving the original SVG paths.
    let bytes = svg.replace("currentColor", "#ffffff").into_bytes();
    egui::Image::from_bytes(uri, bytes)
        .tint(color)
        .paint_at(ui, egui::Rect::from_center_size(center, Vec2::splat(size)));
}
