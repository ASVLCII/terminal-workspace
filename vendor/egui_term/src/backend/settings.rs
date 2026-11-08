use std::path::PathBuf;

const DEFAULT_SHELL: &str = "/bin/bash";

#[derive(Debug, Clone)]
pub struct BackendSettings {
    pub shell: String,
    pub args: Vec<String>,
    pub working_directory: Option<PathBuf>,
    pub env: std::collections::HashMap<String, String>,
    pub scrollback: usize,
    #[cfg(windows)]
    pub job_object: Option<usize>,
}

impl Default for BackendSettings {
    fn default() -> Self {
        Self {
            shell: DEFAULT_SHELL.to_string(),
            args: vec![],
            working_directory: None,
            env: Default::default(),
            scrollback: 10_000,
            #[cfg(windows)]
            job_object: None,
        }
    }
}
