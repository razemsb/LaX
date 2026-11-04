use std::path::PathBuf;

use tauri::{AppHandle, Manager, State};

use crate::config::LaxConfig;
use crate::db;
use crate::discover;
use crate::logs;
use crate::php;
use crate::platform;
use crate::projects;
use crate::state::{AppState, ServiceInfo, Snapshot, TunnelInfo};

fn with_state<T, F>(state: &AppState, f: F) -> Result<T, String>
where
    F: FnOnce(&mut crate::state::Orchestrator) -> Result<T, String>,
{
    let mut g = state
        .inner
        .lock()
        .map_err(|_| rust_i18n::t!("err.lock_poisoned").to_string())?;
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
pub fn switch_web_port(state: State<AppState>, port: u16) -> Result<Snapshot, String> {
    with_state(&state, |o| {
        o.switch_web_port(port).map_err(|e| e.to_string())?;
        Ok(o.snapshot())
    })
}

#[tauri::command]
pub async fn create_project(
    state: State<'_, AppState>,
    name: String,
    kind: Option<String>,
) -> Result<Snapshot, String> {
    let kind = kind.unwrap_or_else(|| "php".into());
    match kind.as_str() {
        "wordpress" => {
            let (root, www, slug) = with_state(&state, |o| {
                let slug =
                    projects::reserve_slug(&o.paths, &o.config, &name).map_err(|e| e.to_string())?;
                Ok((o.paths.root.clone(), o.paths.www(&o.config), slug))
            })?;
            let slug_msg = slug.clone();
            tokio::task::spawn_blocking(move || projects::scaffold_wordpress(&root, &www, &slug))
                .await
                .map_err(|e| e.to_string())?
                .map_err(|e| e.to_string())?;
            with_state(&state, |o| {
                o.prepare_sites().map_err(|e| e.to_string())?;
                o.last_message = Some(rust_i18n::t!("msg.wordpress_ready", slug = slug_msg).into_owned());
                Ok(o.snapshot())
            })
        }
        "laravel" | "vite" => with_state(&state, |o| {
            let msg = crate::projects::start_cli_scaffold(&o.paths, &o.config, &name, &kind)
                .map_err(|e| e.to_string())?;
            o.last_message = Some(msg);
            Ok(o.snapshot())
        }),
        _ => with_state(&state, |o| {
            o.create_project(&name).map_err(|e| e.to_string())?;
            Ok(o.snapshot())
        }),
    }
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
pub fn set_theme(app: AppHandle, state: State<AppState>, theme: String) -> Result<Snapshot, String> {
    let snap = with_state(&state, |o| {
        o.set_theme(&theme).map_err(|e| e.to_string())?;
        Ok(o.snapshot())
    })?;
    crate::apply_window_theme(&app, &theme);
    Ok(snap)
}

#[tauri::command]
pub fn set_db_admin(state: State<AppState>, id: String) -> Result<Snapshot, String> {
    with_state(&state, |o| {
        o.set_db_admin(&id).map_err(|e| e.to_string())?;
        Ok(o.snapshot())
    })
}

#[tauri::command]
pub fn set_language(app: AppHandle, state: State<AppState>, language: String) -> Result<Snapshot, String> {
    let snap = with_state(&state, |o| {
        o.set_language(&language).map_err(|e| e.to_string())?;
        Ok(o.snapshot())
    })?;
    if let Ok(menu) = crate::tray_menu(&app) {
        if let Some(tray) = app.tray_by_id("tray") {
            let _ = tray.set_menu(Some(menu));
        }
    }
    Ok(snap)
}

#[tauri::command]
pub fn set_window_mode(app: AppHandle, state: State<AppState>, mode: String) -> Result<Snapshot, String> {
    let snap = with_state(&state, |o| {
        o.set_window_mode(&mode).map_err(|e| e.to_string())?;
        Ok(o.snapshot())
    })?;
    if let Some(win) = app.get_webview_window("main") {
        crate::apply_window_mode(&win, &snap.config.window_mode);
    }
    Ok(snap)
}

#[tauri::command]
pub async fn install_dbgate(state: State<'_, AppState>) -> Result<Snapshot, String> {
    let paths = with_state(&state, |o| Ok(o.paths.clone()))?;
    tokio::task::spawn_blocking(move || crate::services::install_dbgate(&paths))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;
    with_state(&state, |o| {
        o.set_db_admin("dbgate").map_err(|e| e.to_string())?;
        let _ = crate::services::start_dbgate(&mut o.procs, &o.paths, &o.config);
        o.last_message = Some(rust_i18n::t!("msg.dbgate_installed").into_owned());
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
pub fn php_quick_settings(state: State<AppState>) -> Result<php::PhpQuickSettings, String> {
    with_state(&state, |o| {
        php::quick_settings(&o.paths, &o.config).map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn set_php_quick_settings(
    state: State<AppState>,
    patch: php::PhpQuickPatch,
) -> Result<php::PhpQuickSettings, String> {
    with_state(&state, |o| o.set_php_quick(patch).map_err(|e| e.to_string()))
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
            _ => return Err(rust_i18n::t!("err.unknown_ini", which = which).to_string()),
        };
        platform::open_editor(&path)
    })
}

#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    platform::open_url(&url)
}

#[tauri::command]
pub fn open_path(path: String) -> Result<(), String> {
    platform::open_path(&PathBuf::from(path))
}

#[tauri::command]
pub fn open_terminal(state: State<AppState>, path: String) -> Result<(), String> {
    with_state(&state, |o| {
        let prefix = discover::tools_path_prefix(&o.paths.root, &o.paths.php_dir(&o.config));
        platform::open_terminal(&PathBuf::from(path), None, Some(prefix.as_str()))
    })
}

#[tauri::command]
pub fn open_vscode(path: String) -> Result<(), String> {
    platform::open_vscode(&PathBuf::from(path))
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
        let prefix = discover::tools_path_prefix(&o.paths.root, &php_dir);
        let composer = platform::composer_file(&o.paths.root);
        let line = match action.as_str() {
            "npm-install" => {
                if !dir.join("package.json").exists() {
                    return Err(rust_i18n::t!("err.package_json").to_string());
                }
                "npm install".to_string()
            }
            "composer-install" => {
                if !dir.join("composer.json").exists() {
                    return Err(rust_i18n::t!("err.composer_json").to_string());
                }
                if composer.exists() {
                    format!("\"{}\" install", composer.display())
                } else {
                    "composer install".into()
                }
            }
            other if other.starts_with("npm-run:") => {
                let script = other.trim_start_matches("npm-run:");
                if !crate::projects::is_safe_script(script) {
                    return Err(rust_i18n::t!("err.bad_script_name").to_string());
                }
                if !dir.join("package.json").exists() {
                    return Err(rust_i18n::t!("err.package_json").to_string());
                }
                format!("npm run {script}")
            }
            other => return Err(rust_i18n::t!("err.unknown_action", action = other).to_string()),
        };
        platform::open_terminal(&dir, Some(&line), Some(prefix.as_str()))
    })
}

#[tauri::command]
pub fn share_project(state: State<AppState>, path: String) -> Result<Snapshot, String> {
    let inner = state.inner.clone();
    with_state(&state, |o| {
        let dir = resolve_www_dir(&o.paths, &o.config, &path)?;
        let projects = projects::list_projects(&o.paths, &o.config).map_err(|e| e.to_string())?;
        let p = projects
            .into_iter()
            .find(|p| dunce::canonicalize(&p.path).map(|c| c == dir).unwrap_or(false))
            .ok_or_else(|| rust_i18n::t!("share.not_found").to_string())?;
        if o.tunnels.contains_key(&p.name) {
            return Err(rust_i18n::t!("share.already").to_string());
        }
        if !crate::tunnel::available(&o.paths) {
            return Err(rust_i18n::t!("share.missing").to_string());
        }
        let web_port = if o.config.web_server == "nginx" {
            o.config.nginx_port
        } else {
            o.config.apache_port
        };
        if !discover::port_open(web_port) {
            return Err(rust_i18n::t!("share.web_off").to_string());
        }
        let port = pick_share_port(&o.config)?;
        crate::vhosts::write_share_vhost(&o.paths, &o.config, &p, port).map_err(|e| e.to_string())?;
        o.start_web().map_err(|e| e.to_string())?;
        let local = format!("http://127.0.0.1:{port}");
        o.tunnels.insert(
            p.name.clone(),
            TunnelInfo {
                path: p.path.clone(),
                url: None,
                port: Some(port),
            },
        );
        crate::tunnel::start(&o.paths, &mut o.procs, inner, p.name.clone(), local)
            .map_err(|e| e.to_string())?;
        o.last_message = Some(rust_i18n::t!("share.starting").into_owned());
        Ok(o.snapshot())
    })
}

#[tauri::command]
pub fn unshare_project(state: State<AppState>, path: String) -> Result<Snapshot, String> {
    with_state(&state, |o| {
        let dir = resolve_www_dir(&o.paths, &o.config, &path)?;
        let projects = projects::list_projects(&o.paths, &o.config).map_err(|e| e.to_string())?;
        let name = projects
            .into_iter()
            .find(|p| dunce::canonicalize(&p.path).map(|c| c == dir).unwrap_or(false))
            .map(|p| p.name);
        if let Some(name) = name {
            crate::tunnel::stop(&mut o.procs, &name);
            o.tunnels.remove(&name);
            crate::vhosts::clear_share_vhost(&o.paths, &name);
            let web_port = if o.config.web_server == "nginx" {
                o.config.nginx_port
            } else {
                o.config.apache_port
            };
            if discover::port_open(web_port) {
                let _ = o.start_web();
            }
        }
        Ok(o.snapshot())
    })
}

fn pick_share_port(cfg: &crate::config::LaxConfig) -> Result<u16, String> {
    for port in 12000..=12999u16 {
        if cfg.php_cgi_ports.contains(&port) {
            continue;
        }
        if !discover::port_open(port) {
            return Ok(port);
        }
    }
    Err(rust_i18n::t!("share.no_port").to_string())
}

#[tauri::command]
pub fn set_project_php(
    state: State<AppState>,
    path: String,
    version: Option<String>,
) -> Result<Snapshot, String> {
    with_state(&state, |o| {
        let dir = resolve_www_dir(&o.paths, &o.config, &path)?;
        let slug = dir
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| rust_i18n::t!("share.not_found").to_string())?;
        projects::set_site_php(&o.paths, &slug, version.as_deref()).map_err(|e| e.to_string())?;
        o.prepare_sites().map_err(|e| e.to_string())?;
        o.last_message = Some(rust_i18n::t!("msg.site_php_set").into_owned());
        Ok(o.snapshot())
    })
}

#[tauri::command]
pub fn list_databases(state: State<AppState>) -> Result<Vec<String>, String> {
    with_state(&state, |o| {
        db::list_databases(&o.paths, &o.config).map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn create_database(state: State<AppState>, name: String) -> Result<Vec<String>, String> {
    with_state(&state, |o| {
        db::create_database(&o.paths, &o.config, &name).map_err(|e| e.to_string())?;
        db::list_databases(&o.paths, &o.config).map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn import_sql(state: State<AppState>, db_name: String, sql: String) -> Result<(), String> {
    with_state(&state, |o| {
        crate::db::import_sql(&o.paths, &o.config, &db_name, &sql).map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn check_update(state: State<AppState>) -> Result<Snapshot, String> {
    let info = crate::update::fetch_latest().map_err(|e| e.to_string())?;
    with_state(&state, |o| {
        if crate::update::is_newer(&info.version, crate::update::APP_VERSION) {
            o.update = Some(info);
            o.last_message = None;
        } else {
            o.update = None;
            o.last_message = Some(
                rust_i18n::t!("msg.latest_version", version = crate::update::APP_VERSION).into_owned(),
            );
        }
        Ok(o.snapshot())
    })
}

#[tauri::command]
pub async fn apply_update(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let (root, info) = with_state(&state, |o| {
        let info = o
            .update
            .clone()
            .ok_or_else(|| rust_i18n::t!("msg.no_update").to_string())?;
        o.last_message = Some(rust_i18n::t!("msg.downloading_update").into_owned());
        Ok((o.paths.root.clone(), info))
    })?;

    crate::update::emit_progress(&app, &rust_i18n::t!("update.downloading"));
    let app_dl = app.clone();
    let root_dl = root.clone();
    let info_dl = info.clone();
    let dest = tokio::task::spawn_blocking(move || {
        crate::update::download_asset(&root_dl, &info_dl, |m| {
            crate::update::emit_progress(&app_dl, m);
        })
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    if let Some(sum_url) = info.checksum_url.clone() {
        crate::update::emit_progress(&app, &rust_i18n::t!("update.checksum"));
        let expected = crate::update::fetch_checksum(&sum_url).map_err(|e| e.to_string())?;
        crate::update::verify_sha256(&dest, &expected).map_err(|e| e.to_string())?;
    }

    crate::update::emit_progress(&app, &rust_i18n::t!("update.stopping"));
    with_state(&state, |o| {
        o.stop_all();
        Ok(())
    })?;

    crate::update::emit_progress(&app, &rust_i18n::t!("update.installing"));
    let app_ins = app.clone();
    let root_ins = root.clone();
    tokio::task::spawn_blocking(move || {
        crate::update::install_asset(&root_ins, &dest, |m| {
            crate::update::emit_progress(&app_ins, m);
        })
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    if crate::update::has_staged_exe(&root) {
        crate::update::emit_progress(&app, &rust_i18n::t!("update.restarting"));
        crate::update::spawn_relaunch(&root).map_err(|e| e.to_string())?;
        std::thread::sleep(std::time::Duration::from_millis(400));
        std::process::exit(0);
    }
    with_state(&state, |o| {
        o.update = None;
        o.last_message = Some(rust_i18n::t!("msg.files_updated").into_owned());
        Ok(())
    })
}

#[tauri::command]
pub fn dismiss_update(state: State<AppState>) -> Result<Snapshot, String> {
    dismiss_notice(state, "update".into())
}

#[tauri::command]
pub fn quit_app(app: AppHandle, state: State<AppState>) {
    if let Ok(mut g) = state.inner.lock() {
        g.stop_all();
    }
    app.exit(0);
}

#[tauri::command]
pub fn dismiss_notice(state: State<AppState>, which: String) -> Result<Snapshot, String> {
    with_state(&state, |o| {
        match which.as_str() {
            "update" => o.update = None,
            "message" => o.last_message = None,
            "port" => o.port_conflict = None,
            _ => {
                o.update = None;
                o.last_message = None;
            }
        }
        Ok(o.snapshot())
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
        return Err(rust_i18n::t!("err.project_outside_www").to_string());
    }
    Ok(dir)
}
