use anyhow::Result;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

pub const POWERSHELL: &str = "C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe";
pub fn canonical_directory(path: &Path) -> Result<PathBuf> {
    let canonical = path.canonicalize()?;
    let text = canonical.to_string_lossy();
    Ok(if let Some(rest) = text.strip_prefix(r"\\?\UNC\") {
        PathBuf::from(format!(r"\\{rest}"))
    } else if let Some(rest) = text.strip_prefix(r"\\?\") {
        PathBuf::from(rest)
    } else {
        canonical
    })
}
#[derive(Clone, Debug)]
pub struct Launch {
    pub program: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub directory: PathBuf,
}

pub fn resolve_program(name: &str) -> Result<PathBuf> {
    let requested = Path::new(name);
    if requested.is_absolute() && requested.is_file() {
        return Ok(requested.to_owned());
    }
    for folder in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()) {
        for extension in [".exe", ".cmd", ".bat", ".ps1", ""] {
            let candidate = folder.join(format!("{name}{extension}"));
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }
    anyhow::bail!("Command not found: {name}")
}
pub fn tool(name: &str, directory: &Path, args: Vec<String>) -> Result<Launch> {
    anyhow::ensure!(
        directory.is_dir(),
        "Project folder is unavailable: {}",
        directory.display()
    );
    let executable = if name == "powershell" {
        PathBuf::from(POWERSHELL)
    } else {
        resolve_program(name)?
    };
    let mut launch = Launch {
        program: executable.to_string_lossy().into(),
        args,
        env: HashMap::new(),
        directory: directory.into(),
    };
    match executable
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "ps1" => {
            launch.program = POWERSHELL.into();
            let mut prefix = vec![
                "-NoLogo".into(),
                "-NoProfile".into(),
                "-File".into(),
                executable.to_string_lossy().into(),
            ];
            prefix.append(&mut launch.args);
            launch.args = prefix;
        }
        "cmd" | "bat" => {
            // PowerShell invokes the exact script; user arguments are quoted as literals.
            let command = format!(
                "& '{}' {}",
                ps_quote(&executable.to_string_lossy()),
                launch
                    .args
                    .iter()
                    .map(|s| format!("'{}'", ps_quote(s)))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            launch.program = POWERSHELL.into();
            launch.args = vec![
                "-NoLogo".into(),
                "-NoProfile".into(),
                "-Command".into(),
                command,
            ];
        }
        _ => {}
    }
    Ok(launch)
}
pub fn ps_quote(value: &str) -> String {
    value.replace('\'', "''")
}

#[cfg(windows)]
pub struct Job(std::os::windows::io::OwnedHandle);
#[cfg(windows)]
impl Job {
    pub fn active(&self) -> Result<u32> {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::System::JobObjects::*;
        let mut info: JOBOBJECT_BASIC_ACCOUNTING_INFORMATION = unsafe { std::mem::zeroed() };
        anyhow::ensure!(
            unsafe {
                QueryInformationJobObject(
                    self.0.as_raw_handle(),
                    JobObjectBasicAccountingInformation,
                    &mut info as *mut _ as *mut _,
                    std::mem::size_of_val(&info) as u32,
                    std::ptr::null_mut(),
                )
            } != 0,
            "Query owned processes: {}",
            std::io::Error::last_os_error()
        );
        Ok(info.ActiveProcesses)
    }
    pub fn force_stop(&self) -> Result<()> {
        use std::os::windows::io::AsRawHandle;
        anyhow::ensure!(
            unsafe {
                windows_sys::Win32::System::JobObjects::TerminateJobObject(
                    self.0.as_raw_handle(),
                    1,
                )
            } != 0,
            "Stop owned process job: {}",
            std::io::Error::last_os_error()
        );
        Ok(())
    }
    pub fn new() -> Result<Self> {
        use std::os::windows::io::FromRawHandle;
        use windows_sys::Win32::System::JobObjects::*;
        let raw = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        anyhow::ensure!(
            !raw.is_null(),
            "Create process job: {}",
            std::io::Error::last_os_error()
        );
        let handle = unsafe { std::os::windows::io::OwnedHandle::from_raw_handle(raw) };
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        anyhow::ensure!(
            unsafe {
                SetInformationJobObject(
                    raw,
                    JobObjectExtendedLimitInformation,
                    &limits as *const _ as *const _,
                    std::mem::size_of_val(&limits) as u32,
                )
            } != 0,
            "Configure process job: {}",
            std::io::Error::last_os_error()
        );
        Ok(Self(handle))
    }
    pub fn assign(&self, pid: u32) -> Result<()> {
        use std::os::windows::io::{AsRawHandle, FromRawHandle};
        use windows_sys::Win32::System::{JobObjects::*, Threading::*};
        let raw = unsafe { OpenProcess(PROCESS_SET_QUOTA | PROCESS_TERMINATE, 0, pid) };
        anyhow::ensure!(
            !raw.is_null(),
            "Open owned child: {}",
            std::io::Error::last_os_error()
        );
        let process = unsafe { std::os::windows::io::OwnedHandle::from_raw_handle(raw) };
        anyhow::ensure!(
            unsafe { AssignProcessToJobObject(self.0.as_raw_handle(), process.as_raw_handle()) }
                != 0,
            "Assign child job: {}",
            std::io::Error::last_os_error()
        );
        Ok(())
    }
}

pub fn child(launch: &Launch) -> std::process::Command {
    let mut command = std::process::Command::new(&launch.program);
    command
        .args(&launch.args)
        .current_dir(&launch.directory)
        .envs(&launch.env);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    command
}
#[cfg(test)]mod tests{
    use super::*;
    struct Guard(std::process::Child);impl Drop for Guard{fn drop(&mut self){let _=self.0.kill();let _=self.0.wait();}}
    #[test]fn stopping_an_owned_job_preserves_an_independent_process(){
        let dir=std::env::current_dir().unwrap();
        let launch=tool("powershell",&dir,vec!["-NoProfile".into(),"-Command".into(),"Start-Sleep 60".into()]).unwrap();
        let mut owned=Guard(child(&launch).spawn().unwrap());let mut independent=Guard(child(&launch).spawn().unwrap());let job=Job::new().unwrap();job.assign(owned.0.id()).unwrap();assert!(job.active().unwrap()>0);job.force_stop().unwrap();
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(3);while owned.0.try_wait().unwrap().is_none()&&std::time::Instant::now()<deadline{std::thread::sleep(std::time::Duration::from_millis(20));}
        assert!(owned.0.try_wait().unwrap().is_some());assert!(independent.0.try_wait().unwrap().is_none(),"An independently launched process must remain alive");
    }
    #[test]fn powershell_literal_arguments_cannot_escape_single_quotes(){assert_eq!(ps_quote("a' $(whoami) & b"),"a'' $(whoami) & b");}
}
