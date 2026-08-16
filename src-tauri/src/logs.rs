use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;

use crate::config::{LaxConfig, Paths};
use crate::error::LaxResult;

pub fn read_log(paths: &Paths, cfg: &LaxConfig, which: &str, max: usize) -> LaxResult<String> {
    let file = match which {
        "apache" => paths.apache_dir(cfg).join("logs").join("error.log"),
        "apache-access" => paths.apache_dir(cfg).join("logs").join("access.log"),
        "nginx" => paths.nginx_dir(cfg).join("logs").join("error.log"),
        "mysql" | "mariadb" => paths.datadir().join("mysql.err"),
        "php" => paths.tmp().join("php_errors.log"),
        "mailpit" => paths.root.join("logs").join("mailpit.log"),
        "lax" => paths.root.join("logs").join("lax.log"),
        _ => paths.apache_dir(cfg).join("logs").join("error.log"),
    };
    tail_file(&file, max)
}

fn tail_file(path: &PathBuf, max: usize) -> LaxResult<String> {
    if !path.exists() {
        return Ok(format!("(no log yet: {})", path.display()));
    }
    let mut f = fs::File::open(path)?;
    let len = f.metadata()?.len();
    let read_from = len.saturating_sub(120_000);
    f.seek(SeekFrom::Start(read_from))?;
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;
    let lines: Vec<&str> = buf.lines().collect();
    let start = lines.len().saturating_sub(max);
    Ok(lines[start..].join("\n"))
}
