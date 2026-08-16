use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::{Child, Command, Stdio};

use crate::error::{LaxError, LaxResult};

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
#[cfg(windows)]
const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;

#[derive(Debug)]
pub struct ManagedProcess {
    pub pid: u32,
    child: Option<Child>,
    #[cfg(windows)]
    job: isize,
}

#[derive(Default)]
pub struct ProcessTable {
    inner: HashMap<String, ManagedProcess>,
}

impl ProcessTable {
    pub fn get(&self, name: &str) -> Option<&ManagedProcess> {
        self.inner.get(name)
    }

    pub fn spawn(
        &mut self,
        name: &str,
        program: &Path,
        args: &[&str],
        cwd: &Path,
        env: &[(&str, String)],
    ) -> LaxResult<u32> {
        if !program.exists() {
            return Err(LaxError::msg(format!(
                "binary not found: {}",
                program.display()
            )));
        }
        self.stop(name);

        let mut cmd = Command::new(program);
        cmd.args(args)
            .current_dir(cwd)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        for (k, v) in env {
            cmd.env(k, v);
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP);
        }

        let child = cmd
            .spawn()
            .map_err(|e| LaxError::msg(format!("spawn {} ({}): {e}", name, program.display())))?;
        let pid = child.id();

        #[cfg(windows)]
        let job = unsafe { assign_job(&child) };

        self.inner.insert(
            name.to_string(),
            ManagedProcess {
                pid,
                child: Some(child),
                #[cfg(windows)]
                job,
            },
        );
        Ok(pid)
    }

    pub fn stop(&mut self, name: &str) {
        if let Some(mut proc) = self.inner.remove(name) {
            #[cfg(windows)]
            unsafe {
                if proc.job != 0 {
                    windows_sys::Win32::System::JobObjects::TerminateJobObject(proc.job as _, 1);
                    windows_sys::Win32::Foundation::CloseHandle(proc.job as _);
                }
            }
            if let Some(mut child) = proc.child.take() {
                let _ = child.kill();
            }
            taskkill(proc.pid);
        }
    }

    pub fn stop_prefix(&mut self, prefix: &str) {
        let keys: Vec<String> = self
            .inner
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();
        for k in keys {
            self.stop(&k);
        }
    }

    pub fn stop_all(&mut self) {
        let keys: Vec<String> = self.inner.keys().cloned().collect();
        for k in keys {
            self.stop(&k);
        }
    }
}

pub fn taskkill(pid: u32) {
    if pid == 0 {
        return;
    }
    let mut cmd = Command::new("taskkill");
    cmd.args(["/F", "/T", "/PID", &pid.to_string()]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let _ = cmd.stdout(Stdio::null()).stderr(Stdio::null()).status();
}

pub fn taskkill_image(image: &str) {
    let mut cmd = Command::new("taskkill");
    cmd.args(["/F", "/IM", image, "/T"]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let _ = cmd.stdout(Stdio::null()).stderr(Stdio::null()).status();
}

pub fn run_capture(program: &Path, args: &[&str], cwd: &Path) -> LaxResult<(i32, String)> {
    let mut cmd = Command::new(program);
    cmd.args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let out = cmd
        .output()
        .map_err(|e| LaxError::msg(format!("{}: {e}", program.display())))?;
    let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    Ok((out.status.code().unwrap_or(-1), text))
}

pub fn write_file(path: &Path, body: &str) -> LaxResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, body)?;
    Ok(())
}

#[cfg(windows)]
unsafe fn assign_job(child: &Child) -> isize {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
        SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };

    let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
    if job.is_null() {
        return 0;
    }
    let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
    info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    let _ = SetInformationJobObject(
        job,
        JobObjectExtendedLimitInformation,
        &info as *const _ as *const _,
        std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
    );
    let handle = child.as_raw_handle() as isize;
    let _ = AssignProcessToJobObject(job, handle as _);
    job as isize
}
