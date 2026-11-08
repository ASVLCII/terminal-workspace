use egui_dock::DockState;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Tool {
    PowerShell,
    OpenCode,
    ClaudeCode,
    Codex,
    Custom(String),
}
impl Tool {
    pub fn label(&self) -> &str {
        match self {
            Self::PowerShell => "PowerShell",
            Self::OpenCode => "OpenCode",
            Self::ClaudeCode => "Claude Code",
            Self::Codex => "Codex CLI",
            Self::Custom(_) => "Custom command",
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Pane {
    Terminal(String),
    Image(PathBuf),
    Settings,
    History,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerminalDefinition {
    pub id: String,
    pub name: String,
    pub tool: Tool,
    pub directory: PathBuf,
    pub chat: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub directory: PathBuf,
    pub terminals: Vec<TerminalDefinition>,
    #[serde(serialize_with = "serialize_dock")]
    pub dock: DockState<Pane>,
}
fn serialize_dock<S: serde::Serializer>(
    dock: &DockState<Pane>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let mut snapshot = dock.clone();
    for node in snapshot.main_surface_mut().iter_mut() {
        node.set_rect(eframe::egui::Rect::ZERO);
        if let egui_dock::Node::Leaf(leaf) = node {
            leaf.viewport = eframe::egui::Rect::ZERO;
        }
    }
    snapshot.serialize(serializer)
}
impl Workspace {
    pub fn new(name: String, directory: PathBuf) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            directory,
            terminals: Vec::new(),
            dock: DockState::new(Vec::new()),
        }
    }
}
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
pub enum Closing {
    #[default]
    Ask,
    Stop,
    Background,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub font_size: f32,
    pub scrollback: usize,
    pub closing: Closing,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            font_size: 14.,
            scrollback: 10_000,
            closing: Closing::Ask,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Saved {
    pub version: u32,
    pub workspaces: Vec<Workspace>,
    pub selected: Option<String>,
    pub settings: Settings,
}
impl Default for Saved {
    fn default() -> Self {
        Self {
            version: 1,
            workspaces: Vec::new(),
            selected: None,
            settings: Default::default(),
        }
    }
}
