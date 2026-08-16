use std::fs;
use std::net::{SocketAddr, TcpStream};
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use crate::platform;

pub fn list_subdirs(dir: &Path) -> Vec<String> {
    let mut names = Vec::new();
    let Ok(rd) = fs::read_dir(dir) else {
        return names;
    };
    for ent in rd.flatten() {
        if ent.path().is_dir() {
            names.push(ent.file_name().to_string_lossy().into_owned());
        }
    }
    names.sort();
    names
}

pub fn php_versions(root: &Path) -> Vec<String> {
    list_subdirs(&root.join("bin").join("php"))
        .into_iter()
        .filter(|n| {
            let d = root.join("bin").join("php").join(n);
            crate::platform::bin_path(&d, "php").exists()
                || crate::platform::bin_path(&d, "php-cgi").exists()
        })
        .collect()
}

pub fn mysql_versions(root: &Path) -> Vec<String> {
    list_subdirs(&root.join("bin").join("mysql"))
}

pub fn nginx_versions(root: &Path) -> Vec<String> {
    list_subdirs(&root.join("bin").join("nginx"))
}

pub fn apache_versions(root: &Path) -> Vec<String> {
    list_subdirs(&root.join("bin").join("apache"))
}

/// Directory that contains `node` / `npm` for the portable PATH prefix.
pub fn node_bin_dir(root: &Path) -> Option<std::path::PathBuf> {
    let dirs = [
        root.join("bin").join("node"),
        root.join("bin").join("nodejs"),
    ];
    for d in dirs {
        if crate::platform::bin_path(&d, "node").exists() {
            return Some(d);
        }
        if let Ok(rd) = fs::read_dir(&d) {
            for ent in rd.flatten() {
                let p = ent.path();
                if p.is_dir() && crate::platform::bin_path(&p, "node").exists() {
                    return Some(p);
                }
            }
        }
    }
    None
}

/// `phpDir;nodeDir` (or `:` on Unix) to prepend to PATH in spawned terminals.
pub fn tools_path_prefix(root: &Path, php_dir: &Path) -> String {
    let sep = crate::platform::path_sep();
    let mut parts = Vec::new();
    if php_dir.is_dir() {
        parts.push(php_dir.to_string_lossy().into_owned());
    }
    if let Some(node) = node_bin_dir(root) {
        parts.push(node.to_string_lossy().into_owned());
    }
    parts.join(&sep.to_string())
}

pub fn port_open(port: u16) -> bool {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    TcpStream::connect_timeout(&addr, Duration::from_millis(40)).is_ok()
}

/// Who is listening on TCP port (Windows: netstat; Unix: ss/lsof fallback).
pub fn port_listener(port: u16) -> Option<(u32, String)> {
    #[cfg(windows)]
    {
        return windows_listener(port);
    }
    #[cfg(not(windows))]
    {
        unix_listener(port)
    }
}

#[cfg(windows)]
fn windows_listener(port: u16) -> Option<(u32, String)> {
    let mut cmd = Command::new("netstat");
    cmd.args(["-ano", "-p", "tcp"]);
    platform::hide_window(&mut cmd);
    let out = cmd.output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    for line in text.lines() {
        if !line.contains("LISTENING") {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 {
            continue;
        }
        let local = parts[1];
        let Some((_, pstr)) = local.rsplit_once(':') else {
            continue;
        };
        if pstr.parse::<u16>().ok() != Some(port) {
            continue;
        }
        let pid: u32 = parts.last()?.parse().ok()?;
        let name = process_name(pid).unwrap_or_else(|| format!("PID {pid}"));
        return Some((pid, name));
    }
    None
}

#[cfg(windows)]
fn process_name(pid: u32) -> Option<String> {
    let mut cmd = Command::new("tasklist");
    cmd.args(["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"]);
    platform::hide_window(&mut cmd);
    let out = cmd.output().ok()?;
    let line = String::from_utf8_lossy(&out.stdout);
    let line = line.lines().next()?.trim();
    if line.is_empty() || line.starts_with("INFO:") {
        return None;
    }
    // "httpd.exe","1234","Session","1","12 K"
    let name = line.trim_start_matches('"').split('"').next()?.to_string();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

#[cfg(not(windows))]
fn unix_listener(port: u16) -> Option<(u32, String)> {
    let out = Command::new("ss")
        .args(["-ltnp", &format!("( sport = :{port} )")])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    for line in text.lines() {
        if let Some(idx) = line.find("pid=") {
            let rest = &line[idx + 4..];
            let pid: u32 = rest
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse()
                .ok()?;
            return Some((pid, format!("pid {pid}")));
        }
    }
    None
}
