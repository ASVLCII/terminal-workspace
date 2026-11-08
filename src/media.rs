use eframe::egui;
use std::{
    path::{Path, PathBuf},
    sync::mpsc,
};
pub struct Decoded {
    pub path: PathBuf,
    pub image: egui::ColorImage,
}
pub struct Loader {
    tx: mpsc::Sender<Result<Decoded, (PathBuf,String)>>,
    rx: mpsc::Receiver<Result<Decoded, (PathBuf,String)>>,
}
impl Default for Loader {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        Self { tx, rx }
    }
}
impl Loader {
    pub fn request(&self, path: &Path, ctx: &egui::Context) {
        let (path, tx, ctx) = (path.to_owned(), self.tx.clone(), ctx.clone());
        std::thread::spawn(move || {
            let result = decode(&path).map_err(|e| (path.clone(),e.to_string()));
            let _ = tx.send(result);
            ctx.request_repaint();
        });
    }
    pub fn poll(&self) -> Option<Result<Decoded, (PathBuf,String)>> {
        self.rx.try_recv().ok()
    }
}
pub fn clipboard_image(directory:&Path)->anyhow::Result<PathBuf>{
    let mut clipboard=arboard::Clipboard::new()?;
    let image=clipboard.get_image().map_err(|_|anyhow::anyhow!("The clipboard does not contain an image. Copy a screenshot and try again."))?;
    anyhow::ensure!(image.width.saturating_mul(image.height)<=16*1024*1024,"Clipboard image exceeds the preview limit");
    let folder=directory.join("clipboard");std::fs::create_dir_all(&folder)?;
    let path=folder.join(format!("{}.png",uuid::Uuid::new_v4()));
    image::save_buffer_with_format(&path,&image.bytes,image.width as u32,image.height as u32,image::ColorType::Rgba8,image::ImageFormat::Png)?;
    Ok(path)
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn missing_image_reports_a_recoverable_error(){assert!(decode(Path::new("Z:/this-image-does-not-exist.png")).is_err());}
    #[test] fn oversized_header_is_rejected_before_allocation(){
        let directory=std::env::current_dir().unwrap().join(".work/media-tests");std::fs::create_dir_all(&directory).unwrap();
        let path=directory.join(format!("{}.bmp",uuid::Uuid::new_v4()));
        let mut bytes=vec![0u8;54];bytes[..2].copy_from_slice(b"BM");bytes[14..18].copy_from_slice(&40u32.to_le_bytes());bytes[18..22].copy_from_slice(&100000u32.to_le_bytes());bytes[22..26].copy_from_slice(&100000u32.to_le_bytes());std::fs::write(&path,bytes).unwrap();assert!(decode(&path).is_err());
    }
}
pub fn decode(path: &Path) -> anyhow::Result<Decoded> {
    anyhow::ensure!(
        std::fs::metadata(path)?.len() <= 32 * 1024 * 1024,
        "Image file exceeds the 32 MiB preview limit"
    );
    let mut reader = image::ImageReader::open(path)?.with_guessed_format()?;
    let mut limits = image::Limits::default();
    limits.max_alloc = Some(64 * 1024 * 1024);
    limits.max_image_width = Some(8192);
    limits.max_image_height = Some(8192);
    reader.limits(limits);
    let decoded = reader.decode()?.to_rgba8();
    let size = [decoded.width() as usize, decoded.height() as usize];
    anyhow::ensure!(
        size[0].saturating_mul(size[1]) <= 16 * 1024 * 1024,
        "Image dimensions exceed the preview limit"
    );
    Ok(Decoded {
        path: path.into(),
        image: egui::ColorImage::from_rgba_unmultiplied(size, decoded.as_raw()),
    })
}
