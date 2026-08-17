use std::collections::BTreeMap;
use std::fs;

use serde::{Deserialize, Serialize};

use crate::config::{read_tpl, LaxConfig, Paths};
use crate::error::{LaxError, LaxResult};
use crate::paths::unix;
use crate::process::write_file;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhpExtension {
    pub name: String,
    pub enabled: bool,
    pub kind: String,
}

pub fn apply_php(paths: &Paths, cfg: &LaxConfig) -> LaxResult<()> {
    let php_dir = unix(&paths.php_dir(cfg));
    let root = unix(&paths.root);
    let tpl = read_tpl(&paths.tpl("fcgid.conf.tpl")).unwrap_or_else(|_| default_fcgid());
    let body = tpl
        .replace("<<ROOT>>", &root)
        .replace("<<PHP_DIR>>", &php_dir);
    write_file(&paths.root.join("etc/apache2/fcgid.conf"), &body)?;

    let mod_php = format!(
        "# PHP is served via FastCGI (fcgid.conf). Active version: {}\n",
        cfg.php_version
    );
    write_file(&paths.root.join("etc/apache2/mod_php.conf"), &mod_php)?;

    let ports = if cfg.php_cgi_ports.is_empty() {
        vec![9003, 9004]
    } else {
        cfg.php_cgi_ports.clone()
    };
    let mut upstream = String::from("upstream php_upstream {\n");
    for p in ports {
        upstream.push_str(&format!(
            "\tserver 127.0.0.1:{p} weight=1 max_fails=1 fail_timeout=1;\n"
        ));
    }
    upstream.push_str("}\n");
    write_file(&paths.root.join("etc/nginx/php_upstream.conf"), &upstream)?;

    patch_httpd_includes(paths, cfg)?;
    patch_mail_ini(paths, cfg)?;
    Ok(())
}

fn patch_mail_ini(paths: &Paths, cfg: &LaxConfig) -> LaxResult<()> {
    let ini = php_ini_path(paths, cfg);
    if !ini.exists() {
        return Ok(());
    }
    let text = fs::read_to_string(&ini)?;
    let mut next = set_ini_value(&text, "SMTP", "127.0.0.1");
    next = set_ini_value(&next, "smtp_port", "1025");
    next = set_ini_value(&next, "sendmail_from", "lax@localhost");
    if next != text {
        fs::write(ini, next)?;
    }
    Ok(())
}

fn set_ini_value(text: &str, key: &str, value: &str) -> String {
    let want = key.to_ascii_lowercase();
    let lines: Vec<&str> = text.lines().collect();
    let mut live = None;
    let mut commented = None;
    for (i, line) in lines.iter().enumerate() {
        if let Some((k, is_comment)) = classify_ini_key(line) {
            if k == want {
                if is_comment {
                    commented.get_or_insert(i);
                } else {
                    live.get_or_insert(i);
                }
            }
        }
    }
    let mut out = String::with_capacity(text.len() + 32);
    if let Some(at) = live.or(commented) {
        for (i, line) in lines.iter().enumerate() {
            if i == at {
                out.push_str(&format!("{key} = {value}\n"));
            } else {
                out.push_str(line);
                out.push('\n');
            }
        }
    } else {
        out.push_str(text);
        if !text.is_empty() && !text.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(&format!("{key} = {value}\n"));
    }
    out
}

fn classify_ini_key(line: &str) -> Option<(String, bool)> {
    let t = line.trim();
    if t.is_empty() || t.starts_with('[') {
        return None;
    }
    let commented = t.starts_with(';');
    let t = t.strip_prefix(';').unwrap_or(t).trim();
    let (k, _) = t.split_once('=')?;
    let k = k.trim();
    if k.is_empty() {
        None
    } else {
        Some((k.to_ascii_lowercase(), commented))
    }
}

fn get_ini_value(text: &str, key: &str) -> Option<String> {
    let want = key.to_ascii_lowercase();
    let mut commented = None;
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('[') {
            continue;
        }
        let is_comment = t.starts_with(';');
        let t = t.strip_prefix(';').unwrap_or(t).trim();
        let Some((k, v)) = t.split_once('=') else {
            continue;
        };
        if k.trim().to_ascii_lowercase() != want {
            continue;
        }
        let val = v
            .split(';')
            .next()
            .unwrap_or(v)
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .to_string();
        if !is_comment {
            return Some(val);
        }
        commented.get_or_insert(val);
    }
    commented
}

fn ini_bool(v: &str) -> bool {
    !matches!(
        v.trim().to_ascii_lowercase().as_str(),
        "0" | "off" | "false" | "no" | ""
    )
}

fn ini_bytes(v: &str) -> u64 {
    let v = v.trim().trim_matches('"').to_ascii_uppercase();
    let (num, mul) = if let Some(n) = v.strip_suffix('G') {
        (n, 1024u64 * 1024 * 1024)
    } else if let Some(n) = v.strip_suffix('M') {
        (n, 1024 * 1024)
    } else if let Some(n) = v.strip_suffix('K') {
        (n, 1024)
    } else {
        (v.as_str(), 1)
    };
    num.trim().parse::<u64>().unwrap_or(0).saturating_mul(mul)
}

fn valid_ini_size(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() || s.len() > 8 {
        return false;
    }
    let split = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
    let (n, rest) = s.split_at(split);
    !n.is_empty() && n.len() <= 4 && matches!(rest, "" | "K" | "M" | "G" | "k" | "m" | "g")
}

pub const XDEBUG_IDE_PORT: u16 = 9000;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhpQuickSettings {
    pub display_errors: bool,
    pub memory_limit: String,
    pub upload_max_filesize: String,
    pub post_max_size: String,
    pub xdebug: bool,
    pub xdebug_available: bool,
    pub xdebug_port: u16,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhpQuickPatch {
    pub display_errors: Option<bool>,
    pub memory_limit: Option<String>,
    pub upload_max_filesize: Option<String>,
    pub xdebug: Option<bool>,
}

pub fn quick_settings(paths: &Paths, cfg: &LaxConfig) -> LaxResult<PhpQuickSettings> {
    let ini = php_ini_path(paths, cfg);
    let text = fs::read_to_string(&ini).unwrap_or_default();
    let xdebug = xdebug_enabled(&text);
    Ok(PhpQuickSettings {
        display_errors: get_ini_value(&text, "display_errors")
            .map(|v| ini_bool(&v))
            .unwrap_or(false),
        memory_limit: get_ini_value(&text, "memory_limit").unwrap_or_else(|| "128M".into()),
        upload_max_filesize: get_ini_value(&text, "upload_max_filesize")
            .unwrap_or_else(|| "2M".into()),
        post_max_size: get_ini_value(&text, "post_max_size").unwrap_or_else(|| "8M".into()),
        xdebug,
        xdebug_available: xdebug || xdebug_dll_exists(paths, cfg),
        xdebug_port: get_ini_value(&text, "xdebug.client_port")
            .and_then(|v| v.parse().ok())
            .unwrap_or(XDEBUG_IDE_PORT),
    })
}

pub fn set_quick_settings(paths: &Paths, cfg: &LaxConfig, patch: &PhpQuickPatch) -> LaxResult<()> {
    let ini = php_ini_path(paths, cfg);
    if !ini.exists() {
        return Err(LaxError::msg("php.ini не найден"));
    }
    let mut text = fs::read_to_string(&ini)?;
    if let Some(on) = patch.display_errors {
        text = set_ini_value(&text, "display_errors", if on { "On" } else { "Off" });
    }
    if let Some(limit) = patch.memory_limit.as_deref() {
        if !valid_ini_size(limit) {
            return Err(LaxError::msg("некорректный memory_limit"));
        }
        text = set_ini_value(&text, "memory_limit", limit.trim());
    }
    if let Some(upload) = patch.upload_max_filesize.as_deref() {
        if !valid_ini_size(upload) {
            return Err(LaxError::msg("некорректный upload_max_filesize"));
        }
        let upload = upload.trim();
        text = set_ini_value(&text, "upload_max_filesize", upload);
        let post = get_ini_value(&text, "post_max_size").unwrap_or_else(|| "8M".into());
        if ini_bytes(&post) < ini_bytes(upload) {
            text = set_ini_value(&text, "post_max_size", upload);
        }
    }
    if let Some(on) = patch.xdebug {
        if on && !xdebug_dll_exists(paths, cfg) && !xdebug_enabled(&text) {
            return Err(LaxError::msg(
                "нет php_xdebug в ext/ этой версии PHP — положи dll и включи снова",
            ));
        }
        text = set_xdebug_ext(&text, on);
        if on {
            text = set_ini_value(&text, "xdebug.mode", "debug");
            text = set_ini_value(&text, "xdebug.start_with_request", "yes");
            text = set_ini_value(&text, "xdebug.client_host", "127.0.0.1");
            text = set_ini_value(&text, "xdebug.client_port", &XDEBUG_IDE_PORT.to_string());
        }
    }
    fs::write(ini, text)?;
    Ok(())
}

fn xdebug_enabled(text: &str) -> bool {
    text.lines().any(|line| {
        matches!(
            parse_ext_line(line),
            Some((_, name, true)) if name.eq_ignore_ascii_case("xdebug")
        )
    })
}

fn xdebug_dll_exists(paths: &Paths, cfg: &LaxConfig) -> bool {
    let ext = paths.php_dir(cfg).join("ext");
    let Ok(rd) = fs::read_dir(ext) else {
        return false;
    };
    rd.flatten().any(|ent| {
        ent.file_name()
            .to_string_lossy()
            .to_ascii_lowercase()
            .contains("xdebug")
    })
}

fn set_xdebug_ext(text: &str, enabled: bool) -> String {
    let mut found = false;
    let mut out = String::with_capacity(text.len() + 32);
    for line in text.lines() {
        if let Some((_, n, _)) = parse_ext_line(line) {
            if n.eq_ignore_ascii_case("xdebug") {
                found = true;
                out.push_str(&format_ext_line("zend", "xdebug", enabled));
                out.push('\n');
                continue;
            }
        }
        out.push_str(line);
        out.push('\n');
    }
    if !found {
        out.push_str(&format_ext_line("zend", "xdebug", enabled));
        out.push('\n');
    }
    out
}

fn patch_httpd_includes(paths: &Paths, cfg: &LaxConfig) -> LaxResult<()> {
    let httpd = paths.apache_dir(cfg).join("conf").join("httpd.conf");
    if !httpd.exists() {
        return Ok(());
    }
    let mut body = fs::read_to_string(&httpd)?;
    let root = unix(&paths.root);
    let www = unix(&paths.www(cfg));
    let fcgid = format!("Include \"{root}/etc/apache2/fcgid.conf\"");
    if !body.contains("fcgid.conf") {
        body.push_str("\n");
        body.push_str(&fcgid);
        body.push('\n');
    }
    // Keep document root in sync
    body = replace_directive(&body, "DocumentRoot", &format!("\"{www}\""));
    fs::write(httpd, body)?;
    Ok(())
}

fn replace_directive(src: &str, key: &str, value: &str) -> String {
    let mut out = String::new();
    let mut done = false;
    for line in src.lines() {
        if !done && line.trim_start().starts_with(key) {
            out.push_str(key);
            out.push(' ');
            out.push_str(value);
            out.push('\n');
            done = true;
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

pub fn php_ini_path(paths: &Paths, cfg: &LaxConfig) -> std::path::PathBuf {
    paths.php_dir(cfg).join("php.ini")
}

pub fn list_extensions(paths: &Paths, cfg: &LaxConfig) -> LaxResult<Vec<PhpExtension>> {
    let ini = php_ini_path(paths, cfg);
    let text = fs::read_to_string(&ini).unwrap_or_default();
    let mut map: BTreeMap<String, PhpExtension> = BTreeMap::new();

    for line in text.lines() {
        if let Some((kind, name, enabled)) = parse_ext_line(line) {
            map.insert(
                format!("{kind}:{name}"),
                PhpExtension {
                    name,
                    enabled,
                    kind,
                },
            );
        }
    }

    let ext_dir = paths.php_dir(cfg).join("ext");
    if let Ok(rd) = fs::read_dir(ext_dir) {
        for ent in rd.flatten() {
            let fname = ent.file_name().to_string_lossy().into_owned();
            let Some(name) = dll_to_name(&fname) else {
                continue;
            };
            map.entry(format!("extension:{name}")).or_insert(PhpExtension {
                name,
                enabled: false,
                kind: "extension".into(),
            });
        }
    }

    Ok(map.into_values().collect())
}

pub fn set_extension(paths: &Paths, cfg: &LaxConfig, name: &str, enabled: bool) -> LaxResult<()> {
    let ini = php_ini_path(paths, cfg);
    if !ini.exists() {
        return Err(LaxError::msg("php.ini not found"));
    }
    let text = fs::read_to_string(&ini)?;
    let target = name.trim().to_ascii_lowercase();
    let mut found = false;
    let mut out = String::with_capacity(text.len() + 32);

    for line in text.lines() {
        if let Some((kind, n, _)) = parse_ext_line(line) {
            if n.eq_ignore_ascii_case(&target) {
                found = true;
                out.push_str(&format_ext_line(&kind, &n, enabled));
                out.push('\n');
                continue;
            }
        }
        out.push_str(line);
        out.push('\n');
    }

    if !found {
        out.push_str(&format_ext_line("extension", &target, enabled));
        out.push('\n');
    }
    fs::write(ini, out)?;
    Ok(())
}

fn parse_ext_line(line: &str) -> Option<(String, String, bool)> {
    let raw = line.trim();
    if raw.is_empty() {
        return None;
    }
    let (enabled, rest) = if let Some(stripped) = raw.strip_prefix(';') {
        let rest = stripped.trim();
        if !(rest.starts_with("extension") || rest.starts_with("zend_extension")) {
            return None;
        }
        // documentation samples look like ";   extension=mysqli"
        if stripped.starts_with(' ') || stripped.starts_with('\t') {
            return None;
        }
        (false, rest)
    } else {
        (true, raw)
    };

    let (kind, value) = if let Some(v) = rest.strip_prefix("zend_extension") {
        ("zend", v)
    } else if let Some(v) = rest.strip_prefix("extension") {
        ("extension", v)
    } else {
        return None;
    };
    let value = value.trim().strip_prefix('=')?.trim();
    if value.is_empty() {
        return None;
    }
    let name = normalize_ext_name(value)?;
    if name == "modulename" {
        return None;
    }
    Some((kind.to_string(), name, enabled))
}

fn normalize_ext_name(value: &str) -> Option<String> {
    let mut v = value.trim().trim_matches('"').trim_matches('\'').to_string();
    if let Some(idx) = v.find(';') {
        v = v[..idx].trim().to_string();
    }
    if v.is_empty() {
        return None;
    }
    let file = v.rsplit(['/', '\\']).next().unwrap_or(&v);
    let mut name = file
        .strip_prefix("php_")
        .unwrap_or(file)
        .to_string();
    for suffix in [".dll", ".so"] {
        if let Some(stripped) = name.strip_suffix(suffix) {
            name = stripped.to_string();
        }
    }
    if name.is_empty() || name.contains(' ') {
        return None;
    }
    Some(name.to_ascii_lowercase())
}

fn dll_to_name(fname: &str) -> Option<String> {
    let lower = fname.to_ascii_lowercase();
    if !(lower.ends_with(".dll") || lower.ends_with(".so")) {
        return None;
    }
    normalize_ext_name(&lower)
}

fn format_ext_line(kind: &str, name: &str, enabled: bool) -> String {
    let prefix = if enabled { "" } else { ";" };
    if kind == "zend" {
        format!("{prefix}zend_extension={name}")
    } else {
        format!("{prefix}extension={name}")
    }
}

fn default_fcgid() -> String {
    r#"LoadModule fcgid_module "<<ROOT>>/etc/apache2/modules/mod_fcgid-2.3.9-win64-VC14.so"
<IfModule fcgid_module>
FcgidInitialEnv PATH "<<PHP_DIR>>;C:/Windows/system32;C:/Windows;"
FcgidInitialEnv PHPRC "<<PHP_DIR>>"
FcgidWrapper "<<PHP_DIR>>/php-cgi.exe" .php
<Files ~ "\.php$">
AddHandler fcgid-script .php
Options +ExecCGI
FcgidWrapper "<<PHP_DIR>>/php-cgi.exe" .php
</Files>
</IfModule>
"#
    .into()
}
