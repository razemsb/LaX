use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::config::{LaxConfig, Paths};
use crate::error::{LaxError, LaxResult};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInfo {
    pub name: String,
    pub path: String,
    pub url: String,
    pub localhost_url: String,
    pub has_public: bool,
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
        if name.starts_with('.') {
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
        items.push(ProjectInfo {
            url: url.clone(),
            localhost_url: url,
            name,
            path: path.to_string_lossy().into_owned(),
            has_public,
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
    let slug = sanitize(name)?;
    let dir = paths.www(cfg).join(&slug);
    if dir.exists() {
        return Err(LaxError::msg(format!("project '{slug}' already exists")));
    }
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
    let list = list_projects(paths, cfg)?;
    list.into_iter()
        .find(|p| p.name == slug)
        .ok_or_else(|| LaxError::msg("project created but not listed"))
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
