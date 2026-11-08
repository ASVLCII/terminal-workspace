use std::{path::{Path,PathBuf},sync::mpsc};
use eframe::egui;
pub struct Decoded {pub path:PathBuf,pub image:egui::ColorImage}
pub struct Loader {tx:mpsc::Sender<Result<Decoded,String>>,rx:mpsc::Receiver<Result<Decoded,String>>}
impl Default for Loader {fn default()->Self{let(tx,rx)=mpsc::channel();Self{tx,rx}}}
impl Loader {
    pub fn request(&self,path:&Path,ctx:&egui::Context) {
        let (path,tx,ctx)=(path.to_owned(),self.tx.clone(),ctx.clone());
        std::thread::spawn(move||{let result=decode(&path).map_err(|e|e.to_string());let _=tx.send(result);ctx.request_repaint();});
    }
    pub fn poll(&self)->Option<Result<Decoded,String>>{self.rx.try_recv().ok()}
}
pub fn decode(path:&Path)->anyhow::Result<Decoded> {
    anyhow::ensure!(std::fs::metadata(path)?.len()<=32*1024*1024,"Image file exceeds the 32 MiB preview limit");
    let mut reader=image::ImageReader::open(path)?.with_guessed_format()?;
    let mut limits=image::Limits::default();limits.max_alloc=Some(64*1024*1024);limits.max_image_width=Some(8192);limits.max_image_height=Some(8192);reader.limits(limits);
    let decoded=reader.decode()?.to_rgba8();
    let size=[decoded.width() as usize,decoded.height() as usize];
    anyhow::ensure!(size[0].saturating_mul(size[1])<=16*1024*1024,"Image dimensions exceed the preview limit");
    Ok(Decoded {path:path.into(),image:egui::ColorImage::from_rgba_unmultiplied(size,decoded.as_raw())})
}
