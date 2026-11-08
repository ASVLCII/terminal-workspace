use crate::model::Saved;
use anyhow::{Context, Result};
use std::path::PathBuf;

pub struct Store {
    pub directory: PathBuf,
}
impl Store {
    pub fn user() -> Result<Self> {
        let directory = std::env::var_os("TERMINAL_WORKSPACE_DATA_DIR")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("LOCALAPPDATA").map(|p| PathBuf::from(p).join("TerminalWorkspace"))
            })
            .context("Local application-data directory is unavailable")?;
        anyhow::ensure!(
            directory.is_absolute(),
            "Application-data path must be absolute"
        );
        Ok(Self { directory })
    }
    pub fn load(&self) -> Result<(Saved, bool)> {
        let primary = self.directory.join("state.json");
        let backup = self.directory.join("state.backup.json");
        if !primary.exists() && !backup.exists() {
            return Ok((Saved::default(), false));
        }
        let result = read(&primary);
        if let Ok(ref bytes) = std::fs::read(&primary) {
            if let Ok(value) = serde_json::from_slice::<serde_json::Value>(bytes) {
                if value["version"].as_u64().is_some_and(|v| v != 1) {
                    anyhow::bail!(
                        "State uses an unsupported version; existing files were preserved"
                    )
                }
            }
        }
        match result {
            Ok(state) => Ok((state, false)),
            Err(primary_error) => read(&backup)
                .map(|state| (state, true))
                .with_context(|| format!("Could not read saved workspaces: {primary_error}")),
        }
    }
    pub fn save(&self, state: &Saved) -> Result<()> {
        anyhow::ensure!(
            state.version == 1,
            "Cannot save an unsupported state version"
        );
        std::fs::create_dir_all(&self.directory).context("Create application-data directory")?;
        let primary = self.directory.join("state.json");
        if read(&primary).is_ok() {
            let backup_tmp = self
                .directory
                .join(format!("backup-{}.tmp", uuid::Uuid::new_v4()));
            std::fs::copy(&primary, &backup_tmp)?;
            replace(&backup_tmp, &self.directory.join("state.backup.json"))?;
        }
        let temporary = self
            .directory
            .join(format!("state-{}.tmp", uuid::Uuid::new_v4()));
        let write = (|| -> Result<()> {
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)?;
            file.write_all(&serde_json::to_vec_pretty(state)?)?;
            file.sync_all()?;
            drop(file);
            replace(&temporary, &primary)?;
            Ok(())
        })();
        if write.is_err() {
            let _ = std::fs::remove_file(&temporary);
        }
        write.context("Save workspace state")
    }
}
fn read(path: &std::path::Path) -> Result<Saved> {
    anyhow::ensure!(
        std::fs::metadata(path)?.len() <= 8 * 1024 * 1024,
        "Saved state is too large"
    );
    let raw: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    if let Some(workspaces) = raw["workspaces"].as_array() {
        for workspace in workspaces {
            anyhow::ensure!(
                workspace["dock"]["focused_surface"]
                    .as_u64()
                    .is_none_or(|surface| surface == 0),
                "Invalid focused surface"
            );
        }
    }
    let state: Saved = serde_json::from_value(raw)?;
    anyhow::ensure!(state.version == 1, "Unsupported saved-state version");
    anyhow::ensure!(
        state.settings.font_size.is_finite()
            && (10.0..=28.0).contains(&state.settings.font_size)
            && (1000..=50000).contains(&state.settings.scrollback),
        "Invalid terminal settings"
    );
    let mut workspace_ids = std::collections::HashSet::new();
    let mut terminal_ids = std::collections::HashSet::new();
    for workspace in &state.workspaces {
        anyhow::ensure!(
            workspace_ids.insert(&workspace.id) && workspace.directory.is_absolute(),
            "Invalid or duplicate workspace"
        );
        let mut local_ids = std::collections::HashSet::new();
        for terminal in &workspace.terminals {
            anyhow::ensure!(
                terminal_ids.insert(&terminal.id) && terminal.directory.is_absolute(),
                "Invalid or duplicate terminal identity"
            );
            local_ids.insert(&terminal.id);
        }
        anyhow::ensure!(
            workspace.dock.surfaces_count() == 1,
            "Detached layout surfaces are unsupported"
        );
        let tree = workspace.dock.main_surface();
        let nodes: Vec<_> = tree.iter().collect();
        if let Some(focused) = tree.focused_leaf() {
            anyhow::ensure!(
                focused.0 < nodes.len() && nodes[focused.0].is_leaf(),
                "Invalid focused pane"
            );
        }
        let mut panes = std::collections::HashSet::new();
        for (index, node) in nodes.iter().enumerate() {
            match node {
                egui_dock::Node::Horizontal(split) | egui_dock::Node::Vertical(split) => {
                    anyhow::ensure!(
                        split.fraction.is_finite() && split.fraction > 0. && split.fraction < 1.,
                        "Invalid pane proportion"
                    );
                    let index = egui_dock::NodeIndex(index);
                    anyhow::ensure!(
                        [index.left().0, index.right().0]
                            .into_iter()
                            .all(|child| child < nodes.len() && !nodes[child].is_empty()),
                        "Invalid pane tree"
                    );
                }
                egui_dock::Node::Leaf(leaf) => {
                    anyhow::ensure!(
                        leaf.tabs.is_empty() || leaf.active.0 < leaf.tabs.len(),
                        "Invalid active tab"
                    );
                    for pane in &leaf.tabs {
                        anyhow::ensure!(panes.insert(pane), "Duplicate pane");
                        if let crate::model::Pane::Terminal(id) = pane {
                            anyhow::ensure!(
                                local_ids.contains(id),
                                "Pane references an unknown terminal"
                            );
                        }
                    }
                }
                _ => {}
            }
        }
    }
    Ok(state)
}
fn replace(source: &std::path::Path, destination: &std::path::Path) -> Result<()> {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Storage::FileSystem::*;
        let source: Vec<u16> = source.as_os_str().encode_wide().chain(Some(0)).collect();
        let destination: Vec<u16> = destination
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect();
        anyhow::ensure!(
            unsafe {
                MoveFileExW(
                    source.as_ptr(),
                    destination.as_ptr(),
                    MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
                )
            } != 0,
            "Atomic state replacement: {}",
            std::io::Error::last_os_error()
        );
    }
    #[cfg(not(windows))]
    std::fs::rename(source, destination)?;
    Ok(())
}

pub struct Writer {
    tx: std::sync::mpsc::Sender<(u64, Saved)>,
    rx: std::sync::mpsc::Receiver<(u64, Result<(), String>)>,
    generation: u64,
}
impl Writer {
    pub fn start(directory: PathBuf, ctx: eframe::egui::Context) -> Self {
        let (tx, requests) = std::sync::mpsc::channel::<(u64, Saved)>();
        let (results, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let store = Store { directory };
            while let Ok(mut state) = requests.recv() {
                while let Ok(newer) = requests.try_recv() {
                    state = newer;
                }
                let result = store.save(&state.1).map_err(|e| format!("{e:#}"));
                let _ = results.send((state.0, result));
                ctx.request_repaint();
            }
        });
        Self {
            tx,
            rx,
            generation: 0,
        }
    }
    pub fn save(&mut self, state: Saved) -> u64 {
        self.generation += 1;
        let _ = self.tx.send((self.generation, state));
        self.generation
    }
    pub fn poll(&self) -> Option<(u64, Result<(), String>)> {
        self.rx.try_recv().ok()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Workspace;
    #[test]
    fn invalid_split_is_rejected_instead_of_restoring_a_broken_layout() {
        let store = store();
        let mut state = Saved::default();
        let mut workspace = Workspace::new("Broken".into(), "Z:/project".into());
        workspace.dock = crate::layout::preset(
            vec![crate::model::Pane::Settings, crate::model::Pane::History],
            crate::layout::Preset::Columns,
        );
        if let egui_dock::Node::Horizontal(split) =
            &mut workspace.dock.main_surface_mut()[egui_dock::NodeIndex::root()]
        {
            split.fraction = 2.;
        }
        state.workspaces.push(workspace);
        std::fs::create_dir_all(&store.directory).unwrap();
        std::fs::write(
            store.directory.join("state.json"),
            serde_json::to_vec(&state).unwrap(),
        )
        .unwrap();
        assert!(store.load().is_err());
    }
    #[test]
    fn duplicate_terminal_identity_is_rejected_before_any_process_can_start() {
        let store = store();
        let mut state = Saved::default();
        let mut workspace = Workspace::new("Broken".into(), "Z:/project".into());
        let definition = crate::model::TerminalDefinition {
            id: "duplicate".into(),
            name: "Shell".into(),
            tool: crate::model::Tool::PowerShell,
            directory: "Z:/project".into(),
            chat: None,
        };
        workspace.terminals = vec![definition.clone(), definition];
        state.workspaces.push(workspace);
        std::fs::create_dir_all(&store.directory).unwrap();
        std::fs::write(
            store.directory.join("state.json"),
            serde_json::to_vec(&state).unwrap(),
        )
        .unwrap();
        assert!(store.load().is_err());
    }
    fn store() -> Store {
        Store {
            directory: std::env::current_dir()
                .unwrap()
                .join(".work/state-tests")
                .join(uuid::Uuid::new_v4().to_string()),
        }
    }
    #[test]
    fn saved_workspace_survives_reopen_without_executing_commands() {
        let store = store();
        let mut state = Saved::default();
        state.workspaces.push(Workspace::new(
            "Daily work".into(),
            PathBuf::from("X:/Xelif"),
        ));
        store.save(&state).unwrap();
        let (loaded, _) = store.load().unwrap();
        assert_eq!(loaded.workspaces.len(), 1);
        assert_eq!(loaded.workspaces[0].name, "Daily work");
    }
    #[test]
    fn corrupt_primary_recovers_the_last_good_backup() {
        let store = store();
        let mut state = Saved::default();
        state
            .workspaces
            .push(Workspace::new("First".into(), PathBuf::from("X:/Xelif")));
        store.save(&state).unwrap();
        state.workspaces[0].name = "Second".into();
        store.save(&state).unwrap();
        std::fs::write(store.directory.join("state.json"), "corrupt").unwrap();
        let (loaded, recovered) = store.load().unwrap();
        assert!(recovered);
        assert_eq!(loaded.workspaces[0].name, "First");
    }
    #[test]
    fn future_version_is_reported_instead_of_overwritten() {
        let store = store();
        std::fs::create_dir_all(&store.directory).unwrap();
        let mut state = Saved::default();
        state.version = 999;
        std::fs::write(
            store.directory.join("state.json"),
            serde_json::to_vec(&state).unwrap(),
        )
        .unwrap();
        assert!(store.load().is_err());
    }
}
