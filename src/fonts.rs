use eframe::egui;
/// Reuse installed Windows fonts. No proprietary font files are redistributed.
pub fn install(ctx: &egui::Context) {
    let ctx = ctx.clone();
    std::thread::spawn(move || {
        let windows = std::env::var_os("WINDIR")
            .map(std::path::PathBuf::from)
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
                "Windows mono",
                "consola.ttf",
                egui::FontFamily::Monospace,
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
                if bytes.len() > 32 * 1024 * 1024 {
                    continue;
                }
                fonts
                    .font_data
                    .insert(name.into(), egui::FontData::from_owned(bytes).into());
                let fallbacks = fonts.families.entry(family).or_default();
                if primary {
                    fallbacks.insert(0, name.into());
                } else {
                    fallbacks.push(name.into());
                    fonts
                        .families
                        .entry(egui::FontFamily::Proportional)
                        .or_default()
                        .push(name.into());
                }
            }
        }
        ctx.set_fonts(fonts);
        ctx.request_repaint();
    });
}
