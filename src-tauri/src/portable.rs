use std::fs;
use std::path::{Path, PathBuf};

use crate::config::{LaxConfig, Paths};
use crate::error::LaxResult;
use crate::paths::unix;
use crate::process::write_file;

pub fn rebase(paths: &Paths, cfg: &LaxConfig) -> LaxResult<()> {
    let new_root = unix(&paths.root);
    let marker = paths.root.join("usr").join(".install-root");
    let mut olds: Vec<String> = Vec::new();
    if let Ok(prev) = fs::read_to_string(&marker) {
        let prev = prev.trim();
        if !prev.is_empty() {
            olds.push(prev.replace('\\', "/"));
            olds.push(prev.replace('/', "\\"));
        }
    }
    olds.push("C:/Laragon/www/LaX".into());
    olds.push(r"C:\Laragon\www\LaX".into());
    olds.sort_by_key(|s| std::cmp::Reverse(s.len()));
    olds.dedup();

    for file in config_files(paths, cfg) {
        let Ok(raw) = fs::read_to_string(&file) else {
            continue;
        };
        let mut next = raw.clone();
        for old in &olds {
            next = replace_root(&next, old, &new_root);
            next = replace_root(&next, &old.replace('/', "\\"), &new_root.replace('/', "\\"));
        }
        if next != raw {
            fs::write(file, next)?;
        }
    }

    write_mysql_ini(paths, cfg)?;
    if let Some(parent) = marker.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(marker, &new_root)?;
    Ok(())
}

fn write_mysql_ini(paths: &Paths, cfg: &LaxConfig) -> LaxResult<()> {
    let root = unix(&paths.root);
    let body = format!(
        r#"[client]
port={port}
socket=/tmp/mysql.sock

[mysqld]
datadir="{root}/data/mariadb"
port={port}
socket=/tmp/mysql.sock
skip-external-locking
character-set-server=utf8mb4
collation-server=utf8mb4_general_ci
bind-address=127.0.0.1
innodb_buffer_pool_size=256M
max_allowed_packet=512M
key_buffer_size=32M
skip-log-bin
tmpdir="{root}/tmp"

[mysqldump]
quick
max_allowed_packet=512M
"#,
        port = cfg.mysql_port,
        root = root
    );
    write_file(&paths.mysql_dir(cfg).join("my.ini"), &body)
}

fn replace_root(text: &str, old: &str, new: &str) -> String {
    if old.is_empty() || old == new {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find(old) {
        out.push_str(&rest[..i]);
        let slice = &rest[i..];
        if slice.starts_with(new) {
            out.push_str(new);
            rest = &slice[new.len()..];
        } else {
            out.push_str(new);
            rest = &slice[old.len()..];
        }
    }
    out.push_str(rest);
    out
}

fn config_files(paths: &Paths, cfg: &LaxConfig) -> Vec<PathBuf> {
    let mut files = Vec::new();
    push_tree(&mut files, &paths.root.join("etc"), &["conf", "ini"]);
    push_file(&mut files, paths.apache_dir(cfg).join("conf/httpd.conf"));
    push_file(&mut files, paths.nginx_dir(cfg).join("conf/nginx.conf"));
    push_file(&mut files, paths.mysql_dir(cfg).join("my.ini"));
    if let Ok(rd) = fs::read_dir(paths.root.join("bin/php")) {
        for ent in rd.flatten() {
            push_file(&mut files, ent.path().join("php.ini"));
        }
    }
    files
}

fn push_file(files: &mut Vec<PathBuf>, p: PathBuf) {
    if p.is_file() {
        files.push(p);
    }
}

fn push_tree(files: &mut Vec<PathBuf>, dir: &Path, exts: &[&str]) {
    let Ok(rd) = fs::read_dir(dir) else {
        return;
    };
    for ent in rd.flatten() {
        let p = ent.path();
        if p.is_dir() {
            if p.file_name().and_then(|n| n.to_str()) == Some("tmp") {
                continue;
            }
            push_tree(files, &p, exts);
        } else if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
            if exts.iter().any(|x| x.eq_ignore_ascii_case(ext)) {
                files.push(p);
            }
        }
    }
}
