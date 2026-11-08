use anyhow::Result;
use std::{collections::HashMap,path::{Path,PathBuf}};

pub const POWERSHELL: &str = "C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe";
#[derive(Clone, Debug)]
pub struct Launch {pub program:String,pub args:Vec<String>,pub env:HashMap<String,String>,pub directory:PathBuf}

pub fn resolve_program(name:&str) -> Result<PathBuf> {
    let requested=Path::new(name);
    if requested.is_absolute() && requested.is_file() {return Ok(requested.to_owned())}
    for folder in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()) {
        for extension in [".exe",".cmd",".bat",".ps1",""] {
            let candidate=folder.join(format!("{name}{extension}"));
            if candidate.is_file() {return Ok(candidate)}
        }
    }
    anyhow::bail!("Command not found: {name}")
}
pub fn tool(name:&str,directory:&Path,args:Vec<String>) -> Result<Launch> {
    anyhow::ensure!(directory.is_dir(),"Project folder is unavailable: {}",directory.display());
    let executable=if name=="powershell" {PathBuf::from(POWERSHELL)}else{resolve_program(name)?};
    let mut launch=Launch {program:executable.to_string_lossy().into(),args,env:HashMap::new(),directory:directory.into()};
    match executable.extension().and_then(|e|e.to_str()).unwrap_or("").to_ascii_lowercase().as_str() {
        "ps1" => {launch.program=POWERSHELL.into();let mut prefix=vec!["-NoLogo".into(),"-NoProfile".into(),"-File".into(),executable.to_string_lossy().into()];prefix.append(&mut launch.args);launch.args=prefix;},
        "cmd"|"bat" => {
            // PowerShell invokes the exact script; user arguments are quoted as literals.
            let command=format!("& '{}' {}",ps_quote(&executable.to_string_lossy()),launch.args.iter().map(|s|format!("'{}'",ps_quote(s))).collect::<Vec<_>>().join(" "));
            launch.program=POWERSHELL.into();launch.args=vec!["-NoLogo".into(),"-NoProfile".into(),"-Command".into(),command];
        },_=>{}
    }
    Ok(launch)
}
pub fn ps_quote(value:&str)->String {value.replace('\'',"''")}

#[cfg(windows)]
pub struct Job(std::os::windows::io::OwnedHandle);
#[cfg(windows)]
impl Job {
    pub fn new()->Result<Self> {
        use std::os::windows::io::FromRawHandle;
        use windows_sys::Win32::System::JobObjects::*;
        let raw=unsafe{CreateJobObjectW(std::ptr::null(),std::ptr::null())};
        anyhow::ensure!(!raw.is_null(),"Create process job: {}",std::io::Error::last_os_error());
        let handle=unsafe{std::os::windows::io::OwnedHandle::from_raw_handle(raw)};
        let mut limits:JOBOBJECT_EXTENDED_LIMIT_INFORMATION=unsafe{std::mem::zeroed()};
        limits.BasicLimitInformation.LimitFlags=JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        anyhow::ensure!(unsafe{SetInformationJobObject(raw,JobObjectExtendedLimitInformation,&limits as *const _ as *const _,std::mem::size_of_val(&limits) as u32)}!=0,"Configure process job: {}",std::io::Error::last_os_error());
        Ok(Self(handle))
    }
    pub fn assign(&self,pid:u32)->Result<()> {
        use std::os::windows::io::{AsRawHandle,FromRawHandle};
        use windows_sys::Win32::System::{Threading::*,JobObjects::*};
        let raw=unsafe{OpenProcess(PROCESS_SET_QUOTA|PROCESS_TERMINATE,0,pid)};
        anyhow::ensure!(!raw.is_null(),"Open owned child: {}",std::io::Error::last_os_error());
        let process=unsafe{std::os::windows::io::OwnedHandle::from_raw_handle(raw)};
        anyhow::ensure!(unsafe{AssignProcessToJobObject(self.0.as_raw_handle(),process.as_raw_handle())}!=0,"Assign child job: {}",std::io::Error::last_os_error());
        Ok(())
    }
}

pub fn child(launch:&Launch)->std::process::Command {
    let mut command=std::process::Command::new(&launch.program);
    command.args(&launch.args).current_dir(&launch.directory).envs(&launch.env);
    #[cfg(windows)] {use std::os::windows::process::CommandExt;command.creation_flags(0x08000000);}
    command
}
