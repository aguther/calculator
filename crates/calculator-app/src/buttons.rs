//! Flat buttons retain their hit area while a quick scale and contrast change marks a press.
use eframe::egui::{self, Color32, FontId, Response, Sense, Ui, Vec2, Widget};

/// Match the reference web UI without creating a raised face or changing hitboxes.
const PRESSED_SCALE: f32 = 0.92;
const RELEASE_DURATION_SECONDS: f64 = 0.06;

/// A flat, keyboard-accessible button with short-lived visual press feedback.
pub(super) struct PressButton<'a> {
    text: &'a str,
    size: Vec2,
    fill: Color32,
    color: Color32,
    font: FontId,
    radius: f32,
    held: bool,
    selected: bool,
}
impl<'a> PressButton<'a> {
    pub fn new(text: &'a str, size: Vec2, fill: Color32, color: Color32) -> Self {
        Self {
            text,
            size,
            fill,
            color,
            font: super::bold_font(14.0),
            radius: 12.0,
            held: false,
            selected: false,
        }
    }
    pub fn font(mut self, font: FontId) -> Self {
        self.font = font;
        self
    }
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }
    pub fn held(mut self, held: bool) -> Self {
        self.held = held;
        self
    }
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}
#[derive(Clone, Copy, Default)]
struct PressFeedback {
    held: bool,
    released_at: Option<f64>,
}
impl Widget for PressButton<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(self.size, Sense::click());
        response.widget_info(|| {
            egui::WidgetInfo::selected(
                egui::WidgetType::Button,
                ui.is_enabled(),
                self.selected,
                self.text,
            )
        });
        let pressed = ui.is_enabled()
            && (response.is_pointer_button_down_on() || self.held || response.clicked());
        let now = ui.input(|input| input.time);
        let state_id = response.id.with("press-feedback");
        let mut feedback = ui
            .ctx()
            .data(|d| d.get_temp::<PressFeedback>(state_id))
            .unwrap_or_default();
        if pressed && !feedback.held {
            // A complete tap can arrive in one frame: schedule the release frame
            // even when no further input arrives to wake the renderer.
            ui.ctx().request_repaint();
        }
        if pressed {
            feedback.released_at = None;
        } else if feedback.held {
            feedback.released_at = Some(now);
        }
        feedback.held = pressed;
        let amount = if pressed {
            1.0
        } else {
            feedback.released_at.map_or(0.0, |at| {
                let progress = ((now - at) / RELEASE_DURATION_SECONDS).clamp(0.0, 1.0) as f32;
                (1.0 - progress).powi(2)
            })
        };
        ui.ctx().data_mut(|d| d.insert_temp(state_id, feedback));
        if !pressed && amount > 0.0 {
            ui.ctx().request_repaint();
        }
        let face = face_rect(rect, amount);
        ui.ctx()
            .data_mut(|d| d.insert_temp(response.id.with("face-center"), face.center()));
        if ui.is_rect_visible(rect) {
            let fill = if amount > 0.0 {
                self.fill.lerp_to_gamma(self.color, 0.16 * amount)
            } else if response.hovered() {
                self.fill.lerp_to_gamma(Color32::WHITE, 0.05)
            } else {
                self.fill
            };
            ui.painter().rect_filled(face, self.radius, fill);
            if self.selected || response.has_focus() {
                ui.painter().rect_stroke(
                    face.shrink(1.0),
                    self.radius,
                    egui::Stroke::new(
                        if response.has_focus() {
                            2.0_f32
                        } else {
                            1.0_f32
                        },
                        self.color,
                    ),
                    egui::StrokeKind::Inside,
                );
            }
            let mut font = self.font;
            font.size *= 1.0 - (1.0 - PRESSED_SCALE) * amount;
            let galley = ui
                .painter()
                .layout_no_wrap(self.text.to_owned(), font, self.color);
            ui.painter()
                .with_clip_rect(ui.clip_rect().intersect(face))
                .galley(face.center() - galley.size() * 0.5, galley, self.color);
        }
        response
    }
}
pub(super) fn face_rect(rect: egui::Rect, amount: f32) -> egui::Rect {
    rect.shrink2(rect.size() * ((1.0 - PRESSED_SCALE) * 0.5 * amount))
}
pub(super) fn face_center(ui: &Ui, response: &Response) -> egui::Pos2 {
    ui.ctx()
        .data(|d| d.get_temp(response.id.with("face-center")))
        .unwrap_or(response.rect.center())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn resting_button_is_flat_and_press_scales_about_its_center() {
        let rect = egui::Rect::from_min_size(egui::pos2(10.0, 20.0), Vec2::new(100.0, 60.0));
        assert_eq!(
            face_rect(rect, 0.0),
            rect,
            "Resting buttons must fill their full area without a raised face"
        );
        let pressed = face_rect(rect, 1.0);
        assert_eq!(pressed.center(), rect.center());
        assert!((pressed.width() - 92.0).abs() < 0.1);
        assert!((pressed.height() - 55.2).abs() < 0.1);
    }
    #[test]
    fn quick_touch_is_visible_immediately_and_after_release() {
        let ctx = egui::Context::default();
        super::super::configure_fonts(&ctx);
        let mut widths = Vec::new();
        for (time, held) in [
            (0.0, false),
            (0.001, true),
            (0.015, false),
            (0.035, false),
            (0.090, false),
        ] {
            let mut hit = egui::Rect::NOTHING;
            let output = ctx.run(
                egui::RawInput {
                    time: Some(time),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        hit = ui
                            .add(
                                PressButton::new(
                                    "7",
                                    Vec2::new(80.0, 60.0),
                                    Color32::GRAY,
                                    Color32::WHITE,
                                )
                                .held(held),
                            )
                            .rect;
                    });
                },
            );
            let faces: Vec<_> = output
                .shapes
                .iter()
                .filter_map(|clipped| {
                    if let egui::Shape::Rect(shape) = &clipped.shape
                        && hit.contains_rect(shape.rect)
                    {
                        Some(shape.rect)
                    } else {
                        None
                    }
                })
                .collect();
            assert_eq!(
                faces.len(),
                1,
                "Flat buttons must not paint a second shadow or raised base"
            );
            let width = faces[0].width();
            widths.push(width);
        }
        assert!(
            widths[0] - widths[1] >= 4.0,
            "A short touch must visibly scale immediately: {widths:?}"
        );
        assert!(
            widths[0] - widths[3] >= 0.5,
            "Feedback must survive lifting the finger: {widths:?}"
        );
        assert!(
            (widths[4] - widths[0]).abs() < 0.1,
            "Button must recover within a quick tap: {widths:?}"
        );
    }
    #[test]
    fn pressing_keeps_the_face_center_and_hit_target_stable() {
        let ctx = egui::Context::default();
        super::super::configure_fonts(&ctx);
        let mut centers = Vec::new();
        let mut hit = None;
        for (time, held) in [
            (0.0, false),
            (0.1, true),
            (0.2, true),
            (0.3, false),
            (0.5, false),
        ] {
            let mut y = 0.0;
            let _ = ctx.run(
                egui::RawInput {
                    time: Some(time),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let response = ui.add(
                            PressButton::new(
                                "Test",
                                Vec2::new(100.0, 44.0),
                                Color32::GRAY,
                                Color32::WHITE,
                            )
                            .held(held),
                        );
                        if let Some(rect) = hit {
                            assert_eq!(response.rect, rect);
                        }
                        hit = Some(response.rect);
                        y = face_center(ui, &response).y;
                    });
                },
            );
            centers.push(y);
        }
        assert!(
            (centers[2] - centers[0]).abs() < 0.1,
            "Flat press must stay centered: {centers:?}"
        );
        assert!(
            (centers[4] - centers[0]).abs() < 0.1,
            "Released face did not recover: {centers:?}"
        );
    }
}
