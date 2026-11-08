use std::{
    os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use windows_sys::Win32::{Foundation::*, System::Threading::*};
pub struct Instance {
    _mutex: OwnedHandle,
    event: Arc<OwnedHandle>,
    stop: Arc<AtomicBool>,
}
fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}
impl Instance {
    pub fn acquire() -> anyhow::Result<Option<Self>> {
        use std::hash::{Hash, Hasher};
        let mut hash = std::collections::hash_map::DefaultHasher::new();
        std::env::var_os("LOCALAPPDATA").hash(&mut hash);
        let name = format!("Local\\TerminalWorkspace-{:x}", hash.finish());
        let raw = unsafe { CreateMutexW(std::ptr::null(), 0, wide(&name).as_ptr()) };
        anyhow::ensure!(
            !raw.is_null(),
            "Create instance lock: {}",
            std::io::Error::last_os_error()
        );
        let existing = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;
        let mutex = unsafe { OwnedHandle::from_raw_handle(raw) };
        let raw = unsafe {
            CreateEventW(
                std::ptr::null(),
                0,
                0,
                wide(&format!("{name}-activate")).as_ptr(),
            )
        };
        anyhow::ensure!(
            !raw.is_null(),
            "Create activation event: {}",
            std::io::Error::last_os_error()
        );
        let event = Arc::new(unsafe { OwnedHandle::from_raw_handle(raw) });
        if existing {
            anyhow::ensure!(
                unsafe { SetEvent(event.as_raw_handle()) } != 0,
                "Activate existing instance"
            );
            return Ok(None);
        }
        Ok(Some(Self {
            _mutex: mutex,
            event,
            stop: Arc::new(AtomicBool::new(false)),
        }))
    }
    pub fn listen(&self, ctx: eframe::egui::Context) {
        let (event, stop) = (self.event.clone(), self.stop.clone());
        std::thread::spawn(move || {
            while !stop.load(Ordering::Relaxed) {
                if unsafe { WaitForSingleObject(event.as_raw_handle(), 250) } == WAIT_OBJECT_0 {
                    ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Visible(true));
                    ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Minimized(false));
                    ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Focus);
                    ctx.request_repaint();
                }
            }
        });
    }
}
impl Drop for Instance {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}
