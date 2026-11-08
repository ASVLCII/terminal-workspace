use serde_json::json;
use terminal_workspace::{bridge::Bridge, opencode::Backend};

#[test]
#[ignore = "starts an isolated installed OpenCode backend; no model execution"]
fn real_plugin_admission_targets_only_the_bound_same_folder_chat() -> anyhow::Result<()> {
    let directory = std::env::current_dir()?;
    let bridge = Bridge::start()?;
    let backend = Backend::start(&bridge, &directory, &directory.join("plugin"))?;
    struct Sessions<'a>(&'a Backend, Vec<String>);
    impl Drop for Sessions<'_> {
        fn drop(&mut self) {
            for id in &self.1 {
                let _ = self
                    .0
                    .api(reqwest::Method::DELETE, &format!("/api/session/{id}"), None);
            }
        }
    }
    let mut sessions = Sessions(&backend, Vec::new());
    let a = backend.create(&directory)?;
    sessions.1.push(a.clone());
    let b = backend.create(&directory)?;
    sessions.1.push(b.clone());
    bridge.bind("terminal-a", &a);
    bridge.bind("terminal-b", &b);
    bridge.stage(&a, &directory.join("Cargo.toml"))?;
    bridge.stage(&a, &directory.join("src"))?;
    bridge.stage(
        &a,
        &directory.join("docs/design/01-workspace-and-layouts-v2.png"),
    )?;
    let before = backend.api(
        reqwest::Method::GET,
        &format!("/api/session/{a}/inbox"),
        None,
    )?;
    assert_eq!(
        before["data"].as_array().unwrap().len(),
        0,
        "Staging must not submit a prompt"
    );
    let admitted = backend.api(
        reqwest::Method::POST,
        &format!("/api/session/{a}/prompt"),
        Some(json!({"text":"Attachment admission test; no generation", "resume":false})),
    )?;
    assert_eq!(
        admitted["data"]["payload"]["files"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    let other = backend.api(
        reqwest::Method::GET,
        &format!("/api/session/{b}/inbox"),
        None,
    )?;
    assert_eq!(
        other["data"].as_array().unwrap().len(),
        0,
        "Other same-folder chat must stay untouched"
    );
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    while !bridge.files(&a).is_empty() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert!(
        bridge.files(&a).is_empty(),
        "Only admitted attachments are acknowledged"
    );
    Ok(())
}
