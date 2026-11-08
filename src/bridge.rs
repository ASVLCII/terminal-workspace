use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::Read,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};
use tiny_http::{Header, Method, Response, Server, StatusCode};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Attachment {
    pub uri: String,
    pub name: String,
    pub revision: u64,
}
#[derive(Clone, Debug, Default)]
pub struct BoundSession {
    pub terminal_id: String,
    pub revision: u64,
    pub files: Vec<Attachment>,
    pub tui_connected: bool,
}
#[derive(Default)]
pub struct State {
    pub current: HashMap<String, String>,
    pub sessions: HashMap<String, BoundSession>,
    pub server_connected: bool,
    pub tui_loaded: usize,
    pub error: Option<String>,
}
pub struct Bridge {
    pub url: String,
    token: String,
    pub state: Arc<Mutex<State>>,
    stop: Arc<AtomicBool>,
}

impl Bridge {
    pub fn start() -> Result<Self> {
        let server = Server::http("127.0.0.1:0").map_err(|e| anyhow::anyhow!(e.to_string()))?;
        let url = format!("http://{}", server.server_addr());
        let token = uuid::Uuid::new_v4().to_string();
        let state = Arc::new(Mutex::new(State::default()));
        let stop = Arc::new(AtomicBool::new(false));
        let (thread_state, thread_stop, thread_token) =
            (state.clone(), stop.clone(), token.clone());
        thread::Builder::new()
            .name("workspace-bridge".into())
            .spawn(move || {
                while !thread_stop.load(Ordering::Relaxed) {
                    let Ok(Some(mut request)) =
                        server.recv_timeout(std::time::Duration::from_millis(100))
                    else {
                        continue;
                    };
                    let authorized = request.headers().iter().any(|h| {
                        h.field.equiv("Authorization")
                            && h.value.as_str() == format!("Bearer {thread_token}")
                    });
                    if !authorized {
                        let _ = request.respond(Response::empty(StatusCode(401)));
                        continue;
                    }
                    let method = request.method().clone();
                    let path = request.url().to_owned();
                    let mut body = String::new();
                    if method == Method::Post {
                        if request
                            .as_reader()
                            .take(1024 * 1024 + 1)
                            .read_to_string(&mut body)
                            .is_err()
                            || body.len() > 1024 * 1024
                        {
                            let _ = request.respond(Response::empty(StatusCode(413)));
                            continue;
                        }
                    }
                    let result = handle(&thread_state, &method, &path, &body);
                    let (code, value) = match result {
                        Ok(value) => (200, value),
                        Err(error) => (400, serde_json::json!({"error":error.to_string()})),
                    };
                    let response = Response::from_string(value.to_string())
                        .with_status_code(StatusCode(code))
                        .with_header(
                            Header::from_bytes("Content-Type", "application/json").unwrap(),
                        );
                    let _ = request.respond(response);
                }
            })?;
        Ok(Self {
            url,
            token,
            state,
            stop,
        })
    }
    pub fn token(&self) -> &str {
        &self.token
    }
    pub fn bind(&self, terminal: &str, session: &str) {
        let mut state = self.state.lock().unwrap();
        state.current.insert(terminal.into(), session.into());
        state.sessions.insert(
            session.into(),
            BoundSession {
                terminal_id: terminal.into(),
                ..Default::default()
            },
        );
    }
    pub fn stage(&self, session: &str, path: &std::path::Path) -> Result<()> {
        let absolute = path
            .canonicalize()
            .context("Attachment path is unavailable")?;
        let uri = url::Url::from_file_path(&absolute)
            .map_err(|_| anyhow::anyhow!("Invalid local attachment path"))?
            .to_string();
        let mut state = self.state.lock().unwrap();
        let binding = state
            .sessions
            .get_mut(session)
            .context("Terminal is not bound to an OpenCode chat")?;
        if !binding.files.iter().any(|f| f.uri == uri) {
            binding.revision += 1;
            binding.files.push(Attachment {
                uri,
                name: path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into(),
                revision: binding.revision,
            });
        }
        Ok(())
    }
    pub fn files(&self, session: &str) -> Vec<Attachment> {
        self.state
            .lock()
            .unwrap()
            .sessions
            .get(session)
            .map(|s| s.files.clone())
            .unwrap_or_default()
    }
    pub fn remove(&self, session: &str, uri: &str) {
        if let Some(s) = self.state.lock().unwrap().sessions.get_mut(session) {
            s.files.retain(|f| f.uri != uri);
        }
    }
}
impl Drop for Bridge {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

fn handle(
    state: &Arc<Mutex<State>>,
    method: &Method,
    path: &str,
    body: &str,
) -> Result<serde_json::Value> {
    let mut state = state.lock().unwrap();
    if method == &Method::Get && path.starts_with("/stage/") {
        let id = &path[7..];
        return Ok(match state.sessions.get(id) {
            Some(s) => serde_json::json!({"sessionID":id,"revision":s.revision,"files":s.files}),
            None => serde_json::json!({"sessionID":id,"revision":0,"files":[]}),
        });
    }
    let data: serde_json::Value = serde_json::from_str(body)?;
    match path {
        "/plugin" => {
            if data["version"] != 1 {
                bail!("Bridge version mismatch")
            }
            if data["kind"] == "tui" {
                state.tui_loaded += 1;
            } else {
                state.server_connected = true;
            }
            Ok(serde_json::json!({"version":1}))
        }
        "/plugin-error" => {
            state.error = data["message"].as_str().map(String::from);
            Ok(serde_json::json!({"ok":true}))
        }
        "/tui" => {
            let id = data["sessionID"].as_str().context("Missing chat")?;
            let terminal = data["terminalID"].as_str().context("Missing terminal")?;
            if !state.current.contains_key(terminal) {
                bail!("Unregistered terminal")
            }
            if state
                .sessions
                .get(id)
                .is_some_and(|s| s.terminal_id != terminal)
            {
                bail!("Chat belongs to another terminal")
            }
            if let Some(previous) = state.current.insert(terminal.into(), id.into()) {
                if previous != id {
                    if let Some(binding) = state.sessions.get_mut(&previous) {
                        binding.tui_connected = false;
                    }
                }
            }
            let s = state
                .sessions
                .entry(id.into())
                .or_insert_with(|| BoundSession {
                    terminal_id: terminal.into(),
                    ..Default::default()
                });
            s.tui_connected = true;
            Ok(serde_json::json!({"sessionID":id,"revision":s.revision,"files":s.files}))
        }
        "/ack" => {
            let id = data["sessionID"].as_str().context("Missing chat")?;
            let revision = data["revision"].as_u64().context("Missing revision")?;
            if let Some(s) = state.sessions.get_mut(id) {
                s.files.retain(|f| f.revision > revision);
            }
            Ok(serde_json::json!({"ok":true}))
        }
        _ => bail!("Unknown bridge request"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn switching_chat_in_a_registered_terminal_reassociates_without_broadcasting() {
        let bridge = Bridge::start().unwrap();
        bridge.bind("one", "old");
        bridge.bind("two", "other");
        handle(
            &bridge.state,
            &Method::Post,
            "/tui",
            r#"{"terminalID":"one","sessionID":"new"}"#,
        )
        .unwrap();
        assert_eq!(
            bridge.state.lock().unwrap().sessions["new"].terminal_id,
            "one"
        );
        assert!(
            handle(
                &bridge.state,
                &Method::Post,
                "/tui",
                r#"{"terminalID":"one","sessionID":"other"}"#
            )
            .is_err()
        );
    }
    #[test]
    fn bridge_refuses_unauthenticated_requests() {
        let bridge = Bridge::start().unwrap();
        let response = reqwest::blocking::get(format!("{}/stage/a", bridge.url)).unwrap();
        assert_eq!(response.status(), 401);
    }
    #[test]
    fn acknowledgement_keeps_files_staged_after_submission() {
        let state = Arc::new(Mutex::new(State::default()));
        state.lock().unwrap().sessions.insert(
            "a".into(),
            BoundSession {
                terminal_id: "one".into(),
                revision: 2,
                files: vec![
                    Attachment {
                        uri: "file:///first".into(),
                        name: "first".into(),
                        revision: 1,
                    },
                    Attachment {
                        uri: "file:///second".into(),
                        name: "second".into(),
                        revision: 2,
                    },
                ],
                tui_connected: true,
            },
        );
        handle(
            &state,
            &Method::Post,
            "/ack",
            r#"{"sessionID":"a","revision":1}"#,
        )
        .unwrap();
        assert_eq!(state.lock().unwrap().sessions["a"].files[0].name, "second");
    }
    #[test]
    fn one_terminal_cannot_claim_another_chat() {
        let state = Arc::new(Mutex::new(State::default()));
        state.lock().unwrap().sessions.insert(
            "a".into(),
            BoundSession {
                terminal_id: "one".into(),
                ..Default::default()
            },
        );
        assert!(
            handle(
                &state,
                &Method::Post,
                "/tui",
                r#"{"terminalID":"two","sessionID":"a"}"#
            )
            .is_err()
        );
    }
}
