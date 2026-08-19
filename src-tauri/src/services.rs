use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

use crate::config::{LaxConfig, Paths};
use crate::discover::port_open;
use crate::error::{LaxError, LaxResult};
use crate::paths::unix;
use crate::php;
use crate::process::{run_capture, run_capture_env, taskkill_image, unix_lib_env, write_file, ProcessTable};

pub const MAILPIT_SMTP: u16 = 1025;
pub const MAILPIT_UI: u16 = 8025;
pub const DBGATE_UI: u16 = 8030;

pub fn mailpit_bin(paths: &Paths) -> Option<std::path::PathBuf> {
    let cands = [
        crate::platform::bin_path(&paths.root.join("bin").join("mailpit"), "mailpit"),
        crate::platform::bin_path(&paths.root.join("bin"), "mailpit"),
    ];
    cands.into_iter().find(|p| p.exists())
}

pub fn start_apache(table: &mut ProcessTable, paths: &Paths, cfg: &LaxConfig) -> LaxResult<u32> {
    php::apply_php(paths, cfg)?;
    let dir = paths.apache_dir(cfg);
    let httpd = crate::platform::bin_path(&dir.join("bin"), "httpd");
    if !httpd.exists() {
        #[cfg(unix)]
        {
            return Err(LaxError::msg(
                "Apache на Linux не входит в портативный стек (нет httpd). Включи Nginx — он уже в bin/nginx. Скрипт: scripts/fetch-linux-stack.sh",
            ));
        }
        #[cfg(not(unix))]
        {
            return Err(LaxError::msg(format!("binary not found: {}", httpd.display())));
        }
    }
    let (code, out) = run_capture(&httpd, &["-t"], &dir)?;
    if code != 0 {
        return Err(LaxError::msg(format!("Apache config test failed:\n{out}")));
    }
    if port_open(cfg.apache_port) {
        return Err(LaxError::msg(format!(
            "port {} is already in use — stop Laragon or the other web server first",
            cfg.apache_port
        )));
    }
    table.spawn("apache", &httpd, &[], &dir, &[])
}

pub fn stop_apache(table: &mut ProcessTable) {
    table.stop("apache");
    taskkill_image(&crate::platform::bin("httpd"));
}

pub fn start_nginx(table: &mut ProcessTable, paths: &Paths, cfg: &LaxConfig) -> LaxResult<u32> {
    php::apply_php(paths, cfg)?;
    let dir = paths.nginx_dir(cfg);
    let nginx = crate::platform::bin_path(&dir, "nginx");
    if !nginx.exists() {
        return Err(LaxError::msg(format!(
            "nginx не найден: {}. На Linux: bash scripts/fetch-linux-stack.sh",
            nginx.display()
        )));
    }
    if port_open(cfg.nginx_port) {
        return Err(LaxError::msg(format!(
            "port {} is already in use — stop Laragon or Apache first",
            cfg.nginx_port
        )));
    }
    #[cfg(unix)]
    {
        write_nginx_prefix(paths, cfg)?;
        let prefix = unix(&dir);
        table.spawn(
            "nginx",
            &nginx,
            &["-p", &prefix, "-c", "conf/nginx.conf"],
            &dir,
            &[],
        )
    }
    #[cfg(not(unix))]
    {
        table.spawn("nginx", &nginx, &[], &dir, &[])
    }
}

pub fn stop_nginx(table: &mut ProcessTable, paths: &Paths, cfg: &LaxConfig) {
    let dir = paths.nginx_dir(cfg);
    let nginx = crate::platform::bin_path(&dir, "nginx");
    if nginx.exists() {
        let prefix = unix(&dir);
        let args: Vec<&str> = if cfg!(unix) {
            vec!["-p", &prefix, "-c", "conf/nginx.conf", "-s", "stop"]
        } else {
            vec!["-s", "stop"]
        };
        let _ = run_capture(&nginx, &args, &dir);
    }
    table.stop("nginx");
    taskkill_image(&crate::platform::bin("nginx"));
}

pub fn start_mariadb(table: &mut ProcessTable, paths: &Paths, cfg: &LaxConfig) -> LaxResult<u32> {
    ensure_datadir(paths, cfg)?;
    let dir = paths.mysql_dir(cfg);
    let mysqld = mysql_server_bin(&dir);
    if !mysqld.exists() {
        return Err(LaxError::msg(format!(
            "MariaDB не найден: {}. На Linux: bash scripts/fetch-linux-stack.sh",
            mysqld.display()
        )));
    }
    let ini = if dir.join("my.cnf").is_file() {
        dir.join("my.cnf")
    } else {
        dir.join("my.ini")
    };
    if port_open(cfg.mysql_port) {
        return Err(LaxError::msg(format!(
            "port {} is already in use — stop the other MySQL/MariaDB instance",
            cfg.mysql_port
        )));
    }
    let defaults = format!("--defaults-file={}", unix(&ini));
    let basedir = format!("--basedir={}", unix(&dir));
    let arg_store = if cfg!(unix) {
        vec![defaults, basedir]
    } else {
        vec![defaults]
    };
    let arg_refs: Vec<&str> = arg_store.iter().map(|s| s.as_str()).collect();
    let lib_env = unix_lib_env(&[&dir.join("lib"), &dir.join("lib/private")]);
    let env_refs: Vec<(&str, String)> = lib_env
        .iter()
        .map(|(k, v)| (k.as_str(), v.clone()))
        .collect();
    let pid = table.spawn(
        "mariadb",
        &mysqld,
        &arg_refs,
        &dir.join("bin"),
        &env_refs,
    )?;
    wait_port(cfg.mysql_port, 80)?;
    Ok(pid)
}

pub fn stop_mariadb(table: &mut ProcessTable) {
    table.stop("mariadb");
    taskkill_image(&crate::platform::bin("mysqld"));
    taskkill_image(&crate::platform::bin("mariadbd"));
}

pub fn start_php_cgi(table: &mut ProcessTable, paths: &Paths, cfg: &LaxConfig) -> LaxResult<()> {
    php::apply_php(paths, cfg)?;
    let php_dir = paths.php_dir(cfg);
    let cgi = crate::platform::bin_path(&php_dir, "php-cgi");
    let fpm = crate::platform::bin_path(&php_dir, "php-fpm");
    let phprc = unix(&php_dir);
    table.stop_prefix("php-cgi");

    if cgi.exists() {
        let binds: Vec<(u16, String)> = cfg
            .php_cgi_ports
            .iter()
            .map(|port| (*port, format!("127.0.0.1:{port}")))
            .collect();
        for (port, bind) in &binds {
            table.spawn(
                &format!("php-cgi-{port}"),
                &cgi,
                &["-b", bind],
                &php_dir,
                &[("PHPRC", phprc.clone())],
            )?;
        }
        return Ok(());
    }

    if fpm.exists() {
        let port = cfg.php_cgi_ports.first().copied().unwrap_or(9003);
        let conf = write_php_fpm_conf(paths, cfg, port)?;
        let conf_s = unix(&conf);
        table.spawn(
            &format!("php-cgi-{port}"),
            &fpm,
            &["-F", "-y", &conf_s],
            &php_dir,
            &[("PHPRC", phprc)],
        )?;
        return Ok(());
    }

    Err(LaxError::msg(format!(
        "PHP CGI/FPM не найден в {}. На Linux: bash scripts/fetch-linux-stack.sh",
        php_dir.display()
    )))
}

pub fn stop_php_cgi(table: &mut ProcessTable) {
    table.stop_prefix("php-cgi");
}

pub fn start_mailpit(table: &mut ProcessTable, paths: &Paths) -> LaxResult<()> {
    let Some(bin) = mailpit_bin(paths) else {
        tracing::warn!("mailpit not found in bin/mailpit — skip");
        return Ok(());
    };
    if port_open(MAILPIT_UI) || port_open(MAILPIT_SMTP) {
        return Ok(());
    }
    fs::create_dir_all(paths.root.join("data"))?;
    fs::create_dir_all(paths.root.join("logs"))?;
    let db = paths.root.join("data").join("mailpit.db");
    let log = paths.root.join("logs").join("mailpit.log");
    let db_s = db.to_string_lossy().into_owned();
    let log_s = log.to_string_lossy().into_owned();
    let args = [
        "--smtp".to_string(),
        "0.0.0.0:1025".to_string(),
        "--listen".to_string(),
        "0.0.0.0:8025".to_string(),
        "--smtp-auth-accept-any".to_string(),
        "--database".to_string(),
        db_s,
        "--log-file".to_string(),
        log_s,
    ];
    let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    table.spawn("mailpit", &bin, &refs, &paths.root, &[])?;
    Ok(())
}

pub fn stop_mailpit(table: &mut ProcessTable) {
    table.stop("mailpit");
    taskkill_image(&crate::platform::bin("mailpit"));
}

pub fn dbgate_script(paths: &Paths) -> Option<std::path::PathBuf> {
    let p = paths
        .root
        .join("usr/apps/dbgate/node_modules/dbgate-serve/bin/dbgate-serve.js");
    p.is_file().then_some(p)
}

fn npm_cli(node_dir: &Path) -> Option<PathBuf> {
    let parent = node_dir.parent().unwrap_or(node_dir);
    [
        node_dir.join("node_modules/npm/bin/npm-cli.js"),
        node_dir.join("lib/node_modules/npm/bin/npm-cli.js"),
        parent.join("lib/node_modules/npm/bin/npm-cli.js"),
        parent.join("node_modules/npm/bin/npm-cli.js"),
    ]
    .into_iter()
    .find(|p| p.is_file())
}

const DBGATE_PACKAGE_JSON: &str = r#"{
  "name": "lax-dbgate",
  "private": true,
  "dependencies": {
    "dbgate-serve": "7.2.5"
  }
}
"#;

/// `npm install` dbgate-serve into usr/apps/dbgate. Not shipped in the zip (~350 MB).
pub fn install_dbgate(paths: &Paths) -> LaxResult<()> {
    if dbgate_script(paths).is_some() {
        return Ok(());
    }
    let Some(node_dir) = crate::discover::node_bin_dir(&paths.root) else {
        return Err(LaxError::msg(
            "Node не найден в bin/node. Без него DbGate не ставится.",
        ));
    };
    let node = crate::platform::bin_path(&node_dir, "node");
    if !node.exists() {
        return Err(LaxError::msg("node не найден рядом с bin/node"));
    }
    let Some(npm) = npm_cli(&node_dir) else {
        return Err(LaxError::msg("npm не найден рядом с портативным Node"));
    };
    let dir = paths.root.join("usr/apps/dbgate");
    fs::create_dir_all(&dir)?;
    let pkg = dir.join("package.json");
    if !pkg.is_file() {
        fs::write(&pkg, DBGATE_PACKAGE_JSON)?;
    }
    let cache = paths.root.join("tmp").join("npm-cache");
    fs::create_dir_all(&cache)?;
    let mut path = node_dir.to_string_lossy().into_owned();
    if let Ok(rest) = std::env::var("PATH") {
        path.push(crate::platform::path_sep());
        path.push_str(&rest);
    }
    let mut cmd = Command::new(&node);
    cmd.arg(&npm)
        .args(["install", "--omit=dev", "--no-fund", "--no-audit"])
        .current_dir(&dir)
        .env("PATH", &path)
        .env("npm_config_cache", &cache)
        .env("npm_config_update_notifier", "false");
    crate::platform::hide_window(&mut cmd);
    let out = cmd
        .output()
        .map_err(|e| LaxError::msg(format!("не удалось запустить npm: {e}")))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        let err = err.trim();
        let err = if err.len() > 800 { &err[err.len() - 800..] } else { err };
        return Err(LaxError::msg(if err.is_empty() {
            "npm install dbgate-serve не удался".into()
        } else {
            format!("npm install DbGate:\n{err}")
        }));
    }
    if dbgate_script(paths).is_none() {
        return Err(LaxError::msg(
            "DbGate не появился после npm install. Проверь сеть и логи.",
        ));
    }
    Ok(())
}

pub fn start_dbgate(table: &mut ProcessTable, paths: &Paths, cfg: &LaxConfig) -> LaxResult<()> {
    let Some(script) = dbgate_script(paths) else {
        tracing::warn!("DbGate not found in usr/apps/dbgate — skip");
        return Ok(());
    };
    if port_open(DBGATE_UI) {
        return Ok(());
    }
    let Some(node_dir) = crate::discover::node_bin_dir(&paths.root) else {
        tracing::warn!("Node not found — cannot start DbGate");
        return Ok(());
    };
    let node = crate::platform::bin_path(&node_dir, "node");
    if !node.exists() {
        return Ok(());
    }
    fs::create_dir_all(paths.root.join("data/dbgate"))?;
    fs::create_dir_all(paths.root.join("logs"))?;
    let home = paths.root.join("data/dbgate");
    let cwd = paths.root.join("usr/apps/dbgate");
    let log = paths.root.join("logs").join("dbgate.log");
    let script_s = script.to_string_lossy().into_owned();
    let port = cfg.mysql_port.to_string();
    let home_s = home.to_string_lossy().into_owned();
    let mut path = node_dir.to_string_lossy().into_owned();
    if let Ok(rest) = std::env::var("PATH") {
        path.push(crate::platform::path_sep());
        path.push_str(&rest);
    }
    let env = [
        ("PORT", DBGATE_UI.to_string()),
        ("SKIP_ALL_AUTH", "1".into()),
        ("LANGUAGE", "auto".into()),
        ("CONNECTIONS", "mariadb".into()),
        ("LABEL_mariadb", "MariaDB".into()),
        ("SERVER_mariadb", "127.0.0.1".into()),
        ("USER_mariadb", "root".into()),
        ("PASSWORD_mariadb", String::new()),
        ("PORT_mariadb", port),
        ("ENGINE_mariadb", "mariadb@dbgate-plugin-mysql".into()),
        ("HOME", home_s.clone()),
        ("USERPROFILE", home_s),
        ("PATH", path),
    ];
    let refs: Vec<(&str, String)> = env.into_iter().collect();
    table.spawn_logged(
        "dbgate",
        &node,
        &[script_s.as_str()],
        &cwd,
        &refs,
        &log,
    )?;
    Ok(())
}

pub fn stop_dbgate(table: &mut ProcessTable) {
    table.stop("dbgate");
}

fn ensure_datadir(paths: &Paths, cfg: &LaxConfig) -> LaxResult<()> {
    let data = paths.datadir();
    std::fs::create_dir_all(&data)?;
    if data.join("ibdata1").exists() || data.join("mysql").exists() {
        return Ok(());
    }
    let dir = paths.mysql_dir(cfg);
    let bin = dir.join("bin");
    let datadir = format!("--datadir={}", unix(&data));
    let basedir = format!("--basedir={}", unix(&dir));
    let lib_env = unix_lib_env(&[&dir.join("lib"), &dir.join("lib/private")]);
    let env_refs: Vec<(&str, String)> = lib_env
        .iter()
        .map(|(k, v)| (k.as_str(), v.clone()))
        .collect();
    for name in ["mariadb-install-db", "mysql_install_db"] {
        let exe = crate::platform::bin_path(&bin, name);
        let script = dir.join("scripts").join(name);
        let installer = if exe.exists() {
            Some(exe)
        } else if script.exists() {
            Some(script)
        } else {
            None
        };
        if let Some(exe) = installer {
            let args: Vec<&str> = if cfg!(unix) {
                vec![datadir.as_str(), basedir.as_str(), "--auth-root-authentication-method=normal"]
            } else {
                vec![datadir.as_str(), "--password="]
            };
            let (code, out) = run_capture_env(&exe, &args, &bin, &env_refs)?;
            if code == 0 || data.join("mysql").exists() {
                return Ok(());
            }
            tracing::warn!("install_db {name} exit {code}: {out}");
        }
    }
    let mysqld = mysql_server_bin(&dir);
    let init_args: Vec<&str> = if cfg!(unix) {
        vec![datadir.as_str(), basedir.as_str(), "--initialize-insecure"]
    } else {
        vec![datadir.as_str(), "--initialize-insecure"]
    };
    let (code, out) = run_capture_env(&mysqld, &init_args, &bin, &env_refs)?;
    if code == 0 || data.join("mysql").exists() || data.join("ibdata1").exists() {
        return Ok(());
    }
    Err(LaxError::msg(format!(
        "failed to initialize MariaDB datadir:\n{out}"
    )))
}

fn mysql_server_bin(dir: &std::path::Path) -> std::path::PathBuf {
    let bin = dir.join("bin");
    let mariadbd = crate::platform::bin_path(&bin, "mariadbd");
    if mariadbd.exists() {
        mariadbd
    } else {
        crate::platform::bin_path(&bin, "mysqld")
    }
}

fn write_php_fpm_conf(paths: &Paths, cfg: &LaxConfig, port: u16) -> LaxResult<std::path::PathBuf> {
    let root = unix(&paths.root);
    let php_dir = unix(&paths.php_dir(cfg));
    let path = paths.root.join("tmp").join("php-fpm.conf");
    let body = format!(
        r#"[global]
pid = {root}/tmp/php-fpm.pid
error_log = {root}/logs/php-fpm.log
daemonize = no

[www]
listen = 127.0.0.1:{port}
listen.allowed_clients = 127.0.0.1
pm = dynamic
pm.max_children = 8
pm.start_servers = 2
pm.min_spare_servers = 1
pm.max_spare_servers = 3
chdir = /
php_admin_value[error_log] = {root}/logs/php-fpm-www.log
php_admin_flag[log_errors] = on
env[PHPRC] = {php_dir}
"#
    );
    write_file(&path, &body)?;
    Ok(path)
}

#[cfg(unix)]
fn write_nginx_prefix(paths: &Paths, cfg: &LaxConfig) -> LaxResult<()> {
    let dir = paths.nginx_dir(cfg);
    fs::create_dir_all(dir.join("logs"))?;
    fs::create_dir_all(dir.join("html"))?;
    fs::create_dir_all(dir.join("conf"))?;
    let root = unix(&paths.root);
    let body = format!(
        r#"worker_processes 1;
error_log logs/error.log warn;
pid logs/nginx.pid;
events {{
    worker_connections 1024;
}}
http {{
    include {root}/etc/nginx/mime.types;
    default_type application/octet-stream;
    sendfile on;
    keepalive_timeout 65;
    client_max_body_size 2000M;
    access_log logs/access.log;
    include {root}/etc/nginx/php_upstream.conf;
    include {root}/etc/nginx/sites-enabled/*.conf;
}}
"#
    );
    write_file(&dir.join("conf/nginx.conf"), &body)?;
    Ok(())
}

fn wait_port(port: u16, attempts: u32) -> LaxResult<()> {
    for _ in 0..attempts {
        if port_open(port) {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(150));
    }
    Err(LaxError::msg(format!(
        "service did not open port {port} in time"
    )))
}
