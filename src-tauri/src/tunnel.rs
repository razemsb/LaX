use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

use crate::config::Paths;
use crate::error::{LaxError, LaxResult};
use crate::process::ProcessTable;
use crate::state::Orchestrator;

pub fn cloudflared_bin(paths: &Paths) -> Option<PathBuf> {
    let dir = paths.root.join("bin").join("cloudflared");
    let exe = crate::platform::bin_path(&dir, "cloudflared");
    exe.exists().then_some(exe)
}

pub fn available(paths: &Paths) -> bool {
    cloudflared_bin(paths).is_some()
}

/// Spawn `cloudflared tunnel --url <url>` (quick tunnel, no account).
/// A reader thread scans output for the public trycloudflare URL and
/// publishes it into the orchestrator's tunnel map.
pub fn start(
    paths: &Paths,
    table: &mut ProcessTable,
    state: Arc<Mutex<Orchestrator>>,
    key: String,
    local_url: String,
) -> LaxResult<()> {
    let bin = cloudflared_bin(paths).ok_or_else(|| LaxError::msg(rust_i18n::t!("share.missing")))?;
    let log = paths.root.join("logs").join(format!("tunnel-{key}.log"));
    if let Some(parent) = log.parent() {
        let _ = fs::create_dir_all(parent);
    }
    // fresh log per start (a stale one could contain an old URL)
    fs::write(&log, "").map_err(|e| LaxError::msg(format!("log {}: {e}", log.display())))?;

    let (pipe_reader, pipe_writer) =
        os_pipe::pipe().map_err(|e| LaxError::msg(format!("pipe: {e}")))?;
    let err_writer = pipe_writer
        .try_clone()
        .map_err(|e| LaxError::msg(format!("pipe clone: {e}")))?;

    let mut cmd = Command::new(&bin);
    cmd.args(["tunnel", "--no-autoupdate", "--url", &local_url])
        .current_dir(&paths.root)
        .stdin(Stdio::null())
        .stdout(Stdio::from(pipe_writer))
        .stderr(Stdio::from(err_writer));
    crate::platform::hide_window(&mut cmd);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000 | 0x0000_0200);
    }

    let child = cmd
        .spawn()
        .map_err(|e| LaxError::msg(format!("spawn cloudflared: {e}")))?;
    let pid = child.id();
    table.insert_raw(format!("tunnel-{key}"), child);

    let thread_key = key.clone();
    std::thread::spawn(move || {
        let re = regex_lite::Regex::new(r"https://([a-z0-9-]+)\.trycloudflare\.com").unwrap();
        // service subdomains logged during registration, not the tunnel URL
        let mut reader = BufReader::new(pipe_reader);
        let mut line = String::new();
        let mut log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log)
            .map_err(|e| format!("{e}"))
            .ok();
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) | Err(_) => break,
                Ok(_) => {
                    if let Some(f) = log_file.as_mut() {
                        let _ = std::io::Write::write_all(f, line.as_bytes());
                    }
                    for m in re.find_iter(&line) {
                        let url = m.as_str().to_string();
                        let host = url.trim_start_matches("https://");
                        if matches!(host.split('.').next().unwrap_or(""), "api" | "www" | "dash") {
                            continue;
                        }
                        if let Ok(mut g) = state.lock() {
                            g.tunnel_ready(&thread_key, url);
                        }
                        break;
                    }
                }
            }
        }
    });

    tracing::info!("tunnel {key} spawned (pid {pid}) for {local_url}");
    Ok(())
}

pub fn stop(table: &mut ProcessTable, key: &str) {
    table.stop(&format!("tunnel-{key}"));
}
