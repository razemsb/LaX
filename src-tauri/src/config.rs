use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{LaxError, LaxResult};
use crate::paths::{detect_root, join_unix};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaxConfig {
    #[serde(alias = "document_root")]
    pub document_root: String,
    #[serde(default = "default_tld")]
    pub tld: String,
    #[serde(default, alias = "auto_vhost")]
    pub auto_vhost: bool,
    #[serde(alias = "web_server")]
    pub web_server: String,
    #[serde(alias = "apache_port")]
    pub apache_port: u16,
    #[serde(alias = "nginx_port")]
    pub nginx_port: u16,
    #[serde(alias = "mysql_port")]
    pub mysql_port: u16,
    #[serde(alias = "php_version")]
    pub php_version: String,
    #[serde(alias = "mysql_version")]
    pub mysql_version: String,
    #[serde(alias = "nginx_version")]
    pub nginx_version: String,
    #[serde(alias = "apache_version")]
    pub apache_version: String,
    #[serde(alias = "php_cgi_ports")]
    pub php_cgi_ports: Vec<u16>,
    #[serde(alias = "auto_start")]
    pub auto_start: bool,
    #[serde(default = "default_mysql_enabled", alias = "mysql_enabled")]
    pub mysql_enabled: bool,
}

fn default_mysql_enabled() -> bool {
    true
}

fn default_tld() -> String {
    "localhost".into()
}

impl Default for LaxConfig {
    fn default() -> Self {
        Self {
            document_root: "www".into(),
            tld: "localhost".into(),
            auto_vhost: false,
            web_server: "apache".into(),
            apache_port: 80,
            nginx_port: 80,
            mysql_port: 3306,
            php_version: "php-trash-8.2".into(),
            mysql_version: "mariadb-10.11.13".into(),
            nginx_version: "nginx-1.14.0".into(),
            apache_version: "Apache24".into(),
            php_cgi_ports: vec![9003, 9004],
            auto_start: false,
            mysql_enabled: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Paths {
    pub root: PathBuf,
    pub config_file: PathBuf,
}

impl Paths {
    pub fn new(root: PathBuf) -> Self {
        let config_file = root.join("usr").join("lax.toml");
        Self { root, config_file }
    }

    pub fn detect() -> Self {
        Self::new(detect_root())
    }

    pub fn www(&self, cfg: &LaxConfig) -> PathBuf {
        join_unix(&self.root, &cfg.document_root)
    }

    pub fn php_dir(&self, cfg: &LaxConfig) -> PathBuf {
        self.root.join("bin").join("php").join(&cfg.php_version)
    }

    pub fn apache_dir(&self, cfg: &LaxConfig) -> PathBuf {
        self.root.join("bin").join("apache").join(&cfg.apache_version)
    }

    pub fn nginx_dir(&self, cfg: &LaxConfig) -> PathBuf {
        self.root.join("bin").join("nginx").join(&cfg.nginx_version)
    }

    pub fn mysql_dir(&self, cfg: &LaxConfig) -> PathBuf {
        self.root.join("bin").join("mysql").join(&cfg.mysql_version)
    }

    pub fn datadir(&self) -> PathBuf {
        self.root.join("data").join("mariadb")
    }

    pub fn tmp(&self) -> PathBuf {
        self.root.join("tmp")
    }

    pub fn tpl(&self, name: &str) -> PathBuf {
        self.root.join("usr").join("tpl").join(name)
    }
}

pub fn load_config(paths: &Paths) -> LaxResult<LaxConfig> {
    if !paths.config_file.exists() {
        let cfg = LaxConfig::default();
        save_config(paths, &cfg)?;
        return Ok(cfg);
    }
    let raw = fs::read_to_string(&paths.config_file)?;
    match toml::from_str(&raw) {
        Ok(cfg) => Ok(cfg),
        Err(e) => {
            tracing::error!("lax.toml parse failed: {e}");
            Ok(LaxConfig::default())
        }
    }
}

pub fn save_config(paths: &Paths, cfg: &LaxConfig) -> LaxResult<()> {
    if let Some(parent) = paths.config_file.parent() {
        fs::create_dir_all(parent)?;
    }
    let raw = toml::to_string_pretty(cfg).map_err(|e| LaxError::msg(e.to_string()))?;
    fs::write(&paths.config_file, raw)?;
    Ok(())
}

pub fn ensure_runtime_dirs(paths: &Paths, cfg: &LaxConfig) -> LaxResult<()> {
    for p in [
        paths.www(cfg),
        paths.tmp(),
        paths.datadir(),
        paths.root.join("etc/apache2/sites-enabled"),
        paths.root.join("etc/nginx/sites-enabled"),
        paths.root.join("logs"),
        paths.root.join("etc/apps/phpMyAdmin/tmp"),
        paths.apache_dir(cfg).join("logs"),
        paths.nginx_dir(cfg).join("logs"),
    ] {
        fs::create_dir_all(p)?;
    }
    Ok(())
}

pub fn read_tpl(path: &Path) -> LaxResult<String> {
    fs::read_to_string(path).map_err(|e| LaxError::msg(format!("template {}: {e}", path.display())))
}
