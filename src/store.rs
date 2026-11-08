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
    let state: Saved = serde_json::from_slice(&std::fs::read(path)?)?;
    anyhow::ensure!(state.version == 1, "Unsupported saved-state version");
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
    tx: std::sync::mpsc::Sender<Saved>,
    rx: std::sync::mpsc::Receiver<Result<(), String>>,
}
impl Writer {
    pub fn start(directory: PathBuf, ctx: eframe::egui::Context) -> Self {
        let (tx, requests) = std::sync::mpsc::channel::<Saved>();
        let (results, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let store = Store { directory };
            while let Ok(mut state) = requests.recv() {
                while let Ok(newer) = requests.try_recv() {
                    state = newer;
                }
                let result = store.save(&state).map_err(|e| format!("{e:#}"));
                let _ = results.send(result);
                ctx.request_repaint();
            }
        });
        Self { tx, rx }
    }
    pub fn save(&self, state: Saved) {
        let _ = self.tx.send(state);
    }
    pub fn poll(&self) -> Option<Result<(), String>> {
        self.rx.try_recv().ok()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Workspace;
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
