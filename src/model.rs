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
    pub fn remove_terminal(
        &mut self,
        id: &str,
        starting: bool,
        owned_active: bool,
    ) -> TerminalRemoval {
        if starting {
            return TerminalRemoval::Starting;
        }
        if owned_active {
            return TerminalRemoval::RequiresStop;
        }
        if let Some(tab) = self.dock.find_tab(&Pane::Terminal(id.into())) {
            self.dock.remove_tab(tab);
        }
        self.terminals.retain(|terminal| terminal.id != id);
        TerminalRemoval::Removed
    }
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
#[derive(Debug, PartialEq)]
pub enum TerminalRemoval {
    Starting,
    RequiresStop,
    Removed,
}
#[cfg(test)]
mod removal_tests {
    use super::*;
    #[test]
    fn pending_and_live_terminals_remain_visible_until_they_can_be_stopped() {
        let mut workspace = Workspace::new("Work".into(), "Z:/project".into());
        for id in ["a", "b"] {
            workspace.terminals.push(TerminalDefinition {
                id: id.into(),
                name: id.into(),
                tool: Tool::PowerShell,
                directory: workspace.directory.clone(),
                chat: None,
            });
            workspace.dock.push_to_first_leaf(Pane::Terminal(id.into()));
        }
        assert_eq!(
            workspace.remove_terminal("a", true, false),
            TerminalRemoval::Starting
        );
        assert_eq!(workspace.terminals.len(), 2);
        assert_eq!(workspace.dock.iter_all_tabs().count(), 2);
        assert_eq!(
            workspace.remove_terminal("a", false, true),
            TerminalRemoval::RequiresStop
        );
        assert_eq!(workspace.terminals.len(), 2);
        assert_eq!(
            workspace.remove_terminal("a", false, false),
            TerminalRemoval::Removed
        );
        assert_eq!(workspace.terminals[0].id, "b");
        assert_eq!(workspace.dock.iter_all_tabs().count(), 1);
    }
}
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
pub enum Closing {
    #[default]
    Ask,
    Stop,
    Background,
}
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum Accent {
    #[default]
    Cyan,
    Blue,
    White,
    Black,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub accent: Accent,
    #[serde(default)]
    pub expanded: bool,
    #[serde(default = "default_font_family")]
    pub font_family: String,
    pub font_size: f32,
    pub scrollback: usize,
    pub closing: Closing,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            accent: Accent::default(),
            expanded: false,
            font_size: 14.,
            font_family: default_font_family(),
            scrollback: 10_000,
            closing: Closing::Ask,
        }
    }
}
fn default_font_family() -> String {
    "CaskaydiaMono NFM".into()
}
#[cfg(test)]
mod font_settings_tests {
    use super::*;
    #[test]
    fn older_settings_pick_the_confirmed_nerd_font() {
        let settings: Settings =
            serde_json::from_str(r#"{"font_size":14.0,"scrollback":10000,"closing":"Ask"}"#)
                .unwrap();
        assert_eq!(settings.font_family, "CaskaydiaMono NFM");
    }
    #[test]
    fn a_changed_font_survives_settings_roundtrip() {
        let mut settings = Settings::default();
        settings.font_family = "Consolas".into();
        let restored: Settings =
            serde_json::from_slice(&serde_json::to_vec(&settings).unwrap()).unwrap();
        assert_eq!(restored.font_family, "Consolas");
    }

    #[test]
    fn accent_defaults_for_older_state_and_roundtrips() {
        let mut settings: Settings =
            serde_json::from_str(r#"{"font_size":14.0,"scrollback":10000,"closing":"Ask"}"#)
                .unwrap();
        assert_eq!(settings.accent, Accent::Cyan);
        for accent in [Accent::Cyan, Accent::Blue, Accent::White, Accent::Black] {
            settings.accent = accent;
            let restored: Settings =
                serde_json::from_slice(&serde_json::to_vec(&settings).unwrap()).unwrap();
            assert_eq!(restored.accent, accent);
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
