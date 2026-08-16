use std::thread;
use std::time::Duration;

use crate::config::{LaxConfig, Paths};
use crate::discover::port_open;
use crate::error::{LaxError, LaxResult};
use crate::paths::unix;
use crate::php;
use crate::process::{run_capture, taskkill_image, ProcessTable};

pub fn start_apache(table: &mut ProcessTable, paths: &Paths, cfg: &LaxConfig) -> LaxResult<u32> {
    php::apply_php(paths, cfg)?;
    let dir = paths.apache_dir(cfg);
    let httpd = crate::platform::bin_path(&dir.join("bin"), "httpd");
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
    if port_open(cfg.nginx_port) {
        return Err(LaxError::msg(format!(
            "port {} is already in use — stop Laragon or Apache first",
            cfg.nginx_port
        )));
    }
    table.spawn("nginx", &nginx, &[], &dir, &[])
}

pub fn stop_nginx(table: &mut ProcessTable, paths: &Paths, cfg: &LaxConfig) {
    let dir = paths.nginx_dir(cfg);
    let nginx = crate::platform::bin_path(&dir, "nginx");
    if nginx.exists() {
        let _ = run_capture(&nginx, &["-s", "stop"], &dir);
    }
    table.stop("nginx");
    taskkill_image(&crate::platform::bin("nginx"));
}

pub fn start_mariadb(table: &mut ProcessTable, paths: &Paths, cfg: &LaxConfig) -> LaxResult<u32> {
    ensure_datadir(paths, cfg)?;
    let dir = paths.mysql_dir(cfg);
    let mysqld = crate::platform::bin_path(&dir.join("bin"), "mysqld");
    let ini = dir.join("my.ini");
    if port_open(cfg.mysql_port) {
        return Err(LaxError::msg(format!(
            "port {} is already in use — stop the other MySQL/MariaDB instance",
            cfg.mysql_port
        )));
    }
    let args = [format!("--defaults-file={}", unix(&ini))];
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let pid = table.spawn("mariadb", &mysqld, &arg_refs, &dir.join("bin"), &[])?;
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
    let phprc = unix(&php_dir);
    table.stop_prefix("php-cgi");
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
    Ok(())
}

pub fn stop_php_cgi(table: &mut ProcessTable) {
    table.stop_prefix("php-cgi");
}

fn ensure_datadir(paths: &Paths, cfg: &LaxConfig) -> LaxResult<()> {
    let data = paths.datadir();
    std::fs::create_dir_all(&data)?;
    if data.join("ibdata1").exists() || data.join("mysql").exists() {
        return Ok(());
    }
    let bin = paths.mysql_dir(cfg).join("bin");
    for name in ["mariadb-install-db", "mysql_install_db"] {
        let exe = crate::platform::bin_path(&bin, name);
        if exe.exists() {
            let datadir = format!("--datadir={}", unix(&data));
            let (code, out) = run_capture(&exe, &[&datadir, "--password="], &bin)?;
            if code == 0 || data.join("mysql").exists() {
                return Ok(());
            }
            tracing::warn!("install_db {name} exit {code}: {out}");
        }
    }
    let mysqld = crate::platform::bin_path(&bin, "mysqld");
    let datadir = format!("--datadir={}", unix(&data));
    let (code, out) = run_capture(
        &mysqld,
        &[&datadir, "--initialize-insecure"],
        &bin,
    )?;
    if code == 0 || data.join("mysql").exists() || data.join("ibdata1").exists() {
        return Ok(());
    }
    Err(LaxError::msg(format!(
        "failed to initialize MariaDB datadir:\n{out}"
    )))
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
