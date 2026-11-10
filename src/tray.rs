use std::sync::mpsc;
use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder, TrayIconEvent,
    menu::{Menu, MenuEvent, MenuItem},
};
#[derive(Clone, Copy)]
pub enum Action {
    Open,
    Quit,
}
pub struct Tray {
    _icon: TrayIcon,
    count: MenuItem,
    rx: mpsc::Receiver<Action>,
    last: usize,
}
impl Tray {
    pub fn new(ctx: &eframe::egui::Context) -> anyhow::Result<Self> {
        anyhow::ensure!(
            std::env::var_os("TERMINAL_WORKSPACE_TEST_NO_TRAY").is_none(),
            "Tray is unavailable in this test"
        );
        let menu = Menu::new();
        let open = MenuItem::new("Open window", true, None);
        let count = MenuItem::new("0 running terminals", false, None);
        let quit = MenuItem::new("Quit", true, None);
        menu.append_items(&[&open, &count, &quit])?;
        let (mut pixels, size) = (Vec::new(), 32);
        for y in 0..size {
            for x in 0..size {
                let ink = (x > 6
                    && x < 15
                    && y > 7
                    && y < 22
                    && (x + y > 20 && x + y < 26 || x == y - 1))
                    || (x > 17 && x < 25 && y > 20 && y < 24);
                pixels.extend_from_slice(if ink {
                    &[235, 235, 235, 255]
                } else {
                    &[25, 25, 25, 255]
                });
            }
        }
        let icon = TrayIconBuilder::new()
            .with_tooltip("Terminal Workspace")
            .with_menu(Box::new(menu))
            .with_icon(Icon::from_rgba(pixels, size, size)?)
            .build()?;
        let (tx, rx) = mpsc::channel();
        let tx2 = tx.clone();
        let context = ctx.clone();
        let open_id = open.id().clone();
        let quit_id = quit.id().clone();
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            let action = if event.id == open_id {
                Some(Action::Open)
            } else if event.id == quit_id {
                Some(Action::Quit)
            } else {
                None
            };
            if let Some(action) = action {
                let _ = tx.send(action);
                context.request_repaint();
            }
        }));
        let context = ctx.clone();
        TrayIconEvent::set_event_handler(Some(move |event| {
            if matches!(event, TrayIconEvent::DoubleClick { .. }) {
                let _ = tx2.send(Action::Open);
                context.request_repaint();
            }
        }));
        Ok(Self {
            _icon: icon,
            count,
            rx,
            last: usize::MAX,
        })
    }
    pub fn update(&mut self, count: usize) {
        if count != self.last {
            self.count.set_text(format!("{count} running terminals"));
            self.last = count;
        }
    }
    pub fn poll(&self) -> Option<Action> {
        self.rx.try_recv().ok()
    }
}
