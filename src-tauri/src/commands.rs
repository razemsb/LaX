use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use tauri::State;

use crate::config::LaxConfig;
use crate::logs;
use crate::php;
use crate::projects::ProjectInfo;
use crate::state::{AppState, ServiceInfo, Snapshot};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;

fn with_state<T, F>(state: &AppState, f: F) -> Result<T, String>
where
    F: FnOnce(&mut crate::state::Orchestrator) -> Result<T, String>,
{
    let mut g = state
        .inner
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?;
    f(&mut g)
}

#[tauri::command]
pub fn snapshot(state: State<AppState>) -> Result<Snapshot, String> {
    with_state(&state, |o| Ok(o.snapshot()))
}

#[tauri::command]
pub fn status(state: State<AppState>) -> Result<Vec<ServiceInfo>, String> {
    with_state(&state, |o| Ok(o.status()))
}

#[tauri::command]
pub fn start_all(state: State<AppState>) -> Result<Snapshot, String> {
    with_state(&state, |o| {
        o.start_all().map_err(|e| e.to_string())?;
        Ok(o.snapshot())
    })
}

#[tauri::command]
pub fn stop_all(state: State<AppState>) -> Result<Snapshot, String> {
    with_state(&state, |o| {
        o.stop_all();
        Ok(o.snapshot())
    })
}

#[tauri::command]
pub fn start_service(state: State<AppState>, id: String) -> Result<Snapshot, String> {
    with_state(&state, |o| {
        o.start_service(&id).map_err(|e| e.to_string())?;
        Ok(o.snapshot())
    })
}

#[tauri::command]
pub fn stop_service(state: State<AppState>, id: String) -> Result<Snapshot, String> {
    with_state(&state, |o| {
        o.stop_service(&id).map_err(|e| e.to_string())?;
        Ok(o.snapshot())
    })
}

#[tauri::command]
pub fn switch_php(state: State<AppState>, version: String) -> Result<Snapshot, String> {
    with_state(&state, |o| {
        o.switch_php(&version).map_err(|e| e.to_string())?;
        Ok(o.snapshot())
    })
}

#[tauri::command]
pub fn create_project(state: State<AppState>, name: String) -> Result<ProjectInfo, String> {
    with_state(&state, |o| o.create_project(&name).map_err(|e| e.to_string()))
}

#[tauri::command]
pub fn get_config(state: State<AppState>) -> Result<LaxConfig, String> {
    with_state(&state, |o| Ok(o.config.clone()))
}

#[tauri::command]
pub fn save_config(state: State<AppState>, config: LaxConfig) -> Result<Snapshot, String> {
    with_state(&state, |o| {
        o.save(config).map_err(|e| e.to_string())?;
        Ok(o.snapshot())
    })
}

#[tauri::command]
pub fn read_logs(state: State<AppState>, which: String) -> Result<String, String> {
    with_state(&state, |o| {
        logs::read_log(&o.paths, &o.config, &which, 250).map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn list_php_extensions(state: State<AppState>) -> Result<Vec<php::PhpExtension>, String> {
    with_state(&state, |o| {
        php::list_extensions(&o.paths, &o.config).map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn set_php_extension(
    state: State<AppState>,
    name: String,
    enabled: bool,
) -> Result<(), String> {
    with_state(&state, |o| {
        o.set_php_extension(&name, enabled)
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn open_ini(state: State<AppState>, which: String) -> Result<(), String> {
    with_state(&state, |o| {
        let path = match which.as_str() {
            "php" => php::php_ini_path(&o.paths, &o.config),
            "mysql" | "mariadb" => o.paths.mysql_dir(&o.config).join("my.ini"),
            "apache" => o.paths.apache_dir(&o.config).join("conf").join("httpd.conf"),
            "nginx" => o.paths.nginx_dir(&o.config).join("conf").join("nginx.conf"),
            "lax" => o.paths.config_file.clone(),
            _ => return Err(format!("unknown ini: {which}")),
        };
        open_editor(&path)
    })
}

#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    spawn_cmd(&["/C", "start", "", &url])
}

#[tauri::command]
pub fn open_path(path: String) -> Result<(), String> {
    let p = PathBuf::from(&path);
    if !p.exists() {
        return Err(format!("path not found: {path}"));
    }
    Command::new("explorer.exe")
        .arg(&p)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn open_terminal(path: String) -> Result<(), String> {
    let dir = PathBuf::from(&path);
    if !dir.exists() {
        return Err(format!("path not found: {path}"));
    }
    spawn_console(&dir, None, None)
}

#[tauri::command]
pub fn open_vscode(path: String) -> Result<(), String> {
    Command::new("cmd.exe")
        .args(["/C", "code", &path])
        .creation_flags(CREATE_NO_WINDOW)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("VS Code not found: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn run_project_action(
    state: State<AppState>,
    path: String,
    action: String,
) -> Result<(), String> {
    with_state(&state, |o| {
        let dir = resolve_www_dir(&o.paths, &o.config, &path)?;
        let php_dir = o.paths.php_dir(&o.config);
        let composer = o.paths.root.join("bin").join("composer").join("composer.bat");
        let (line, extra_path) = match action.as_str() {
            "npm-install" => {
                if !dir.join("package.json").exists() {
                    return Err("package.json не найден".into());
                }
                ("npm install".to_string(), None)
            }
            "composer-install" => {
                if !dir.join("composer.json").exists() {
                    return Err("composer.json не найден".into());
                }
                let composer_cmd = if composer.exists() {
                    format!("\"{}\" install", composer.display())
                } else {
                    "composer install".into()
                };
                (composer_cmd, Some(php_dir))
            }
            other if other.starts_with("npm-run:") => {
                let script = other.trim_start_matches("npm-run:");
                if !crate::projects::is_safe_script(script) {
                    return Err("недопустимое имя скрипта".into());
                }
                if !dir.join("package.json").exists() {
                    return Err("package.json не найден".into());
                }
                (format!("npm run {script}"), None)
            }
            _ => return Err(format!("unknown action: {action}")),
        };
        spawn_console(&dir, Some(&line), extra_path.as_deref())
    })
}

fn resolve_www_dir(
    paths: &crate::config::Paths,
    cfg: &LaxConfig,
    path: &str,
) -> Result<PathBuf, String> {
    let www = dunce::canonicalize(paths.www(cfg)).map_err(|e| e.to_string())?;
    let dir = dunce::canonicalize(PathBuf::from(path)).map_err(|e| e.to_string())?;
    if !dir.starts_with(&www) || !dir.is_dir() {
        return Err("проект должен быть внутри www".into());
    }
    Ok(dir)
}

fn spawn_cmd(args: &[&str]) -> Result<(), String> {
    Command::new("cmd.exe")
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn spawn_console(dir: &Path, cmdline: Option<&str>, extra_path: Option<&Path>) -> Result<(), String> {
    let mut cmd = Command::new("cmd.exe");
    if let Some(line) = cmdline {
        cmd.args(["/K", line]);
    } else {
        cmd.arg("/K");
    }
    cmd.current_dir(dir).creation_flags(CREATE_NEW_CONSOLE);
    if let Some(bin) = extra_path {
        let rest = std::env::var("PATH").unwrap_or_default();
        cmd.env("PATH", format!("{};{rest}", bin.display()));
    }
    cmd.spawn().map_err(|e| e.to_string())?;
    Ok(())
}

fn open_editor(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("file not found: {}", path.display()));
    }
    Command::new("notepad.exe")
        .arg(path)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}
