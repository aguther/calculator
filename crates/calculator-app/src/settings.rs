//! Settings navigation is transient; only preferences are persisted.
use super::{
    COMMIT, CalculatorApp, Language, Text, Theme, VERSION, bold_font, buttons::PressButton,
    settings_switch,
};
use eframe::egui::{self, RichText, Vec2};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum SettingsTab {
    #[default]
    General,
    Help,
    Diagnostics,
}
impl CalculatorApp {
    pub(super) fn settings(&mut self, ctx: &egui::Context) {
        if !self.settings_open {
            return;
        }
        let width = (ctx.content_rect().width() - 48.0).clamp(220.0, 360.0);
        let height = (ctx.content_rect().height() - 100.0).max(220.0);
        egui::Window::new(self.language.text(Text::InformationSettings))
            .id(egui::Id::new("calculator-settings"))
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .fixed_size(Vec2::new(width, height))
            .vscroll(false)
            .frame(
                egui::Frame::new()
                    .fill(self.theme.panel())
                    .inner_margin(18.0)
                    .corner_radius(20.0),
            )
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = Vec2::splat(10.0);
                ui.label(RichText::new(self.language.text(Text::Settings)).font(bold_font(23.0)));
                ui.label(
                    RichText::new(format!(
                        "Calculator {VERSION} · {}",
                        COMMIT.chars().take(12).collect::<String>()
                    ))
                    .size(14.0)
                    .weak(),
                );
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    let tab_width = (ui.available_width() - 12.0) / 3.0;
                    for (tab, label) in [
                        (SettingsTab::General, self.language.text(Text::General)),
                        (SettingsTab::Help, self.language.text(Text::Help)),
                        (
                            SettingsTab::Diagnostics,
                            self.language.text(Text::Diagnostics),
                        ),
                    ] {
                        let selected = self.settings_tab == tab;
                        if ui
                            .add(
                                PressButton::new(
                                    label,
                                    Vec2::new(tab_width, 44.0),
                                    self.theme.surface(),
                                    if selected {
                                        self.theme.accent()
                                    } else {
                                        ui.visuals().text_color()
                                    },
                                )
                                .font(bold_font(13.0))
                                .selected(selected),
                            )
                            .clicked()
                        {
                            self.settings_tab = tab;
                        }
                    }
                });
                ui.add_space(4.0);
                let area = ui.available_rect_before_wrap();
                let footer = egui::Rect::from_min_max(
                    egui::pos2(area.left(), area.bottom() - 44.0),
                    area.max,
                );
                let content = egui::Rect::from_min_max(
                    area.min,
                    egui::pos2(area.right(), footer.top() - 20.0),
                );
                let mut body = super::child_ui(ui, content, "settings-body");
                egui::ScrollArea::vertical()
                    .id_salt(("settings-content", self.settings_tab as u8))
                    .max_height(content.height())
                    .auto_shrink([false, false])
                    .show(&mut body, |ui| {
                        ui.set_max_width(content.width());
                        match self.settings_tab {
                            SettingsTab::General => self.general_settings(ui, ctx),
                            SettingsTab::Help => self.help_settings(ui),
                            SettingsTab::Diagnostics => self.diagnostic_settings(ui, ctx),
                        }
                    });
                ui.painter().hline(
                    area.x_range(),
                    footer.top() - 10.0,
                    egui::Stroke::new(1.0_f32, ui.visuals().weak_text_color().gamma_multiply(0.2)),
                );
                if ui
                    .put(
                        footer,
                        PressButton::new(
                            self.language.text(Text::Done),
                            footer.size(),
                            self.theme.surface(),
                            ui.visuals().text_color(),
                        )
                        .font(bold_font(15.0)),
                    )
                    .clicked()
                {
                    self.settings_open = false;
                }
                ui.advance_cursor_after_rect(area);
            });
    }
    fn general_settings(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.label(RichText::new(self.language.text(Text::Appearance)).font(bold_font(15.0)));
        ui.horizontal(|ui| {
            let width = (ui.available_width() - ui.spacing().item_spacing.x * 2.0) / 3.0;
            for &theme in Theme::ALL {
                let selected = self.theme == theme;
                if ui
                    .add(
                        PressButton::new(
                            theme.label(self.language),
                            Vec2::new(width, 44.0),
                            self.theme.surface(),
                            if selected {
                                self.theme.accent()
                            } else {
                                ui.visuals().text_color()
                            },
                        )
                        .selected(selected),
                    )
                    .clicked()
                {
                    self.theme = theme;
                    self.theme.apply(ctx);
                    self.dirty = true;
                }
            }
        });
        ui.add_space(8.0);
        ui.label(RichText::new(self.language.text(Text::Language)).font(bold_font(15.0)));
        ui.horizontal(|ui| {
            let width = (ui.available_width() - ui.spacing().item_spacing.x) / 2.0;
            for language in [Language::German, Language::English] {
                if ui
                    .add(
                        PressButton::new(
                            language.name(),
                            Vec2::new(width, 44.0),
                            self.theme.surface(),
                            ui.visuals().text_color(),
                        )
                        .selected(self.language == language),
                    )
                    .clicked()
                {
                    self.set_language(language, ctx);
                }
            }
        });
        ui.add_space(8.0);
        ui.label(RichText::new(self.language.text(Text::Behavior)).font(bold_font(15.0)));
        egui::Frame::new()
            .fill(self.theme.surface())
            .inner_margin(12.0)
            .corner_radius(12.0)
            .show(ui, |ui| {
                if settings_switch(
                    ui,
                    &mut self.history_right,
                    self.language.text(Text::HistoryRight),
                    true,
                ) {
                    self.dirty = true;
                }
                ui.separator();
                if settings_switch(ui, &mut self.sound, self.language.text(Text::Clicks), true) {
                    self.dirty = true;
                }
                ui.separator();
                if settings_switch(
                    ui,
                    &mut self.haptics,
                    self.language.text(Text::Haptics),
                    crate::feedback::supports_haptics(),
                ) {
                    self.dirty = true;
                }
                if !crate::feedback::supports_haptics() {
                    ui.label(
                        RichText::new(self.language.text(Text::NoVibration))
                            .small()
                            .weak(),
                    );
                }
            });
    }
    fn help_settings(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new(self.language.text(Text::Shortcuts)).font(bold_font(15.0)));
        for (label, key) in [
            (self.language.text(Text::ToggleSettings), "F1"),
            (self.language.text(Text::TimeShortcut), "F2"),
            (self.language.text(Text::NumberShortcut), "F3"),
            (
                self.language.text(Text::Backspace),
                self.language.text(Text::BackspaceKey),
            ),
            (
                self.language.text(Text::ClearEntry),
                self.language.text(Text::DeleteKey),
            ),
            (self.language.text(Text::Clear), "Esc"),
        ] {
            Self::settings_row(ui, label, key);
        }
        ui.add_space(6.0);
        ui.label(
            RichText::new(self.language.text(Text::ShortcutHelp))
                .small()
                .weak(),
        );
    }
    fn settings_row(ui: &mut egui::Ui, label: &str, value: &str) {
        ui.horizontal(|ui| {
            let width = ui.available_width();
            ui.allocate_ui_with_layout(
                Vec2::new((width - 88.0).max(80.0), 38.0),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.add(egui::Label::new(label).wrap());
                },
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(value).font(bold_font(13.0)));
            });
        });
        ui.separator();
    }
    fn diagnostic_settings(&self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let size = ctx.content_rect().size();
        let browser: serde_json::Value = {
            #[cfg(target_arch = "wasm32")]
            {
                web_sys::window()
                    .and_then(|w| w.document())
                    .and_then(|d| d.document_element())
                    .and_then(|e| e.get_attribute("data-viewport"))
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or(serde_json::Value::Null)
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                serde_json::Value::Null
            }
        };
        ui.label(RichText::new(self.language.text(Text::AppDevice)).font(bold_font(15.0)));
        Self::settings_row(ui, self.language.text(Text::Version), VERSION);
        Self::settings_row(
            ui,
            self.language.text(Text::Screen),
            &format!("{} × {}", size.x.round(), size.y.round()),
        );
        Self::settings_row(
            ui,
            self.language.text(Text::Scale),
            &format!("{}×", ctx.pixels_per_point()),
        );
        Self::settings_row(
            ui,
            self.language.text(Text::Theme),
            self.theme.label(self.language),
        );
        ui.label(RichText::new(format!("Commit: {COMMIT}")).small().weak());
        let text = serde_json::to_string_pretty(&serde_json::json!({"version":VERSION,"commit":COMMIT,"viewport":{"width":size.x,"height":size.y,"pixelsPerPoint":ctx.pixels_per_point()},"theme":self.theme,"language":self.language,"browser":browser})).unwrap_or_default();
        let copied = ui.id().with("diagnostics-copied");
        let now = ctx.input(|i| i.time);
        let recent = ctx
            .data(|d| d.get_temp::<f64>(copied))
            .is_some_and(|at| now - at < 2.0);
        if ui
            .add(PressButton::new(
                if recent {
                    self.language.text(Text::Copied)
                } else {
                    self.language.text(Text::Copy)
                },
                Vec2::new(ui.available_width(), 44.0),
                self.theme.surface(),
                self.theme.accent(),
            ))
            .clicked()
        {
            ctx.copy_text(text);
            ctx.data_mut(|d| d.insert_temp(copied, now));
            ctx.request_repaint_after(std::time::Duration::from_secs(2));
        }
        ui.label(
            RichText::new(self.language.text(Text::CopyHelp))
                .small()
                .weak(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_tab_keeps_navigation_and_footer_visible() {
        for size in [
            Vec2::new(320.0, 568.0),
            Vec2::new(844.0, 390.0),
            Vec2::new(568.0, 320.0),
        ] {
            for tab in [
                SettingsTab::General,
                SettingsTab::Help,
                SettingsTab::Diagnostics,
            ] {
                let ctx = egui::Context::default();
                super::super::configure_fonts(&ctx);
                let mut app = CalculatorApp {
                    settings_open: true,
                    settings_tab: tab,
                    language: Language::German,
                    ..Default::default()
                };
                for _ in 0..3 {
                    let output = ctx.run(
                        egui::RawInput {
                            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                            ..Default::default()
                        },
                        |ctx| app.settings(ctx),
                    );
                    for clipped in &output.shapes {
                        if let egui::Shape::Text(text) = &clipped.shape
                            && ["Allgemein", "Bedienung", "Diagnose", "Fertig"]
                                .contains(&text.galley.text())
                        {
                            let rect = text.galley.rect.translate(text.pos.to_vec2());
                            assert!(
                                egui::Rect::from_min_size(egui::Pos2::ZERO, size)
                                    .contains_rect(rect),
                                "Navigation outside {size:?}: {rect:?}"
                            );
                            assert!(clipped.clip_rect.contains_rect(rect));
                        }
                    }
                }
                assert!(
                    !serde_json::to_string(&app)
                        .unwrap()
                        .contains("settings_tab")
                );
            }
        }
    }
}

#[cfg(test)]
mod interaction_tests {
    use super::*;
    fn frame(
        ctx: &egui::Context,
        app: &mut CalculatorApp,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    Vec2::new(400.0, 760.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| app.settings(ctx),
        )
    }
    fn center(output: &egui::FullOutput, label: &str) -> egui::Pos2 {
        output
            .shapes
            .iter()
            .find_map(|shape| {
                if let egui::Shape::Text(text) = &shape.shape
                    && text.galley.text() == label
                {
                    return Some(text.pos + text.galley.size() * 0.5);
                }
                None
            })
            .unwrap_or_else(|| panic!("Missing {label}"))
    }
    fn click(ctx: &egui::Context, app: &mut CalculatorApp, pos: egui::Pos2) -> egui::FullOutput {
        for pressed in [true, false] {
            let output = frame(
                ctx,
                app,
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::default(),
                    },
                ],
            );
            if !pressed {
                return output;
            }
        }
        unreachable!()
    }
    #[test]
    fn language_buttons_switch_visible_labels_and_persist_preferences() {
        let ctx = egui::Context::default();
        super::super::configure_fonts(&ctx);
        let mut app = CalculatorApp {
            settings_open: true,
            language: Language::German,
            ..Default::default()
        };
        for _ in 0..3 {
            frame(&ctx, &mut app, vec![]);
        }
        let output = frame(&ctx, &mut app, vec![]);
        click(&ctx, &mut app, center(&output, "English"));
        let output = frame(&ctx, &mut app, vec![]);
        center(&output, "Settings");
        center(&output, "Done");
        assert_eq!(app.language, Language::English);
        assert!(app.dirty);
        let restored = CalculatorApp::restore(Some(&serde_json::to_string(&app).unwrap()));
        assert_eq!(restored.language, Language::English);
        click(&ctx, &mut app, center(&output, "Deutsch"));
        let output = frame(&ctx, &mut app, vec![]);
        center(&output, "Einstellungen");
        assert_eq!(app.language, Language::German);
    }
    #[test]
    fn tabs_show_shortcuts_and_copy_full_diagnostics_without_changing_preferences() {
        let ctx = egui::Context::default();
        super::super::configure_fonts(&ctx);
        let mut app = CalculatorApp {
            settings_open: true,
            language: Language::German,
            ..Default::default()
        };
        let saved = serde_json::to_string(&app).unwrap();
        for _ in 0..3 {
            frame(&ctx, &mut app, vec![]);
        }
        let output = frame(&ctx, &mut app, vec![]);
        click(&ctx, &mut app, center(&output, "Bedienung"));
        let output = frame(&ctx, &mut app, vec![]);
        center(&output, "F2");
        center(&output, "F3");
        assert!(app.settings_tab == SettingsTab::Help);
        click(&ctx, &mut app, center(&output, "Diagnose"));
        let output = frame(&ctx, &mut app, vec![]);
        let output = click(&ctx, &mut app, center(&output, "Diagnose kopieren"));
        let copied = output
            .platform_output
            .commands
            .iter()
            .find_map(|command| {
                if let egui::OutputCommand::CopyText(text) = command {
                    Some(text)
                } else {
                    None
                }
            })
            .expect("Clipboard command missing");
        let report: serde_json::Value = serde_json::from_str(copied).unwrap();
        assert_eq!(report["viewport"]["width"], 400.0);
        assert_eq!(report["version"], VERSION);
        assert_eq!(report["commit"], COMMIT);
        assert!(report.get("browser").is_some());
        assert_eq!(serde_json::to_string(&app).unwrap(), saved);
        assert!(!app.dirty);
    }
}
