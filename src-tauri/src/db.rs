use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::config::{LaxConfig, Paths};
use crate::error::{LaxError, LaxResult};
use crate::platform;
use crate::process::run_capture;

const SKIP: &[&str] = &["information_schema", "performance_schema", "mysql", "sys"];

fn mysql_bin(paths: &Paths, cfg: &LaxConfig) -> PathBuf {
    let dir = paths.mysql_dir(cfg).join("bin");
    let mysql = platform::bin_path(&dir, "mysql");
    if mysql.exists() {
        mysql
    } else {
        platform::bin_path(&dir, "mariadb")
    }
}

fn mysql_args(cfg: &LaxConfig) -> Vec<String> {
    vec![
        "-uroot".into(),
        "--password=".into(),
        "-h".into(),
        "127.0.0.1".into(),
        "-P".into(),
        cfg.mysql_port.to_string(),
    ]
}

pub fn sanitize_db(name: &str) -> LaxResult<String> {
    let s = name.trim();
    if s.is_empty() || s.len() > 64 {
        return Err(LaxError::msg("некорректное имя базы"));
    }
    let first = s.chars().next().unwrap();
    if !(first.is_ascii_alphabetic() || first == '_') {
        return Err(LaxError::msg("имя базы должно начинаться с буквы"));
    }
    if !s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(LaxError::msg("в имени базы только латиница, цифры и _"));
    }
    Ok(s.to_string())
}

pub fn list_databases(paths: &Paths, cfg: &LaxConfig) -> LaxResult<Vec<String>> {
    let bin = mysql_bin(paths, cfg);
    if !bin.exists() {
        return Err(LaxError::msg("mysql-клиент не найден"));
    }
    let mut args = mysql_args(cfg);
    args.extend(["-N".into(), "-e".into(), "SHOW DATABASES".into()]);
    let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let (code, out) = run_capture(&bin, &refs, &paths.root)?;
    if code != 0 {
        return Err(LaxError::msg(format!(
            "не удалось получить список баз (MariaDB запущена?)\n{out}"
        )));
    }
    let mut names: Vec<String> = out
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && !SKIP.contains(&l.as_str()))
        .collect();
    names.sort();
    Ok(names)
}

pub fn create_database(paths: &Paths, cfg: &LaxConfig, name: &str) -> LaxResult<()> {
    let name = sanitize_db(name)?;
    let bin = mysql_bin(paths, cfg);
    let sql = format!("CREATE DATABASE IF NOT EXISTS `{name}` CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci");
    let mut args = mysql_args(cfg);
    args.extend(["-e".into(), sql]);
    let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let (code, out) = run_capture(&bin, &refs, &paths.root)?;
    if code != 0 {
        return Err(LaxError::msg(format!("не удалось создать базу:\n{out}")));
    }
    Ok(())
}

pub fn import_sql(paths: &Paths, cfg: &LaxConfig, db: &str, sql: &str) -> LaxResult<()> {
    let db = sanitize_db(db)?;
    if sql.len() > 40 * 1024 * 1024 {
        return Err(LaxError::msg("дамп больше 40 МБ — импортируй через mysql CLI"));
    }
    let tmp = paths.tmp().join("import.sql");
    fs::create_dir_all(paths.tmp())?;
    fs::write(&tmp, sql)?;
    let bin = mysql_bin(paths, cfg);
    let mut cmd = Command::new(&bin);
    cmd.args(mysql_args(cfg))
        .arg("--default-character-set=utf8mb4")
        .arg(&db)
        .current_dir(&paths.root)
        .stdin(Stdio::from(fs::File::open(&tmp)?))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    platform::hide_window(&mut cmd);
    let out = cmd
        .output()
        .map_err(|e| LaxError::msg(format!("mysql import: {e}")))?;
    let _ = fs::remove_file(&tmp);
    if !out.status.success() {
        let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
        text.push_str(&String::from_utf8_lossy(&out.stderr));
        return Err(LaxError::msg(format!("импорт не удался:\n{text}")));
    }
    Ok(())
}
