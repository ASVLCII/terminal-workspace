use eframe::egui;
use std::{path::PathBuf, sync::mpsc};
#[derive(Clone, Debug)]
pub struct Choice {
    pub family: String,
    pub aliases: Vec<String>,
    pub path: PathBuf,
    pub index: u32,
}
pub enum Event {
    Catalog(Vec<Choice>),
    Applied {
        requested: String,
        actual: String,
        error: Option<String>,
    },
}
pub struct Loader {
    tx: mpsc::Sender<String>,
    rx: mpsc::Receiver<Event>,
}
impl Loader {
    pub fn start(ctx: egui::Context, initial: String) -> Self {
        let (tx, requests) = mpsc::channel::<String>();
        let (results, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let choices = catalog();
            if results.send(Event::Catalog(choices.clone())).is_err() {
                return;
            }
            let mut requested = initial;
            loop {
                while let Ok(newer) = requests.try_recv() {
                    requested = newer;
                }
                let selected = choices.iter().find(|choice| {
                    choice.family.eq_ignore_ascii_case(&requested)
                        || choice
                            .aliases
                            .iter()
                            .any(|alias| alias.eq_ignore_ascii_case(&requested))
                });
                let fallback = choices
                    .iter()
                    .find(|choice| choice.family.eq_ignore_ascii_case("Consolas"));
                let choice = selected.or(fallback);
                let actual = choice
                    .map(|choice| choice.family.clone())
                    .unwrap_or("Default monospace".into());
                let mut error = if selected.is_none() {
                    Some(format!("{requested} is unavailable. Using {actual}."))
                } else {
                    None
                };
                match definitions(choice) {
                    Ok(fonts) => ctx.set_fonts(fonts),
                    Err(problem) => error = Some(problem),
                }
                if results
                    .send(Event::Applied {
                        requested: requested.clone(),
                        actual,
                        error,
                    })
                    .is_err()
                {
                    break;
                }
                ctx.request_repaint();
                match requests.recv() {
                    Ok(next) => requested = next,
                    Err(_) => break,
                }
            }
        });
        Self { tx, rx }
    }
    pub fn apply(&self, family: String) {
        let _ = self.tx.send(family);
    }
    pub fn poll(&self) -> Option<Event> {
        self.rx.try_recv().ok()
    }
}
pub fn catalog() -> Vec<Choice> {
    let mut database = fontdb::Database::new();
    database.load_system_fonts();
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        database.load_fonts_dir(PathBuf::from(local).join("Microsoft/Windows/Fonts"));
    }
    let mut families: std::collections::BTreeMap<String, (i32, Choice)> = Default::default();
    for face in database.faces() {
        if !face.monospaced {
            continue;
        }
        let path = match &face.source {
            fontdb::Source::File(path) | fontdb::Source::SharedFile(path, _) => path.clone(),
            _ => continue,
        };
        let Some((family, _)) = face
            .families
            .iter()
            .find(|(name, _)| name == "CaskaydiaMono NFM")
            .or_else(|| face.families.first())
        else {
            continue;
        };
        let score = (face.weight.0 as i32 - 400).abs()
            + if face.style == fontdb::Style::Normal {
                0
            } else {
                1000
            };
        let key = family.to_lowercase();
        if families.get(&key).is_none_or(|(old, _)| score < *old) {
            families.insert(
                key,
                (
                    score,
                    Choice {
                        family: family.clone(),
                        aliases: face.families.iter().map(|(name, _)| name.clone()).collect(),
                        path,
                        index: face.index,
                    },
                ),
            );
        }
    }
    families.into_values().map(|(_, choice)| choice).collect()
}
fn definitions(choice: Option<&Choice>) -> Result<egui::FontDefinitions, String> {
    let windows = std::env::var_os("WINDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| "C:/Windows".into());
    let mut fonts = egui::FontDefinitions::default();
    for (name, file, family, primary) in [
        (
            "Windows UI",
            "segoeui.ttf",
            egui::FontFamily::Proportional,
            true,
        ),
        (
            "Windows CJK",
            "msyh.ttc",
            egui::FontFamily::Monospace,
            false,
        ),
    ] {
        if let Ok(bytes) = std::fs::read(windows.join("Fonts").join(file)) {
            fonts
                .font_data
                .insert(name.into(), egui::FontData::from_owned(bytes).into());
            let list = fonts.families.entry(family).or_default();
            if primary {
                list.insert(0, name.into());
            } else {
                list.push(name.into());
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .push(name.into());
            }
        }
    }
    if let Some(choice) = choice {
        let metadata = std::fs::metadata(&choice.path).map_err(|error| error.to_string())?;
        if metadata.len() > 64 * 1024 * 1024 {
            return Err("The selected font exceeds the 64 MiB font limit.".into());
        }
        let bytes = std::fs::read(&choice.path).map_err(|error| error.to_string())?;
        let mut data = egui::FontData::from_owned(bytes);
        data.index = choice.index;
        fonts.font_data.insert("Terminal font".into(), data.into());
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "Terminal font".into());
    }
    Ok(fonts)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "checks the installed font on this Windows machine"]
    fn confirmed_nerd_font_is_available_as_a_monospace_face() {
        let choices = catalog();
        let choice = choices
            .iter()
            .find(|choice| choice.family == "CaskaydiaMono NFM")
            .expect("Confirmed Xelif font must be found");
        let fonts = definitions(Some(choice)).unwrap();
        assert_eq!(
            fonts.families[&egui::FontFamily::Monospace][0],
            "Terminal font"
        );
    }
}
