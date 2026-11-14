use super::*;
use crate::icons::{self, Icon};
use crate::surfaces::{CHROME, FG, LINE, MUTED, SIDEBAR};
use crate::surfaces::{FIELD_RADIUS, accent_ink, glass_edge, primary};
use egui::{Align, FontId, Frame, Layout, Margin, Sense, Stroke};

fn form_input(text: &mut String) -> egui::TextEdit<'_> {
    egui::TextEdit::singleline(text)
        .margin(Margin::symmetric(12, 6))
        .vertical_align(Align::Center)
}

fn search_input(ui: &mut egui::Ui, text: &mut String, hint: &str) -> egui::Response {
    crate::surfaces::glass_inset(ui.ctx())
        .inner_margin(Margin::symmetric(12, 0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                icons::inline(ui, Icon::Search, MUTED);
                ui.add_sized(
                    [ui.available_width(), 36.],
                    egui::TextEdit::singleline(text)
                        .hint_text(hint)
                        .frame(Frame::NONE)
                        .vertical_align(Align::Center),
                )
            })
            .inner
        })
        .inner
}

fn flat(ui: &mut egui::Ui) {
    ui.visuals_mut().widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    ui.visuals_mut().widgets.inactive.bg_fill = Color32::TRANSPARENT;
    ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::NONE;
}
fn navigation_focus(ui: &egui::Ui, rect: egui::Rect, response: &egui::Response) {
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect,
            FIELD_RADIUS,
            ui.visuals().selection.stroke,
            egui::StrokeKind::Inside,
        );
    }
}
fn navigation_title(ui: &mut egui::Ui, rect: egui::Rect, text: &str) {
    let mut job = egui::text::LayoutJob::simple(
        text.to_owned(),
        FontId::proportional(13.),
        FG,
        (rect.width() - 54.).max(1.),
    );
    job.wrap.max_rows = 1;
    job.wrap.break_anywhere = true;
    job.wrap.overflow_character = Some('…');
    let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
    ui.painter().with_clip_rect(rect).galley(
        egui::pos2(rect.left() + 46., rect.center().y - galley.size().y / 2.),
        galley,
        FG,
    );
}
impl App {
    pub(super) fn history_surface(&mut self, ui: &mut egui::Ui) {
        let view_height = ui.available_height();
        let margin = if ui.available_width() < 640. { 24 } else { 40 };
        egui::ScrollArea::vertical()
            .id_salt("history-page")
            .show(ui, |ui| {
                Frame::NONE
                    .inner_margin(Margin::symmetric(margin, 32))
                    .show(ui, |ui| {
                        ui.set_max_width(ui.available_width().min(920.));
                        ui.label(RichText::new("Past chats").size(30.).strong());
                        ui.label(
                            RichText::new("Find and resume a conversation.")
                                .size(13.)
                                .color(MUTED),
                        );
                        ui.add_space(24.);
                        if let Some(index) = self.index() {
                            let current = self.saved.workspaces[index].id.clone();
                            let title = self.saved.workspaces[index]
                                .directory
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .into_owned();
                            let projects: Vec<_> = self
                                .saved
                                .workspaces
                                .iter()
                                .map(|workspace| {
                                    (
                                        workspace.id.clone(),
                                        workspace
                                            .directory
                                            .file_name()
                                            .unwrap_or_default()
                                            .to_string_lossy()
                                            .into_owned(),
                                    )
                                })
                                .collect();
                            ui.horizontal(|ui| {
                                icons::inline(ui, Icon::Folder, FG);
                                let combo_width = (ui.available_width() - 120.).clamp(80., 240.);
                                ui.allocate_ui_with_layout(
                                    egui::vec2(combo_width, 32.),
                                    Layout::left_to_right(Align::Center),
                                    |ui| {
                                        egui::ComboBox::from_id_salt("history-project")
                                            .width(combo_width)
                                            .truncate()
                                            .selected_text(title)
                                            .show_ui(ui, |ui| {
                                                for (id, name) in projects {
                                                    if ui
                                                        .selectable_label(current == id, name)
                                                        .clicked()
                                                    {
                                                        self.saved.selected = Some(id);
                                                        self.history_selected = None;
                                                        self.actions.push(Action::History(None));
                                                    }
                                                }
                                            });
                                    },
                                );
                                icons::inline(ui, Icon::Terminal, MUTED);
                                ui.label("OpenCode");
                            });
                        }
                        ui.add_space(12.);
                        if search_input(ui, &mut self.history.search, "Search chats").changed() {
                            self.history.changed = Some(Instant::now());
                        }
                        ui.add_space(18.);
                        let (heading_rect, _) = ui.allocate_exact_size(
                            egui::vec2(ui.available_width(), 24.),
                            Sense::hover(),
                        );
                        ui.painter().text(
                            egui::pos2(heading_rect.left() + 36., heading_rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            "Chat",
                            FontId::proportional(12.),
                            MUTED,
                        );
                        ui.painter().text(
                            egui::pos2(heading_rect.right() - 16., heading_rect.center().y),
                            egui::Align2::RIGHT_CENTER,
                            "Updated",
                            FontId::proportional(12.),
                            MUTED,
                        );
                        ui.add_space(6.);
                        if self.history.busy {
                            ui.spinner();
                        }
                        if let Some(error) = &self.history.error {
                            ui.label(error);
                            if ui.button("Try again").clicked() {
                                self.actions.push(Action::History(None));
                            }
                        }
                        if let Some(page) = &self.history.page {
                            egui::ScrollArea::vertical()
                                .id_salt("past-chat-rows")
                                .max_height((view_height * 0.45).max(120.))
                                .show(ui, |ui| {
                                    if page.data.is_empty() {
                                        ui.add_space(18.);
                                        ui.label(
                                            RichText::new(
                                                "No OpenCode chats found for this project.",
                                            )
                                            .color(MUTED),
                                        );
                                    }
                                    for chat in &page.data {
                                        let selected = self
                                            .history_selected
                                            .as_ref()
                                            .is_some_and(|item| item.id == chat.id);
                                        let (rect, response) = ui.allocate_exact_size(
                                            egui::vec2(ui.available_width(), 40.),
                                            Sense::click(),
                                        );
                                        if selected || response.hovered() {
                                            ui.painter().rect_filled(
                                                rect,
                                                FIELD_RADIUS,
                                                if selected {
                                                    crate::surfaces::selected(ui.ctx())
                                                } else {
                                                    crate::surfaces::hover(ui.ctx())
                                                },
                                            );
                                        }
                                        navigation_focus(ui, rect, &response);
                                        icons::paint(
                                            ui.painter(),
                                            egui::Rect::from_center_size(
                                                egui::pos2(rect.left() + 19., rect.center().y),
                                                egui::vec2(16., 16.),
                                            ),
                                            Icon::Chat,
                                            FG,
                                        );
                                        let mut title = egui::text::LayoutJob::simple(
                                            chat.title.clone().unwrap_or("Untitled chat".into()),
                                            FontId::proportional(13.),
                                            FG,
                                            (rect.width() - 170.).max(40.),
                                        );
                                        title.wrap.max_rows = 1;
                                        title.wrap.overflow_character = Some('…');
                                        let galley = ui.fonts_mut(|fonts| fonts.layout_job(title));
                                        ui.painter().galley(
                                            egui::pos2(
                                                rect.left() + 36.,
                                                rect.center().y - galley.size().y / 2.,
                                            ),
                                            galley,
                                            FG,
                                        );
                                        let updated = chat
                                            .time
                                            .get("updated")
                                            .and_then(|value| value.as_u64())
                                            .map(|millis| {
                                                let now = std::time::SystemTime::now()
                                                    .duration_since(std::time::UNIX_EPOCH)
                                                    .unwrap_or_default()
                                                    .as_secs();
                                                let age = now.saturating_sub(millis / 1000);
                                                if age < 60 {
                                                    "Just now".into()
                                                } else if age < 3600 {
                                                    format!("{} min ago", age / 60)
                                                } else if age < 86400 {
                                                    format!("{} h ago", age / 3600)
                                                } else {
                                                    format!("{} d ago", age / 86400)
                                                }
                                            })
                                            .unwrap_or_else(|| "—".into());
                                        ui.painter().text(
                                            egui::pos2(rect.right() - 16., rect.center().y),
                                            egui::Align2::RIGHT_CENTER,
                                            updated,
                                            FontId::proportional(12.),
                                            MUTED,
                                        );
                                        if response.double_clicked() {
                                            self.actions.push(Action::Resume(chat.clone()));
                                        } else if response.clicked() {
                                            self.history_selected = Some(chat.clone());
                                        }
                                    }
                                });
                            ui.add_space(10.);
                            ui.horizontal(|ui| {
                                if let Some(cursor) = &page.cursor.previous {
                                    if ui.small_button("Newer").clicked() {
                                        self.actions.push(Action::History(Some(cursor.clone())));
                                    }
                                }
                                if let Some(cursor) = &page.cursor.next {
                                    if ui.small_button("Older").clicked() {
                                        self.actions.push(Action::History(Some(cursor.clone())));
                                    }
                                }
                            });
                        }
                        if let Some(chat) = self.history_selected.clone() {
                            ui.add_space(18.);
                            ui.separator();
                            ui.add_space(18.);
                            ui.label(
                                RichText::new(chat.title.as_deref().unwrap_or("Untitled chat"))
                                    .size(18.)
                                    .strong(),
                            );
                            ui.add_space(8.);
                            ui.label(
                                RichText::new(format!(
                                    "OpenCode · {}",
                                    chat.location.directory.display()
                                ))
                                .color(MUTED),
                            );
                            ui.add_space(18.);
                            if ui
                                .add_sized([220., 36.], primary("Resume in new terminal"))
                                .clicked()
                            {
                                self.actions.push(Action::Resume(chat));
                            }
                            ui.add_space(8.);
                            ui.label(
                                RichText::new(
                                    "Opens this conversation without sending a new prompt.",
                                )
                                .size(12.)
                                .color(MUTED),
                            );
                        }
                    });
            });
    }
    pub(super) fn terminal_failure(&mut self, ui: &mut egui::Ui, id: &str, error: &str) {
        egui::ScrollArea::vertical()
            .id_salt(("terminal-failure", id))
            .show(ui, |ui| {
                Frame::NONE.inner_margin(24).show(ui, |ui| {
                    ui.add_space(24.);
                    ui.horizontal_wrapped(|ui| {
                        icons::inline(ui, Icon::Terminal, FG);
                        ui.label(
                            RichText::new("Could not start this terminal")
                                .size(19.)
                                .strong(),
                        );
                    });
                    if let Some(definition) = self.definition(id) {
                        ui.add_space(10.);
                        ui.label(format!(
                            "{} · {}",
                            definition.tool.label(),
                            definition.directory.display()
                        ));
                    }
                    ui.add_space(16.);
                    ui.label(error);
                    ui.add_space(20.);
                    ui.horizontal_wrapped(|ui| {
                        if ui.add_sized([130., 34.], primary("Try again")).clicked() {
                            self.actions.push(Action::Launch(id.into()));
                        }
                        if ui
                            .add_sized([140., 34.], egui::Button::new("Edit command"))
                            .clicked()
                        {
                            self.actions.push(Action::EditCommand(id.into()));
                        }
                        if ui
                            .add_sized([150., 34.], egui::Button::new("Open PowerShell"))
                            .clicked()
                        {
                            self.actions.push(Action::NewTerminal(Tool::PowerShell));
                        }
                    });
                    ui.add_space(16.);
                    if let Some(workspace) = self.index() {
                        if ui.button("Locate project folder…").clicked() {
                            self.actions.push(Action::LocateWorkspace(
                                self.saved.workspaces[workspace].id.clone(),
                            ));
                        }
                    }
                });
            });
    }
    pub(super) fn creation_surface(&mut self, ctx: &egui::Context) {
        if self.form.is_none() {
            return;
        }
        let (mut browse, mut create, mut cancel) = (false, false, false);
        let form = self.form.as_mut().unwrap();
        let modal_id = egui::Id::new("workspace-form");
        let response = egui::Modal::new(modal_id)
            .area(
                egui::Modal::default_area(modal_id)
                    .default_height(ctx.content_rect().height() - 32.),
            )
            .frame(Frame::window(&ctx.style_of(egui::Theme::Dark)).inner_margin(24))
            .show(ctx, |ui| {
                ui.set_width(500.);
                egui::ScrollArea::vertical()
                    .max_height((ctx.content_rect().height() - 80.).max(100.))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(if form.custom {
                                    "Custom command"
                                } else {
                                    "New workspace"
                                })
                                .size(22.)
                                .strong(),
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                cancel = icons::button(ui, Icon::Close, "Cancel").clicked();
                            });
                        });
                        ui.add_space(20.);
                        if !form.custom {
                            ui.label(RichText::new("Name").strong());
                            ui.add_space(4.);
                            ui.add_sized(
                                [ui.available_width(), 36.],
                                form_input(&mut form.name).hint_text("Daily work"),
                            );
                            ui.add_space(16.);
                        } else {
                            ui.label(RichText::new("Command").strong());
                            ui.add_space(4.);
                            ui.add_sized(
                                [ui.available_width(), 36.],
                                form_input(&mut form.command)
                                    .font(egui::TextStyle::Monospace)
                                    .hint_text("npm run dev"),
                            );
                            ui.add_space(16.);
                        }
                        ui.label(
                            RichText::new(if form.custom {
                                "Working folder"
                            } else {
                                "Project folder"
                            })
                            .strong(),
                        );
                        ui.add_space(4.);
                        ui.horizontal(|ui| {
                            ui.add_sized(
                                [(ui.available_width() - 102.).max(80.), 36.],
                                form_input(&mut form.folder),
                            );
                            browse = ui
                                .add_sized([94., 36.], egui::Button::new("Browse…"))
                                .clicked();
                        });
                        ui.add_space(5.);
                        ui.label(
                            RichText::new(if form.custom {
                                "Custom commands run through PowerShell."
                            } else {
                                "Choose a folder for this workspace."
                            })
                            .size(12.)
                            .color(MUTED),
                        );
                        if !form.custom {
                            ui.add_space(18.);
                            ui.label(RichText::new("Start with").strong());
                            ui.add_space(4.);
                            egui::ComboBox::from_id_salt("first-terminal")
                                .width(ui.available_width())
                                .selected_text(form.start.label())
                                .show_ui(ui, |ui| {
                                    for tool in [
                                        Tool::PowerShell,
                                        Tool::OpenCode,
                                        Tool::ClaudeCode,
                                        Tool::Codex,
                                        Tool::Custom(String::new()),
                                    ] {
                                        let label = tool.label().to_string();
                                        ui.selectable_value(&mut form.start, tool, label);
                                    }
                                });
                            if matches!(form.start, Tool::Custom(_)) {
                                ui.add_space(12.);
                                ui.label(RichText::new("Command").strong());
                                ui.add_space(4.);
                                ui.add_sized(
                                    [ui.available_width(), 36.],
                                    form_input(&mut form.command).font(egui::TextStyle::Monospace),
                                );
                            }
                        }
                        if !form.error.is_empty() {
                            ui.add_space(12.);
                            ui.colored_label(Color32::from_rgb(234, 137, 144), &form.error);
                        }
                        if form.validating {
                            ui.spinner();
                        }
                        ui.add_space(24.);
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            create = ui
                                .add_enabled(
                                    !form.validating,
                                    crate::surfaces::primary_sized(
                                        if form.custom {
                                            "Open terminal"
                                        } else {
                                            "Create workspace"
                                        },
                                        egui::vec2(148., 34.),
                                    ),
                                )
                                .clicked();
                            cancel |= ui
                                .add_sized([90., 34.], egui::Button::new("Cancel"))
                                .clicked();
                        });
                        ui.add_space(8.);
                    });
            });
        if browse {
            self.picker(None, true, ctx);
        }
        if cancel || response.should_close() {
            self.form = None;
        } else if create {
            let form = self.form.as_ref().unwrap();
            if (form.custom || matches!(form.start, Tool::Custom(_)))
                && form.command.trim().is_empty()
            {
                self.form.as_mut().unwrap().error = "Enter a command to run.".into();
            } else if form.custom {
                let form = self.form.take().unwrap();
                self.add_terminal_at(Tool::Custom(form.command), PathBuf::from(form.folder), ctx);
            } else {
                self.actions.push(Action::CreateWorkspace);
            }
        }
    }
    pub(super) fn pane_name(&self, pane: &Pane) -> String {
        match pane {
            Pane::Terminal(id) => self
                .definition(id)
                .map(|d| d.name.clone())
                .unwrap_or("Terminal".into()),
            Pane::Image(path) => path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into(),
            Pane::Settings => "Settings".into(),
            Pane::History => "Past chats".into(),
        }
    }
    pub(super) fn paint_header(&mut self, pane: &Pane, response: &egui::Response) {
        let painter = response
            .ctx
            .layer_painter(response.layer_id)
            .with_clip_rect(response.rect);
        let icon = match pane {
            Pane::Terminal(_) => Icon::Terminal,
            Pane::Image(_) => Icon::Image,
            Pane::Settings => Icon::Settings,
            Pane::History => Icon::Chat,
        };
        let r = response.rect;
        glass_edge(&painter, r, 0.68);
        icons::paint(
            &painter,
            egui::Rect::from_center_size(
                egui::pos2(r.left() + 18., r.center().y),
                egui::vec2(16., 16.),
            ),
            icon,
            FG,
        );
        let mut right = r.right() - 32.;
        if let Pane::Terminal(id) = pane {
            if r.width() > 275. {
                let status = self.status(id);
                let short = status.split('·').next().unwrap_or(&status).trim();
                let color = if short == "Running" {
                    accent_ink(self.saved.settings.accent)
                } else if short.starts_with("Failed") {
                    Color32::from_rgb(229, 91, 110)
                } else {
                    MUTED
                };
                let width = if short == "Failed to start" {
                    108.
                } else {
                    82.
                };
                right -= width;
                painter.circle_filled(egui::pos2(right + 6., r.center().y), 3.5, color);
                painter.text(
                    egui::pos2(right + 16., r.center().y),
                    egui::Align2::LEFT_CENTER,
                    short,
                    FontId::proportional(12.),
                    MUTED,
                );
                right -= 8.;
            }
        }
        let mut job = egui::text::LayoutJob::simple(
            self.pane_name(pane),
            FontId::proportional(13.),
            FG,
            (right - r.left() - 40.).max(20.),
        );
        job.wrap.max_rows = 1;
        job.wrap.break_anywhere = true;
        job.wrap.overflow_character = Some('…');
        let galley = response.ctx.fonts_mut(|fonts| fonts.layout_job(job));
        painter.galley(
            egui::pos2(r.left() + 36., r.center().y - galley.size().y / 2.),
            galley,
            FG,
        );
        let edge = if matches!(pane, Pane::Terminal(id) if self.focus.as_ref() == Some(id)) {
            accent_ink(self.saved.settings.accent)
        } else {
            LINE
        };
        painter.line_segment([r.left_bottom(), r.right_bottom()], Stroke::new(1., edge));
        if response.double_clicked() {
            self.actions.push(Action::Maximize(pane.clone()));
        } else if response.clicked() {
            self.actions.push(Action::Focus(pane.clone()));
        }
    }
    pub(super) fn workspace_chrome(&mut self, ui: &mut egui::Ui) {
        if !self.saved.settings.expanded {
            self.workspace_rail(ui);
            self.workspace_toolbar(ui);
            return;
        }
        egui::Panel::bottom("footer")
            .exact_size(36.)
            .frame(Frame::NONE.fill(CHROME).stroke(Stroke::new(1., LINE)))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.;
                    let (rect, response) =
                        ui.allocate_exact_size(egui::vec2(self.sidebar_width, 34.), Sense::click());
                    response.widget_info(|| {
                        egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "Settings")
                    });
                    if response.hovered() {
                        ui.painter()
                            .rect_filled(rect, 0., crate::surfaces::hover(ui.ctx()));
                    }
                    navigation_focus(ui, rect, &response);
                    icons::paint(
                        ui.painter(),
                        egui::Rect::from_center_size(
                            egui::pos2(rect.left() + 23., rect.center().y),
                            egui::vec2(16., 16.),
                        ),
                        Icon::Settings,
                        FG,
                    );
                    ui.painter().text(
                        egui::pos2(rect.left() + 44., rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        "Settings",
                        FontId::proportional(13.),
                        FG,
                    );
                    ui.painter()
                        .vline(rect.right(), rect.y_range(), Stroke::new(1., LINE));
                    if response.clicked() {
                        self.actions.push(Action::Focus(Pane::Settings));
                    }
                    if let Some(i) = self.index() {
                        let workspace = &self.saved.workspaces[i];
                        let count = workspace.terminals.len();
                        ui.add_space(18.);
                        let width = (ui.available_width() - 125.).max(20.);
                        let (rect, response) =
                            ui.allocate_exact_size(egui::vec2(width, 34.), Sense::hover());
                        ui.painter().with_clip_rect(rect).text(
                            rect.left_center(),
                            egui::Align2::LEFT_CENTER,
                            workspace.directory.to_string_lossy(),
                            FontId::monospace(13.),
                            FG,
                        );
                        response.on_hover_text(workspace.directory.to_string_lossy());
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.add_space(16.);
                            ui.label(format!(
                                "{count} terminal{}",
                                if count == 1 { "" } else { "s" }
                            ));
                        });
                    }
                });
            });
        let sidebar = egui::Panel::left("sidebar")
            .default_size(220.)
            .size_range(220.0..=380.0)
            .frame(Frame::NONE.fill(SIDEBAR).stroke(Stroke::new(1., LINE)))
            .show(ui, |ui| {
                glass_edge(ui.painter(), ui.max_rect(), 0.55);
                Frame::NONE
                    .inner_margin(Margin::symmetric(12, 12))
                    .show(ui, |ui| {
                        search_input(
                            ui,
                            &mut self.navigation_search,
                            "Search workspaces and chats",
                        );
                        ui.add_space(10.);
                        ui.scope(|ui| {
                            flat(ui);
                            ui.horizontal(|ui| {
                                icons::inline(ui, Icon::Plus, FG);
                                if ui
                                    .add_sized(
                                        [ui.available_width(), 30.],
                                        egui::Button::new("").left_text("New workspace"),
                                    )
                                    .clicked()
                                {
                                    self.actions.push(Action::NewWorkspace);
                                }
                            });
                        });
                        ui.add_space(16.);
                        ui.label(RichText::new("Workspaces").size(12.).color(MUTED));
                        ui.add_space(6.);
                        let search = self.navigation_search.to_lowercase();
                        let mut projects: std::collections::BTreeMap<
                            String,
                            (PathBuf, Vec<Workspace>),
                        > = Default::default();
                        for workspace in &self.saved.workspaces {
                            if !search.is_empty()
                                && !workspace.name.to_lowercase().contains(&search)
                                && !workspace
                                    .directory
                                    .to_string_lossy()
                                    .to_lowercase()
                                    .contains(&search)
                                && !workspace
                                    .terminals
                                    .iter()
                                    .any(|t| t.name.to_lowercase().contains(&search))
                            {
                                continue;
                            }
                            let key = workspace.directory.to_string_lossy().to_lowercase();
                            projects
                                .entry(key)
                                .or_insert_with(|| (workspace.directory.clone(), Vec::new()))
                                .1
                                .push(workspace.clone());
                        }
                        egui::ScrollArea::vertical()
                            .id_salt("workspace-list")
                            .max_height((ui.available_height() - 190.).max(160.))
                            .show(ui, |ui| {
                                for (key, (directory, workspaces)) in projects {
                                    let selected = workspaces
                                        .iter()
                                        .any(|w| Some(&w.id) == self.saved.selected.as_ref());
                                    let expanded = *self
                                        .project_expansion
                                        .entry(key.clone())
                                        .or_insert(selected);
                                    let (rect, response) = ui.allocate_exact_size(
                                        egui::vec2(ui.available_width(), 32.),
                                        Sense::click(),
                                    );
                                    if response.hovered() {
                                        ui.painter().rect_filled(
                                            rect,
                                            FIELD_RADIUS,
                                            crate::surfaces::hover(ui.ctx()),
                                        );
                                    }
                                    navigation_focus(ui, rect, &response);
                                    icons::paint(
                                        ui.painter(),
                                        egui::Rect::from_center_size(
                                            egui::pos2(rect.left() + 8., rect.center().y),
                                            egui::vec2(12., 12.),
                                        ),
                                        Icon::Chevron(expanded),
                                        FG,
                                    );
                                    icons::paint(
                                        ui.painter(),
                                        egui::Rect::from_center_size(
                                            egui::pos2(rect.left() + 30., rect.center().y),
                                            egui::vec2(16., 16.),
                                        ),
                                        Icon::Folder,
                                        FG,
                                    );
                                    let name = directory
                                        .file_name()
                                        .unwrap_or(directory.as_os_str())
                                        .to_string_lossy();
                                    navigation_title(ui, rect, &name);
                                    if response
                                        .on_hover_text(directory.to_string_lossy())
                                        .clicked()
                                    {
                                        self.project_expansion.insert(key, !expanded);
                                    }
                                    if expanded {
                                        for workspace in workspaces {
                                            let active =
                                                self.saved.selected.as_ref() == Some(&workspace.id);
                                            let open = *self
                                                .workspace_expansion
                                                .entry(workspace.id.clone())
                                                .or_insert(active);
                                            ui.indent((&workspace.id, "nav"), |ui| {
                                                let (rect, response) = ui.allocate_exact_size(
                                                    egui::vec2(ui.available_width(), 32.),
                                                    Sense::click(),
                                                );
                                                if active || response.hovered() {
                                                    ui.painter().rect_filled(
                                                        rect,
                                                        FIELD_RADIUS,
                                                        if active {
                                                            crate::surfaces::selected(ui.ctx())
                                                        } else {
                                                            crate::surfaces::hover(ui.ctx())
                                                        },
                                                    );
                                                }
                                                navigation_focus(ui, rect, &response);
                                                icons::paint(
                                                    ui.painter(),
                                                    egui::Rect::from_center_size(
                                                        egui::pos2(
                                                            rect.left() + 10.,
                                                            rect.center().y,
                                                        ),
                                                        egui::vec2(12., 12.),
                                                    ),
                                                    Icon::Chevron(open),
                                                    MUTED,
                                                );
                                                icons::paint(
                                                    ui.painter(),
                                                    egui::Rect::from_center_size(
                                                        egui::pos2(
                                                            rect.left() + 30.,
                                                            rect.center().y,
                                                        ),
                                                        egui::vec2(16., 16.),
                                                    ),
                                                    Icon::Folder,
                                                    FG,
                                                );
                                                navigation_title(ui, rect, &workspace.name);
                                                if response.on_hover_text(&workspace.name).clicked()
                                                {
                                                    self.saved.selected =
                                                        Some(workspace.id.clone());
                                                    self.workspace_expansion.insert(
                                                        workspace.id.clone(),
                                                        if active { !open } else { true },
                                                    );
                                                    self.maximized = None;
                                                    self.changed();
                                                }
                                                if open {
                                                    for terminal in &workspace.terminals {
                                                        ui.indent((&terminal.id, "nav"), |ui| {
                                                            let (rect, response) = ui
                                                                .allocate_exact_size(
                                                                    egui::vec2(
                                                                        ui.available_width(),
                                                                        44.,
                                                                    ),
                                                                    Sense::click(),
                                                                );
                                                            let chosen = self.focus.as_ref()
                                                                == Some(&terminal.id)
                                                                && active;
                                                            if chosen || response.hovered() {
                                                                ui.painter().rect_filled(
                                                                    rect,
                                                                    FIELD_RADIUS,
                                                                    if chosen {
                                                                        crate::surfaces::selected(
                                                                            ui.ctx(),
                                                                        )
                                                                    } else {
                                                                        crate::surfaces::hover(
                                                                            ui.ctx(),
                                                                        )
                                                                    },
                                                                );
                                                            }
                                                            navigation_focus(ui, rect, &response);
                                                            icons::paint(
                                                                ui.painter(),
                                                                egui::Rect::from_center_size(
                                                                    egui::pos2(
                                                                        rect.left() + 16.,
                                                                        rect.top() + 16.,
                                                                    ),
                                                                    egui::vec2(16., 16.),
                                                                ),
                                                                Icon::Terminal,
                                                                FG,
                                                            );
                                                            let text_rect =
                                                                egui::Rect::from_min_max(
                                                                    egui::pos2(
                                                                        rect.left() + 34.,
                                                                        rect.top() + 5.,
                                                                    ),
                                                                    egui::pos2(
                                                                        rect.right() - 4.,
                                                                        rect.top() + 26.,
                                                                    ),
                                                                );
                                                            let mut title =
                                                                egui::text::LayoutJob::simple(
                                                                    terminal.name.clone(),
                                                                    FontId::proportional(13.),
                                                                    FG,
                                                                    text_rect.width(),
                                                                );
                                                            title.wrap.max_rows = 1;
                                                            title.wrap.overflow_character =
                                                                Some('…');
                                                            let galley = ui.fonts_mut(|fonts| {
                                                                fonts.layout_job(title)
                                                            });
                                                            ui.painter().galley(
                                                                egui::pos2(
                                                                    text_rect.left(),
                                                                    text_rect.center().y
                                                                        - galley.size().y / 2.,
                                                                ),
                                                                galley,
                                                                FG,
                                                            );
                                                            let status = self.status(&terminal.id);
                                                            let color = if status == "Running" {
                                                                accent_ink(
                                                                    self.saved.settings.accent,
                                                                )
                                                            } else if status.starts_with("Failed") {
                                                                Color32::from_rgb(229, 91, 110)
                                                            } else {
                                                                MUTED
                                                            };
                                                            ui.painter().circle_filled(
                                                                egui::pos2(
                                                                    rect.left() + 37.,
                                                                    rect.top() + 32.,
                                                                ),
                                                                3.,
                                                                color,
                                                            );
                                                            ui.painter().text(
                                                                egui::pos2(
                                                                    rect.left() + 47.,
                                                                    rect.top() + 32.,
                                                                ),
                                                                egui::Align2::LEFT_CENTER,
                                                                status,
                                                                FontId::proportional(12.),
                                                                MUTED,
                                                            );
                                                            if response.clicked() {
                                                                self.saved.selected =
                                                                    Some(workspace.id.clone());
                                                                self.actions.push(Action::Focus(
                                                                    Pane::Terminal(
                                                                        terminal.id.clone(),
                                                                    ),
                                                                ));
                                                            }
                                                        });
                                                    }
                                                }
                                            });
                                        }
                                    }
                                }
                            });
                        ui.add_space(16.);
                        ui.separator();
                        ui.add_space(10.);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Past chats").size(12.).color(MUTED));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui.small_button("View all").clicked() {
                                    self.actions.push(Action::Focus(Pane::History));
                                }
                            });
                        });
                        ui.add_space(6.);
                        if let Some(page) = &self.history.page {
                            for chat in page
                                .data
                                .iter()
                                .filter(|chat| {
                                    search.is_empty()
                                        || chat
                                            .title
                                            .as_deref()
                                            .unwrap_or("")
                                            .to_lowercase()
                                            .contains(&search)
                                })
                                .take(3)
                            {
                                ui.horizontal(|ui| {
                                    icons::inline(ui, Icon::Chat, FG);
                                    let title = chat.title.as_deref().unwrap_or("Untitled chat");
                                    let width = ui.available_width();
                                    if ui
                                        .add_sized(
                                            [width, 28.],
                                            egui::Button::new("")
                                                .left_text(title)
                                                .frame(false)
                                                .truncate(),
                                        )
                                        .on_hover_text(title)
                                        .clicked()
                                    {
                                        self.actions.push(Action::Resume(chat.clone()));
                                    }
                                });
                                ui.horizontal(|ui| {
                                    ui.add_space(26.);
                                    ui.label(RichText::new("OpenCode").size(12.).color(MUTED));
                                });
                                ui.add_space(7.);
                            }
                        } else {
                            ui.label(
                                RichText::new("Open Past Chats to load recent chats.")
                                    .size(12.)
                                    .color(MUTED),
                            );
                        }
                        let response = ui.allocate_response(ui.available_size(), Sense::click());
                        response.widget_info(|| {
                            egui::WidgetInfo::labeled(
                                egui::WidgetType::Button,
                                true,
                                "Return to docked layout",
                            )
                        });
                        if response.clicked() {
                            self.actions.push(Action::WorkspaceView(false));
                        }
                    });
            });
        self.sidebar_width = sidebar.response.rect.width();
        self.workspace_toolbar(ui);
    }

    fn workspace_rail(&mut self, ui: &mut egui::Ui) {
        egui::Panel::left("workspace-rail")
            .exact_size(56.)
            .resizable(false)
            .frame(Frame::NONE.fill(SIDEBAR).stroke(Stroke::new(1., LINE)))
            .show(ui, |ui| {
                glass_edge(ui.painter(), ui.max_rect(), 0.53);
                ui.vertical_centered(|ui| {
                    ui.add_space(8.);
                    if icons::button(ui, Icon::Folder, "Workspaces").clicked() {
                        self.actions.push(Action::WorkspaceView(true));
                    }
                    if icons::button(ui, Icon::Chat, "Past chats").clicked() {
                        self.actions.push(Action::Focus(Pane::History));
                    }
                    let spare = (ui.available_height() - 45.).max(0.);
                    let response = ui
                        .allocate_response(egui::vec2(ui.available_width(), spare), Sense::click());
                    response.widget_info(|| {
                        egui::WidgetInfo::labeled(
                            egui::WidgetType::Button,
                            true,
                            "Expand workspace",
                        )
                    });
                    if response.clicked() {
                        self.actions.push(Action::WorkspaceView(true));
                    }
                    if icons::button(ui, Icon::Settings, "Settings").clicked() {
                        self.actions.push(Action::Focus(Pane::Settings));
                    }
                });
            });
    }

    fn workspace_toolbar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("toolbar")
            .exact_size(44.)
            .frame(
                Frame::NONE
                    .fill(CHROME)
                    .stroke(Stroke::new(1., LINE))
                    .inner_margin(Margin::symmetric(16, 5)),
            )
            .show(ui, |ui| {
                glass_edge(
                    ui.painter(),
                    ui.max_rect().expand2(egui::vec2(15., 4.)),
                    0.65,
                );
                ui.horizontal(|ui| {
                    if let Some(i) = self.index() {
                        let workspace = &self.saved.workspaces[i];
                        let project = workspace
                            .directory
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy();
                        let breadcrumb = format!("{} / {}", project, workspace.name);
                        let mut job = egui::text::LayoutJob::default();
                        let muted = egui::TextFormat {
                            font_id: FontId::proportional(13.),
                            color: MUTED,
                            ..Default::default()
                        };
                        let title_format = egui::TextFormat {
                            font_id: FontId::proportional(15.),
                            color: FG,
                            ..Default::default()
                        };
                        job.append(&workspace.name, 0., title_format);
                        job.append(&format!("   /   {project}"), 0., muted);
                        let width = (ui.available_width() - 220.).max(0.);
                        ui.allocate_ui_with_layout(
                            egui::vec2(width, 30.),
                            Layout::left_to_right(Align::Center),
                            |ui| ui.add(egui::Label::new(job).truncate()),
                        )
                        .inner
                        .on_hover_text(breadcrumb);
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if icons::button(ui, Icon::Image, "Open image preview").clicked() {
                                self.pick_preview(ui.ctx());
                            }
                            let layout_menu = ui.menu_button(" ", |ui| {
                                for (label, preset) in [
                                    ("Columns", Preset::Columns),
                                    ("Rows", Preset::Rows),
                                    ("2 × 2 grid", Preset::Grid),
                                ] {
                                    if ui.button(label).clicked() {
                                        self.actions.push(Action::Layout(preset));
                                        ui.close();
                                    }
                                }
                            });
                            icons::paint(
                                ui.painter(),
                                layout_menu.response.rect,
                                Icon::Columns,
                                FG,
                            );
                            layout_menu.response.on_hover_text("Pane layout");
                            ui.separator();
                            egui::Popup::menu(&ui.add(primary("+  New terminal"))).show(|ui| {
                                self.terminal_menu(ui);
                            });
                        });
                    } else {
                        ui.add_sized([140., 30.], egui::Label::new("Workspaces"));
                    }
                });
            });
    }
    fn pick_preview(&self, ctx: &egui::Context) {
        let tx = self.tx.clone();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Images", &["png", "jpg", "jpeg", "webp", "gif", "bmp"])
                .pick_file()
            {
                let _ = tx.send(Action::Preview(path));
                ctx.request_repaint();
            }
        });
    }
    fn terminal_menu(&mut self, ui: &mut egui::Ui) {
        ui.set_min_width(210.);
        for tool in [
            Tool::PowerShell,
            Tool::OpenCode,
            Tool::ClaudeCode,
            Tool::Codex,
        ] {
            ui.horizontal(|ui| {
                icons::inline(ui, Icon::Terminal, FG);
                if ui
                    .add_sized([165., 28.], egui::Button::new(tool.label()).frame(false))
                    .clicked()
                {
                    self.actions.push(Action::NewTerminal(tool));
                    ui.close();
                }
            });
        }
        ui.separator();
        if ui
            .add_sized(
                [200., 30.],
                egui::Button::new("Custom command…").frame(false),
            )
            .clicked()
        {
            self.actions
                .push(Action::NewTerminal(Tool::Custom(String::new())));
            ui.close();
        }
        ui.separator();
        if let Some(i) = self.index() {
            ui.label(
                RichText::new(format!(
                    "In {}",
                    self.saved.workspaces[i].directory.display()
                ))
                .size(12.)
                .color(MUTED),
            );
        }
    }
    pub(super) fn attachment_footer(&mut self, ui: &mut egui::Ui, id: &str) {
        let Some(def) = self.definition(id).cloned() else {
            return;
        };
        let files = def
            .chat
            .as_ref()
            .map(|chat| self.runtime.bridge.files(chat))
            .unwrap_or_default();
        Frame::NONE
            .inner_margin(Margin::symmetric(12, 8))
            .show(ui, |ui| {
                if !files.is_empty() {
                    let staged = crate::surfaces::glass_inset(ui.ctx())
                        .inner_margin(12)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let color = accent_ink(self.saved.settings.accent);
                                ui.label(RichText::new("Staged").size(12.).color(color));
                                let target = format!("To {}", def.name);
                                let title_width = (ui.available_width() - 40.).max(0.);
                                ui.allocate_ui_with_layout(
                                    egui::vec2(title_width, 30.),
                                    Layout::left_to_right(Align::Center),
                                    |ui| {
                                        ui.add(
                                            egui::Label::new(
                                                RichText::new(&target).size(13.).color(MUTED),
                                            )
                                            .truncate(),
                                        )
                                    },
                                )
                                .inner
                                .on_hover_text(target);
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    self.attach_menu(ui, id);
                                });
                            });
                            egui::ScrollArea::vertical()
                                .id_salt(("staged-attachments", id))
                                .min_scrolled_height(0.)
                                .max_height(34.)
                                .show(ui, |ui| {
                                    ui.horizontal_wrapped(|ui| {
                                        for file in &files {
                                            let remove_label =
                                                format!("Remove staged reference: {}", file.name);
                                            let remove = ui
                                                .add_sized(
                                                    [ui.available_width().clamp(60., 200.), 24.],
                                                    egui::Button::new(format!("×  {}", file.name))
                                                        .truncate(),
                                                )
                                                .on_hover_text(&remove_label);
                                            remove.widget_info(|| {
                                                egui::WidgetInfo::labeled(
                                                    egui::WidgetType::Button,
                                                    ui.is_enabled(),
                                                    &remove_label,
                                                )
                                            });
                                            if remove.clicked() {
                                                if let Some(chat) = &def.chat {
                                                    self.runtime.bridge.remove(chat, &file.uri);
                                                }
                                            }
                                            if let Ok(url) = url::Url::parse(&file.uri) {
                                                if let Ok(path) = url.to_file_path() {
                                                    if path.extension().is_some_and(|ext| {
                                                        ["png", "jpg", "jpeg", "webp", "gif", "bmp"]
                                                            .contains(
                                                                &ext.to_string_lossy()
                                                                    .to_ascii_lowercase()
                                                                    .as_str(),
                                                            )
                                                    }) {
                                                        if icons::button(
                                                            ui,
                                                            Icon::Image,
                                                            "Preview image",
                                                        )
                                                        .clicked()
                                                        {
                                                            self.actions
                                                                .push(Action::Preview(path));
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    });
                                });
                        });
                    glass_edge(ui.painter(), staged.response.rect.shrink(3.), 0.42);
                }
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 7.;
                    flat(ui);
                    if ui.small_button("Add files").clicked() {
                        self.picker(Some(id.into()), false, ui.ctx());
                    }
                    ui.label(RichText::new("·").color(MUTED));
                    if ui.small_button("Add folder").clicked() {
                        self.picker(Some(id.into()), true, ui.ctx());
                    }
                    if files.is_empty() {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            self.attach_menu(ui, id);
                        });
                    }
                });
            });
    }
    fn attach_menu(&self, ui: &mut egui::Ui, id: &str) {
        let attachment_menu = ui.menu_button(" ", |ui| {
            ui.label(
                RichText::new(format!(
                    "To {}",
                    self.definition(id)
                        .map(|d| d.name.as_str())
                        .unwrap_or("terminal")
                ))
                .color(MUTED),
            );
            if ui.button("Add files…").clicked() {
                self.picker(Some(id.into()), false, ui.ctx());
                ui.close();
            }
            if ui.button("Add folder…").clicked() {
                self.picker(Some(id.into()), true, ui.ctx());
                ui.close();
            }
            if ui.button("Paste screenshot").clicked() {
                let _ = self.tx.send(Action::Screenshot(id.into()));
                ui.ctx().request_repaint();
                ui.close();
            }
            ui.separator();
            ui.label(
                RichText::new("References are inserted without sending.")
                    .size(12.)
                    .color(MUTED),
            );
        });
        icons::paint(
            ui.painter(),
            attachment_menu.response.rect,
            Icon::Paperclip,
            FG,
        );
        attachment_menu.response.on_hover_text("Add attachments");
    }
    pub(super) fn maximized_header(&mut self, ui: &mut egui::Ui, pane: &Pane) {
        let header = Frame::NONE
            .fill(CHROME)
            .inner_margin(Margin::symmetric(10, 2))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    icons::inline(
                        ui,
                        match pane {
                            Pane::Settings => Icon::Settings,
                            Pane::History => Icon::Chat,
                            Pane::Image(_) => Icon::Image,
                            _ => Icon::Terminal,
                        },
                        FG,
                    );
                    let title = self.pane_name(pane);
                    let actions_width = if matches!(pane, Pane::Terminal(_) | Pane::Image(_)) {
                        80.
                    } else {
                        40.
                    };
                    let title_width = (ui.available_width() - actions_width).max(20.);
                    ui.allocate_ui_with_layout(
                        egui::vec2(title_width, 30.),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            ui.add(
                                egui::Label::new(RichText::new(&title).size(15.).strong())
                                    .truncate(),
                            )
                        },
                    )
                    .inner
                    .on_hover_text(title);
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if icons::button(ui, Icon::Close, "Close pane").clicked() {
                            self.actions.push(Action::Close(pane.clone()));
                        }
                        if matches!(pane, Pane::Terminal(_) | Pane::Image(_))
                            && icons::button(ui, Icon::Maximize, "Restore pane").clicked()
                        {
                            self.actions.push(Action::Maximize(pane.clone()));
                        }
                    });
                });
            });
        glass_edge(ui.painter(), header.response.rect, 0.62);
    }
    pub(super) fn image_surface(&mut self, ui: &mut egui::Ui, path: &PathBuf) {
        if !self.images.contains_key(path)
            && !self.image_errors.contains_key(path)
            && self.loading.insert(path.clone())
        {
            self.media.request(path, ui.ctx());
        }
        if let Some(image) = self.images.get_mut(path) {
            let narrow_footer = ui.available_width() < 260.;
            let footer_height = if narrow_footer { 114. } else { 52. };
            let size = egui::vec2(
                ui.available_width(),
                (ui.available_height() - footer_height).max(40.),
            );
            let (rect, response) = ui.allocate_exact_size(size, Sense::drag());
            let bounds = rect.shrink(18.);
            let native = image.texture.size_vec2();
            if image.fit {
                image.zoom = (bounds.width() / native.x)
                    .min(bounds.height() / native.y)
                    .min(1.)
                    .max(0.01);
                image.pan = egui::Vec2::ZERO;
            }
            if response.dragged() {
                image.fit = false;
                image.pan += response.drag_delta();
            }
            if response.hovered() {
                let delta = ui.ctx().input(|input| input.smooth_scroll_delta.y);
                if delta != 0. {
                    let before = image.zoom;
                    let after = media::wheel_zoom(before, delta);
                    let pointer = response.hover_pos().unwrap_or(bounds.center()) - bounds.center();
                    image.pan = pointer - (pointer - image.pan) * (after / before);
                    image.zoom = after;
                    image.fit = false;
                    ui.ctx().request_repaint();
                }
            }
            if response.double_clicked() {
                image.fit = true;
                image.pan = egui::Vec2::ZERO;
            }
            let target =
                egui::Rect::from_center_size(bounds.center() + image.pan, native * image.zoom);
            ui.painter().with_clip_rect(rect).image(
                image.texture.id(),
                target,
                egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1., 1.)),
                Color32::WHITE,
            );
            Frame::NONE
                .inner_margin(Margin::symmetric(14, 6))
                .show(ui, |ui| {
                    let caption = format!(
                        "{} · {} × {}",
                        path.file_name().unwrap_or_default().to_string_lossy(),
                        native.x as u32,
                        native.y as u32
                    );
                    let mut controls = |ui: &mut egui::Ui| {
                        ui.label(
                            RichText::new(format!("{:.0}%", image.zoom * 100.))
                                .size(12.)
                                .color(MUTED),
                        );
                        if ui.small_button("Fit").clicked() {
                            image.fit = true;
                            image.pan = egui::Vec2::ZERO;
                        }
                        if ui.small_button("−").clicked() {
                            image.fit = false;
                            image.zoom = (image.zoom / 1.25).max(0.02);
                        }
                        if ui.small_button("+").clicked() {
                            image.fit = false;
                            image.zoom = (image.zoom * 1.25).min(16.);
                        }
                    };
                    if narrow_footer {
                        ui.add(
                            egui::Label::new(RichText::new(&caption).size(12.).color(MUTED))
                                .truncate(),
                        )
                        .on_hover_text(&caption);
                        ui.horizontal_wrapped(|ui| controls(ui));
                    } else {
                        ui.horizontal(|ui| {
                            let label_width = (ui.available_width() - 190.).max(0.);
                            ui.allocate_ui_with_layout(
                                egui::vec2(label_width, 32.),
                                Layout::left_to_right(Align::Center),
                                |ui| {
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(&caption).size(12.).color(MUTED),
                                        )
                                        .truncate(),
                                    )
                                },
                            )
                            .inner
                            .on_hover_text(&caption);
                            ui.allocate_ui_with_layout(
                                egui::vec2(182., 32.),
                                Layout::left_to_right(Align::Center),
                                |ui| controls(ui),
                            );
                        });
                    }
                });
        } else if let Some(error) = self.image_errors.get(path).cloned() {
            let top_space = ui.available_height() * 0.18;
            egui::ScrollArea::vertical()
                .id_salt(("image-error", path))
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(top_space);
                        icons::inline(ui, Icon::Image, FG);
                        ui.heading("Image unavailable");
                        ui.label("The file may have moved or cannot be read.");
                        ui.add_space(16.);
                        if ui.add(primary("Locate image…")).clicked() {
                            self.actions.push(Action::LocateImage(path.clone()));
                        }
                        if ui.button("Try again").clicked() {
                            self.image_errors.remove(path);
                            self.loading.insert(path.clone());
                            self.media.request(path, ui.ctx());
                        }
                        ui.add_space(12.);
                        ui.collapsing("Details", |ui| {
                            ui.label(error);
                            ui.label(path.to_string_lossy());
                        });
                    });
                });
        } else {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() * 0.4);
                ui.spinner();
                ui.label("Loading image…");
            });
        }
    }
}
