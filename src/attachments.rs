use std::collections::HashMap;

#[derive(Default)]
pub struct Staging {
    sessions: HashMap<String, Vec<String>>,
}

impl Staging {
    pub fn stage(&mut self, session: &str, uri: String) {
        let files = self.sessions.entry(session.to_owned()).or_default();
        if !files.contains(&uri) {
            files.push(uri);
        }
    }
    pub fn files(&self, session: &str) -> Vec<String> {
        self.sessions.get(session).cloned().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn same_folder_terminals_have_isolated_unsent_attachments() {
        let mut staged = Staging::default();
        staged.stage("session-a", "file:///C:/Project/layout.png".into());
        assert_eq!(staged.files("session-a"), ["file:///C:/Project/layout.png"]);
        assert!(staged.files("session-b").is_empty());
    }
}
