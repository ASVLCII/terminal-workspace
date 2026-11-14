mod chrome;
mod settings;
#[cfg(windows)]
mod title_bar;
use eframe::egui::{self, Color32, RichText};
use egui_dock::{DockArea, DockState, TabViewer};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::mpsc,
    time::{Duration, Instant},
};
use terminal_workspace::{
    layout::{self, Preset},
    media,
    model::*,
    runtime::{Life, Manager},
    store::{Store, Writer},
};
enum Action {
    NewWorkspace,
    CreateWorkspace,
    NewTerminal(Tool),
    Focus(Pane),
    Launch(String),
    Layout(Preset),
    WorkspaceView(bool),
    Maximize(Pane),
    Close(Pane),
    Attach(String, Vec<PathBuf>),
    PickedFolder(String, PathBuf),
    WorkspaceReady(String, String, PathBuf),
    FormError(String, String),
    Preview(PathBuf),
    LocateImage(PathBuf),
    ImageLocated(PathBuf, PathBuf),
    Screenshot(String),
    Error(String),
    History(Option<String>),
    Resume(terminal_workspace::history::Chat),
    LocateWorkspace(String),
    WorkspaceMoved(String, PathBuf),
    EditCommand(String),
}
struct Form {
    id: String,
    validating: bool,
    start: Tool,
    name: String,
    folder: String,
    command: String,
    custom: bool,
    error: String,
}
struct Preview {
    texture: egui::TextureHandle,
    zoom: f32,
    fit: bool,
    pan: egui::Vec2,
}
#[derive(Default)]
struct History {
    search: String,
    page: Option<terminal_workspace::history::Page>,
    busy: bool,
    generation: u64,
    error: Option<String>,
    changed: Option<Instant>,
}
pub struct App {
    glass: bool,
    saved: Saved,
    writer: Writer,
    runtime: Manager,
    dirty: Option<Instant>,
    load_error: bool,
    save_failed: bool,
    actions: Vec<Action>,
    tx: mpsc::Sender<Action>,
    rx: mpsc::Receiver<Action>,
    form: Option<Form>,
    error: Option<String>,
    starting: HashSet<String>,
    failures: HashMap<String, String>,
    focus: Option<String>,
    focus_request: Option<String>,
    terminal_widgets: HashMap<String, egui::Id>,
    maximized: Option<Pane>,
    media: media::Loader,
    images: HashMap<PathBuf, Preview>,
    loading: HashSet<PathBuf>,
    image_errors: HashMap<PathBuf, String>,
    navigation_search: String,
    project_expansion: HashMap<String, bool>,
    workspace_expansion: HashMap<String, bool>,
    sidebar_width: f32,
    history_selected: Option<terminal_workspace::history::Chat>,
    font_loader: crate::fonts::Loader,
    font_choices: Vec<crate::fonts::Choice>,
    font_actual: String,
    font_error: Option<String>,
    close: bool,
    remember: bool,
    closing: bool,
    drag_snapshot: Option<DockState<Pane>>,
    dock_epoch: u64,
    drag_cancelled: bool,
    history: History,
    history_tx: mpsc::Sender<terminal_workspace::runtime::HistoryResult>,
    history_rx: mpsc::Receiver<terminal_workspace::runtime::HistoryResult>,
    resume_question: Option<terminal_workspace::history::Chat>,
    _instance: terminal_workspace::instance::Instance,
    tray: Option<terminal_workspace::tray::Tray>,
    explicit_quit: bool,
    save_gate: terminal_workspace::closing::SaveGate,
    saved_action: Option<terminal_workspace::closing::Choice>,
    pending_stop_save: bool,
    final_exit_save: bool,
    discard_on_exit: bool,
    shutdown_waiting: bool,
    shutdown_started: Option<Instant>,
    global_pane: Option<Pane>,
    close_seen: bool,
    terminal_question: Option<String>,
    terminal_stopping: Option<(String, Instant)>,
}
impl App {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        instance: terminal_workspace::instance::Instance,
    ) -> anyhow::Result<Self> {
        let store = Store::user()?;
        let (saved, load_error, error) = match store.load() {
            Ok((s, recovered)) => (
                s,
                false,
                recovered.then(|| {
                    "Recovered the last good workspace save. Recent changes may be missing.".into()
                }),
            ),
            Err(e) => (
                Saved::default(),
                true,
                Some(format!(
                    "{e:#}. Existing state is preserved; saving is disabled."
                )),
            ),
        };
        crate::surfaces::apply(&cc.egui_ctx, saved.settings.accent);
        cc.egui_ctx.set_theme(egui::Theme::Dark);
        let font_loader =
            crate::fonts::Loader::start(cc.egui_ctx.clone(), saved.settings.font_family.clone());
        let writer = Writer::start(store.directory, cc.egui_ctx.clone());
        let plugin = std::env::current_exe()?.parent().unwrap().join("plugin");
        let plugin = if plugin.is_dir() {
            plugin
        } else {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("plugin")
        };
        let runtime = Manager::new(plugin)?;
        let (tx, rx) = mpsc::channel();
        let (history_tx, history_rx) = mpsc::channel();
        let tray = terminal_workspace::tray::Tray::new(&cc.egui_ctx);
        let tray = match tray {
            Ok(tray) => Some(tray),
            Err(_) => None,
        };
        let mut app = Self {
            glass: crate::backdrop::enable(cc),
            saved,
            writer,
            runtime,
            dirty: None,
            load_error,
            save_failed: false,
            actions: Vec::new(),
            tx,
            rx,
            form: None,
            error,
            starting: HashSet::new(),
            failures: HashMap::new(),
            focus: None,
            focus_request: None,
            terminal_widgets: HashMap::new(),
            maximized: None,
            media: Default::default(),
            images: HashMap::new(),
            loading: HashSet::new(),
            image_errors: HashMap::new(),
            navigation_search: String::new(),
            project_expansion: HashMap::new(),
            workspace_expansion: HashMap::new(),
            sidebar_width: 272.,
            history_selected: None,
            font_loader,
            font_choices: Vec::new(),
            font_actual: "Loading font…".into(),
            font_error: None,
            close: false,
            remember: false,
            closing: false,
            drag_snapshot: None,
            dock_epoch: 0,
            drag_cancelled: false,
            history: History::default(),
            history_tx,
            history_rx,
            resume_question: None,
            _instance: instance,
            tray,
            explicit_quit: false,
            save_gate: Default::default(),
            saved_action: None,
            pending_stop_save: false,
            final_exit_save: false,
            discard_on_exit: false,
            shutdown_waiting: false,
            shutdown_started: None,
            global_pane: None,
            close_seen: false,
            terminal_question: None,
            terminal_stopping: None,
        };
        if std::env::args().any(|a| a == "--benchmark-workload") {
            anyhow::ensure!(
                std::env::var_os("TERMINAL_WORKSPACE_DATA_DIR").is_some()
                    && app.saved.workspaces.is_empty(),
                "Benchmark requires an explicitly isolated empty data directory"
            );
            let workspace = Workspace::new("Memory workload".into(), std::env::current_dir()?);
            app.saved.selected = Some(workspace.id.clone());
            app.saved.workspaces.push(workspace);
            for _ in 0..4 {
                app.add_terminal(Tool::PowerShell, &cc.egui_ctx);
            }
            let panes = app.saved.workspaces[0]
                .dock
                .iter_all_tabs()
                .map(|(_, p)| p.clone())
                .collect();
            app.saved.workspaces[0].dock = layout::preset(panes, Preset::Grid);
            if let Some(path) = std::env::var_os("TERMINAL_WORKSPACE_BENCH_IMAGE") {
                app.saved.workspaces[0].dock.main_surface_mut().split_right(
                    egui_dock::NodeIndex::root(),
                    0.74,
                    vec![Pane::Image(path.into())],
                );
            }
        }
        Ok(app)
    }
    fn choose_close(&mut self, choice: terminal_workspace::closing::Choice, ctx: &egui::Context) {
        use terminal_workspace::closing::{self, Choice};
        if choice == Choice::Cancel {
            self.focus_request = self.focus.clone();
            self.close = false;
            self.remember = false;
            self.explicit_quit = false;
            return;
        }
        if choice == Choice::Background && self.tray.is_none() {
            self.error=Some("A usable system tray could not be created. The window will stay open so you can return to your terminals.".into());
            return;
        }
        if self.load_error {
            self.error=Some("Saved state could not be loaded safely. Existing files are preserved. Return to the app to resolve the error, or choose Close without saving to stop this run and keep the existing files.".into());
            return;
        }
        if !self.explicit_quit {
            self.saved.settings.closing =
                closing::preference(self.saved.settings.closing, choice, self.remember);
        }
        self.close = false;
        self.close_seen = false;
        self.remember = false;
        if choice == Choice::Background {
            self.focus_request = self.focus.clone();
        }
        self.saved_action = Some(choice);
        if choice == Choice::Stop {
            self.form = None;
            self.resume_question = None;
            self.terminal_question = None;
        }
        if choice == Choice::Stop && !self.starting.is_empty() {
            self.pending_stop_save = true;
            self.dirty = None;
            ctx.request_repaint();
            return;
        }
        let generation = self.writer.save(self.saved.clone());
        self.save_gate.wait(generation);
        self.dirty = None;
        ctx.request_repaint();
    }
    fn close_ui(&mut self, ctx: &egui::Context) {
        use terminal_workspace::closing::Choice;
        if self.pending_stop_save {
            egui::Modal::new(egui::Id::new("startup-before-close")).show(ctx, |ui| {
                ui.set_width(420.);
                ui.heading("Closing workspace");
                ui.add_space(16.);
                ui.spinner();
                ui.label("Waiting for terminal startup to finish.");
            });
        }
        if let Some(id) = self.terminal_question.clone() {
            let name = self
                .definition(&id)
                .map(|d| d.name.clone())
                .unwrap_or("terminal".into());
            let (mut cancel, mut stop) = (false, false);
            let response = egui::Modal::new(egui::Id::new("stop-terminal"))
                .frame(egui::Frame::window(&ctx.style_of(egui::Theme::Dark)).inner_margin(24))
                .show(ctx, |ui| {
                    ui.set_width(420.);
                    ui.add(egui::Label::new(RichText::new(format!("Stop {name}?")).size(20.).strong()).wrap());
                    ui.add_space(8.);
                    ui.label("This ends this terminal and its owned processes. Other terminals keep running.");
                    ui.add_space(20.);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        stop = ui.add_sized([120., 34.], egui::Button::new("Stop terminal")).clicked();
                        cancel = ui.add_sized([90., 34.], egui::Button::new("Cancel")).clicked();
                    });
                });
            if cancel || response.should_close() {
                self.terminal_question = None;
            }
            if stop {
                self.terminal_question = None;
                self.runtime.stop_terminal(&id);
                self.terminal_stopping = Some((id, Instant::now()));
            }
        }
        if let Some((id, since)) = self.terminal_stopping.clone() {
            if since.elapsed() > Duration::from_secs(5) {
                let (mut force, mut cancel) = (false, false);
                egui::Modal::new(egui::Id::new("force-terminal"))
                    .frame(egui::Frame::window(&ctx.style_of(egui::Theme::Dark)).inner_margin(24))
                    .show(ctx, |ui| {
                        ui.set_width(420.);
                        ui.heading("This terminal is still running");
                        ui.add_space(8.);
                        ui.label("Normal shutdown has not finished.");
                        ui.add_space(20.);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            force = ui
                                .add_sized([150., 34.], egui::Button::new("Force stop terminal"))
                                .clicked();
                            cancel = ui
                                .add_sized([120., 34.], egui::Button::new("Keep terminal"))
                                .clicked();
                        });
                    });
                if cancel {
                    self.terminal_stopping = None;
                }
                if force {
                    if let Some(t) = self.runtime.terminals.get(&id) {
                        if let Err(e) = t.job.force_stop() {
                            self.error = Some(format!("{e:#}"));
                            self.terminal_stopping = None;
                        }
                    }
                }
            }
        }
        if !self.close {
            self.close_seen = false;
        }
        if self.close {
            let first = !self.close_seen;
            self.close_seen = true;
            let mut action = None;
            let response = egui::Modal::new(egui::Id::new("close-warning"))
                .frame(egui::Frame::window(&ctx.style_of(egui::Theme::Dark)).inner_margin(24))
                .show(ctx, |ui| {
                    ui.set_width(470.);
                    ui.heading(if self.explicit_quit {
                        "Quit Terminal Workspace?"
                    } else {
                        "Close window?"
                    });
                    ui.add_space(12.);
                    ui.label(format!(
                        "{} terminals are running. Stopping them may interrupt work.",
                        self.runtime.running()
                    ));
                    if !self.explicit_quit {
                        ui.add_space(8.);
                        ui.checkbox(&mut self.remember, "Don’t show this again");
                        ui.horizontal(|ui| {
                            ui.add_space(ui.spacing().icon_width + ui.spacing().icon_spacing);
                            ui.label(
                                RichText::new("Remember the action you choose below.")
                                    .size(12.)
                                    .color(crate::surfaces::MUTED),
                            );
                        });
                    }
                    ui.add_space(24.);
                    ui.horizontal(|ui| {
                        let total = if self.explicit_quit { 310. } else { 372. };
                        ui.add_space(((ui.available_width() - total) / 2.).max(0.));
                        let cancel = ui.add_sized([76., 34.], egui::Button::new("Cancel"));
                        if first {
                            cancel.request_focus();
                        }
                        if cancel.clicked() {
                            action = Some(Choice::Cancel);
                        }
                        if !self.explicit_quit
                            && ui
                                .add_sized([148., 34.], egui::Button::new("Run in background"))
                                .clicked()
                        {
                            action = Some(Choice::Background);
                        }
                        if ui
                            .add_sized(
                                [if self.explicit_quit { 226. } else { 132. }, 34.],
                                crate::surfaces::primary(if self.explicit_quit {
                                    "Stop terminals and quit"
                                } else {
                                    "Stop and close"
                                }),
                            )
                            .clicked()
                        {
                            action = Some(Choice::Stop);
                        }
                    });
                });
            if response.should_close() {
                action = Some(Choice::Cancel);
            }
            if let Some(action) = action {
                self.choose_close(action, ctx);
            }
        }
        if self.shutdown_waiting || self.shutdown_started.is_some() {
            let force = self
                .shutdown_started
                .is_some_and(|t| t.elapsed() > Duration::from_secs(5));
            let mut force_stop = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("shutdown"))
                .frame(egui::Frame::window(&ctx.style_of(egui::Theme::Dark)).inner_margin(24))
                .show(ctx, |ui| {
                    ui.set_width(440.);
                    ui.heading(if force { "Some processes are still running" } else { "Stopping terminals…" });
                    ui.add_space(12.);
                    if force {
                        ui.label("Normal shutdown has not finished. Force stop affects only processes owned by this app.");
                        ui.add_space(20.);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            force_stop = ui.add_sized([160., 34.], egui::Button::new("Force stop and quit")).clicked();
                            cancel = ui.add_sized([120., 34.], egui::Button::new("Return to app")).clicked();
                        });
                    } else {
                        ui.spinner();
                        ui.label("Waiting for owned terminals and the OpenCode backend.");
                    }
                });
            if cancel {
                self.shutdown_waiting = false;
                self.shutdown_started = None;
                self.explicit_quit = false;
            }
            if force_stop {
                match self.runtime.force_stop() {
                    Ok(()) => {
                        self.shutdown_started = Some(Instant::now());
                    }
                    Err(error) => {
                        self.error = Some(format!("{error:#}"));
                        self.shutdown_started = None;
                    }
                }
            }
        }
    }
    fn index(&self) -> Option<usize> {
        self.saved
            .workspaces
            .iter()
            .position(|w| Some(&w.id) == self.saved.selected.as_ref())
    }
    fn changed(&mut self) {
        self.dirty = Some(Instant::now());
    }
    fn definition(&self, id: &str) -> Option<&TerminalDefinition> {
        self.saved
            .workspaces
            .iter()
            .flat_map(|w| &w.terminals)
            .find(|t| t.id == id)
    }
    fn status(&self, id: &str) -> String {
        if self.starting.contains(id) {
            "Starting".into()
        } else if self.failures.contains_key(id) {
            "Failed to start".into()
        } else {
            self.runtime
                .terminals
                .get(id)
                .map(|t| t.life.label())
                .unwrap_or("Stopped".into())
        }
    }
    fn add_terminal(&mut self, tool: Tool, ctx: &egui::Context) {
        if let Some(i) = self.index() {
            self.add_terminal_at(tool, self.saved.workspaces[i].directory.clone(), ctx);
        }
    }
    fn add_terminal_at(&mut self, tool: Tool, directory: PathBuf, ctx: &egui::Context) {
        if let Some(i) = self.index() {
            let w = &mut self.saved.workspaces[i];
            let id = uuid::Uuid::new_v4().to_string();
            let def = TerminalDefinition {
                id: id.clone(),
                name: tool.label().into(),
                tool,
                directory,
                chat: None,
            };
            w.terminals.push(def.clone());
            w.dock.push_to_focused_leaf(Pane::Terminal(id.clone()));
            if w.terminals.len() == 2 {
                w.dock = layout::workspace_view(w, self.saved.settings.expanded);
            }
            self.starting.insert(id.clone());
            self.focus_request = Some(id.clone());
            self.focus = Some(id);
            self.runtime.spawn(def, ctx, self.saved.settings.scrollback);
            self.changed();
        }
    }
    fn picker(&self, recipient: Option<String>, folder: bool, ctx: &egui::Context) {
        let tx = self.tx.clone();
        let form_id = self
            .form
            .as_ref()
            .map(|form| form.id.clone())
            .unwrap_or_default();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let dialog = rfd::FileDialog::new();
            let paths = if folder {
                dialog.pick_folder().map(|p| vec![p])
            } else {
                dialog.pick_files()
            };
            if let Some(paths) = paths {
                let action = if let Some(id) = recipient {
                    Action::Attach(id, paths)
                } else {
                    Action::PickedFolder(form_id, paths[0].clone())
                };
                let _ = tx.send(action);
                ctx.request_repaint();
            }
        });
    }
    fn focus_pane(&mut self, pane: Pane) {
        if matches!(pane, Pane::Settings | Pane::History) {
            self.global_pane = Some(pane);
            return;
        }
        self.global_pane = None;
        if let Some(i) = self.index() {
            let dock = &mut self.saved.workspaces[i].dock;
            if let Some(path) = dock.find_tab(&pane) {
                let _ = dock.set_active_tab(path);
            } else {
                dock.push_to_focused_leaf(pane.clone());
                if matches!(pane, Pane::Image(_)) {
                    self.saved.workspaces[i].dock = layout::workspace_view(
                        &self.saved.workspaces[i],
                        self.saved.settings.expanded,
                    );
                    let dock = &mut self.saved.workspaces[i].dock;
                    if self.saved.settings.expanded
                        && let Some(tab) = dock.find_tab(&pane)
                    {
                        let _ = dock.set_active_tab(tab);
                    }
                }
            }
            if let Pane::Terminal(id) = pane {
                self.focus_request = Some(id.clone());
                self.focus = Some(id);
            }
            self.changed();
        }
    }
    fn process_actions(&mut self, ctx: &egui::Context) {
        if self.saved_action == Some(terminal_workspace::closing::Choice::Stop)
            || self.final_exit_save
            || self.shutdown_waiting
            || self.shutdown_started.is_some()
        {
            return;
        }
        self.actions.extend(self.rx.try_iter());
        for action in std::mem::take(&mut self.actions) {
            match action {
                Action::LocateImage(old) => {
                    let tx = self.tx.clone();
                    let context = ctx.clone();
                    std::thread::spawn(move || {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp", "bmp"])
                            .pick_file()
                        {
                            let _ = tx.send(Action::ImageLocated(old, path));
                            context.request_repaint();
                        }
                    });
                }
                Action::ImageLocated(old, path) => {
                    for workspace in &mut self.saved.workspaces {
                        if let Some(old_tab) = workspace.dock.find_tab(&Pane::Image(old.clone())) {
                            if workspace
                                .dock
                                .find_tab(&Pane::Image(path.clone()))
                                .is_some()
                                && old != path
                            {
                                workspace.dock.remove_tab(old_tab);
                            } else {
                                for (_, tab) in workspace.dock.iter_all_tabs_mut() {
                                    if *tab == Pane::Image(old.clone()) {
                                        *tab = Pane::Image(path.clone());
                                    }
                                }
                            }
                        }
                    }
                    if self.maximized == Some(Pane::Image(old.clone())) {
                        self.maximized = Some(Pane::Image(path));
                    }
                    self.images.remove(&old);
                    self.image_errors.remove(&old);
                    self.loading.remove(&old);
                    self.changed();
                }
                Action::LocateWorkspace(id) => {
                    let tx = self.tx.clone();
                    let ctx = ctx.clone();
                    std::thread::spawn(move || {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            let action =
                                match terminal_workspace::process::canonical_directory(&path) {
                                    Ok(path) => Action::WorkspaceMoved(id, path),
                                    Err(error) => Action::Error(format!("{error:#}")),
                                };
                            let _ = tx.send(action);
                            ctx.request_repaint();
                        }
                    });
                }
                Action::WorkspaceMoved(id, path) => {
                    if let Some(workspace) = self
                        .saved
                        .workspaces
                        .iter_mut()
                        .find(|workspace| workspace.id == id)
                    {
                        let old = workspace.directory.clone();
                        workspace.directory = path.clone();
                        for terminal in &mut workspace.terminals {
                            if let Ok(relative) = terminal.directory.strip_prefix(&old) {
                                terminal.directory = path.join(relative);
                            }
                        }
                        self.changed();
                    }
                }
                Action::EditCommand(id) => {
                    if let Some(definition) = self.definition(&id) {
                        let command = match &definition.tool {
                            Tool::Custom(command) => command.clone(),
                            Tool::OpenCode => "opencode".into(),
                            Tool::ClaudeCode => "claude".into(),
                            Tool::Codex => "codex".into(),
                            Tool::PowerShell => "powershell".into(),
                        };
                        self.form = Some(Form {
                            id: uuid::Uuid::new_v4().to_string(),
                            validating: false,
                            start: Tool::PowerShell,
                            name: "Custom command".into(),
                            folder: definition.directory.to_string_lossy().into(),
                            command,
                            custom: true,
                            error: String::new(),
                        });
                    }
                }
                Action::NewWorkspace => {
                    self.form = Some(Form {
                        id: uuid::Uuid::new_v4().to_string(),
                        validating: false,
                        start: Tool::PowerShell,
                        name: String::new(),
                        folder: String::new(),
                        command: String::new(),
                        custom: false,
                        error: String::new(),
                    })
                }
                Action::CreateWorkspace => {
                    if let Some(form) = self.form.as_mut() {
                        if form.validating {
                            continue;
                        }
                        form.validating = true;
                        let (id, name, folder) = (
                            form.id.clone(),
                            form.name.clone(),
                            PathBuf::from(&form.folder),
                        );
                        let tx = self.tx.clone();
                        let ctx = ctx.clone();
                        std::thread::spawn(move || {
                            let result = (|| -> anyhow::Result<PathBuf> {
                                anyhow::ensure!(
                                    folder.is_absolute() && folder.is_dir(),
                                    "Choose an existing absolute project folder."
                                );
                                terminal_workspace::process::canonical_directory(&folder)
                            })();
                            let action = match result {
                                Ok(directory) => Action::WorkspaceReady(id, name, directory),
                                Err(error) => Action::FormError(id, format!("{error:#}")),
                            };
                            let _ = tx.send(action);
                            ctx.request_repaint();
                        });
                    }
                }
                Action::WorkspaceReady(id, name, directory) => {
                    if self.form.as_ref().is_some_and(|form| form.id == id) {
                        let name = if name.trim().is_empty() {
                            directory
                                .file_name()
                                .unwrap_or(directory.as_os_str())
                                .to_string_lossy()
                                .into()
                        } else {
                            name.trim().into()
                        };
                        let workspace = Workspace::new(name, directory);
                        self.saved.selected = Some(workspace.id.clone());
                        self.saved.workspaces.push(workspace);
                        let tool = self
                            .form
                            .as_ref()
                            .map(|form| {
                                if matches!(form.start, Tool::Custom(_)) {
                                    Tool::Custom(form.command.clone())
                                } else {
                                    form.start.clone()
                                }
                            })
                            .unwrap_or(Tool::PowerShell);
                        self.form = None;
                        self.add_terminal(tool, ctx);
                    }
                }
                Action::FormError(id, error) => {
                    if let Some(form) = self.form.as_mut() {
                        if form.id == id {
                            form.validating = false;
                            form.error = error;
                        }
                    }
                }
                Action::NewTerminal(Tool::Custom(command)) if command.is_empty() => {
                    if let Some(i) = self.index() {
                        self.form = Some(Form {
                            id: uuid::Uuid::new_v4().to_string(),
                            validating: false,
                            start: Tool::PowerShell,
                            name: "Custom command".into(),
                            folder: self.saved.workspaces[i].directory.to_string_lossy().into(),
                            command: String::new(),
                            custom: true,
                            error: String::new(),
                        });
                    }
                }
                Action::NewTerminal(tool) => self.add_terminal(tool, ctx),
                Action::Focus(pane) => {
                    if pane == Pane::History {
                        self.actions.push(Action::History(None));
                    }
                    self.focus_pane(pane);
                }
                Action::History(cursor) => {
                    if let Some(i) = self.index() {
                        self.history.generation += 1;
                        self.history.busy = true;
                        self.history.error = None;
                        self.runtime.history(
                            self.saved.workspaces[i].directory.clone(),
                            self.history.search.clone(),
                            cursor,
                            self.history.generation,
                            self.history_tx.clone(),
                            ctx,
                        );
                    }
                }
                Action::Resume(chat) => {
                    let existing = self.saved.workspaces.iter().find_map(|w| {
                        w.terminals
                            .iter()
                            .find(|t| t.chat.as_deref() == Some(&chat.id))
                            .map(|t| (w.id.clone(), t.id.clone()))
                    });
                    if let Some((workspace, id)) = existing {
                        self.saved.selected = Some(workspace);
                        self.focus_pane(Pane::Terminal(id.clone()));
                        self.actions.push(Action::Launch(id));
                        continue;
                    }
                    if let Some(i) = self.index() {
                        if self.saved.workspaces[i]
                            .directory
                            .to_string_lossy()
                            .to_lowercase()
                            != chat.location.directory.to_string_lossy().to_lowercase()
                        {
                            self.resume_question = Some(chat);
                            continue;
                        }
                    }
                    self.resume_chat(chat, ctx);
                }
                Action::Launch(id) => {
                    if self.starting.contains(&id)
                        || self
                            .runtime
                            .terminals
                            .get(&id)
                            .is_some_and(|t| matches!(t.life, Life::Running | Life::Stopping))
                    {
                        continue;
                    }
                    if self
                        .runtime
                        .terminals
                        .get(&id)
                        .is_some_and(|t| t.job.active().unwrap_or(1) > 0)
                    {
                        self.error = Some("This terminal still owns running processes. Stop it before starting another command.".into());
                        continue;
                    }
                    if let Some(def) = self.definition(&id).cloned() {
                        self.runtime.terminals.remove(&id);
                        self.starting.insert(id.clone());
                        self.failures.remove(&id);
                        self.runtime.spawn(def, ctx, self.saved.settings.scrollback);
                    }
                }
                Action::Layout(preset) => {
                    if let Some(i) = self.index() {
                        let panes = self.saved.workspaces[i]
                            .dock
                            .iter_all_tabs()
                            .map(|(_, p)| p.clone())
                            .collect();
                        self.saved.workspaces[i].dock = layout::preset(panes, preset);
                        self.maximized = None;
                        self.changed();
                    }
                }
                Action::WorkspaceView(expanded) => {
                    if self.saved.settings.expanded != expanded {
                        self.saved.settings.expanded = expanded;
                        if let Some(i) = self.index() {
                            let workspace = &mut self.saved.workspaces[i];
                            workspace.dock = layout::workspace_view(workspace, expanded);
                            if let Some(tab) = self
                                .focus
                                .as_ref()
                                .and_then(|id| workspace.dock.find_tab(&Pane::Terminal(id.clone())))
                            {
                                let _ = workspace.dock.set_active_tab(tab);
                            }
                            self.focus_request = self.focus.clone();
                            self.maximized = None;
                            self.drag_snapshot = None;
                            self.dock_epoch += 1;
                        }
                        self.changed();
                    }
                }
                Action::Maximize(pane) => {
                    if let Pane::Terminal(id) = &pane {
                        self.focus_request = Some(id.clone());
                    }
                    self.maximized = if self.maximized.as_ref() == Some(&pane) {
                        None
                    } else {
                        Some(pane)
                    };
                }
                Action::Close(pane) => {
                    if matches!(pane, Pane::Settings | Pane::History) {
                        self.global_pane = None;
                        self.focus_request = self.focus.clone();
                        continue;
                    }
                    if let Some(index) = self.index() {
                        if let Pane::Terminal(id) = &pane {
                            let active = self
                                .runtime
                                .terminals
                                .get(id)
                                .is_some_and(|terminal| terminal.job.active().unwrap_or(1) > 0);
                            match self.saved.workspaces[index].remove_terminal(
                                id,
                                self.starting.contains(id),
                                active,
                            ) {
                                TerminalRemoval::Starting => {
                                    self.error=Some("This terminal is still starting. Wait for startup to finish, then close it.".into());
                                    continue;
                                }
                                TerminalRemoval::RequiresStop => {
                                    self.terminal_question = Some(id.clone());
                                    continue;
                                }
                                TerminalRemoval::Removed => {
                                    self.runtime.terminals.remove(id);
                                    self.runtime.bridge.forget(id);
                                }
                            }
                        } else if let Some(path) = self.saved.workspaces[index].dock.find_tab(&pane)
                        {
                            self.saved.workspaces[index].dock.remove_tab(path);
                        }
                    }
                    self.maximized = None;
                    self.changed();
                }
                Action::Attach(id, paths) => self.attach(&id, paths, ctx),
                Action::Error(error) => self.error = Some(error),
                Action::Screenshot(id) => {
                    let tx = self.tx.clone();
                    let ctx = ctx.clone();
                    std::thread::spawn(move || {
                        let result = Store::user()
                            .and_then(|store| media::clipboard_image(&store.directory));
                        let action = match result {
                            Ok(path) => Action::Attach(id, vec![path]),
                            Err(e) => Action::Error(format!("{e:#}")),
                        };
                        let _ = tx.send(action);
                        ctx.request_repaint();
                    });
                }
                Action::PickedFolder(id, path) => {
                    if let Some(form) = self.form.as_mut() {
                        if form.id == id {
                            form.folder = path.to_string_lossy().into();
                        }
                    }
                }
                Action::Preview(path) => {
                    if !self.images.contains_key(&path) && self.loading.insert(path.clone()) {
                        self.media.request(&path, ctx);
                    }
                    self.focus_pane(Pane::Image(path));
                }
            }
        }
    }
    fn resume_chat(&mut self, chat: terminal_workspace::history::Chat, ctx: &egui::Context) {
        let directory = chat.location.directory;
        let found = self.saved.workspaces.iter().position(|w| {
            w.directory.to_string_lossy().to_lowercase()
                == directory.to_string_lossy().to_lowercase()
        });
        let i = found.unwrap_or_else(|| {
            let name = directory
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into();
            self.saved
                .workspaces
                .push(Workspace::new(name, directory.clone()));
            self.saved.workspaces.len() - 1
        });
        let w = &mut self.saved.workspaces[i];
        self.saved.selected = Some(w.id.clone());
        let id = uuid::Uuid::new_v4().to_string();
        let def = TerminalDefinition {
            id: id.clone(),
            name: chat.title.unwrap_or("OpenCode".into()),
            directory,
            tool: Tool::OpenCode,
            chat: Some(chat.id),
        };
        w.terminals.push(def.clone());
        w.dock.push_to_focused_leaf(Pane::Terminal(id.clone()));
        if w.terminals.len() == 2 {
            w.dock = layout::workspace_view(w, self.saved.settings.expanded);
        }
        self.starting.insert(id.clone());
        self.focus_request = Some(id.clone());
        self.focus = Some(id);
        self.runtime.spawn(def, ctx, self.saved.settings.scrollback);
        self.changed();
    }
    fn history_ui(&mut self, ui: &mut egui::Ui) {
        self.history_surface(ui);
    }
    fn attach(&mut self, id: &str, paths: Vec<PathBuf>, ctx: &egui::Context) {
        let Some(def) = self.definition(id).cloned() else {
            return;
        };
        if def.tool == Tool::OpenCode {
            if let Some(chat) = def.chat {
                let bridge = self.runtime.bridge.clone();
                let tx = self.tx.clone();
                let ctx = ctx.clone();
                std::thread::spawn(move || {
                    for path in paths {
                        if let Err(e) = bridge.stage(&chat, &path) {
                            let _ = tx.send(Action::Error(format!(
                                "Could not stage {}: {e:#}",
                                path.display()
                            )));
                        }
                    }
                    ctx.request_repaint();
                });
            } else {
                self.error = Some(
                    "OpenCode is not connected yet. Wait for it to start and try again.".into(),
                );
            }
        } else if let Some(terminal) = self.runtime.terminals.get_mut(id) {
            let text = paths
                .iter()
                .map(|p| {
                    format!(
                        "'{}'",
                        terminal_workspace::process::ps_quote(&p.to_string_lossy())
                    )
                })
                .collect::<Vec<_>>()
                .join(" ");
            terminal
                .backend
                .process_command(egui_term::BackendCommand::Write(text.into_bytes()));
        } else {
            self.error = Some("Start the recipient terminal before inserting a path.".into());
        }
    }
    fn terminal(&mut self, ui: &mut egui::Ui, id: &str) {
        let opencode = self
            .definition(id)
            .is_some_and(|definition| definition.tool == Tool::OpenCode);
        let staged = self
            .definition(id)
            .and_then(|definition| definition.chat.as_ref())
            .map(|chat| !self.runtime.bridge.files(chat).is_empty())
            .unwrap_or(false);
        let footer = if opencode {
            let narrow = ui.available_width() < 244.;
            match (staged, narrow) {
                (true, true) => 204.,
                (true, false) => 164.,
                (false, true) => 96.,
                (false, false) => 56.,
            }
        } else {
            0.
        };
        if let Some(error) = self.failures.get(id).cloned() {
            self.terminal_failure(ui, id, &error);
            return;
        }
        egui::Frame::NONE
            .inner_margin(egui::Margin::symmetric(12, 12))
            .show(ui, |ui| {
                let size = egui::vec2(
                    ui.available_width(),
                    (ui.available_height() - footer).max(30.),
                );
                if let Some(terminal) = self.runtime.terminals.get_mut(id) {
                    let current_focus = ui.memory(|memory| memory.focused());
                    let focused = (self.focus_request.as_deref() == Some(id)
                        || current_focus
                            .is_some_and(|focus| self.terminal_widgets.get(id) == Some(&focus)))
                        && self.form.is_none()
                        && !self.close
                        && self.error.is_none()
                        && self.resume_question.is_none()
                        && self.saved_action.is_none()
                        && !self.shutdown_waiting
                        && self.shutdown_started.is_none()
                        && self.terminal_question.is_none()
                        && self.terminal_stopping.is_none();
                    let view = egui_term::TerminalView::new(ui, &mut terminal.backend)
                        .set_focus(focused)
                        .set_size(size)
                        .set_background(Color32::TRANSPARENT)
                        .set_font(egui_term::TerminalFont::new(egui_term::FontSettings {
                            font_type: egui::FontId::monospace(self.saved.settings.font_size),
                        }));
                    let response = ui.add(view);
                    self.terminal_widgets.insert(id.into(), response.id);
                    if focused && self.focus_request.as_deref() == Some(id) {
                        self.focus_request = None;
                    }
                    if response.clicked() {
                        self.focus = Some(id.into());
                        self.focus_request = Some(id.into());
                        ui.ctx().request_repaint();
                    }
                    if response.hovered() {
                        let files = ui.ctx().input(|i| i.raw.dropped_files.clone());
                        if !files.is_empty() {
                            self.actions.push(Action::Attach(
                                id.into(),
                                files
                                    .into_iter()
                                    .map(|file| file.path().to_owned())
                                    .collect(),
                            ));
                        }
                        if ui.ctx().input(|i| !i.raw.hovered_files.is_empty()) {
                            ui.painter().rect_stroke(
                                response.rect,
                                4.,
                                egui::Stroke::new(2., Color32::from_gray(200)),
                                egui::StrokeKind::Inside,
                            );
                        }
                    }
                    for link in terminal.backend.take_opened_links() {
                        if let Ok(url) = url::Url::parse(&link) {
                            if let Ok(path) = url.to_file_path() {
                                self.actions.push(Action::Preview(path));
                            }
                        }
                    }
                } else {
                    ui.allocate_ui(size, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space((size.y * 0.33).max(10.));
                            ui.label(
                                RichText::new("Saved terminal is stopped")
                                    .size(18.)
                                    .strong(),
                            );
                            ui.add_space(8.);
                            ui.weak("Commands never restart automatically.");
                            ui.add_space(18.);
                            if ui
                                .add_sized([150., 34.], egui::Button::new("Start terminal"))
                                .clicked()
                            {
                                self.actions.push(Action::Launch(id.into()));
                            }
                        });
                    });
                }
            });
        if opencode {
            let available = ui.available_height().max(0.);
            egui::ScrollArea::vertical()
                .id_salt(("attachment-footer-container", id))
                .min_scrolled_height(0.)
                .max_height(available)
                .show(ui, |ui| self.attachment_footer(ui, id));
        }
    }
    fn settings(&mut self, ui: &mut egui::Ui) {
        self.settings_surface(ui);
    }
    fn image(&mut self, ui: &mut egui::Ui, path: &PathBuf) {
        self.image_surface(ui, path);
    }
    fn form(&mut self, ctx: &egui::Context) {
        self.creation_surface(ctx);
    }
}
struct Viewer<'a>(&'a mut App);
impl TabViewer for Viewer<'_> {
    type Tab = Pane;
    fn id(&mut self, tab: &mut Pane) -> egui::Id {
        egui::Id::new(tab)
    }
    fn title(&mut self, tab: &mut Pane) -> egui::WidgetText {
        egui::RichText::new(self.0.pane_name(tab))
            .color(Color32::TRANSPARENT)
            .into()
    }
    fn on_tab_button(&mut self, tab: &mut Pane, response: &egui::Response) {
        self.0.paint_header(tab, response);
    }
    fn context_menu(&mut self, ui: &mut egui::Ui, tab: &mut Pane, _: egui_dock::NodePath) {
        if ui
            .button(if self.0.maximized.is_some() {
                "Restore pane"
            } else {
                "Maximize pane"
            })
            .clicked()
        {
            self.0.actions.push(Action::Maximize(tab.clone()));
            ui.close();
        }
        if let Pane::Terminal(id) = tab {
            ui.separator();
            if ui.button("Add files…").clicked() {
                self.0.picker(Some(id.clone()), false, ui.ctx());
                ui.close();
            }
            if ui.button("Add folder reference…").clicked() {
                self.0.picker(Some(id.clone()), true, ui.ctx());
                ui.close();
            }
            if ui.button("Paste screenshot").clicked() {
                self.0.actions.push(Action::Screenshot(id.clone()));
                ui.close();
            }
        }
    }
    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Pane) {
        match tab {
            Pane::Terminal(id) => self.0.terminal(ui, id),
            Pane::Image(path) => self.0.image(ui, path),
            Pane::Settings => self.0.settings(ui),
            Pane::History => {
                self.0.history_ui(ui);
            }
        }
    }
    fn allowed_in_windows(&self, _: &mut Pane) -> bool {
        false
    }
    fn scroll_bars(&self, _: &Pane) -> [bool; 2] {
        [false, false]
    }
    fn on_close(&mut self, tab: &mut Pane) -> egui_dock::widgets::tab_viewer::OnCloseResponse {
        self.0.actions.push(Action::Close(tab.clone()));
        egui_dock::widgets::tab_viewer::OnCloseResponse::Ignore
    }
}
impl eframe::App for App {
    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        if self.glass {
            [0.; 4]
        } else {
            [0.015, 0.025, 0.045, 1.]
        }
    }

    fn logic(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        let ctx = context.clone();
        while let Some(event) = self.font_loader.poll() {
            match event {
                crate::fonts::Event::Catalog(choices) => self.font_choices = choices,
                crate::fonts::Event::Applied {
                    requested,
                    actual,
                    error,
                } => {
                    if requested == self.saved.settings.font_family {
                        self.font_actual = actual;
                        self.font_error = error;
                    }
                }
            }
        }

        for response in self.history_rx.try_iter() {
            if response.generation == self.history.generation {
                self.history.busy = false;
                match response.result {
                    Ok(page) => self.history.page = Some(page),
                    Err(error) => self.history.error = Some(error),
                }
            }
        }
        if self
            .history
            .changed
            .is_some_and(|t| t.elapsed() > Duration::from_millis(400))
        {
            self.history.changed = None;
            self.actions.push(Action::History(None));
        }
        if self.history.changed.is_some() {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
        self.runtime.events(&ctx);
        if let Some((id, _)) = self.terminal_stopping.clone() {
            if self
                .runtime
                .terminals
                .get(&id)
                .is_none_or(|t| t.job.active().unwrap_or(1) == 0)
            {
                self.terminal_stopping = None;
                self.actions.push(Action::Close(Pane::Terminal(id)));
            } else {
                ctx.request_repaint_after(Duration::from_millis(100));
            }
        }
        if self
            .runtime
            .terminals
            .values()
            .any(|t| t.definition.tool == Tool::OpenCode && t.life == Life::Running)
        {
            ctx.request_repaint_after(Duration::from_millis(500));
        }
        for started in self.runtime.poll() {
            let id = started.definition.id.clone();
            self.starting.remove(&id);
            for w in &mut self.saved.workspaces {
                if let Some(def) = w.terminals.iter_mut().find(|d| d.id == id) {
                    *def = started.definition.clone();
                }
            }
            match started.result {
                Ok(terminal) => {
                    self.runtime.terminals.insert(id, terminal);
                }
                Err(error) => {
                    self.failures.insert(id, error);
                }
            }
            self.changed();
        }
        let associations = self.runtime.bridge.state.lock().unwrap().current.clone();
        let mut association_changed = false;
        for workspace in &mut self.saved.workspaces {
            for definition in &mut workspace.terminals {
                if let Some(chat) = associations.get(&definition.id) {
                    if definition.chat.as_ref() != Some(chat) {
                        definition.chat = Some(chat.clone());
                        association_changed = true;
                    }
                }
            }
        }
        if association_changed {
            self.changed();
        }
        if self.pending_stop_save {
            if self.starting.is_empty() {
                self.pending_stop_save = false;
                let generation = self.writer.save(self.saved.clone());
                self.save_gate.wait(generation);
                self.dirty = None;
            } else {
                ctx.request_repaint_after(Duration::from_millis(100));
            }
        }
        while let Some((generation, result)) = self.writer.poll() {
            match result {
                Err(error) => {
                    self.save_failed = true;
                    self.error = Some(error);
                    self.saved_action = None;
                    self.final_exit_save = false;
                    self.closing = false;
                }
                Ok(()) => {
                    self.save_failed = false;
                    if self.save_gate.acknowledge(generation) {
                        if self.final_exit_save {
                            self.final_exit_save = false;
                            self.closing = true;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        match self.saved_action.take() {
                            Some(terminal_workspace::closing::Choice::Background) => {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false))
                            }
                            Some(terminal_workspace::closing::Choice::Stop) => {
                                self.shutdown_waiting = true
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        if self.shutdown_waiting && self.starting.is_empty() {
            self.shutdown_waiting = false;
            self.runtime.request_shutdown();
            self.shutdown_started = Some(Instant::now());
        }
        if self.shutdown_started.is_some() {
            match self.runtime.remaining() {
                Ok(0) => {
                    self.shutdown_started = None;
                    if self.discard_on_exit {
                        self.closing = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    } else {
                        let generation = self.writer.save(self.saved.clone());
                        self.save_gate.wait(generation);
                        self.final_exit_save = true;
                        self.dirty = None;
                    }
                }
                Ok(_) => {}
                Err(error) => {
                    self.error = Some(format!("{error:#}"));
                    self.shutdown_started = None;
                }
            }
            ctx.request_repaint_after(Duration::from_millis(100));
        }
        if let Some(tray) = self.tray.as_mut() {
            tray.update(self.runtime.running());
        }
        let mut tray_actions = Vec::new();
        if let Some(tray) = &self.tray {
            while let Some(action) = tray.poll() {
                tray_actions.push(action);
            }
        }
        for action in tray_actions {
            match action {
                terminal_workspace::tray::Action::Open => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                }
                terminal_workspace::tray::Action::Quit => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                    self.explicit_quit = true;
                    if self.runtime.running() > 0 || !self.starting.is_empty() {
                        self.close = true;
                    } else {
                        self.choose_close(terminal_workspace::closing::Choice::Stop, &ctx);
                    }
                }
            }
        }
        while let Some(result) = self.media.poll() {
            match result {
                Ok(decoded) => {
                    self.loading.remove(&decoded.path);
                    if !self.saved.workspaces.iter().any(|workspace| {
                        workspace
                            .dock
                            .find_tab(&Pane::Image(decoded.path.clone()))
                            .is_some()
                    }) {
                        continue;
                    }
                    let texture = ctx.load_texture(
                        decoded.path.to_string_lossy(),
                        decoded.image,
                        Default::default(),
                    );
                    self.images.insert(
                        decoded.path,
                        Preview {
                            texture,
                            zoom: 1.,
                            fit: true,
                            pan: egui::Vec2::ZERO,
                        },
                    );
                }
                Err((path, error)) => {
                    self.loading.remove(&path);
                    self.image_errors.insert(path, error);
                }
            }
        }
        if self
            .dirty
            .is_some_and(|t| t.elapsed() > Duration::from_millis(700))
            && !self.load_error
            && self.saved_action.is_none()
            && self.shutdown_started.is_none()
            && !self.shutdown_waiting
            && !self.pending_stop_save
            && !self.final_exit_save
            && self.drag_snapshot.is_none()
        {
            self.writer.save(self.saved.clone());
            self.dirty = None;
        }
        if self.dirty.is_some() || !self.starting.is_empty() || self.final_exit_save {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        crate::surfaces::window_glass(ui.painter(), ui.max_rect());
        let exiting = self.pending_stop_save
            || self.saved_action == Some(terminal_workspace::closing::Choice::Stop)
            || self.shutdown_waiting
            || self.shutdown_started.is_some()
            || self.final_exit_save;
        if exiting {
            ui.disable();
        }
        #[cfg(windows)]
        title_bar::show(ui);
        if ctx.input(|i| i.viewport().close_requested()) && !self.closing {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            if self.saved_action.is_none()
                && self.shutdown_started.is_none()
                && !self.final_exit_save
            {
                self.explicit_quit = false;
                match self.saved.settings.closing {
                    Closing::Ask => self.close = true,
                    Closing::Stop => {
                        self.choose_close(terminal_workspace::closing::Choice::Stop, &ctx)
                    }
                    Closing::Background => {
                        self.choose_close(terminal_workspace::closing::Choice::Background, &ctx)
                    }
                }
            }
        }
        if ctx.input_mut(|i| {
            i.consume_key(egui::Modifiers::CTRL | egui::Modifiers::SHIFT, egui::Key::N)
        }) {
            self.actions.push(Action::NewWorkspace);
        }
        if ctx.input_mut(|i| {
            i.consume_key(egui::Modifiers::CTRL | egui::Modifiers::SHIFT, egui::Key::T)
        }) {
            self.actions.push(Action::NewTerminal(Tool::PowerShell));
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Comma)) {
            self.actions.push(Action::Focus(Pane::Settings));
        }
        if ctx.input_mut(|i| {
            i.consume_key(egui::Modifiers::CTRL | egui::Modifiers::SHIFT, egui::Key::I)
        }) {
            if let Some(id) = self.focus.clone() {
                self.actions.push(Action::Screenshot(id));
            }
        }
        if !exiting && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.maximized = None;
        }
        self.workspace_chrome(ui);
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                if let Some(pane) = self.global_pane.clone() {
                    self.maximized_header(ui, &pane);
                    Viewer(self).ui(ui, &mut pane.clone());
                } else if let Some(pane) = self.maximized.clone() {
                    self.maximized_header(ui, &pane);
                    Viewer(self).ui(ui, &mut pane.clone());
                } else if let Some(i) = self.index() {
                    if !exiting && ctx.input(|input| input.pointer.any_pressed()) {
                        self.drag_snapshot = Some(self.saved.workspaces[i].dock.clone());
                        self.drag_cancelled = false;
                    }
                    if !exiting
                        && self.drag_snapshot.is_some()
                        && ctx.input(|input| input.key_pressed(egui::Key::Escape) || !input.focused)
                    {
                        if let Some(previous) = self.drag_snapshot.take() {
                            self.saved.workspaces[i].dock = previous;
                        }
                        self.dock_epoch += 1;
                        self.drag_cancelled = true;
                    }
                    let mut dock = std::mem::replace(
                        &mut self.saved.workspaces[i].dock,
                        DockState::new(Vec::new()),
                    );
                    let mut dock_style = egui_dock::Style::from_egui(ui.style());
                    dock_style.tab_bar.height = 34.;
                    dock_style.tab_bar.fill_tab_bar = true;
                    dock_style.tab.minimum_width = Some(140.);
                    dock_style.tab_bar.bg_fill = crate::surfaces::CHROME;
                    dock_style.tab.tab_body.inner_margin = egui::Margin::ZERO;
                    dock_style.tab.tab_body.stroke = egui::Stroke::new(1., crate::surfaces::LINE);
                    dock_style.tab.tab_body.bg_fill = Color32::TRANSPARENT;
                    for interaction in [
                        &mut dock_style.tab.active,
                        &mut dock_style.tab.inactive,
                        &mut dock_style.tab.focused,
                        &mut dock_style.tab.hovered,
                    ] {
                        interaction.corner_radius = egui::CornerRadius::ZERO;
                        interaction.outline_color = crate::surfaces::LINE;
                        interaction.bg_fill = crate::surfaces::CHROME;
                    }
                    dock_style.tab.active.bg_fill = crate::surfaces::CHROME;
                    dock_style.tab.focused.bg_fill = Color32::TRANSPARENT;
                    dock_style.tab.hovered.bg_fill = crate::surfaces::HOVER;
                    dock_style.separator.width = 1.;
                    dock_style.separator.extra = 220.;
                    dock_style.separator.color_idle = crate::surfaces::LINE;
                    let mut horizontal = true;
                    let mut vertical = true;
                    for node in dock.main_surface().iter() {
                        if let egui_dock::Node::Leaf(leaf) = node {
                            if leaf.rect.is_finite() {
                                horizontal &= leaf.rect.width() >= 640.;
                                vertical &= leaf.rect.height() >= 420.;
                            }
                        }
                    }
                    let splits = match (horizontal, vertical) {
                        (true, true) => egui_dock::AllowedSplits::All,
                        (true, false) => egui_dock::AllowedSplits::LeftRightOnly,
                        (false, true) => egui_dock::AllowedSplits::TopBottomOnly,
                        _ => egui_dock::AllowedSplits::None,
                    };
                    DockArea::new(&mut dock)
                        .id(egui::Id::new((
                            &self.saved.workspaces[i].id,
                            self.dock_epoch,
                        )))
                        .style(dock_style)
                        .show_leaf_close_all_buttons(false)
                        .show_leaf_collapse_buttons(false)
                        .allowed_splits(splits)
                        .draggable_tabs(!self.drag_cancelled)
                        .show_inside(ui, &mut Viewer(self));
                    self.saved.workspaces[i].dock = dock;
                    if !exiting && ctx.input(|i| i.pointer.any_released()) {
                        self.drag_snapshot = None;
                        self.drag_cancelled = false;
                        self.changed();
                    }
                } else if self.global_pane == Some(Pane::Settings) {
                    self.settings(ui);
                } else {
                    ui.vertical_centered(|ui| {
                        ui.add_space(ui.available_height() * 0.32);
                        ui.heading("Create a workspace");
                        ui.add_space(8.);
                        ui.weak("Choose a project folder and open your first terminal.");
                        ui.add_space(20.);
                        if ui.add(crate::surfaces::primary("New workspace")).clicked() {
                            self.actions.push(Action::NewWorkspace);
                        }
                    });
                }
            });
        if !exiting {
            self.form(&ctx);
        }
        if let Some(chat) = self.resume_question.clone().filter(|_| !exiting) {
            let (mut cancel, mut resume) = (false, false);
            egui::Modal::new(egui::Id::new("resume-project")).show(&ctx, |ui| {
                ui.set_width(440.);
                ui.heading("Open this chat’s project?");
                ui.label(chat.location.directory.to_string_lossy());
                ui.add_space(16.);
                ui.horizontal(|ui| {
                    cancel = ui.button("Cancel").clicked();
                    resume = ui.button("Switch workspace and resume").clicked();
                });
            });
            if cancel {
                self.resume_question = None;
            }
            if resume {
                self.resume_question = None;
                self.resume_chat(chat, &ctx);
            }
        }
        self.close_ui(&ctx);
        if self.error.is_some() {
            let (mut dismiss, mut discard) = (false, false);
            let error_id = egui::Id::new("error");
            let response = egui::Modal::new(error_id)
                .area(
                    egui::Modal::default_area(error_id)
                        .default_height(ctx.content_rect().height() - 32.),
                )
                .frame(egui::Frame::window(&ctx.style_of(egui::Theme::Dark)).inner_margin(24))
                .show(&ctx, |ui| {
                    ui.set_width(480.);
                    ui.heading("Something needs attention");
                    ui.add_space(16.);
                    egui::ScrollArea::vertical()
                        .max_height((ctx.content_rect().height() - 220.).max(80.))
                        .show(ui, |ui| {
                            ui.add(
                                egui::Label::new(self.error.as_deref().unwrap_or_default()).wrap(),
                            );
                        });
                    ui.add_space(24.);
                    ui.horizontal(|ui| {
                        dismiss = ui.button("Return to app").clicked();
                        if self.load_error || self.save_failed {
                            discard = ui.button("Close without saving").clicked();
                        }
                    });
                });
            if dismiss || response.should_close() {
                self.error = None;
            }
            if discard {
                self.error = None;
                self.close = false;
                self.remember = false;
                self.saved_action = None;
                self.pending_stop_save = false;
                self.discard_on_exit = true;
                self.shutdown_waiting = true;
                self.dirty = None;
            }
        }
        self.process_actions(&ctx);
        if self.dirty.is_some() || !self.starting.is_empty() || self.final_exit_save {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
        let live_images: HashSet<_> = self
            .saved
            .workspaces
            .iter()
            .flat_map(|w| w.dock.iter_all_tabs())
            .filter_map(|(_, p)| {
                if let Pane::Image(path) = p {
                    Some(path.clone())
                } else {
                    None
                }
            })
            .collect();
        self.images.retain(|p, _| live_images.contains(p));
    }
}
