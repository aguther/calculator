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
    pub fn background(self) -> Color32 {
        match self {
            Self::Original => Color32::from_rgb(12, 12, 14),
            Self::Light => Color32::from_rgb(245, 243, 239),
            Self::Dark => Color32::from_rgb(20, 29, 42),
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
}
pub fn layout(width: f32, height: f32) -> Layout {
    let content_width = width.min(520.0);
    let available = (height - 150.0).max(170.0);
    let key_height = (available * 0.56 / 5.0).clamp(28.0, 76.0);
    Layout {
        key_height,
        tape_height: (height - 150.0 - key_height * 5.0 - 24.0).max(30.0),
        content_width,
    }
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
            touch: touch::TouchController::default(),
            targets: Vec::new(),
            touch_frame: false,
            viewport_size: None,
        }
    }
}
impl CalculatorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app: Self = cc
            .storage
            .and_then(|s| s.get_string("calculator-v1"))
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        app.calculator.validate();
        app.theme.apply(&cc.egui_ctx);
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
        .size((size.y * 0.4).clamp(15.0, 27.0))
        .color(foreground);
        let text = if matches!(
            action,
            Action::Clear | Action::ClearEntry | Action::Operator(_)
        ) {
            text.strong()
        } else {
            text
        };
        let mut button = egui::Button::new(text).corner_radius(12.0);
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
        let gap = 6.0;
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
                        Step::Sum(value) => {
                            ("=".into(), self.calculator.label(*value, false), None)
                        }
                        Step::Open(op) => {
                            groups.push(i);
                            (
                                op.map(symbol).unwrap_or("").to_string(),
                                "Gruppe (".into(),
                                None,
                            )
                        }
                        Step::Close => {
                            let start = groups.pop().unwrap_or(i);
                            let mut steps = self.calculator.state().steps[start..=i].to_vec();
                            if let Some(Step::Open(op)) = steps.first_mut() {
                                *op = None;
                            }
                            (
                                "= )".into(),
                                self.calculator
                                    .label(calculator_core::evaluate(&steps), false),
                                None,
                            )
                        }
                    };
                    if matches!(step, Step::Sum(_)) {
                        ui.separator();
                    }
                    ui.horizontal(|ui| {
                        ui.add_space((depth.min(8) * 8) as f32);
                        ui.add_sized(
                            [20.0, 20.0],
                            egui::Label::new(
                                RichText::new(if matches!(step, Step::Value { .. }) {
                                    number.to_string()
                                } else {
                                    String::new()
                                })
                                .small()
                                .weak(),
                            ),
                        );
                        ui.add_sized(
                            [24.0, 20.0],
                            egui::Label::new(RichText::new(operator).color(self.theme.accent())),
                        );
                        let content_width = ui.available_width();
                        let alignment = if self.history_right {
                            egui::Align::Max
                        } else {
                            egui::Align::Min
                        };
                        ui.allocate_ui_with_layout(
                            Vec2::new(content_width, 0.0),
                            egui::Layout::top_down(alignment),
                            |ui| {
                                ui.set_min_width(content_width);
                                ui.label(RichText::new(label).size(19.0));
                                if let Some(minutes) = minutes {
                                    ui.label(
                                        RichText::new(format!("{minutes} min")).small().weak(),
                                    );
                                }
                            },
                        );
                    });
                    if matches!(step, Step::Open(_)) {
                        depth += 1;
                    }
                }
            });
    }
    fn calculator_ui(&mut self, ui: &mut egui::Ui, width: f32, height: f32) {
        let l = layout(width, height);
        ui.horizontal(|ui| {
            if ui
                .selectable_label(self.calculator.mode == Mode::Time, "Zeit")
                .clicked()
            {
                self.dispatch(Action::Mode(Mode::Time));
            }
            if ui
                .selectable_label(self.calculator.mode == Mode::Number, "Rechner")
                .clicked()
            {
                self.dispatch(Action::Mode(Mode::Number));
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .button("⚙")
                    .on_hover_text("Informationen und Einstellungen")
                    .clicked()
                {
                    self.settings_open = true;
                }
            });
        });
        ui.separator();
        self.tape(ui, l.tape_height);
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(RichText::new("SUMME").small().weak());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new(self.calculator.label(self.calculator.total(), false))
                        .size(23.0)
                        .color(self.theme.accent()),
                );
                if ui
                    .button("Undo")
                    .on_hover_text("Letzten Schritt zurück")
                    .clicked()
                {
                    self.dispatch(Action::Undo);
                }
                if self.calculator.mode == Mode::Time {
                    ui.label(
                        RichText::new(format!(
                            "{} min",
                            format_number(self.calculator.total() / 60.0)
                        ))
                        .small()
                        .weak(),
                    );
                }
            });
        });
        let text = self.calculator.current_text();
        ui.horizontal(|ui| {
            ui.label(self.calculator.state().pending.map(symbol).unwrap_or(""));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(&text).size(if text.len() > 18 { 19.0 } else { 32.0 }));
            });
        });
        if !self.calculator.notice.is_empty() {
            ui.label(
                RichText::new(&self.calculator.notice)
                    .small()
                    .color(self.theme.accent()),
            );
        }
        ui.horizontal(|ui| {
            let size = Vec2::new((ui.available_width() - 6.0) / 2.0, 28.0);
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
        self.keypad(ui, &l);
    }
    fn settings(&mut self, ctx: &egui::Context) {
        if !self.settings_open {
            return;
        }
        let mut open = true;
        egui::Window::new("Informationen und Einstellungen").open(&mut open).collapsible(false).resizable(false).default_width(280.0).show(ctx,|ui|{
            ui.heading("Zeitrechner");let short_commit=COMMIT.chars().take(12).collect::<String>();
                ui.label(format!("Calculator {VERSION} · {short_commit}"));ui.separator();
            ui.label("Theme");
            for theme in [Theme::Original,Theme::Light,Theme::Dark]{if ui.selectable_value(&mut self.theme,theme,theme.label()).changed(){self.theme.apply(ctx);self.dirty=true;}}
            if ui.checkbox(&mut self.sound,"Tastenklicks").changed(){self.dirty=true;}
            if ui.checkbox(&mut self.haptics,"Haptik (wo unterstützt)").changed(){self.dirty=true;}
            if ui.checkbox(&mut self.history_right,"Verlauf rechtsbündig").changed(){self.dirty=true;}
            ui.separator();ui.label("Tastatur: Ziffern, + − * /, Klammern, Enter, Rücktaste. Escape löscht alles; Entf löscht die Eingabe. Punkt/Komma fügt im Zeitmodus einen Doppelpunkt ein.");
            ui.label("Berechnungen und Einstellungen bleiben auf diesem Gerät.");
            ui.collapsing("Diagnose",|ui|{
                let size=ctx.content_rect().size();
                let report=serde_json::json!({"version":VERSION,"commit":COMMIT,"target":std::env::consts::ARCH,"viewport":{"width":size.x,"height":size.y,"pixelsPerPoint":ctx.pixels_per_point()},"theme":self.theme});
                let text=serde_json::to_string_pretty(&report).unwrap_or_default();
                ui.label(egui::RichText::new(&text).monospace().small());
                if ui.button("Diagnose kopieren").clicked(){ctx.copy_text(text);}
            });
            if ui.button("Schließen").clicked(){self.settings_open=false;}
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
                if width > height * 1.35 {
                    let half = (width - 12.0) / 2.0;
                    ui.horizontal_top(|ui| {
                        ui.allocate_ui_with_layout(
                            Vec2::new(half, height),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.heading("Verlauf");
                                self.tape(ui, (height - 36.0).max(30.0));
                            },
                        );
                        ui.allocate_ui_with_layout(
                            Vec2::new(half, height),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                // The compact panel omits the second tape in landscape.
                                let l = Layout {
                                    key_height: ((height - 100.0) / 5.0).max(24.0),
                                    tape_height: 0.0,
                                    content_width: half,
                                };
                                ui.horizontal(|ui| {
                                    if ui.button("Zeit / Rechner").clicked() {
                                        let mode = if self.calculator.mode == Mode::Time {
                                            Mode::Number
                                        } else {
                                            Mode::Time
                                        };
                                        self.dispatch(Action::Mode(mode));
                                    }
                                    if ui.button("⚙").clicked() {
                                        self.settings_open = true;
                                    }
                                });
                                ui.horizontal(|ui| {
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label(
                                                RichText::new(self.calculator.current_text())
                                                    .size(24.0),
                                            );
                                        },
                                    );
                                });
                                ui.horizontal(|ui| {
                                    if ui.button("(").clicked() {
                                        self.dispatch(Action::Open);
                                    }
                                    if ui
                                        .add_enabled(
                                            self.calculator.can_close(),
                                            egui::Button::new(")"),
                                        )
                                        .clicked()
                                    {
                                        self.dispatch(Action::Close);
                                    }
                                    if ui.button("Undo").clicked() {
                                        self.dispatch(Action::Undo);
                                    }
                                });
                                self.keypad(ui, &l);
                            },
                        );
                    });
                } else {
                    let content_width = width.min(520.0);
                    let offset = (width - content_width) * 0.5;
                    ui.horizontal_top(|ui| {
                        ui.add_space(offset);
                        ui.allocate_ui_with_layout(
                            Vec2::new(content_width, height),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                self.calculator_ui(ui, content_width, height);
                            },
                        );
                    });
                }
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
        if let Ok(data) = serde_json::to_string(self) {
            storage.set_string("calculator-v1", data);
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
    fn bottom_keys_remain_visible() {
        for (width, height, long_entry) in [
            (320.0, 568.0, false),
            (400.0, 760.0, false),
            (320.0, 568.0, true),
        ] {
            let ctx = egui::Context::default();
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
    fn right_alignment_aligns_values_and_minutes_and_toggle_moves_them() {
        for (mode, labels) in [
            (
                Mode::Time,
                vec!["1:45", "12:30", "14:15", "105 min", "750 min"],
            ),
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
                rects[0].right() - l.right() > 100.0,
                "Setting has no visible effect: {right:?} vs {left:?}"
            );
        }
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
