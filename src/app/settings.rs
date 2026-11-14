use super::*;
use crate::surfaces::{self, CANVAS, FG, LINE, MUTED};
use egui::{Align, FontId, Frame, Layout, Margin, Stroke};

// Settings share one content edge. On narrow windows, controls stack instead of
// squeezing labels or shrinking the terminal font.
fn setting_row(ui: &mut egui::Ui, label: &str, help: &str, control: impl FnOnce(&mut egui::Ui)) {
    let width = ui.available_width();
    if width < 560. {
        ui.label(RichText::new(label).strong());
        ui.label(RichText::new(help).size(12.).color(MUTED));
        ui.add_space(4.);
        control(ui);
    } else {
        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(width - 296., 0.),
                Layout::top_down(Align::Min),
                |ui| {
                    ui.set_min_width(width - 296.);
                    ui.label(RichText::new(label).strong());
                    ui.label(RichText::new(help).size(12.).color(MUTED));
                },
            );
            ui.allocate_ui_with_layout(egui::vec2(288., 0.), Layout::top_down(Align::Min), |ui| {
                ui.set_min_width(288.);
                control(ui);
            });
        });
    }
}

impl App {
    pub(super) fn settings_surface(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let margin = if ui.available_width() < 640. { 24 } else { 40 };
            Frame::NONE
                .inner_margin(Margin::symmetric(margin, 32))
                .show(ui, |ui| {
                    ui.set_max_width(ui.available_width().min(920.));
                    ui.label(RichText::new("Theme").size(18.).strong());
                    ui.add_space(8.);
                    setting_row(
                        ui,
                        "Appearance",
                        "Choose a glossy glass or solid neutral workspace.",
                        |ui| {
                            let before = self.saved.settings.appearance;
                            egui::ComboBox::from_id_salt("appearance-mode")
                                .width(ui.available_width())
                                .selected_text(format!("{before:?}"))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.saved.settings.appearance,
                                        Appearance::Glossy,
                                        "Glossy",
                                    );
                                    ui.selectable_value(
                                        &mut self.saved.settings.appearance,
                                        Appearance::Opaque,
                                        "Opaque",
                                    );
                                });
                            if before != self.saved.settings.appearance {
                                crate::surfaces::apply(
                                    ui.ctx(),
                                    self.saved.settings.accent,
                                    self.saved.settings.appearance,
                                );
                                self.changed();
                            }
                        },
                    );
                    ui.add_space(16.);
                    setting_row(
                        ui,
                        "Accent color",
                        "Colors links, status indicators, and focus outlines.",
                        |ui| {
                            let before = self.saved.settings.accent;
                            egui::ComboBox::from_id_salt("theme-accent")
                                .width(ui.available_width())
                                .selected_text(format!("{before:?}"))
                                .show_ui(ui, |ui| {
                                    for accent in
                                        [Accent::Cyan, Accent::Blue, Accent::White, Accent::Black]
                                    {
                                        ui.selectable_value(
                                            &mut self.saved.settings.accent,
                                            accent,
                                            format!("{accent:?}"),
                                        );
                                    }
                                });
                            if before != self.saved.settings.accent {
                                crate::surfaces::apply(
                                    ui.ctx(),
                                    self.saved.settings.accent,
                                    self.saved.settings.appearance,
                                );
                                self.changed();
                            }
                        },
                    );
                    ui.add_space(28.);
                    ui.separator();
                    ui.add_space(24.);
                    ui.label(RichText::new("TERMINAL").size(12.).color(MUTED));
                    ui.add_space(12.);
                    let sample = surfaces::glass_inset(ui.ctx())
                        .inner_margin(Margin::symmetric(20, 16))
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.label(
                                RichText::new("PS X:\\project> echo Hello")
                                    .font(FontId::monospace(self.saved.settings.font_size))
                                    .color(FG),
                            );
                            ui.label(
                                RichText::new("Hello")
                                    .font(FontId::monospace(self.saved.settings.font_size))
                                    .color(MUTED),
                            );
                        });
                    surfaces::glass_edge(ui.painter(), sample.response.rect.shrink(3.), 0.48);
                    ui.add_space(20.);
                    let width = ui.available_width();
                    let size_width = 106. + 2. * ui.spacing().item_spacing.x;
                    let font_width = (width - size_width - 32.).min(440.);
                    if width >= 560. {
                        ui.horizontal_top(|ui| {
                            ui.allocate_ui_with_layout(
                                egui::vec2(font_width, 0.),
                                Layout::top_down(Align::Min),
                                |ui| {
                                    ui.set_min_width(font_width);
                                    ui.label(RichText::new("Font").strong());
                                    ui.add_space(4.);
                                    self.settings_font(ui);
                                },
                            );
                            ui.add_space(width - font_width - size_width - 8.);
                            ui.allocate_ui_with_layout(
                                egui::vec2(size_width, 0.),
                                Layout::top_down(Align::Min),
                                |ui| {
                                    ui.label(RichText::new("Size").strong());
                                    ui.add_space(4.);
                                    self.settings_size(ui);
                                },
                            );
                        });
                    } else {
                        ui.label(RichText::new("Font").strong());
                        self.settings_font(ui);
                        ui.add_space(12.);
                        ui.label(RichText::new("Size").strong());
                        self.settings_size(ui);
                    }
                    if let Some(error) = &self.font_error {
                        ui.label(
                            RichText::new(error)
                                .size(12.)
                                .color(Color32::from_rgb(234, 137, 144)),
                        );
                    }
                    ui.add_space(32.);
                    setting_row(
                        ui,
                        "Scrollback",
                        "How many lines of terminal output to keep. Older lines are discarded.",
                        |ui| self.settings_scrollback(ui),
                    );
                    ui.add_space(24.);
                    setting_row(
                        ui,
                        "Closing behavior",
                        "What to do when you close the window.",
                        |ui| self.settings_closing(ui),
                    );
                    if self.saved.settings.closing == Closing::Background {
                        ui.add_space(8.);
                        ui.label(
                            RichText::new("Background terminals keep running and using memory.")
                                .size(12.)
                                .color(MUTED),
                        );
                    }
                    ui.add_space(28.);
                    ui.separator();
                    ui.add_space(24.);
                    ui.label(RichText::new("Keyboard shortcuts").size(16.).strong());
                    ui.add_space(12.);
                    // Keep the real application shortcuts; mockup shortcuts are illustrative.
                    for (label, keys) in [
                        ("New workspace", "Ctrl+Shift+N"),
                        ("New terminal", "Ctrl+Shift+T"),
                        ("Settings", "Ctrl+,"),
                        ("Paste screenshot", "Ctrl+Shift+I"),
                        ("Restore pane", "Escape"),
                    ] {
                        ui.horizontal(|ui| {
                            ui.label(label);
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                for key in keys.split('+').rev() {
                                    Frame::new()
                                        .fill(CANVAS)
                                        .stroke(Stroke::new(1., LINE))
                                        .corner_radius(4)
                                        .inner_margin(Margin::symmetric(8, 5))
                                        .show(ui, |ui| {
                                            ui.label(RichText::new(key).size(12.).color(MUTED));
                                        });
                                }
                            });
                        });
                        ui.add_space(8.);
                    }
                    ui.add_space(8.);
                    ui.label(
                        RichText::new("Terminal shortcuts stay with the running program.")
                            .size(12.)
                            .color(MUTED),
                    );
                });
        });
    }
    fn settings_font(&mut self, ui: &mut egui::Ui) {
        let previous = self.saved.settings.font_family.clone();
        egui::ComboBox::from_id_salt("terminal-font-family")
            .width(ui.available_width())
            .selected_text(&self.saved.settings.font_family)
            .show_ui(ui, |ui| {
                for choice in &self.font_choices {
                    ui.selectable_value(
                        &mut self.saved.settings.font_family,
                        choice.family.clone(),
                        &choice.family,
                    );
                }
            });
        if self.saved.settings.font_family != previous {
            self.font_loader
                .apply(self.saved.settings.font_family.clone());
            self.changed();
        }
    }
    fn settings_size(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui
                .add_sized([32., 32.], egui::Button::new("+"))
                .on_hover_text("Increase font size")
                .clicked()
            {
                self.saved.settings.font_size = (self.saved.settings.font_size + 1.).min(28.);
                self.changed();
            }
            ui.add_sized(
                [42., 32.],
                egui::Label::new(format!("{:.0}", self.saved.settings.font_size)),
            );
            if ui
                .add_sized([32., 32.], egui::Button::new("−"))
                .on_hover_text("Decrease font size")
                .clicked()
            {
                self.saved.settings.font_size = (self.saved.settings.font_size - 1.).max(10.);
                self.changed();
            }
        });
    }
    fn settings_scrollback(&mut self, ui: &mut egui::Ui) {
        let before = self.saved.settings.scrollback;
        egui::ComboBox::from_id_salt("scrollback")
            .width(ui.available_width())
            .selected_text(format!("{} lines", self.saved.settings.scrollback))
            .show_ui(ui, |ui| {
                for count in [1000, 5000, 10000, 20000, 50000] {
                    ui.selectable_value(
                        &mut self.saved.settings.scrollback,
                        count,
                        format!("{} lines", count),
                    );
                }
            });
        if before != self.saved.settings.scrollback {
            for terminal in self.runtime.terminals.values_mut() {
                terminal
                    .backend
                    .set_scrollback(self.saved.settings.scrollback);
            }
            self.changed();
        }
    }
    fn settings_closing(&mut self, ui: &mut egui::Ui) {
        let before = self.saved.settings.closing;
        egui::ComboBox::from_id_salt("closing-policy")
            .width(ui.available_width())
            .selected_text(match self.saved.settings.closing {
                Closing::Ask => "Ask every time",
                Closing::Stop => "Stop terminals and quit",
                Closing::Background => "Run in background",
            })
            .show_ui(ui, |ui| {
                for (value, label) in [
                    (Closing::Ask, "Ask every time"),
                    (Closing::Stop, "Stop terminals and quit"),
                    (Closing::Background, "Run in background"),
                ] {
                    ui.selectable_value(&mut self.saved.settings.closing, value, label);
                }
            });
        if before != self.saved.settings.closing {
            self.changed();
        }
    }
}
