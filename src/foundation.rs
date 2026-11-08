use eframe::egui;
use egui_term::{BackendSettings, TerminalBackend, TerminalView};
use std::sync::mpsc;
use terminal_workspace::{bridge::Bridge,opencode::Backend,process::Job};

struct Foundation {
    terminals: Vec<TerminalBackend>,
    events: mpsc::Receiver<(u64,egui_term::PtyEvent)>,
    focus: usize,
    error: Option<String>,
    jobs: Vec<Job>,
    backend: Option<Backend>,
    bridge: Option<Bridge>,
    sessions: Vec<String>,
    preview: Option<egui::TextureHandle>,
    media: terminal_workspace::media::Loader,
}
impl Foundation {
    fn new(ctx: &egui::Context) -> Self {
        let (tx,rx)=mpsc::channel();
        let mut app=Self {terminals:Vec::new(),events:rx,focus:0,error:None,jobs:Vec::new(),backend:None,bridge:None,sessions:Vec::new(),preview:None,media:Default::default()};
        if std::env::args().any(|a|a=="--opencode-gate") {
            let setup=(||->anyhow::Result<()> {
                let directory=std::env::current_dir()?;
                let bridge=Bridge::start()?;
                let backend=Backend::start(&bridge,&directory,&directory.join("plugin"))?;
                for index in 0..2 {
                    let id=index.to_string();
                    let session=backend.create(&directory)?;
                    bridge.bind(&id,&session);
                    let launch=backend.launch(&bridge,&id,&session,&directory)?;
                    let settings=BackendSettings{shell:launch.program,args:launch.args,working_directory:Some(launch.directory),env:launch.env,..Default::default()};
                    let terminal=TerminalBackend::new(index,ctx.clone(),tx.clone(),settings)?;
                    let job=Job::new()?;job.assign(terminal.pty_id())?;
                    app.jobs.push(job);app.terminals.push(terminal);app.sessions.push(session);
                }
                for path in [directory.join(".work/gate-fixtures/notes.txt"),directory.join(".work/gate-fixtures/folder"),directory.join(".work/gate-fixtures/screenshot.png")] {bridge.stage(&app.sessions[0],&path)?;}
                app.backend=Some(backend);app.bridge=Some(bridge);
                Ok(())
            })();
            if let Err(error)=setup {app.error=Some(format!("{error:#}"));}
            return app;
        }
        for index in 0..2 {
            let settings=BackendSettings {shell:"C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe".into(),args:vec!["-NoLogo".into(),"-NoProfile".into()],working_directory:std::env::current_dir().ok(),..Default::default()};
            match TerminalBackend::new(index as u64,ctx.clone(),tx.clone(),settings) {Ok(t)=>{
                match Job::new().and_then(|job|{job.assign(t.pty_id())?;Ok(job)}) {Ok(job)=>app.jobs.push(job),Err(error)=>app.error=Some(error.to_string())}
                app.terminals.push(t)
            },Err(e)=>app.error=Some(e.to_string())}
        }
        app
    }
}
impl eframe::App for Foundation {
    fn ui(&mut self,ui: &mut egui::Ui,_frame:&mut eframe::Frame) {
        while let Ok((_id,event))=self.events.try_recv() {if let egui_term::PtyEvent::ClipboardStore(_,text)=event {ui.ctx().copy_text(text)}}
        while let Some(result)=self.media.poll() {match result{Ok(decoded)=>{
            let _=std::fs::write("artifacts/foundation-preview.json",serde_json::json!({"path":decoded.path,"size":decoded.image.size}).to_string());
            self.preview=Some(ui.ctx().load_texture(decoded.path.to_string_lossy(),decoded.image,Default::default()));
        },Err(error)=>self.error=Some(error)}}
        ui.heading("Terminal Workspace · foundation gate");
        ui.label("Two independent PowerShell processes. This is a development build.");
        if let Some(error)=&self.error {ui.colored_label(egui::Color32::LIGHT_RED,error);}
        if let Some(bridge)=&self.bridge {
            let state=bridge.state.lock().unwrap();
            let connected=state.sessions.values().filter(|s|s.tui_connected).count();
            ui.label(format!("OpenCode bridge: {} TUI connections; staged files: {} / {}",connected,self.sessions.first().map(|id|state.sessions[id].files.len()).unwrap_or(0),self.sessions.get(1).map(|id|state.sessions[id].files.len()).unwrap_or(0)));
            let report=serde_json::json!({"connected":connected,"loaded":state.tui_loaded,"server":state.server_connected,"staged":self.sessions.iter().map(|id|state.sessions[id].files.iter().map(|f|f.name.clone()).collect::<Vec<_>>()).collect::<Vec<_>>(),"error":self.error.as_ref().or(state.error.as_ref())});
            let _=std::fs::write("artifacts/native-opencode-gate.json",report.to_string());
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(500));
        }
        ui.separator();
        if let Some(texture)=&self.preview {
            egui::Panel::right("image-preview").default_size(300.).show(ui,|ui|{ui.label("Image preview");ui.image(texture);});
        }
        let height=ui.available_height();
        ui.columns(2,|columns|{
            for (index,terminal) in self.terminals.iter_mut().enumerate() {
                let column=&mut columns[index];
                column.label(format!("PowerShell {} · PID {}",index+1,terminal.pty_id()));
                let size=egui::vec2(column.available_width(),(height-28.).max(80.));
                let view=TerminalView::new(column,terminal).set_focus(self.focus==index).set_size(size);
                let response=column.add(view);
                if response.clicked() {self.focus=index;}
                for link in terminal.take_opened_links() {
                    if let Ok(url)=url::Url::parse(&link) {if let Ok(path)=url.to_file_path() {self.media.request(&path,column.ctx());}}
                }
            }
        });
    }
}
pub fn run() -> eframe::Result {
    let options=eframe::NativeOptions {viewport:egui::ViewportBuilder::default().with_title("Terminal Workspace · foundation").with_inner_size([1100.,700.]),..Default::default()};
    eframe::run_native("Terminal Workspace",options,Box::new(|cc|Ok(Box::new(Foundation::new(&cc.egui_ctx)))))
}

