use crate::{
    bridge::Bridge,
    model::{TerminalDefinition, Tool},
    opencode::Backend,
    process::{self, Job, Launch},
};
use eframe::egui;
use egui_term::{BackendSettings, PtyEvent, TerminalBackend};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
};

#[derive(Clone, Debug, PartialEq)]
pub enum Life {
    Running,
    Exited(Option<i32>),
    Failed(String),
    Stopping,
}
impl Life {
    pub fn label(&self) -> String {
        match self {
            Self::Running => "Running".into(),
            Self::Exited(Some(0)) => "Exited · code 0".into(),
            Self::Exited(Some(code)) => format!("Failed · code {code}"),
            Self::Exited(None) => "Exited".into(),
            Self::Failed(_) => "Failed to start".into(),
            Self::Stopping => "Stopping".into(),
        }
    }
}
pub struct Terminal {
    pub backend: TerminalBackend,
    pub life: Life,
    pub definition: TerminalDefinition,
    pub job: Job,
    pub numeric_id: u64,
}
pub struct Started {
    pub definition: TerminalDefinition,
    pub result: Result<Terminal, String>,
}
pub struct Manager {
    pub bridge: Arc<Bridge>,
    backend: Arc<Mutex<Option<Backend>>>,
    pub terminals: HashMap<String, Terminal>,
    tx: mpsc::Sender<Started>,
    rx: mpsc::Receiver<Started>,
    event_tx: mpsc::Sender<(u64, PtyEvent)>,
    event_rx: mpsc::Receiver<(u64, PtyEvent)>,
    serial: Arc<AtomicU64>,
    pub plugin: PathBuf,
}
pub struct HistoryResult {
    pub generation: u64,
    pub result: Result<crate::history::Page, String>,
}
impl Manager {
    pub fn history(
        &self,
        directory: PathBuf,
        search: String,
        cursor: Option<String>,
        generation: u64,
        tx: mpsc::Sender<HistoryResult>,
        ctx: &egui::Context,
    ) {
        let (backend, bridge, plugin, ctx) = (
            self.backend.clone(),
            self.bridge.clone(),
            self.plugin.clone(),
            ctx.clone(),
        );
        std::thread::spawn(move || {
            let result = (|| -> anyhow::Result<crate::history::Page> {
                let directory = process::canonical_directory(&directory)?;
                let mut owned = backend
                    .lock()
                    .map_err(|_| anyhow::anyhow!("OpenCode worker failed"))?;
                if owned.is_none() {
                    *owned = Some(Backend::start(&bridge, &directory, &plugin)?);
                }
                let value = owned.as_ref().unwrap().api(
                    reqwest::Method::GET,
                    &crate::history::query(&directory, &search, cursor.as_deref()),
                    None,
                )?;
                Ok(serde_json::from_value(value)?)
            })()
            .map_err(|e| format!("{e:#}"));
            let _ = tx.send(HistoryResult { generation, result });
            ctx.request_repaint();
        });
    }
    pub fn new(plugin: PathBuf) -> anyhow::Result<Self> {
        let (tx, rx) = mpsc::channel();
        let (event_tx, event_rx) = mpsc::channel();
        Ok(Self {
            bridge: Arc::new(Bridge::start()?),
            backend: Arc::new(Mutex::new(None)),
            terminals: HashMap::new(),
            tx,
            rx,
            event_tx,
            event_rx,
            serial: Arc::new(AtomicU64::new(1)),
            plugin,
        })
    }
    pub fn spawn(&self, definition: TerminalDefinition, ctx: &egui::Context, scrollback: usize) {
        let (backend, bridge, tx, event_tx, serial, plugin, ctx) = (
            self.backend.clone(),
            self.bridge.clone(),
            self.tx.clone(),
            self.event_tx.clone(),
            self.serial.clone(),
            self.plugin.clone(),
            ctx.clone(),
        );
        std::thread::spawn(move || {
            let mut definition = definition;
            let result = (|| -> anyhow::Result<Terminal> {
                definition.directory = process::canonical_directory(&definition.directory)?;
                let launch = match &definition.tool {
                    Tool::PowerShell => {
                        process::tool("powershell", &definition.directory, vec!["-NoLogo".into()])?
                    }
                    Tool::ClaudeCode => process::tool("claude", &definition.directory, Vec::new())?,
                    Tool::Codex => process::tool("codex", &definition.directory, Vec::new())?,
                    Tool::Custom(command) => Launch {
                        program: process::POWERSHELL.into(),
                        args: vec![
                            "-NoLogo".into(),
                            "-NoProfile".into(),
                            "-NoExit".into(),
                            "-Command".into(),
                            command.clone(),
                        ],
                        directory: definition.directory.clone(),
                        env: Default::default(),
                    },
                    Tool::OpenCode => {
                        let mut owned = backend
                            .lock()
                            .map_err(|_| anyhow::anyhow!("OpenCode worker failed"))?;
                        if owned.is_none() {
                            *owned = Some(Backend::start(&bridge, &definition.directory, &plugin)?);
                        }
                        let owned = owned.as_ref().unwrap();
                        let session = match &definition.chat {
                            Some(id) => {
                                owned.api(
                                    reqwest::Method::GET,
                                    &format!("/api/session/{id}"),
                                    None,
                                )?;
                                id.clone()
                            }
                            None => owned.create(&definition.directory)?,
                        };
                        bridge.bind(&definition.id, &session);
                        definition.chat = Some(session.clone());
                        owned.launch(&bridge, &definition.id, &session, &definition.directory)?
                    }
                };
                let numeric_id = serial.fetch_add(1, Ordering::Relaxed);
                let job = Job::new()?;
                let terminal = TerminalBackend::new(
                    numeric_id,
                    ctx.clone(),
                    event_tx,
                    BackendSettings {
                        job_object: Some(job.raw()),
                        shell: launch.program,
                        args: launch.args,
                        working_directory: Some(launch.directory),
                        env: launch.env,
                        scrollback,
                    },
                )?;
                Ok(Terminal {
                    backend: terminal,
                    life: Life::Running,
                    definition: definition.clone(),
                    job,
                    numeric_id,
                })
            })()
            .map_err(|error| format!("{error:#}"));
            let _ = tx.send(Started { definition, result });
            ctx.request_repaint();
        });
    }
    pub fn poll(&mut self) -> Vec<Started> {
        self.rx.try_iter().collect()
    }
    pub fn events(&mut self, ctx: &egui::Context) {
        while let Ok((numeric, event)) = self.event_rx.try_recv() {
            if let Some(terminal) = self
                .terminals
                .values_mut()
                .find(|t| t.numeric_id == numeric)
            {
                match event {
                    PtyEvent::ChildExit(status) => terminal.life = Life::Exited(status.code()),
                    PtyEvent::Exit => {
                        if terminal.life == Life::Running {
                            terminal.life = Life::Exited(None)
                        }
                    }
                    PtyEvent::ClipboardStore(_, text) => ctx.copy_text(text),
                    PtyEvent::ColorRequest(index, format) => {
                        let color = terminal.backend.default_color(index);
                        terminal
                            .backend
                            .process_command(egui_term::BackendCommand::Write(
                                format(color).into_bytes(),
                            ))
                    }
                    PtyEvent::TextAreaSizeRequest(format) => {
                        let size = terminal.backend.window_size();
                        terminal
                            .backend
                            .process_command(egui_term::BackendCommand::Write(
                                format(size).into_bytes(),
                            ));
                    }
                    _ => {}
                }
            }
        }
    }
    pub fn running(&self) -> usize {
        self.terminals
            .values()
            .filter(|t| t.job.active().map(|n| n > 0).unwrap_or(true))
            .count()
    }
    pub fn request_shutdown(&mut self) {
        for terminal in self.terminals.values_mut() {
            if terminal.job.active().unwrap_or(1) > 0 {
                terminal.life = Life::Stopping;
                terminal
                    .backend
                    .process_command(egui_term::BackendCommand::Write(vec![3]));
                if matches!(terminal.definition.tool, Tool::PowerShell | Tool::Custom(_)) {
                    terminal
                        .backend
                        .process_command(egui_term::BackendCommand::Write(b"exit\r".to_vec()));
                }
            }
        }
        let backend = self.backend.clone();
        std::thread::spawn(move || {
            if let Ok(mut backend) = backend.lock() {
                if let Some(backend) = backend.as_mut() {
                    backend.request_shutdown();
                }
            }
        });
    }
    pub fn stop_terminal(&mut self, id: &str) {
        if let Some(terminal) = self.terminals.get_mut(id) {
            terminal.life = Life::Stopping;
            terminal
                .backend
                .process_command(egui_term::BackendCommand::Write(vec![3]));
            if matches!(terminal.definition.tool, Tool::PowerShell | Tool::Custom(_)) {
                terminal
                    .backend
                    .process_command(egui_term::BackendCommand::Write(b"exit\r".to_vec()));
            }
        }
    }
    pub fn remaining(&self) -> anyhow::Result<usize> {
        let mut count = 0;
        for t in self.terminals.values() {
            count += t.job.active()? as usize;
        }
        if let Ok(backend) = self.backend.try_lock() {
            if let Some(backend) = backend.as_ref() {
                count += backend.remaining()? as usize;
            }
        } else {
            count += 1;
        }
        Ok(count)
    }
    pub fn force_stop(&self) -> anyhow::Result<()> {
        for t in self.terminals.values() {
            t.job.force_stop()?;
        }
        let backend = self
            .backend
            .try_lock()
            .map_err(|_| anyhow::anyhow!("OpenCode is still starting or handling a request. Wait for it to finish, then try quitting again."))?;
        if let Some(backend) = backend.as_ref() {
            backend.force_stop()?;
        }
        Ok(())
    }
}
