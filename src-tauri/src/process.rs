use std::collections::HashMap;
use std::fs;
use std::fs::OpenOptions;
use std::path::Path;
use std::process::{Child, Command, Stdio};

use crate::error::{LaxError, LaxResult};
use crate::platform;

#[cfg(windows)]
const CREATE_NO_WINDOW_GROUP: u32 = 0x0800_0000 | 0x0000_0200;

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
        self.spawn_inner(name, program, args, cwd, env, None)
    }

    pub fn spawn_logged(
        &mut self,
        name: &str,
        program: &Path,
        args: &[&str],
        cwd: &Path,
        env: &[(&str, String)],
        log: &Path,
    ) -> LaxResult<u32> {
        self.spawn_inner(name, program, args, cwd, env, Some(log))
    }

    fn spawn_inner(
        &mut self,
        name: &str,
        program: &Path,
        args: &[&str],
        cwd: &Path,
        env: &[(&str, String)],
        log: Option<&Path>,
    ) -> LaxResult<u32> {
        if !program.exists() {
            return Err(LaxError::msg(format!(
                "binary not found: {}",
                program.display()
            )));
        }
        self.stop(name);

        let mut cmd = Command::new(program);
        cmd.args(args).current_dir(cwd).stdin(Stdio::null());
        if let Some(log) = log {
            if let Some(parent) = log.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(log)
                .map_err(|e| LaxError::msg(format!("log {}: {e}", log.display())))?;
            let err = file
                .try_clone()
                .map_err(|e| LaxError::msg(format!("log clone: {e}")))?;
            cmd.stdout(Stdio::from(file)).stderr(Stdio::from(err));
        } else {
            cmd.stdout(Stdio::null()).stderr(Stdio::null());
        }
        for (k, v) in env {
            cmd.env(k, v);
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(CREATE_NO_WINDOW_GROUP);
        }
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            cmd.process_group(0);
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
            platform::kill_pid(proc.pid);
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

#[allow(dead_code)]
pub fn taskkill(pid: u32) {
    platform::kill_pid(pid);
}

pub fn taskkill_image(image: &str) {
    platform::kill_image(image);
}

pub fn run_capture(program: &Path, args: &[&str], cwd: &Path) -> LaxResult<(i32, String)> {
    run_capture_env(program, args, cwd, &[])
}

pub fn run_capture_env(
    program: &Path,
    args: &[&str],
    cwd: &Path,
    env: &[(&str, String)],
) -> LaxResult<(i32, String)> {
    let mut cmd = Command::new(program);
    cmd.args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (k, v) in env {
        cmd.env(k, v);
    }
    platform::hide_window(&mut cmd);
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

/// Extra `LD_LIBRARY_PATH` entries for vendored ELF libs (MariaDB generic tarball).
pub fn unix_lib_env(dirs: &[&Path]) -> Vec<(String, String)> {
    #[cfg(unix)]
    {
        let extra: Vec<String> = dirs
            .iter()
            .filter(|p| p.is_dir())
            .map(|p| p.to_string_lossy().into_owned())
            .collect();
        if extra.is_empty() {
            return Vec::new();
        }
        let rest = std::env::var("LD_LIBRARY_PATH").unwrap_or_default();
        let joined = if rest.is_empty() {
            extra.join(":")
        } else {
            format!("{}:{rest}", extra.join(":"))
        };
        vec![("LD_LIBRARY_PATH".into(), joined)]
    }
    #[cfg(not(unix))]
    {
        let _ = dirs;
        Vec::new()
    }
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
