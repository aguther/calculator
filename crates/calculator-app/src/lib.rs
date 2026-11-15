//! Shared desktop and browser UI.
use calculator_core::{Action, Calculator, Mode, Step, format_number};
use eframe::egui::{self, Color32, RichText, Vec2};
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const COMMIT: &str = env!("CALCULATOR_COMMIT");
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    #[default]
    Original,
    Light,
    Dark,
}
impl Theme {
    pub const ALL: &[Self] = &[Self::Original, Self::Light, Self::Dark];
    pub fn background(self) -> Color32 {
        match self {
            Self::Original => Color32::from_rgb(12, 12, 14),
            Self::Light => Color32::from_rgb(245, 243, 239),
            Self::Dark => Color32::from_rgb(20, 29, 42),
        }
    }
    fn panel(self) -> Color32 {
        match self {
            Self::Original => Color32::from_rgb(22, 22, 26),
            Self::Light => Color32::from_rgb(232, 229, 224),
            Self::Dark => Color32::from_rgb(25, 37, 52),
        }
    }
    fn surface(self) -> Color32 {
        match self {
            Self::Original => Color32::from_rgb(34, 34, 39),
            Self::Light => Color32::from_rgb(255, 253, 249),
            Self::Dark => Color32::from_rgb(34, 49, 66),
        }
    }
    fn accent(self) -> Color32 {
        match self {
            Self::Original => Color32::from_rgb(232, 148, 58),
            Self::Light => Color32::from_rgb(150, 75, 8),
            Self::Dark => Color32::from_rgb(103, 192, 233),
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Original => "Original",
            Self::Light => "Hell",
            Self::Dark => "Dunkel",
        }
    }
    fn apply(self, ctx: &egui::Context) {
        let mode = if self == Self::Light {
            egui::Theme::Light
        } else {
            egui::Theme::Dark
        };
        ctx.set_theme(mode);
        ctx.set_visuals_of(mode, self.visuals());
    }
    fn visuals(self) -> egui::Visuals {
        let mut v = if self == Self::Light {
            egui::Visuals::light()
        } else {
            egui::Visuals::dark()
        };
        v.panel_fill = self.background();
        v.window_fill = self.background();
        v.selection.bg_fill = self.accent();
        v.widgets.active.bg_fill = self.accent();
        if self == Self::Original {
            v.override_text_color = Some(Color32::from_rgb(245, 243, 239));
            v.widgets.inactive.weak_bg_fill = Color32::from_rgb(34, 34, 39);
            v.widgets.inactive.bg_fill = Color32::from_rgb(34, 34, 39);
            v.widgets.hovered.weak_bg_fill = Color32::from_rgb(44, 44, 51);
            v.widgets.hovered.bg_fill = Color32::from_rgb(44, 44, 51);
        }
        v
    }
}
pub struct Layout {
    pub key_height: f32,
    pub tape_height: f32,
    pub content_width: f32,
    pub header_height: f32,
    pub display_height: f32,
    pub paren_height: f32,
    pub total_height: f32,
    pub gap: f32,
    pub landscape: bool,
}
pub fn layout(width: f32, height: f32) -> Layout {
    let landscape = (width >= 740.0 && height >= 500.0) || (width >= 540.0 && width > height);
    let gap = if height < 500.0 { 4.0 } else { 8.0 };
    let header_height = (height * 0.065).clamp(32.0, 48.0);
    let display_height = (height * 0.085).clamp(36.0, 72.0);
    let paren_height = (height * 0.05).clamp(24.0, 42.0);
    let total_height = (height * 0.075).clamp(38.0, 56.0);
    let key_height = if landscape {
        ((height - header_height - display_height - paren_height - gap * 7.0) / 5.0)
            .clamp(28.0, 76.0)
    } else {
        (height * 0.087).clamp(28.0, 72.0).min(
            ((height
                - header_height
                - display_height
                - paren_height
                - total_height
                - 48.0
                - gap * 8.0)
                / 5.0)
                .max(28.0),
        )
    };
    Layout {
        key_height,
        tape_height: if landscape {
            (height - total_height - 16.0).max(30.0)
        } else {
            (height
                - header_height
                - display_height
                - paren_height
                - total_height
                - key_height * 5.0
                - gap * 8.0)
                .max(30.0)
        },
        content_width: width.min(if landscape { 1100.0 } else { 480.0 }),
        header_height,
        display_height,
        paren_height,
        total_height,
        gap,
        landscape,
    }
}
fn settings_switch(ui: &mut egui::Ui, value: &mut bool, label: &str, enabled: bool) -> bool {
    ui.add_enabled_ui(enabled, |ui| {
        let (rect, mut response) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 44.0), egui::Sense::click());
        if response.clicked() {
            *value = !*value;
            response.mark_changed();
        }
        response.widget_info(|| {
            egui::WidgetInfo::selected(egui::WidgetType::Checkbox, enabled, *value, label)
        });
        let amount = ui.ctx().animate_bool(response.id, *value);
        let track = egui::Rect::from_center_size(
            egui::pos2(rect.right() - 25.0, rect.center().y),
            Vec2::new(50.0, 30.0),
        );
        let off = ui.visuals().weak_text_color().gamma_multiply(0.35);
        let on = Color32::from_rgb(48, 178, 91);
        let fill = if *value { on } else { off };
        ui.painter().rect_filled(
            track,
            15.0,
            fill.gamma_multiply(if enabled { 1.0 } else { 0.35 }),
        );
        ui.painter().circle_filled(
            egui::pos2(track.left() + 15.0 + 20.0 * amount, track.center().y),
            12.0,
            if enabled {
                Color32::WHITE
            } else {
                Color32::GRAY
            },
        );
        paint_text(
            ui,
            egui::Rect::from_min_max(rect.min, egui::pos2(track.left() - 12.0, rect.bottom())),
            label,
            14.0,
            ui.visuals().text_color(),
            false,
            egui::Align::Min,
        );
        if response.has_focus() {
            ui.painter().rect_stroke(
                track.expand(3.0),
                18.0,
                egui::Stroke::new(2.0_f32, ui.visuals().selection.stroke.color),
                egui::StrokeKind::Outside,
            );
        }
        response.changed()
    })
    .inner
}
fn bold_font(size: f32) -> egui::FontId {
    egui::FontId::new(size, egui::FontFamily::Name("semibold".into()))
}
fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "regular".into(),
        egui::FontData::from_static(include_bytes!("../../../assets/fonts/NotoSans-Regular.ttf"))
            .into(),
    );
    fonts.font_data.insert(
        "semibold".into(),
        egui::FontData::from_static(include_bytes!(
            "../../../assets/fonts/NotoSans-SemiBold.ttf"
        ))
        .into(),
    );
    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "regular".into());
    let mut bold = fonts.families[&egui::FontFamily::Proportional].clone();
    bold.insert(0, "semibold".into());
    fonts
        .families
        .insert(egui::FontFamily::Name("semibold".into()), bold);
    ctx.set_fonts(fonts);
}
fn paint_text(
    ui: &egui::Ui,
    rect: egui::Rect,
    text: &str,
    size: f32,
    color: Color32,
    bold: bool,
    align: egui::Align,
) {
    if rect.width() <= 0.0 || rect.height() <= 0.0 {
        return;
    }
    ui.interact(
        rect,
        ui.id()
            .with(("text", text, rect.min.x.to_bits(), rect.min.y.to_bits())),
        egui::Sense::hover(),
    )
    .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, text));
    let mut font = if bold {
        bold_font(size)
    } else {
        egui::FontId::proportional(size)
    };
    let mut galley = ui
        .painter()
        .layout_no_wrap(text.to_owned(), font.clone(), color);
    if galley.size().x > rect.width() {
        font.size = (size * rect.width() / galley.size().x).max(9.0);
        galley = ui.painter().layout_no_wrap(text.to_owned(), font, color);
    }
    let x = match align {
        egui::Align::Min => rect.left(),
        egui::Align::Center => rect.center().x - galley.size().x * 0.5,
        egui::Align::Max => rect.right() - galley.size().x,
    };
    ui.painter()
        .with_clip_rect(ui.clip_rect().intersect(rect))
        .galley(
            egui::pos2(x, rect.center().y - galley.size().y * 0.5),
            galley,
            color,
        );
}
fn child_ui(ui: &mut egui::Ui, rect: egui::Rect, salt: &'static str) -> egui::Ui {
    ui.new_child(
        egui::UiBuilder::new()
            .id_salt(salt)
            .max_rect(rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    )
}
#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct CalculatorApp {
    pub calculator: Calculator,
    pub theme: Theme,
    pub sound: bool,
    pub haptics: bool,
    pub history_right: bool,
    #[serde(skip)]
    settings_open: bool,
    #[serde(skip)]
    dirty: bool,
    #[serde(skip)]
    last_status: String,
    #[serde(skip)]
    last_saved: Option<String>,
    #[serde(skip)]
    touch: touch::TouchController,
    #[serde(skip)]
    targets: Vec<touch::Target>,
    #[serde(skip)]
    touch_frame: bool,
    #[serde(skip)]
    viewport_size: Option<Vec2>,
}
impl Default for CalculatorApp {
    fn default() -> Self {
        Self {
            calculator: Calculator::default(),
            theme: Theme::default(),
            sound: false,
            haptics: false,
            history_right: true,
            settings_open: false,
            dirty: false,
            last_status: String::new(),
            last_saved: None,
            touch: touch::TouchController::default(),
            targets: Vec::new(),
            touch_frame: false,
            viewport_size: None,
        }
    }
}
impl CalculatorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let stored = cc
            .storage
            .and_then(|storage| storage.get_string("calculator-v1"));
        let app = Self::restore(stored.as_deref());
        configure_fonts(&cc.egui_ctx);
        app.theme.apply(&cc.egui_ctx);
        app
    }
    fn restore(data: Option<&str>) -> Self {
        let mut app: Self = data
            .and_then(|data| serde_json::from_str(data).ok())
            .unwrap_or_default();
        app.calculator.validate();
        if data.is_some() {
            app.last_saved = serde_json::to_string(&app).ok();
        }
        app
    }
    pub fn dispatch(&mut self, action: Action) {
        if matches!(action, Action::Mode(_)) {
            self.touch.cancel();
        }
        if self.calculator.apply(action) {
            self.dirty = true;
            if self.sound {
                crate::feedback::click();
            }
            if self.haptics {
                crate::feedback::vibrate();
            }
        }
    }
    fn keyboard(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(egui::Key::F1)) {
            self.settings_open = !self.settings_open;
            return;
        }
        if self.settings_open {
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.settings_open = false;
            }
            return;
        }
        let events = ctx.input(|i| i.events.clone());
        for event in events {
            let action = match event {
                egui::Event::Text(text) => {
                    for c in text.chars() {
                        if let Some(action) = Action::from_char(c) {
                            self.dispatch(action);
                        }
                    }
                    None
                }
                egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } if !modifiers.ctrl && !modifiers.command && !modifiers.alt => match key {
                    egui::Key::Enter => Some(Action::Equals),
                    egui::Key::Backspace => Some(Action::Backspace),
                    egui::Key::Delete => Some(Action::ClearEntry),
                    egui::Key::Escape => Some(Action::Clear),
                    _ => None,
                },
                _ => None,
            };
            if let Some(action) = action {
                self.dispatch(action);
            }
        }
    }
    fn key(
        &mut self,
        ui: &mut egui::Ui,
        label: &str,
        action: Action,
        size: Vec2,
        accent: bool,
        enabled: bool,
    ) {
        let danger = match self.theme {
            Theme::Original => Color32::from_rgb(210, 91, 80),
            Theme::Light => Color32::from_rgb(174, 49, 39),
            Theme::Dark => Color32::from_rgb(238, 142, 133),
        };
        let danger_fill = match self.theme {
            Theme::Original => Color32::from_rgb(58, 32, 29),
            Theme::Light => Color32::from_rgb(248, 221, 216),
            Theme::Dark => Color32::from_rgb(73, 43, 47),
        };
        let operator_fill = match self.theme {
            Theme::Original => Color32::from_rgb(58, 44, 24),
            Theme::Light => Color32::from_rgb(247, 231, 208),
            Theme::Dark => Color32::from_rgb(28, 57, 73),
        };
        let success = match self.theme {
            Theme::Original => Color32::from_rgb(79, 157, 97),
            Theme::Light => Color32::from_rgb(46, 125, 65),
            Theme::Dark => Color32::from_rgb(90, 166, 106),
        };
        let base = if accent {
            self.theme.accent()
        } else {
            ui.visuals().text_color()
        };
        let (foreground, background) = match action {
            Action::Clear => (danger, Some(danger_fill)),
            Action::ClearEntry => (danger, None),
            Action::Backspace => (ui.visuals().weak_text_color(), None),
            Action::Operator(op) if self.calculator.state().pending == Some(op) => {
                (Color32::BLACK, Some(self.theme.accent()))
            }
            Action::Operator(_) => (self.theme.accent(), Some(operator_fill)),
            Action::Equals => (
                if self.theme == Theme::Light {
                    Color32::WHITE
                } else {
                    Color32::from_rgb(12, 26, 15)
                },
                Some(success),
            ),
            _ => (base, None),
        };
        let text = RichText::new(if matches!(action, Action::Backspace) {
            ""
        } else {
            label
        })
        .font(bold_font((size.y * 0.4).clamp(15.0, 28.0)))
        .color(foreground);
        let text = if matches!(
            action,
            Action::Clear | Action::ClearEntry | Action::Operator(_)
        ) {
            text.strong()
        } else {
            text
        };
        let mut button = egui::Button::new(text).corner_radius(15.0);
        if let Some(background) = background {
            button = button.fill(background);
        }
        let response = ui
            .add_enabled_ui(enabled, |ui| ui.add_sized(size, button))
            .inner;
        if matches!(action, Action::Backspace) {
            response.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::WidgetType::Button,
                    enabled,
                    "Letzte Stelle löschen",
                )
            });
            let center = response.rect.center();
            let h = (size.y * 0.32).clamp(12.0, 24.0);
            let w = h * 1.5;
            let stroke = egui::Stroke::new(1.7_f32, foreground);
            let points = vec![
                egui::pos2(center.x - w * 0.5, center.y),
                egui::pos2(center.x - w * 0.2, center.y - h * 0.5),
                egui::pos2(center.x + w * 0.5, center.y - h * 0.5),
                egui::pos2(center.x + w * 0.5, center.y + h * 0.5),
                egui::pos2(center.x - w * 0.2, center.y + h * 0.5),
            ];
            ui.painter().add(egui::Shape::closed_line(points, stroke));
            let x = center.x + w * 0.12;
            let d = h * 0.2;
            ui.painter().line_segment(
                [
                    egui::pos2(x - d, center.y - d),
                    egui::pos2(x + d, center.y + d),
                ],
                stroke,
            );
            ui.painter().line_segment(
                [
                    egui::pos2(x - d, center.y + d),
                    egui::pos2(x + d, center.y - d),
                ],
                stroke,
            );
        }
        if enabled {
            self.targets.push(touch::Target {
                rect: response.rect,
                action,
                group: if matches!(action, Action::Open | Action::Close) {
                    1
                } else {
                    0
                },
            });
        }
        if response.clicked() && !self.touch_frame {
            self.dispatch(action);
        }
    }
    fn keypad(&mut self, ui: &mut egui::Ui, l: &Layout) {
        let gap = l.gap;
        let width = (ui.available_width() - gap * 3.0) / 4.0;
        let size = Vec2::new(width, l.key_height);
        let rows = [
            [
                ("AC", Action::Clear),
                ("C", Action::ClearEntry),
                ("⌫", Action::Backspace),
                ("÷", Action::Operator('/')),
            ],
            [
                ("7", Action::Digit('7')),
                ("8", Action::Digit('8')),
                ("9", Action::Digit('9')),
                ("×", Action::Operator('*')),
            ],
            [
                ("4", Action::Digit('4')),
                ("5", Action::Digit('5')),
                ("6", Action::Digit('6')),
                ("−", Action::Operator('-')),
            ],
            [
                ("1", Action::Digit('1')),
                ("2", Action::Digit('2')),
                ("3", Action::Digit('3')),
                ("+", Action::Operator('+')),
            ],
        ];
        ui.spacing_mut().item_spacing = Vec2::splat(gap);
        for row in rows {
            ui.horizontal(|ui| {
                for (label, action) in row {
                    self.key(
                        ui,
                        label,
                        action,
                        size,
                        matches!(action, Action::Operator(_)),
                        true,
                    );
                }
            });
        }
        ui.horizontal(|ui| {
            self.key(
                ui,
                "0",
                Action::Digit('0'),
                Vec2::new(width * 2.0 + gap, l.key_height),
                false,
                true,
            );
            self.key(
                ui,
                if self.calculator.mode == Mode::Time {
                    ":"
                } else {
                    "."
                },
                Action::Separator,
                size,
                true,
                true,
            );
            self.key(
                ui,
                "=",
                Action::Equals,
                size,
                true,
                self.calculator.can_equal(),
            );
        });
    }
    fn tape(&self, ui: &mut egui::Ui, height: f32) {
        egui::ScrollArea::vertical()
            .id_salt("tape")
            .auto_shrink([false, false])
            .max_height(height)
            .min_scrolled_height(height)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                let mut number = 0;
                let mut depth = 0usize;
                let mut groups = Vec::new();
                for (i, step) in self.calculator.state().steps.iter().enumerate() {
                    if matches!(step, Step::Close) {
                        depth = depth.saturating_sub(1);
                    }
                    let (operator, label, minutes) = match step {
                        Step::Value { op, value, scalar } => {
                            number += 1;
                            (
                                op.map(symbol).unwrap_or("").to_string(),
                                self.calculator.label(*value, *scalar),
                                if self.calculator.mode == Mode::Time && !*scalar {
                                    Some(format_number(*value / 60.0))
                                } else {
                                    None
                                },
                            )
                        }
                        Step::Sum(value) => (
                            "=".into(),
                            self.calculator.label(*value, false),
                            (self.calculator.mode == Mode::Time)
                                .then(|| format_number(*value / 60.0)),
                        ),
                        Step::Open(op) => {
                            groups.push(i);
                            (
                                op.map(symbol).unwrap_or("").to_string(),
                                "Gruppe".into(),
                                None,
                            )
                        }
                        Step::Close => {
                            let start = groups.pop().unwrap_or(i);
                            let mut steps = self.calculator.state().steps[start..=i].to_vec();
                            if let Some(Step::Open(op)) = steps.first_mut() {
                                *op = None;
                            }
                            let value = calculator_core::evaluate(&steps);
                            (
                                "=".into(),
                                self.calculator.label(value, false),
                                (self.calculator.mode == Mode::Time)
                                    .then(|| format_number(value / 60.0)),
                            )
                        }
                    };
                    let sum = matches!(step, Step::Sum(_) | Step::Close);
                    let group_header = matches!(step, Step::Open(_));
                    let row_height = 32.0;
                    let (mut rect, _) = ui.allocate_exact_size(
                        Vec2::new(ui.available_width(), row_height),
                        egui::Sense::hover(),
                    );
                    let base_left = rect.left();
                    let visible_depth = depth.min(8);
                    let indentation =
                        (visible_depth as f32 * 12.0).min((rect.width() - 220.0).max(0.0));
                    let rail_step = if visible_depth > 0 {
                        indentation / visible_depth as f32
                    } else {
                        0.0
                    };
                    rect.min.x += indentation;
                    for level in 0..visible_depth {
                        let x = base_left + level as f32 * rail_step + 2.0;
                        ui.painter().vline(
                            x,
                            rect.top() - 3.0..=rect.bottom() + 3.0,
                            egui::Stroke::new(2.0_f32, self.theme.accent().gamma_multiply(0.4)),
                        );
                    }
                    let fill = if sum || group_header {
                        self.theme.accent().gamma_multiply(0.12)
                    } else {
                        self.theme.surface()
                    };
                    ui.painter().rect_filled(rect, 11.0, fill);
                    if sum || group_header {
                        ui.painter().rect_stroke(
                            rect,
                            11.0,
                            egui::Stroke::new(1.0_f32, self.theme.accent().gamma_multiply(0.35)),
                            egui::StrokeKind::Inside,
                        );
                    }
                    let inner = rect.shrink2(Vec2::new(10.0, 0.0));
                    if matches!(step, Step::Value { .. }) {
                        let badge = egui::Rect::from_center_size(
                            egui::pos2(inner.left() + 10.0, inner.center().y),
                            Vec2::splat(22.0),
                        );
                        ui.painter().circle_filled(
                            badge.center(),
                            11.0,
                            ui.visuals().weak_text_color().gamma_multiply(0.12),
                        );
                        paint_text(
                            ui,
                            badge,
                            &number.to_string(),
                            11.0,
                            ui.visuals().weak_text_color(),
                            true,
                            egui::Align::Center,
                        );
                    }
                    let op_rect = egui::Rect::from_min_max(
                        egui::pos2(inner.left() + 26.0, inner.top()),
                        egui::pos2(inner.left() + 50.0, inner.bottom()),
                    );
                    paint_text(
                        ui,
                        op_rect,
                        &operator,
                        21.0,
                        self.theme.accent(),
                        true,
                        egui::Align::Center,
                    );
                    let value_width = (inner.width() * 0.32)
                        .clamp(78.0, 180.0)
                        .min((inner.width() - 54.0).max(20.0));
                    let value_rect = egui::Rect::from_min_max(
                        egui::pos2(inner.right() - value_width, inner.top()),
                        inner.max,
                    );
                    paint_text(
                        ui,
                        value_rect,
                        &label,
                        if group_header { 15.0 } else { 21.0 },
                        if sum {
                            self.theme.accent()
                        } else if group_header {
                            ui.visuals().weak_text_color()
                        } else {
                            ui.visuals().text_color()
                        },
                        true,
                        egui::Align::Max,
                    );
                    if let Some(minutes) = minutes {
                        let minutes_rect = egui::Rect::from_min_max(
                            egui::pos2(op_rect.right() + 4.0, inner.top()),
                            egui::pos2(value_rect.left() - 12.0, inner.bottom()),
                        );
                        paint_text(
                            ui,
                            minutes_rect,
                            &format!("{minutes} min"),
                            12.0,
                            ui.visuals().weak_text_color(),
                            true,
                            egui::Align::Max,
                        );
                    }
                    if matches!(step, Step::Open(_)) {
                        depth += 1;
                    }
                }
            });
    }
    fn mode_bar(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        ui.painter().rect_filled(rect, 16.0, self.theme.panel());
        let inset = rect.shrink(5.0);
        let info_width = inset.height();
        #[cfg(target_arch = "wasm32")]
        if let Some(window) = web_sys::window()
            && let Some(document) = window.document()
            && let Some(button) = document.get_element_by_id("update")
            && let Some(canvas) = document.get_element_by_id("calculator-canvas")
        {
            let bounds = canvas.get_bounding_client_rect();
            let scale = bounds.width() as f32 / ui.ctx().content_rect().width();
            let x = bounds.left() as f32 + (inset.right() - info_width - 40.0) * scale;
            let y = bounds.top() as f32 + (rect.center().y - 16.0) * scale;
            let visibility = if self.settings_open {
                "hidden"
            } else {
                "visible"
            };
            let _ = button.set_attribute(
                "style",
                &format!("left:{x}px;top:{y}px;visibility:{visibility}"),
            );
        }
        let segment_width = (inset.width() - info_width - 48.0) * 0.5;
        for (index, mode, label) in [(0, Mode::Time, "Zeit"), (1, Mode::Number, "Rechner")] {
            let r = egui::Rect::from_min_size(
                egui::pos2(inset.left() + index as f32 * segment_width, inset.top()),
                Vec2::new(segment_width, inset.height()),
            );
            let selected = self.calculator.mode == mode;
            let response =
                ui.put(
                    r,
                    egui::Button::new(RichText::new(label).font(bold_font(15.0)).color(
                        if selected {
                            ui.visuals().text_color()
                        } else {
                            ui.visuals().weak_text_color()
                        },
                    ))
                    .fill(if selected {
                        self.theme.surface()
                    } else {
                        Color32::TRANSPARENT
                    })
                    .stroke(egui::Stroke::NONE)
                    .corner_radius(12.0),
                );
            if response.clicked() {
                self.dispatch(Action::Mode(mode));
            }
        }
        let r = egui::Rect::from_min_size(
            egui::pos2(inset.right() - info_width, inset.top()),
            Vec2::splat(info_width),
        );
        let response = ui.put(
            r,
            egui::Button::new(RichText::new("i").font(bold_font(19.0)))
                .fill(self.theme.surface())
                .corner_radius(100.0),
        );
        if response
            .on_hover_text("Informationen und Einstellungen")
            .clicked()
        {
            self.settings_open = true;
        }
    }
    fn total_bar(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        let r = rect.shrink2(Vec2::new(12.0, 0.0));
        ui.painter().hline(
            rect.x_range(),
            rect.top(),
            egui::Stroke::new(1.0_f32, ui.visuals().weak_text_color().gamma_multiply(0.2)),
        );
        let total = self.calculator.label(self.calculator.total(), false);
        let value_width = (r.width() * 0.3).clamp(76.0, 160.0);
        let value_rect =
            egui::Rect::from_min_max(egui::pos2(r.right() - value_width, r.top()), r.max);
        paint_text(
            ui,
            value_rect,
            &total,
            25.0,
            self.theme.accent(),
            true,
            egui::Align::Max,
        );
        let undo_rect = egui::Rect::from_center_size(
            egui::pos2(value_rect.left() - 23.0, r.center().y),
            Vec2::splat(34.0),
        );
        let undo = ui.put(
            undo_rect,
            egui::Button::new("")
                .fill(self.theme.surface())
                .corner_radius(100.0),
        );
        undo.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "Letzten Schritt zurück")
        });
        if undo.on_hover_text("Letzten Schritt zurück").clicked() {
            self.dispatch(Action::Undo);
        }
        let center = undo_rect.center();
        let stroke = egui::Stroke::new(2.0_f32, ui.visuals().weak_text_color());
        let points = (0..=16)
            .map(|index| {
                let angle = -2.4 + index as f32 * 4.7 / 16.0;
                center + Vec2::angled(angle) * 7.0
            })
            .collect();
        ui.painter().add(egui::Shape::line(points, stroke));
        let tip = center + Vec2::angled(-2.4) * 7.0;
        ui.painter()
            .line_segment([tip, tip + Vec2::new(-1.0, -5.0)], stroke);
        ui.painter()
            .line_segment([tip, tip + Vec2::new(5.0, -1.0)], stroke);
        paint_text(
            ui,
            egui::Rect::from_min_max(r.min, egui::pos2(r.left() + 60.0, r.bottom())),
            "SUMME",
            11.0,
            ui.visuals().weak_text_color(),
            true,
            egui::Align::Min,
        );
        if self.calculator.mode == Mode::Time {
            let minutes = egui::Rect::from_min_max(
                egui::pos2(r.left() + 64.0, r.top()),
                egui::pos2(undo_rect.left() - 6.0, r.bottom()),
            );
            if minutes.width() > 26.0 {
                paint_text(
                    ui,
                    minutes,
                    &format!("{} min", format_number(self.calculator.total() / 60.0)),
                    12.0,
                    ui.visuals().weak_text_color(),
                    true,
                    egui::Align::Max,
                );
            }
        }
    }
    fn history_panel(&mut self, ui: &mut egui::Ui, rect: egui::Rect, l: &Layout) {
        ui.painter().rect_filled(rect, 18.0, self.theme.panel());
        let tape_rect = egui::Rect::from_min_max(
            rect.min + Vec2::splat(8.0),
            egui::pos2(rect.right() - 8.0, rect.bottom() - l.total_height - 4.0),
        );
        let mut tape_ui = child_ui(ui, tape_rect, "history");
        tape_ui.spacing_mut().item_spacing.y = 2.0;
        self.tape(&mut tape_ui, tape_rect.height().max(0.0));
        self.total_bar(
            ui,
            egui::Rect::from_min_max(
                egui::pos2(rect.left(), rect.bottom() - l.total_height),
                rect.max,
            ),
        );
    }
    fn controls(&mut self, ui: &mut egui::Ui, rect: egui::Rect, l: &Layout) {
        let keypad_height = l.key_height * 5.0 + l.gap * 4.0;
        let keypad_rect = egui::Rect::from_min_max(
            egui::pos2(rect.left(), rect.bottom() - keypad_height),
            rect.max,
        );
        let paren_rect = egui::Rect::from_min_size(
            egui::pos2(rect.left(), keypad_rect.top() - l.gap - l.paren_height),
            Vec2::new(rect.width(), l.paren_height),
        );
        let display_rect = egui::Rect::from_min_size(
            egui::pos2(
                rect.left() + 8.0,
                paren_rect.top() - l.gap - l.display_height,
            ),
            Vec2::new(rect.width() - 16.0, l.display_height),
        );
        let text = self.calculator.current_text();
        paint_text(
            ui,
            display_rect,
            &text,
            if text.len() > 18 {
                25.0
            } else {
                l.display_height * 0.78
            },
            self.theme.accent(),
            true,
            egui::Align::Max,
        );
        if !self.calculator.notice.is_empty() {
            paint_text(
                ui,
                egui::Rect::from_min_size(display_rect.min, Vec2::new(display_rect.width(), 16.0)),
                &self.calculator.notice,
                11.0,
                self.theme.accent(),
                false,
                egui::Align::Min,
            );
        }
        let mut paren_ui = child_ui(ui, paren_rect, "parentheses");
        paren_ui.spacing_mut().item_spacing = Vec2::splat(l.gap);
        paren_ui.horizontal(|ui| {
            let size = Vec2::new((rect.width() - l.gap) * 0.5, l.paren_height);
            self.key(
                ui,
                "(",
                Action::Open,
                size,
                false,
                !self.calculator.state().error,
            );
            self.key(
                ui,
                ")",
                Action::Close,
                size,
                false,
                self.calculator.can_close(),
            );
        });
        self.keypad(&mut child_ui(ui, keypad_rect, "keypad"), l);
    }
    fn calculator_ui(&mut self, ui: &mut egui::Ui, width: f32, height: f32) {
        let l = layout(width, height);
        let size = Vec2::new(l.content_width, height);
        let rect = egui::Rect::from_min_size(
            ui.min_rect().min + Vec2::new((width - size.x) * 0.5, (height - size.y) * 0.5),
            size,
        );
        if l.landscape {
            let column_width = (rect.width() - 16.0) * 0.5;
            let left = egui::Rect::from_min_size(rect.min, Vec2::new(column_width, rect.height()));
            let right =
                egui::Rect::from_min_size(egui::pos2(left.right() + 16.0, rect.top()), left.size());
            let (controls, history) = if self.history_right {
                (left, right)
            } else {
                (right, left)
            };
            self.mode_bar(
                ui,
                egui::Rect::from_min_size(
                    controls.min,
                    Vec2::new(controls.width(), l.header_height),
                ),
            );
            self.controls(ui, controls, &l);
            self.history_panel(ui, history, &l);
        } else {
            self.mode_bar(
                ui,
                egui::Rect::from_min_size(rect.min, Vec2::new(rect.width(), l.header_height)),
            );
            let controls_height =
                l.key_height * 5.0 + l.gap * 6.0 + l.paren_height + l.display_height;
            let history = egui::Rect::from_min_max(
                egui::pos2(rect.left(), rect.top() + l.header_height + l.gap),
                egui::pos2(rect.right(), rect.bottom() - controls_height - l.gap),
            );
            self.history_panel(ui, history, &l);
            self.controls(ui, rect, &l);
        }
        ui.allocate_rect(rect, egui::Sense::hover());
    }
    fn settings(&mut self, ctx: &egui::Context) {
        if !self.settings_open {
            return;
        }
        let mut open = true;
        let width = (ctx.content_rect().width() - 48.0).clamp(220.0, 360.0);
        egui::Window::new("Informationen und Einstellungen")
            .open(&mut open).title_bar(false).collapsible(false).resizable(false)
            .anchor(egui::Align2::CENTER_CENTER,Vec2::ZERO)
            .default_width(width)
            .max_width(width)
            .max_height((ctx.content_rect().height()-100.0).max(160.0))
            .vscroll(true)
            .frame(egui::Frame::new().fill(self.theme.panel()).inner_margin(18.0).corner_radius(20.0))
            .show(ctx,|ui|{
                ui.spacing_mut().item_spacing=Vec2::splat(12.0);
                ui.label(RichText::new("Einstellungen").font(bold_font(23.0)));
                ui.label(RichText::new(format!("Calculator {VERSION} · {}",COMMIT.chars().take(12).collect::<String>())).small().weak());
                ui.add_space(4.0);
                ui.label(RichText::new("Darstellung").font(bold_font(15.0)));
                ui.horizontal_wrapped(|ui|{
                    let width=((ui.available_width()-12.0)/2.0).max(96.0);
                    for &theme in Theme::ALL {
                        let selected=self.theme==theme;
                        if ui.add_sized([width,40.0],egui::Button::new(RichText::new(theme.label()).font(bold_font(14.0)).color(if selected {Color32::from_rgb(20,20,22)}else{ui.visuals().text_color()})).fill(if selected {self.theme.accent()}else{self.theme.surface()}).corner_radius(12.0)).clicked(){self.theme=theme;self.theme.apply(ctx);self.dirty=true;}
                    }
                });
                egui::Frame::new().fill(self.theme.surface()).inner_margin(12.0).corner_radius(12.0).show(ui,|ui|{
                    ui.set_min_width(width-24.0);
                    if settings_switch(ui,&mut self.history_right,"Verlauf rechts im Querformat",true){self.dirty=true;}
                    if settings_switch(ui,&mut self.sound,"Tastenklicks",true){self.dirty=true;}
                    if settings_switch(ui,&mut self.haptics,"Haptik",crate::feedback::supports_haptics()){self.dirty=true;}
                    if !crate::feedback::supports_haptics(){ui.label(RichText::new("Vibration wird auf diesem Gerät nicht unterstützt.").small().weak());}
                });
                ui.collapsing("Bedienung",|ui|{ui.label("Ziffern, + − * /, Klammern und Enter. Rücktaste löscht eine Stelle, Escape alles und Entf die Eingabe.");});
                ui.collapsing("Diagnose",|ui|{
                    let size=ctx.content_rect().size();
                    let text=serde_json::to_string_pretty(&serde_json::json!({"version":VERSION,"commit":COMMIT,"viewport":{"width":size.x,"height":size.y,"pixelsPerPoint":ctx.pixels_per_point()},"theme":self.theme})).unwrap_or_default();
                    ui.label(RichText::new(&text).monospace().small());
                    if ui.button("Diagnose kopieren").clicked(){ctx.copy_text(text);}
                });
                ui.label(RichText::new("Berechnungen und Einstellungen bleiben auf diesem Gerät.").small().weak());
                if ui.add_sized([ui.available_width(),42.0],egui::Button::new(RichText::new("Fertig").font(bold_font(15.0))).fill(self.theme.surface()).corner_radius(12.0)).clicked(){self.settings_open=false;}
            });
        if !open {
            self.settings_open = false;
        }
    }
}
fn symbol(c: char) -> &'static str {
    match c {
        '+' => "+",
        '-' => "−",
        '*' => "×",
        '/' => "÷",
        _ => "",
    }
}
impl eframe::App for CalculatorApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let events = ctx.input(|i| i.events.clone());
        let size = ctx.content_rect().size();
        if self.viewport_size.is_some_and(|old| old != size) {
            self.touch.cancel();
        }
        self.viewport_size = Some(size);
        self.touch_frame = self.touch.is_active()
            || events
                .iter()
                .any(|e| matches!(e, egui::Event::Touch { .. }));
        self.targets.clear();
        self.keyboard(ctx);
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(self.theme.background())
                    .inner_margin(10.0),
            )
            .show(ctx, |ui| {
                if self.settings_open {
                    ui.disable();
                }
                let width = ui.available_width();
                let height = ui.available_height();
                self.calculator_ui(ui, width, height);
            });
        self.settings(ctx);
        if self.settings_open {
            self.touch.cancel();
        } else {
            for event in events {
                match event {
                    egui::Event::Touch { id, phase, pos, .. } => match phase {
                        egui::TouchPhase::Start => self.touch.start(id.0, pos, &self.targets),
                        egui::TouchPhase::End => {
                            if let Some(action) = self.touch.end(id.0, pos, &self.targets) {
                                self.dispatch(action);
                            }
                        }
                        egui::TouchPhase::Cancel => self.touch.cancel_contact(id.0),
                        egui::TouchPhase::Move => {}
                    },
                    egui::Event::WindowFocused(false) => self.touch.cancel(),
                    _ => {}
                }
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            let status = format!(
                "{} | Summe {}",
                self.calculator.current_text(),
                self.calculator.label(self.calculator.total(), false)
            );
            if status != self.last_status {
                if let Some(doc) = web_sys::window().and_then(|w| w.document())
                    && let Some(el) = doc.get_element_by_id("calculator-status")
                {
                    el.set_text_content(Some(&status));
                }
                self.last_status = status;
            }
        }
        if self.dirty {
            if let Some(storage) = frame.storage_mut() {
                self.save(storage);
                storage.flush();
            }
            self.dirty = false;
        }
    }
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Ok(data) = serde_json::to_string(self)
            && self.last_saved.as_ref() != Some(&data)
        {
            // An unchanged tab must not overwrite another tab during an update/reload.
            storage.set_string("calculator-v1", data.clone());
            self.last_saved = Some(data);
        }
    }
    fn auto_save_interval(&self) -> Duration {
        Duration::from_secs(1)
    }
}
mod feedback;
mod touch;

#[cfg(test)]
mod rendering_tests {
    use super::*;
    #[test]
    fn history_side_moves_the_panel_and_keeps_keys_bounded() {
        for (width, height) in [
            (1440.0, 900.0),
            (390.0, 1100.0),
            (844.0, 390.0),
            (568.0, 320.0),
            (400.0, 760.0),
            (320.0, 568.0),
            (300.0, 400.0),
        ] {
            for right in [true, false] {
                let ctx = egui::Context::default();
                configure_fonts(&ctx);
                let mut app = CalculatorApp {
                    history_right: right,
                    ..Default::default()
                };
                for _ in 0..3 {
                    app.targets.clear();
                    let _ = ctx.run(
                        egui::RawInput {
                            screen_rect: Some(egui::Rect::from_min_size(
                                egui::Pos2::ZERO,
                                Vec2::new(width, height),
                            )),
                            ..Default::default()
                        },
                        |ctx| {
                            egui::CentralPanel::default()
                                .frame(egui::Frame::new().inner_margin(10.0))
                                .show(ctx, |ui| {
                                    app.calculator_ui(
                                        ui,
                                        ui.available_width(),
                                        ui.available_height(),
                                    )
                                });
                        },
                    );
                }
                let metrics = layout(width - 20.0, height - 20.0);
                if !metrics.landscape {
                    let history_height = height
                        - 20.0
                        - metrics.header_height
                        - metrics.display_height
                        - metrics.paren_height
                        - metrics.key_height * 5.0
                        - metrics.gap * 8.0;
                    assert!(
                        history_height >= metrics.total_height + 47.9,
                        "No full history row at {width}x{height}"
                    );
                }
                let seven = app
                    .targets
                    .iter()
                    .find(|target| matches!(target.action, Action::Digit('7')))
                    .unwrap();
                if layout(width - 20.0, height - 20.0).landscape {
                    assert_eq!(
                        seven.rect.center().x < width * 0.5,
                        right,
                        "History side ignored at {width}x{height}"
                    );
                }
                for target in &app.targets {
                    assert!(
                        target.rect.top() >= 0.0
                            && target.rect.bottom() <= height
                            && target.rect.left() >= 0.0
                            && target.rect.right() <= width,
                        "Key outside viewport: {:?} at {width}x{height}",
                        target.rect
                    );
                    assert!(target.rect.height() <= 76.1, "Desktop key grew too tall");
                }
                let bottom = app
                    .targets
                    .iter()
                    .map(|t| t.rect.bottom())
                    .fold(0.0_f32, f32::max);
                assert!(
                    height - bottom <= 32.0,
                    "Unused space below keypad at {width}x{height}"
                );
            }
        }
    }
    #[test]
    fn phone_safe_area_keeps_a_simple_group_header_visible() {
        let ctx = egui::Context::default();
        configure_fonts(&ctx);
        let mut app = CalculatorApp::default();
        for c in "(25+25)".chars() {
            app.calculator.apply(Action::from_char(c).unwrap());
        }
        let mut visible = false;
        for _ in 0..3 {
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        Vec2::new(390.0, 751.0),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default()
                        .frame(egui::Frame::new().inner_margin(10.0))
                        .show(ctx, |ui| {
                            app.calculator_ui(ui, ui.available_width(), ui.available_height())
                        });
                },
            );
            visible = output.shapes.iter().any(|clipped| {
                if let egui::Shape::Text(text) = &clipped.shape {
                    text.galley.text() == "Gruppe"
                        && clipped
                            .clip_rect
                            .contains_rect(text.galley.rect.translate(text.pos.to_vec2()))
                } else {
                    false
                }
            });
        }
        assert!(
            visible,
            "Simple group header scrolled out of a normal iPhone safe area"
        );
    }
    #[test]
    fn bottom_keys_remain_visible() {
        for (width, height, long_entry) in [
            (320.0, 568.0, false),
            (400.0, 760.0, false),
            (320.0, 568.0, true),
        ] {
            let ctx = egui::Context::default();
            configure_fonts(&ctx);
            let mut app = CalculatorApp::default();
            if long_entry {
                app.calculator.apply(Action::Mode(Mode::Number));
                app.calculator.apply(Action::Separator);
                for _ in 0..63 {
                    app.calculator.apply(Action::Digit('1'));
                }
            }
            let mut found = false;
            for _ in 0..3 {
                let output = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            Vec2::new(width, height),
                        )),
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| {
                            let content_width = ui.available_width().min(520.0);
                            let height = ui.available_height();
                            let offset = (ui.available_width() - content_width) * 0.5;
                            ui.horizontal_top(|ui| {
                                ui.add_space(offset);
                                ui.allocate_ui_with_layout(
                                    Vec2::new(content_width, height),
                                    egui::Layout::top_down(egui::Align::Min),
                                    |ui| {
                                        app.calculator_ui(ui, content_width, height);
                                    },
                                );
                            });
                        });
                    },
                );
                for egui::epaint::ClippedShape { shape, .. } in output.shapes {
                    if let egui::Shape::Text(text) = shape
                        && text.galley.text() == "="
                    {
                        found = true;
                        assert!(
                            text.pos.y + text.galley.size().y <= height,
                            "Equals key outside viewport: {:?}",
                            text.pos
                        );
                    }
                }
            }
            assert!(found, "Equals key was clipped");
        }
    }
}

#[cfg(test)]
mod touch_tests {
    use super::*;
    use crate::touch::{Target, TouchController};
    #[test]
    fn overlapping_contacts_commit_on_release_and_cancellation_discards() {
        let target = Target {
            rect: egui::Rect::from_min_size(egui::pos2(0.0, 0.0), Vec2::splat(50.0)),
            action: Action::Digit('1'),
            group: 0,
        };
        let other = Target {
            rect: egui::Rect::from_min_size(egui::pos2(55.0, 0.0), Vec2::splat(50.0)),
            action: Action::Digit('2'),
            group: 0,
        };
        let targets = [target, other];
        let mut input = TouchController::default();
        input.start(1, egui::pos2(25.0, 25.0), &targets);
        input.start(2, egui::pos2(75.0, 25.0), &targets);
        assert!(matches!(
            input.end(2, egui::pos2(75.0, 25.0), &targets),
            Some(Action::Digit('2'))
        ));
        assert!(matches!(
            input.end(1, egui::pos2(75.0, 25.0), &targets),
            Some(Action::Digit('1'))
        ));
        input.start(3, egui::pos2(25.0, 25.0), &targets);
        input.cancel();
        assert!(input.end(3, egui::pos2(25.0, 25.0), &targets).is_none());
        input.start(4, egui::pos2(25.0, 25.0), &targets);
        assert!(input.end(4, egui::pos2(200.0, 25.0), &targets).is_none());
    }
}

#[cfg(test)]
mod cancellation_tests {
    use super::*;
    #[test]
    fn canceling_one_contact_preserves_another() {
        let target = touch::Target {
            rect: egui::Rect::from_min_size(egui::Pos2::ZERO, Vec2::splat(50.0)),
            action: Action::Digit('1'),
            group: 0,
        };
        let mut touch = touch::TouchController::default();
        touch.start(1, egui::pos2(10.0, 10.0), &[target]);
        touch.start(2, egui::pos2(20.0, 20.0), &[target]);
        touch.cancel_contact(1);
        assert!(touch.end(1, egui::pos2(10.0, 10.0), &[target]).is_none());
        assert!(matches!(
            touch.end(2, egui::pos2(20.0, 20.0), &[target]),
            Some(Action::Digit('1'))
        ));
    }
}

#[cfg(test)]
mod history_alignment_tests {
    use super::*;
    fn positions(mode: Mode, right: bool) -> Vec<(String, egui::Rect)> {
        let ctx = egui::Context::default();
        configure_fonts(&ctx);
        let mut app = CalculatorApp {
            history_right: right,
            ..Default::default()
        };
        app.calculator.apply(Action::Mode(mode));
        let keys = if mode == Mode::Time {
            "145+1230="
        } else {
            "1234+5="
        };
        for c in keys.chars() {
            app.calculator.apply(Action::from_char(c).unwrap());
        }
        let mut rows = Vec::new();
        for _ in 0..3 {
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        Vec2::new(400.0, 400.0),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| app.tape(ui, 300.0));
                },
            );
            rows.clear();
            for clipped in output.shapes {
                if let egui::Shape::Text(text) = clipped.shape {
                    rows.push((
                        text.galley.text().to_owned(),
                        text.galley.rect.translate(text.pos.to_vec2()),
                    ));
                }
            }
        }
        rows
    }
    #[test]
    fn values_align_independently_of_history_column_position() {
        for (mode, labels) in [
            (Mode::Time, vec!["1:45", "12:30", "14:15"]),
            (Mode::Number, vec!["1 234", "5", "1 239"]),
        ] {
            let right = positions(mode, true);
            let left = positions(mode, false);
            let rects: Vec<_> = labels
                .iter()
                .map(|label| right.iter().find(|(text, _)| text == label).unwrap().1)
                .collect();
            let edge = rects[0].right();
            for rect in &rects {
                assert!(
                    (rect.right() - edge).abs() < 1.0,
                    "Unaligned right edges: {rects:?}"
                );
            }
            let label = labels[0];
            let l = left.iter().find(|(text, _)| text == label).unwrap().1;
            assert!(
                (rects[0].right() - l.right()).abs() < 1.0,
                "Numeric alignment depends on history side: {right:?} vs {left:?}"
            );
        }
    }
    #[test]
    fn nested_group_rows_remain_readable_and_accessible_at_narrow_width() {
        let ctx = egui::Context::default();
        configure_fonts(&ctx);
        ctx.enable_accesskit();
        let mut app = CalculatorApp::default();
        for c in "((((((((25+25))))))))".chars() {
            app.calculator.apply(Action::from_char(c).unwrap());
        }
        let mut labels = Vec::new();
        let mut rows = Vec::new();
        for _ in 0..3 {
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        Vec2::new(250.0, 1000.0),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| app.tape(ui, 950.0));
                },
            );
            labels = output
                .platform_output
                .accesskit_update
                .unwrap()
                .nodes
                .into_iter()
                .filter_map(|(_, node)| node.value().or_else(|| node.label()).map(str::to_owned))
                .collect();
            rows.clear();
            for clipped in output.shapes {
                if let egui::Shape::Text(text) = clipped.shape {
                    rows.push((
                        text.galley.text().to_owned(),
                        text.galley.rect.translate(text.pos.to_vec2()),
                    ));
                }
            }
        }
        for expected in ["Gruppe", "0:25", "25 min", "0:50", "50 min"] {
            assert!(
                labels.iter().any(|label| label == expected),
                "Missing accessible history text: {expected}"
            );
        }
        let value = rows.iter().find(|(text, _)| text == "0:25").unwrap().1;
        let minutes = rows.iter().find(|(text, _)| text == "25 min").unwrap().1;
        assert!(minutes.width() > 20.0 && minutes.left() >= 0.0);
        assert!(minutes.right() + 6.0 < value.left());
        assert!((minutes.center().y - value.center().y).abs() < 3.0);
    }
    #[test]
    fn time_and_minutes_share_a_compact_history_row() {
        let rows = positions(Mode::Time, true);
        let value = rows.iter().find(|(text, _)| text == "1:45").unwrap().1;
        let minutes = rows.iter().find(|(text, _)| text == "105 min").unwrap().1;
        assert!((value.center().y - minutes.center().y).abs() < 3.0);
        assert!(minutes.right() + 6.0 < value.left());
    }
    #[test]
    fn unchanged_tab_does_not_overwrite_newer_calculation_on_update() {
        #[derive(Default)]
        struct Memory {
            data: std::collections::HashMap<String, String>,
        }
        impl eframe::Storage for Memory {
            fn get_string(&self, key: &str) -> Option<String> {
                self.data.get(key).cloned()
            }
            fn set_string(&mut self, key: &str, value: String) {
                self.data.insert(key.into(), value);
            }
            fn flush(&mut self) {}
        }
        use eframe::{App, Storage};
        let mut storage = Memory::default();
        let mut stale = CalculatorApp::default();
        for c in "45=".chars() {
            stale.dispatch(Action::from_char(c).unwrap());
        }
        stale.save(&mut storage);
        let mut restored_stale =
            CalculatorApp::restore(storage.get_string("calculator-v1").as_deref());
        let mut active = CalculatorApp::restore(storage.get_string("calculator-v1").as_deref());
        for c in "+15=".chars() {
            active.dispatch(Action::from_char(c).unwrap());
        }
        active.save(&mut storage);
        stale.save(&mut storage);
        restored_stale.save(&mut storage);
        let restored: CalculatorApp =
            serde_json::from_str(&storage.get_string("calculator-v1").unwrap()).unwrap();
        assert_eq!(restored.calculator.current_text(), "1:00");
    }
    #[test]
    fn new_installations_default_to_right_aligned_history() {
        assert!(CalculatorApp::default().history_right);
        let restored: CalculatorApp = serde_json::from_str("{}").unwrap();
        assert!(restored.history_right);
        let restored: CalculatorApp = serde_json::from_str(r#"{"history_right":false}"#).unwrap();
        assert!(!restored.history_right);
    }
}
