use crate::{
    bridge::Bridge,
    process::{self, Job},
};
use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Stdio},
    time::Duration,
};

pub struct Backend {
    pub url: String,
    password: String,
    client: reqwest::blocking::Client,
    child: Child,
    lease: Option<ChildStdin>,
    _job: Job,
    pub plugin: PathBuf,
}
impl Backend {
    pub fn request_shutdown(&mut self) {
        self.lease.take();
    }
    pub fn remaining(&self) -> Result<u32> {
        self._job.active()
    }
    pub fn force_stop(&self) -> Result<()> {
        self._job.force_stop()
    }
    pub fn start(bridge: &Bridge, directory: &Path, plugin: &Path) -> Result<Self> {
        let password = uuid::Uuid::new_v4().to_string();
        let mut launch = process::tool(
            "opencode",
            directory,
            vec![
                "serve".into(),
                "--stdio".into(),
                "--hostname".into(),
                "127.0.0.1".into(),
                "--port".into(),
                "0".into(),
            ],
        )?;
        launch.env.extend([
            ("OPENCODE_PASSWORD".into(), password.clone()),
            ("OPENCODE_SERVER_PASSWORD".into(), password.clone()),
            (
                "OPENCODE_CONFIG_CONTENT".into(),
                json!({"plugins":[plugin]}).to_string(),
            ),
            ("TERMINAL_WORKSPACE_BRIDGE".into(), bridge.url.clone()),
            ("TERMINAL_WORKSPACE_TOKEN".into(), bridge.token().into()),
        ]);
        let job = Job::new()?;
        let mut child = process::child(&launch)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .context("Start OpenCode backend")?;
        job.assign(child.id())?;
        let stdout = child.stdout.take().context("Backend stdout missing")?;
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout)
                .lines()
                .map_while(std::result::Result::ok)
            {
                if let Ok(value) = serde_json::from_str::<Value>(&line) {
                    if let Some(url) = value["url"].as_str() {
                        let _ = tx.send(url.to_owned());
                        break;
                    }
                }
            }
        });
        let url = rx
            .recv_timeout(Duration::from_secs(25))
            .context("OpenCode backend startup timed out")?;
        let lease = child.stdin.take();
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(15))
            .redirect(reqwest::redirect::Policy::none())
            .build()?;
        Ok(Self {
            url,
            password,
            client,
            child,
            lease,
            _job: job,
            plugin: plugin.into(),
        })
    }
    pub fn api(&self, method: reqwest::Method, path: &str, body: Option<Value>) -> Result<Value> {
        let mut request = self
            .client
            .request(method, format!("{}{}", self.url, path))
            .basic_auth("opencode", Some(&self.password));
        if let Some(body) = body {
            request = request.json(&body)
        }
        let response = request.send()?.error_for_status()?;
        if response.status() == 204 {
            return Ok(Value::Null);
        }
        use std::io::Read;
        let mut bytes=Vec::new();response.take(8*1024*1024+1).read_to_end(&mut bytes)?;
        anyhow::ensure!(
            bytes.len() <= 8 * 1024 * 1024,
            "OpenCode response too large"
        );
        if bytes.is_empty() {
            Ok(Value::Null)
        } else {
            Ok(serde_json::from_slice(&bytes)?)
        }
    }
    pub fn create(&self, directory: &Path) -> Result<String> {
        let value = self.api(
            reqwest::Method::POST,
            "/api/session",
            Some(json!({"location":{"directory":directory},"title":"New chat"})),
        )?;
        Ok(value["data"]["id"]
            .as_str()
            .context("OpenCode returned no chat ID")?
            .into())
    }
    pub fn launch(
        &self,
        bridge: &Bridge,
        terminal: &str,
        session: &str,
        directory: &Path,
    ) -> Result<process::Launch> {
        let mut launch = process::tool(
            "opencode",
            directory,
            vec![
                "--server".into(),
                self.url.clone(),
                "--session".into(),
                session.into(),
                directory.to_string_lossy().into(),
            ],
        )?;
        launch.env.extend([
            ("OPENCODE_PASSWORD".into(), self.password.clone()),
            ("OPENCODE_SERVER_PASSWORD".into(), self.password.clone()),
            (
                "OPENCODE_CLI_CONFIG_CONTENT".into(),
                json!({"plugins":[self.plugin]}).to_string(),
            ),
            ("TERMINAL_WORKSPACE_BRIDGE".into(), bridge.url.clone()),
            ("TERMINAL_WORKSPACE_TOKEN".into(), bridge.token().into()),
            ("TERMINAL_WORKSPACE_TERMINAL".into(), terminal.into()),
        ]);
        Ok(launch)
    }
}
impl Drop for Backend {
    fn drop(&mut self) {
        self.lease.take();
        let _ = self.child.try_wait();
    }
}
