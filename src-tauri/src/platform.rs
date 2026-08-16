//! OS helpers. Windows is the real portable stack; Linux is the GUI + the same
//! layout, with native binaries (`httpd`, `php-cgi`) instead of `*.exe`.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
#[cfg(windows)]
const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;

/// `httpd` on Unix, `httpd.exe` on Windows.
pub fn bin(name: &str) -> String {
    #[cfg(windows)]
    {
        if name.ends_with(".exe") || name.ends_with(".bat") {
            return name.to_string();
        }
        format!("{name}.exe")
    }
    #[cfg(not(windows))]
    {
        name.trim_end_matches(".exe")
            .trim_end_matches(".bat")
            .to_string()
    }
}

pub fn bin_path(dir: &Path, name: &str) -> PathBuf {
    dir.join(bin(name))
}

pub fn path_sep() -> char {
    if cfg!(windows) { ';' } else { ':' }
}

pub fn hide_window(cmd: &mut Command) {
    #[cfg(windows)]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let _ = cmd;
}

pub fn open_url(url: &str) -> Result<(), String> {
    let url = url.trim();
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err("разрешены только http(s) ссылки".into());
    }
    #[cfg(windows)]
    {
        return spawn_hidden("cmd.exe", &["/C", "start", "", url]);
    }
    #[cfg(not(windows))]
    {
        spawn_hidden("xdg-open", &[url])
    }
}

pub fn open_path(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("path not found: {}", path.display()));
    }
    #[cfg(windows)]
    {
        Command::new("explorer.exe")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }
    #[cfg(not(windows))]
    {
        let s = path.display().to_string();
        spawn_hidden("xdg-open", &[&s])
    }
}

pub fn open_editor(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("file not found: {}", path.display()));
    }
    #[cfg(windows)]
    {
        Command::new("notepad.exe")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }
    #[cfg(not(windows))]
    {
        let s = path.display().to_string();
        spawn_hidden("xdg-open", &[&s])
    }
}

pub fn open_vscode(path: &Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        let s = path.display().to_string();
        return spawn_hidden("cmd.exe", &["/C", "code", &s]);
    }
    #[cfg(not(windows))]
    {
        Command::new("code")
            .arg(path)
            .spawn()
            .map_err(|e| format!("VS Code not found: {e}"))?;
        Ok(())
    }
}

pub fn open_terminal(dir: &Path, cmdline: Option<&str>, path_prefix: Option<&str>) -> Result<(), String> {
    if !dir.exists() {
        return Err(format!("path not found: {}", dir.display()));
    }
    #[cfg(windows)]
    {
        let mut cmd = Command::new("cmd.exe");
        if let Some(line) = cmdline {
            cmd.args(["/K", line]);
        } else {
            cmd.arg("/K");
        }
        cmd.current_dir(dir).creation_flags(CREATE_NEW_CONSOLE);
        apply_path_prefix(&mut cmd, path_prefix);
        cmd.spawn().map_err(|e| e.to_string())?;
        return Ok(());
    }
    #[cfg(not(windows))]
    {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".into());
        let run = match cmdline {
            Some(line) => format!("{line}; exec {shell}"),
            None => shell.clone(),
        };
        let mut cmd = Command::new("x-terminal-emulator");
        cmd.args(["-e", &shell, "-lc", &run]).current_dir(dir);
        apply_path_prefix(&mut cmd, path_prefix);
        match cmd.spawn() {
            Ok(_) => Ok(()),
            Err(_) => {
                let mut fallback = Command::new(&shell);
                fallback.arg("-lc").arg(&run).current_dir(dir);
                apply_path_prefix(&mut fallback, path_prefix);
                fallback
                    .spawn()
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            }
        }
    }
}

fn apply_path_prefix(cmd: &mut Command, path_prefix: Option<&str>) {
    if let Some(prefix) = path_prefix {
        if !prefix.is_empty() {
            let rest = std::env::var("PATH").unwrap_or_default();
            cmd.env("PATH", format!("{prefix}{}{rest}", path_sep()));
        }
    }
}

pub fn kill_pid(pid: u32) {
    if pid == 0 {
        return;
    }
    #[cfg(windows)]
    {
        let mut cmd = Command::new("taskkill");
        cmd.args(["/F", "/T", "/PID", &pid.to_string()]);
        hide_window(&mut cmd);
        let _ = cmd.stdout(Stdio::null()).stderr(Stdio::null()).status();
    }
    #[cfg(not(windows))]
    {
        let _ = Command::new("kill")
            .args(["-TERM", "--", &format!("-{pid}")])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        let _ = Command::new("kill")
            .args(["-KILL", &pid.to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

pub fn kill_image(image: &str) {
    #[cfg(windows)]
    {
        let mut cmd = Command::new("taskkill");
        cmd.args(["/F", "/IM", image, "/T"]);
        hide_window(&mut cmd);
        let _ = cmd.stdout(Stdio::null()).stderr(Stdio::null()).status();
    }
    #[cfg(not(windows))]
    {
        let name = image.trim_end_matches(".exe");
        let _ = Command::new("pkill")
            .args(["-f", name])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

pub fn hosts_path() -> &'static str {
    #[cfg(windows)]
    {
        r"C:\Windows\System32\drivers\etc\hosts"
    }
    #[cfg(not(windows))]
    {
        "/etc/hosts"
    }
}

pub fn composer_file(root: &Path) -> PathBuf {
    let win = root.join("bin").join("composer").join("composer.bat");
    let unix = root.join("bin").join("composer").join("composer");
    if cfg!(windows) && win.exists() {
        win
    } else if unix.exists() {
        unix
    } else {
        win
    }
}

fn spawn_hidden(program: &str, args: &[&str]) -> Result<(), String> {
    let mut cmd = Command::new(program);
    cmd.args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    hide_window(&mut cmd);
    cmd.spawn().map_err(|e| e.to_string())?;
    Ok(())
}
