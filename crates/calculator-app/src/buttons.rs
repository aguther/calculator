//! Raised buttons keep a stable hit area while their face animates on press.
use eframe::egui::{self, Color32, FontId, Response, Sense, Ui, Vec2, Widget};

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
                let progress = ((now - at - 0.06) / 0.12).clamp(0.0, 1.0) as f32;
                1.0 - progress * progress
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
            ui.painter()
                .rect_filled(rect, self.radius, self.fill.gamma_multiply(0.55));
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
            let galley = ui
                .painter()
                .layout_no_wrap(self.text.to_owned(), self.font, self.color);
            ui.painter()
                .with_clip_rect(ui.clip_rect().intersect(face))
                .galley(face.center() - galley.size() * 0.5, galley, self.color);
        }
        response
    }
}
pub(super) fn face_rect(rect: egui::Rect, amount: f32) -> egui::Rect {
    let depth = (rect.height() * 0.09).clamp(3.5, 6.0);
    egui::Rect::from_min_max(
        rect.min + egui::vec2(1.5 * amount, depth * amount),
        rect.max - egui::vec2(1.5 * amount, depth * (1.0 - amount)),
    )
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
    fn quick_touch_is_visible_immediately_and_after_release() {
        let ctx = egui::Context::default();
        super::super::configure_fonts(&ctx);
        let mut centers = Vec::new();
        for (time, held) in [
            (0.0, false),
            (0.001, true),
            (0.015, false),
            (0.070, false),
            (0.300, false),
        ] {
            let mut center = 0.0;
            let _ = ctx.run(
                egui::RawInput {
                    time: Some(time),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let response = ui.add(
                            PressButton::new(
                                "7",
                                Vec2::new(80.0, 60.0),
                                Color32::GRAY,
                                Color32::WHITE,
                            )
                            .held(held),
                        );
                        center = face_center(ui, &response).y;
                    });
                },
            );
            centers.push(center);
        }
        assert!(
            centers[1] - centers[0] >= 4.0,
            "A short touch must visibly depress immediately: {centers:?}"
        );
        assert!(
            centers[3] - centers[0] >= 3.0,
            "Feedback must survive lifting the finger: {centers:?}"
        );
        assert!(
            (centers[4] - centers[0]).abs() < 0.1,
            "Button must recover: {centers:?}"
        );
    }
    #[test]
    fn pressing_moves_the_face_without_moving_the_hit_target() {
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
            centers[2] > centers[0] + 2.0,
            "Pressed face did not sink: {centers:?}"
        );
        assert!(
            (centers[4] - centers[0]).abs() < 0.1,
            "Released face did not recover: {centers:?}"
        );
    }
}
