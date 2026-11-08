use serde::Deserialize;
use std::path::{Path, PathBuf};
#[derive(Clone, Debug, Deserialize)]
pub struct Chat {
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    pub location: Location,
    #[serde(default)]
    pub time: serde_json::Value,
}
#[derive(Clone, Debug, Deserialize)]
pub struct Location {
    pub directory: PathBuf,
}
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Cursor {
    pub next: Option<String>,
    pub previous: Option<String>,
}
#[derive(Clone, Debug, Deserialize)]
pub struct Page {
    pub data: Vec<Chat>,
    pub cursor: Cursor,
}
pub fn query(directory: &Path, search: &str, cursor: Option<&str>) -> String {
    let mut query = url::form_urlencoded::Serializer::new(String::new());
    query
        .append_pair("directory", &directory.to_string_lossy())
        .append_pair("limit", "50")
        .append_pair("order", "desc");
    if !search.trim().is_empty() {
        query.append_pair("search", search.trim());
    }
    if let Some(cursor) = cursor {
        query.append_pair("cursor", cursor);
    }
    format!("/api/session?{}", query.finish())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn query_keeps_unicode_paths_titles_and_opaque_cursor_distinct() {
        let path = Path::new("Z:/Dự án/space folder");
        let query = query(path, "a & b", Some("opaque+cursor/="));
        let url = url::Url::parse(&format!("http://localhost{query}")).unwrap();
        let pairs: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(pairs["directory"], path.to_string_lossy());
        assert_eq!(pairs["search"], "a & b");
        assert_eq!(pairs["cursor"], "opaque+cursor/=");
        assert_eq!(pairs["limit"], "50");
    }
}
