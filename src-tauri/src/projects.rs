use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::Serialize;

use crate::config::{LaxConfig, Paths};
use crate::discover;
use crate::error::{LaxError, LaxResult};
use crate::platform;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInfo {
    pub name: String,
    pub path: String,
    pub url: String,
    pub localhost_url: String,
    pub has_public: bool,
    pub kind: String,
    pub scripts: Vec<String>,
    pub has_package: bool,
    pub has_composer: bool,
    pub has_node_modules: bool,
    pub has_vendor: bool,
}

pub fn list_projects(paths: &Paths, cfg: &LaxConfig) -> LaxResult<Vec<ProjectInfo>> {
    let www = paths.www(cfg);
    fs::create_dir_all(&www)?;
    let mut items = Vec::new();
    let rd = fs::read_dir(&www)?;
    for ent in rd.flatten() {
        let path = ent.path();
        if !path.is_dir() {
            continue;
        }
        let name = ent.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') || is_stack_dir(&name) {
            continue;
        }
        let has_public = document_root_for(&path) != path;
        let port = if cfg.web_server == "nginx" {
            cfg.nginx_port
        } else {
            cfg.apache_port
        };
        let hostport = if port == 80 {
            String::new()
        } else {
            format!(":{port}")
        };
        let suffix = if has_public { format!("{name}/public/") } else { format!("{name}/") };
        let url = format!("http://localhost{hostport}/{suffix}");
        let info = inspect(&path);
        items.push(ProjectInfo {
            url: url.clone(),
            localhost_url: url,
            name,
            path: path.to_string_lossy().into_owned(),
            has_public,
            kind: info.kind,
            scripts: info.scripts,
            has_package: info.has_package,
            has_composer: info.has_composer,
            has_node_modules: info.has_node_modules,
            has_vendor: info.has_vendor,
        });
    }
    items.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(items)
}

pub fn document_root_for(project: &Path) -> PathBuf {
    let public = project.join("public");
    if public.join("index.php").exists() || public.join("index.html").exists() {
        public
    } else {
        project.to_path_buf()
    }
}

pub fn create_project(paths: &Paths, cfg: &LaxConfig, name: &str) -> LaxResult<ProjectInfo> {
    create_php(paths, cfg, name)
}

pub fn reserve_slug(paths: &Paths, cfg: &LaxConfig, name: &str) -> LaxResult<String> {
    let slug = sanitize(name)?;
    let dir = paths.www(cfg).join(&slug);
    if dir.exists() {
        return Err(LaxError::msg(format!("проект '{slug}' уже есть")));
    }
    Ok(slug)
}

pub fn create_php(paths: &Paths, cfg: &LaxConfig, name: &str) -> LaxResult<ProjectInfo> {
    let slug = reserve_slug(paths, cfg, name)?;
    let dir = paths.www(cfg).join(&slug);
    fs::create_dir_all(&dir)?;
    let index = format!(
        r#"<?php
?><!doctype html>
<html lang="ru">
<head>
  <meta charset="utf-8">
  <title>{slug}</title>
  <style>
    body {{ margin:0; min-height:100vh; display:grid; place-items:center;
      font-family: Segoe UI, sans-serif; background:#07090f; color:#e8eef7; }}
    main {{ text-align:center; }}
    h1 {{ font-size:42px; margin:0 0 8px; }}
    p {{ color:#9aa8bd; }}
    code {{ color:#22d3ee; }}
  </style>
</head>
<body>
  <main>
    <h1>{slug}</h1>
    <p>PHP <?= PHP_VERSION ?> · <code>http://localhost/{slug}/</code></p>
  </main>
</body>
</html>
"#,
        slug = slug
    );
    fs::write(dir.join("index.php"), index)?;
    listed(paths, cfg, &slug)
}

/// Opens a terminal in `www` and runs create-project / npm create.
pub fn start_cli_scaffold(paths: &Paths, cfg: &LaxConfig, name: &str, kind: &str) -> LaxResult<String> {
    let slug = reserve_slug(paths, cfg, name)?;
    let www = paths.www(cfg);
    fs::create_dir_all(&www)?;
    let php_dir = paths.php_dir(cfg);
    let prefix = discover::tools_path_prefix(&paths.root, &php_dir);
    let line = match kind {
        "laravel" => {
            let composer = platform::composer_cmdline(&paths.root, &php_dir);
            format!("{composer} create-project --prefer-dist --no-interaction laravel/laravel {slug}")
        }
        "vite" => {
            if discover::node_bin_dir(&paths.root).is_none() {
                return Err(LaxError::msg(
                    "Node не найден в bin/node. Запусти npm run fetch-tools",
                ));
            }
            format!("npm create --yes vite@latest {slug} -- --template vue")
        }
        other => return Err(LaxError::msg(format!("неизвестный шаблон: {other}"))),
    };
    platform::open_terminal(&www, Some(&line), Some(prefix.as_str()))
        .map_err(LaxError::msg)?;
    let hint = match kind {
        "laravel" => format!("Laravel: в терминале composer create-project → {slug}"),
        "vite" => format!("Vite + Vue: в терминале npm create → {slug}"),
        _ => slug.clone(),
    };
    Ok(hint)
}

pub fn scaffold_wordpress(root: &Path, www: &Path, slug: &str) -> LaxResult<()> {
    let dest = www.join(slug);
    if dest.exists() {
        return Err(LaxError::msg(format!("проект '{slug}' уже есть")));
    }
    fs::create_dir_all(root.join("tmp"))?;
    let zip_path = root.join("tmp").join("wordpress-latest.zip");
    download_wordpress(&zip_path)?;
    fs::create_dir_all(&dest)?;
    if let Err(e) = extract_zip_prefix(&zip_path, &dest, "wordpress/") {
        let _ = fs::remove_dir_all(&dest);
        return Err(e);
    }
    if !dest.join("wp-config-sample.php").exists() && !dest.join("index.php").exists() {
        let _ = fs::remove_dir_all(&dest);
        return Err(LaxError::msg("архив WordPress пустой или неожиданный"));
    }
    Ok(())
}

fn download_wordpress(dest: &Path) -> LaxResult<()> {
    if dest.is_file() {
        if let Ok(meta) = dest.metadata() {
            if meta.len() > 1_000_000 {
                return Ok(());
            }
        }
    }
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(20))
        .timeout(Duration::from_secs(180))
        .user_agent("LaX/wordpress-scaffold")
        .build();
    let resp = agent
        .get("https://wordpress.org/latest.zip")
        .call()
        .map_err(|e| LaxError::msg(format!("скачивание WordPress: {e}")))?;
    let tmp = dest.with_extension("part");
    let mut file = fs::File::create(&tmp)?;
    io::copy(&mut resp.into_reader(), &mut file)?;
    file.flush()?;
    drop(file);
    if dest.exists() {
        let _ = fs::remove_file(dest);
    }
    fs::rename(&tmp, dest)?;
    Ok(())
}

fn extract_zip_prefix(zip_path: &Path, dest: &Path, prefix: &str) -> LaxResult<()> {
    let file = fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| LaxError::msg(e.to_string()))?;
    let dest = dunce::canonicalize(dest).map_err(|e| LaxError::msg(e.to_string()))?;
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| LaxError::msg(e.to_string()))?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().replace('\\', "/");
        let rel = name
            .strip_prefix(prefix)
            .unwrap_or(name.as_str())
            .trim_start_matches('/');
        if rel.is_empty() || rel.contains("..") {
            continue;
        }
        let out = dest.join(rel);
        if !out.starts_with(&dest) {
            continue;
        }
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut target = fs::File::create(&out)?;
        io::copy(&mut entry, &mut target)?;
    }
    Ok(())
}

fn listed(paths: &Paths, cfg: &LaxConfig, slug: &str) -> LaxResult<ProjectInfo> {
    let list = list_projects(paths, cfg)?;
    list.into_iter()
        .find(|p| p.name == slug)
        .ok_or_else(|| LaxError::msg("проект создан, но не попал в список"))
}

fn sanitize(name: &str) -> LaxResult<String> {
    let s = name
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if s.is_empty() {
        return Err(LaxError::msg("empty project name"));
    }
    Ok(s)
}

struct Inspect {
    kind: String,
    scripts: Vec<String>,
    has_package: bool,
    has_composer: bool,
    has_node_modules: bool,
    has_vendor: bool,
}

fn inspect(dir: &Path) -> Inspect {
    let has_package = dir.join("package.json").exists();
    let has_composer = dir.join("composer.json").exists();
    let has_artisan = dir.join("artisan").exists();
    let has_vite_cfg = ["vite.config.ts", "vite.config.js", "vite.config.mts", "vite.config.mjs"]
        .iter()
        .any(|n| dir.join(n).exists());
    let wordpress = dir.join("wp-config.php").exists() || dir.join("wp-config-sample.php").exists();

    let mut scripts = Vec::new();
    let mut has_vite_dep = false;
    if has_package {
        if let Ok(raw) = fs::read_to_string(dir.join("package.json")) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&raw) {
                if let Some(obj) = json.get("scripts").and_then(|s| s.as_object()) {
                    scripts = obj.keys().cloned().collect();
                    scripts.sort();
                }
                has_vite_dep = dep_has(&json, "vite") || dep_has(&json, "@vitejs/plugin-vue");
            }
        }
    }

    let kind = if has_artisan && has_composer {
        "laravel"
    } else if has_vite_cfg || has_vite_dep {
        "vite"
    } else if wordpress {
        "wordpress"
    } else if has_package {
        "node"
    } else {
        "php"
    }
    .to_string();

    Inspect {
        kind,
        scripts,
        has_package,
        has_composer,
        has_node_modules: dir.join("node_modules").is_dir(),
        has_vendor: dir.join("vendor").is_dir(),
    }
}

fn dep_has(json: &serde_json::Value, name: &str) -> bool {
    json.get("dependencies")
        .and_then(|d| d.get(name))
        .is_some()
        || json
            .get("devDependencies")
            .and_then(|d| d.get(name))
            .is_some()
}

pub fn is_safe_script(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ':' || c == '.')
}

fn is_stack_dir(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "api"
            | "assets"
            | "data"
            | "vendor"
            | "node_modules"
            | "storage"
            | "tmp"
            | "cache"
            | "files"
            | "uploads"
            | "cgi-bin"
    )
}
