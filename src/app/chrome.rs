use super::*;
use crate::icons::{self, Icon};
use egui::{Align, FontId, Frame, Layout, Margin, Sense, Stroke};
const MUTED: Color32 = Color32::from_rgb(170, 170, 176);
const LINE: Color32 = Color32::from_rgb(58, 58, 62);
const CHROME: Color32 = Color32::from_rgb(31, 31, 33);
const FG: Color32 = Color32::from_rgb(237, 237, 237);
fn flat(ui: &mut egui::Ui) {
    ui.visuals_mut().widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    ui.visuals_mut().widgets.inactive.bg_fill = Color32::TRANSPARENT;
    ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::NONE;
}
fn primary(label: &str) -> egui::Button<'_> {
    egui::Button::new(RichText::new(label).color(Color32::from_gray(23)))
        .fill(Color32::from_gray(240))
        .stroke(Stroke::new(1., Color32::from_gray(249)))
}
fn section(ui: &mut egui::Ui, name: &str) {
    ui.add_space(24.);
    ui.label(RichText::new(name).size(17.).strong());
    ui.add_space(12.);
}
fn field_label(ui: &mut egui::Ui, label: &str, width: f32) {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 32.), Sense::hover());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, label));
    ui.painter().text(
        rect.left_center(),
        egui::Align2::LEFT_CENTER,
        label,
        FontId::proportional(14.),
        FG,
    );
}
impl App {
    pub(super) fn history_surface(&mut self, ui: &mut egui::Ui) {
        Frame::NONE
            .inner_margin(Margin::symmetric(24, 22))
            .show(ui, |ui| {
                ui.label(RichText::new("Past chats").size(24.).strong());
                ui.add_space(18.);
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
                        egui::ComboBox::from_id_salt("history-project")
                            .width(240.)
                            .selected_text(title)
                            .show_ui(ui, |ui| {
                                for (id, name) in projects {
                                    if ui.selectable_label(current == id, name).clicked() {
                                        self.saved.selected = Some(id);
                                        self.history_selected = None;
                                        self.actions.push(Action::History(None));
                                    }
                                }
                            });
                        icons::inline(ui, Icon::Terminal, MUTED);
                        ui.label("OpenCode");
                    });
                }
                ui.add_space(12.);
                Frame::new()
                    .fill(Color32::from_rgb(28, 28, 30))
                    .stroke(Stroke::new(1., LINE))
                    .corner_radius(4)
                    .inner_margin(Margin::symmetric(10, 5))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            icons::inline(ui, Icon::Search, MUTED);
                            if ui
                                .add_sized(
                                    [ui.available_width(), 26.],
                                    egui::TextEdit::singleline(&mut self.history.search)
                                        .hint_text("Search chats")
                                        .frame(Frame::NONE),
                                )
                                .changed()
                            {
                                self.history.changed = Some(Instant::now());
                            }
                        });
                    });
                ui.add_space(18.);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Chat").size(12.).color(MUTED));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.add_space(28.);
                        ui.label(RichText::new("Updated").size(12.).color(MUTED));
                    });
                });
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
                        .max_height((ui.available_height() * 0.55).max(150.))
                        .show(ui, |ui| {
                            if page.data.is_empty() {
                                ui.add_space(18.);
                                ui.label(
                                    RichText::new("No OpenCode chats found for this project.")
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
                                        4.,
                                        Color32::from_gray(if selected { 58 } else { 36 }),
                                    );
                                }
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
                                    FontId::proportional(14.),
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
                        RichText::new(format!("OpenCode · {}", chat.location.directory.display()))
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
                        RichText::new("Opens this conversation without sending a new prompt.")
                            .size(12.)
                            .color(MUTED),
                    );
                }
            });
    }
    pub(super) fn terminal_failure(&mut self, ui: &mut egui::Ui, id: &str, error: &str) {
        Frame::NONE.inner_margin(24).show(ui, |ui| {
            ui.add_space(24.);
            ui.horizontal(|ui| {
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
            ui.horizontal(|ui| {
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
    }
    pub(super) fn creation_surface(&mut self, ctx: &egui::Context) {
        if self.form.is_none() {
            return;
        }
        let (mut browse, mut create, mut cancel) = (false, false, false);
        let form = self.form.as_mut().unwrap();
        let response = egui::Modal::new(egui::Id::new("workspace-form"))
            .frame(Frame::window(&ctx.style_of(egui::Theme::Dark)).inner_margin(24))
            .show(ctx, |ui| {
                ui.set_width(430.);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(if form.custom {
                            "Custom command"
                        } else {
                            "New workspace"
                        })
                        .size(19.)
                        .strong(),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        cancel = icons::button(ui, Icon::Close, "Cancel").clicked();
                    });
                });
                ui.add_space(20.);
                if !form.custom {
                    ui.label("Name");
                    ui.add_sized(
                        [ui.available_width(), 32.],
                        egui::TextEdit::singleline(&mut form.name).hint_text("Daily work"),
                    );
                    ui.add_space(16.);
                } else {
                    ui.label("Command");
                    ui.add_sized(
                        [ui.available_width(), 32.],
                        egui::TextEdit::singleline(&mut form.command)
                            .font(egui::TextStyle::Monospace)
                            .hint_text("npm run dev"),
                    );
                    ui.add_space(16.);
                }
                ui.label(if form.custom {
                    "Working folder"
                } else {
                    "Project folder"
                });
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [(ui.available_width() - 104.).max(80.), 32.],
                        egui::TextEdit::singleline(&mut form.folder),
                    );
                    browse = ui
                        .add_sized([94., 32.], egui::Button::new("Browse…"))
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
                    ui.label("Start with");
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
                        ui.label("Command");
                        ui.add_sized(
                            [ui.available_width(), 32.],
                            egui::TextEdit::singleline(&mut form.command),
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
                            primary(if form.custom {
                                "Open terminal"
                            } else {
                                "Create workspace"
                            })
                            .min_size(egui::vec2(148., 34.)),
                        )
                        .clicked();
                    cancel |= ui
                        .add_sized([90., 34.], egui::Button::new("Cancel"))
                        .clicked();
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
                    Color32::from_rgb(24, 190, 151)
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
                    FG,
                );
                right -= 8.;
            }
        }
        let mut job = egui::text::LayoutJob::simple(
            self.pane_name(pane),
            FontId::proportional(14.),
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
        painter.line_segment([r.left_bottom(), r.right_bottom()], Stroke::new(1., LINE));
        if response.double_clicked() {
            self.actions.push(Action::Maximize(pane.clone()));
        } else if response.clicked() {
            self.actions.push(Action::Focus(pane.clone()));
        }
    }
    pub(super) fn workspace_chrome(&mut self, ui: &mut egui::Ui) {
        egui::Panel::bottom("footer")
            .exact_size(42.)
            .frame(Frame::NONE.fill(CHROME).stroke(Stroke::new(1., LINE)))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.;
                    let (rect, response) =
                        ui.allocate_exact_size(egui::vec2(self.sidebar_width, 40.), Sense::click());
                    response.widget_info(|| {
                        egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "Settings")
                    });
                    if response.hovered() {
                        ui.painter().rect_filled(rect, 0., Color32::from_gray(39));
                    }
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
                        FontId::proportional(14.),
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
                            ui.allocate_exact_size(egui::vec2(width, 38.), Sense::hover());
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
            .default_size(272.)
            .size_range(220.0..=380.0)
            .frame(
                Frame::NONE
                    .fill(Color32::from_rgb(29, 29, 31))
                    .stroke(Stroke::new(1., LINE)),
            )
            .show(ui, |ui| {
                Frame::NONE
                    .inner_margin(Margin::symmetric(14, 14))
                    .show(ui, |ui| {
                        Frame::new()
                            .fill(Color32::from_rgb(37, 37, 39))
                            .stroke(Stroke::new(1., Color32::from_gray(63)))
                            .corner_radius(4)
                            .inner_margin(Margin::symmetric(10, 6))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    icons::inline(ui, Icon::Search, MUTED);
                                    let width = ui.available_width();
                                    ui.add_sized(
                                        [width, 22.],
                                        egui::TextEdit::singleline(&mut self.navigation_search)
                                            .hint_text("Search workspaces and chats")
                                            .font(FontId::proportional(13.))
                                            .frame(Frame::NONE),
                                    );
                                });
                            });
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
                        ui.add_space(18.);
                        ui.label(RichText::new("Workspaces").color(MUTED));
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
                                        ui.painter().rect_filled(rect, 4., Color32::from_gray(37));
                                    }
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
                                    ui.painter().text(
                                        egui::pos2(rect.left() + 46., rect.center().y),
                                        egui::Align2::LEFT_CENTER,
                                        &name,
                                        FontId::proportional(14.),
                                        FG,
                                    );
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
                                                        4.,
                                                        Color32::from_gray(if active {
                                                            48
                                                        } else {
                                                            38
                                                        }),
                                                    );
                                                }
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
                                                ui.painter().text(
                                                    egui::pos2(rect.left() + 46., rect.center().y),
                                                    egui::Align2::LEFT_CENTER,
                                                    &workspace.name,
                                                    FontId::proportional(14.),
                                                    FG,
                                                );
                                                if response.clicked() {
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
                                                                        48.,
                                                                    ),
                                                                    Sense::click(),
                                                                );
                                                            let chosen = self.focus.as_ref()
                                                                == Some(&terminal.id)
                                                                && active;
                                                            if chosen || response.hovered() {
                                                                ui.painter().rect_filled(
                                                                    rect,
                                                                    4.,
                                                                    Color32::from_gray(if chosen {
                                                                        64
                                                                    } else {
                                                                        39
                                                                    }),
                                                                );
                                                            }
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
                                                                    FontId::proportional(14.),
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
                                                                Color32::from_rgb(24, 190, 151)
                                                            } else if status.starts_with("Failed") {
                                                                Color32::from_rgb(229, 91, 110)
                                                            } else {
                                                                MUTED
                                                            };
                                                            ui.painter().circle_filled(
                                                                egui::pos2(
                                                                    rect.left() + 37.,
                                                                    rect.top() + 35.,
                                                                ),
                                                                3.,
                                                                color,
                                                            );
                                                            ui.painter().text(
                                                                egui::pos2(
                                                                    rect.left() + 47.,
                                                                    rect.top() + 35.,
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
                            ui.label("Past chats");
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
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                chat.title.as_deref().unwrap_or("Untitled chat"),
                                            )
                                            .frame(false),
                                        )
                                        .clicked()
                                    {
                                        self.actions.push(Action::Resume(chat.clone()));
                                    }
                                });
                                ui.indent(&chat.id, |ui| {
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
                    });
            });
        self.sidebar_width = sidebar.response.rect.width();
        egui::Panel::top("toolbar")
            .exact_size(54.)
            .frame(
                Frame::NONE
                    .fill(CHROME)
                    .stroke(Stroke::new(1., LINE))
                    .inner_margin(Margin::symmetric(16, 9)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if let Some(i) = self.index() {
                        let workspace = &self.saved.workspaces[i];
                        let project = workspace
                            .directory
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy();
                        ui.label(RichText::new(project).size(17.).strong());
                        ui.label(RichText::new("/").size(17.).color(MUTED));
                        ui.label(RichText::new(&workspace.name).size(17.).strong());
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
                            ui.scope(|ui| {
                                flat(ui);
                                ui.menu_button("+  Terminal", |ui| {
                                    self.terminal_menu(ui);
                                });
                            });
                        });
                    } else {
                        ui.label(RichText::new("Terminal Workspace").size(17.).strong());
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
                    Frame::new()
                        .fill(Color32::from_rgb(31, 31, 33))
                        .stroke(Stroke::new(1., Color32::from_gray(64)))
                        .corner_radius(4)
                        .inner_margin(10)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format!("To {}", def.name))
                                        .size(13.)
                                        .color(MUTED),
                                );
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    self.attach_menu(ui, id);
                                });
                            });
                            ui.horizontal_wrapped(|ui| {
                                for file in &files {
                                    if ui
                                        .small_button(format!("{}   ×", file.name))
                                        .on_hover_text("Remove staged reference")
                                        .clicked()
                                    {
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
                                                if icons::button(ui, Icon::Image, "Preview image")
                                                    .clicked()
                                                {
                                                    self.actions.push(Action::Preview(path));
                                                }
                                            }
                                        }
                                    }
                                }
                            });
                        });
                }
                ui.horizontal(|ui| {
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
        Frame::NONE
            .fill(Color32::from_gray(36))
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
                    ui.label(self.pane_name(pane));
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
    }
    pub(super) fn settings_surface(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            Frame::NONE
                .inner_margin(Margin::symmetric(30, 24))
                .show(ui, |ui| {
                    ui.set_max_width(720.);
                    ui.label(RichText::new("Settings").size(26.).strong());
                    ui.add_space(14.);
                    ui.separator();
                    section(ui, "Terminal");
                    ui.horizontal(|ui| {
                        field_label(ui, "Font", 170.);
                        let previous = self.saved.settings.font_family.clone();
                        egui::ComboBox::from_id_salt("terminal-font-family")
                            .width(270.)
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
                    });
                    if let Some(error) = &self.font_error {
                        ui.label(
                            RichText::new(error)
                                .size(12.)
                                .color(Color32::from_rgb(234, 137, 144)),
                        );
                    }
                    ui.add_space(12.);
                    ui.horizontal(|ui| {
                        field_label(ui, "Font size", 170.);
                        if ui.add_sized([32., 32.], egui::Button::new("−")).clicked() {
                            self.saved.settings.font_size =
                                (self.saved.settings.font_size - 1.).max(10.);
                            self.changed();
                        }
                        ui.add_sized(
                            [42., 32.],
                            egui::Label::new(format!("{:.0}", self.saved.settings.font_size)),
                        );
                        if ui.add_sized([32., 32.], egui::Button::new("+")).clicked() {
                            self.saved.settings.font_size =
                                (self.saved.settings.font_size + 1.).min(28.);
                            self.changed();
                        }
                        ui.label(RichText::new("pt").color(MUTED));
                    });
                    ui.add_space(12.);
                    Frame::new()
                        .fill(Color32::from_rgb(21, 21, 22))
                        .stroke(Stroke::new(1., LINE))
                        .corner_radius(4)
                        .inner_margin(Margin::symmetric(16, 12))
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.label(
                                RichText::new("PS X:\\project> echo Hello")
                                    .font(FontId::monospace(self.saved.settings.font_size)),
                            );
                        });
                    ui.add_space(16.);
                    ui.horizontal(|ui| {
                        field_label(ui, "Scrollback", 170.);
                        let before = self.saved.settings.scrollback;
                        egui::ComboBox::from_id_salt("scrollback")
                            .width(200.)
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
                    });
                    ui.horizontal(|ui| {
                        ui.add_space(178.);
                        ui.label(
                            RichText::new("Older lines are discarded.")
                                .size(12.)
                                .color(MUTED),
                        );
                    });
                    ui.add_space(24.);
                    ui.separator();
                    section(ui, "Closing behavior");
                    ui.horizontal(|ui| {
                        field_label(ui, "When I close the window", 170.);
                        let before = self.saved.settings.closing;
                        egui::ComboBox::from_id_salt("closing-policy")
                            .width(250.)
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
                                    ui.selectable_value(
                                        &mut self.saved.settings.closing,
                                        value,
                                        label,
                                    );
                                }
                            });
                        if before != self.saved.settings.closing {
                            self.changed();
                        }
                    });
                    ui.add_space(8.);
                    ui.label(
                        RichText::new("Background terminals keep running and using memory.")
                            .size(12.)
                            .color(MUTED),
                    );
                    ui.add_space(24.);
                    ui.separator();
                    section(ui, "Keyboard shortcuts");
                    for (label, key) in [
                        ("New workspace", "Ctrl+Shift+N"),
                        ("New terminal", "Ctrl+Shift+T"),
                        ("Settings", "Ctrl+,"),
                        ("Paste screenshot", "Ctrl+Shift+I"),
                        ("Restore pane", "Escape"),
                    ] {
                        ui.add_space(10.);
                        ui.horizontal(|ui| {
                            ui.add_sized([240., 32.], egui::Label::new(label));
                            Frame::new()
                                .stroke(Stroke::new(1., LINE))
                                .corner_radius(4)
                                .inner_margin(Margin::symmetric(8, 5))
                                .show(ui, |ui| {
                                    ui.monospace(key);
                                });
                        });
                        ui.add_space(10.);
                        ui.separator();
                    }
                    ui.add_space(10.);
                    ui.label(
                        RichText::new("Terminal shortcuts stay with the running program.")
                            .size(12.)
                            .color(MUTED),
                    );
                });
        });
    }
    pub(super) fn image_surface(&mut self, ui: &mut egui::Ui, path: &PathBuf) {
        if !self.images.contains_key(path)
            && !self.image_errors.contains_key(path)
            && self.loading.insert(path.clone())
        {
            self.media.request(path, ui.ctx());
        }
        if let Some(image) = self.images.get_mut(path) {
            let size = egui::vec2(ui.available_width(), (ui.available_height() - 46.).max(40.));
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
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::Label::new(
                                RichText::new(format!(
                                    "{} · {} × {}",
                                    path.file_name().unwrap_or_default().to_string_lossy(),
                                    native.x as u32,
                                    native.y as u32
                                ))
                                .size(12.)
                                .color(MUTED),
                            )
                            .truncate(),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.small_button("+").clicked() {
                                image.fit = false;
                                image.zoom = (image.zoom * 1.25).min(16.);
                            }
                            if ui.small_button("−").clicked() {
                                image.fit = false;
                                image.zoom = (image.zoom / 1.25).max(0.02);
                            }
                            if ui.small_button("Fit").clicked() {
                                image.fit = true;
                                image.pan = egui::Vec2::ZERO;
                            }
                            ui.label(
                                RichText::new(format!("{:.0}%", image.zoom * 100.))
                                    .size(12.)
                                    .color(MUTED),
                            );
                        });
                    });
                });
        } else if let Some(error) = self.image_errors.get(path).cloned() {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() * 0.28);
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
        } else {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() * 0.4);
                ui.spinner();
                ui.label("Loading image…");
            });
        }
    }
}
