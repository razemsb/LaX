use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use serde::Serialize;

use crate::config::{self, LaxConfig, Paths};
use crate::discover::{self, php_versions, port_open};
use crate::error::{LaxError, LaxResult};
use crate::hosts;
use crate::php;
use crate::process::ProcessTable;
use crate::projects::{self, ProjectInfo};
use crate::services::{self, MAILPIT_SMTP, MAILPIT_UI};
use crate::vhosts;

#[derive(Clone)]
pub struct AppState {
    pub inner: Arc<Mutex<Orchestrator>>,
}

impl AppState {
    pub fn new() -> LaxResult<Self> {
        Ok(Self {
            inner: Arc::new(Mutex::new(Orchestrator::load()?)),
        })
    }

    pub fn fallback() -> Self {
        let paths = Paths::detect();
        let config = LaxConfig::default();
        let _ = config::ensure_runtime_dirs(&paths, &config);
        Self {
            inner: Arc::new(Mutex::new(Orchestrator {
                paths,
                config,
                procs: ProcessTable::default(),
                last_message: Some("Started with default config".into()),
                port_conflict: None,
                update: None,
            })),
        }
    }
}

pub struct Orchestrator {
    pub paths: Paths,
    pub config: LaxConfig,
    pub procs: ProcessTable,
    pub last_message: Option<String>,
    pub port_conflict: Option<PortConflict>,
    pub update: Option<crate::update::UpdateInfo>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceInfo {
    pub id: String,
    pub name: String,
    pub running: bool,
    pub pid: Option<u32>,
    pub port: Option<u16>,
    pub version: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortConflict {
    pub port: u16,
    pub pid: u32,
    pub process: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub root: String,
    pub config: LaxConfig,
    pub services: Vec<ServiceInfo>,
    pub projects: Vec<ProjectInfo>,
    pub php_versions: Vec<String>,
    pub mysql_versions: Vec<String>,
    pub nginx_versions: Vec<String>,
    pub apache_versions: Vec<String>,
    pub hosts_writable: bool,
    pub message: Option<String>,
    pub port_conflict: Option<PortConflict>,
    pub node_available: bool,
    pub mailpit_available: bool,
    pub app_version: String,
    pub repo_url: String,
    pub issues_url: String,
    pub feedback_url: String,
    pub update: Option<crate::update::UpdateInfo>,
}

impl Orchestrator {
    pub fn load() -> LaxResult<Self> {
        let paths = Paths::detect();
        let config = config::load_config(&paths)?;
        config::ensure_runtime_dirs(&paths, &config)?;
        let _ = crate::portable::rebase(&paths, &config);
        Ok(Self {
            paths,
            config,
            procs: ProcessTable::default(),
            last_message: None,
            port_conflict: None,
            update: None,
        })
    }

    pub fn snapshot(&self) -> Snapshot {
        let projects = projects::list_projects(&self.paths, &self.config).unwrap_or_default();
        Snapshot {
            root: self.paths.root.to_string_lossy().into_owned(),
            config: self.config.clone(),
            services: self.services(),
            projects,
            php_versions: php_versions(&self.paths.root),
            mysql_versions: discover::mysql_versions(&self.paths.root),
            nginx_versions: discover::nginx_versions(&self.paths.root),
            apache_versions: discover::apache_versions(&self.paths.root),
            hosts_writable: hosts::writable(),
            message: self.last_message.clone(),
            port_conflict: self.port_conflict.clone(),
            node_available: discover::node_bin_dir(&self.paths.root).is_some(),
            mailpit_available: services::mailpit_bin(&self.paths).is_some(),
            app_version: crate::update::APP_VERSION.to_string(),
            repo_url: crate::update::REPO_URL.to_string(),
            issues_url: crate::update::ISSUES_URL.to_string(),
            feedback_url: crate::update::FEEDBACK_URL.to_string(),
            update: self.update.clone(),
        }
    }

    pub fn status(&self) -> Vec<ServiceInfo> {
        self.services()
    }

    fn services(&self) -> Vec<ServiceInfo> {
        let apache_on = self.config.web_server == "apache";
        let nginx_on = self.config.web_server == "nginx";
        let mailpit = services::mailpit_bin(&self.paths).is_some();
        vec![
            ServiceInfo {
                id: "apache".into(),
                name: "Apache".into(),
                running: port_open(self.config.apache_port) && apache_on,
                pid: self.procs.get("apache").map(|p| p.pid),
                port: Some(self.config.apache_port),
                version: self.config.apache_version.clone(),
                enabled: apache_on,
            },
            ServiceInfo {
                id: "nginx".into(),
                name: "Nginx".into(),
                running: port_open(self.config.nginx_port) && nginx_on,
                pid: self.procs.get("nginx").map(|p| p.pid),
                port: Some(self.config.nginx_port),
                version: self.config.nginx_version.clone(),
                enabled: nginx_on,
            },
            ServiceInfo {
                id: "mariadb".into(),
                name: "MariaDB".into(),
                running: port_open(self.config.mysql_port),
                pid: self.procs.get("mariadb").map(|p| p.pid),
                port: Some(self.config.mysql_port),
                version: self.config.mysql_version.clone(),
                enabled: self.config.mysql_enabled,
            },
            ServiceInfo {
                id: "php".into(),
                name: "PHP".into(),
                running: self.php_running(),
                pid: self.procs.get("php-cgi-9003").map(|p| p.pid),
                port: self.config.php_cgi_ports.first().copied(),
                version: self.config.php_version.clone(),
                enabled: true,
            },
            ServiceInfo {
                id: "mailpit".into(),
                name: "Mailpit".into(),
                running: port_open(MAILPIT_UI) || port_open(MAILPIT_SMTP),
                pid: self.procs.get("mailpit").map(|p| p.pid),
                port: Some(MAILPIT_UI),
                version: "SMTP :1025".into(),
                enabled: mailpit,
            },
        ]
    }

    fn php_running(&self) -> bool {
        if self.config.web_server == "nginx" {
            self.config.php_cgi_ports.iter().any(|p| port_open(*p))
        } else {
            port_open(self.config.apache_port)
        }
    }

    fn web_port(&self) -> u16 {
        if self.config.web_server == "nginx" {
            self.config.nginx_port
        } else {
            self.config.apache_port
        }
    }

    fn remember_conflict(&mut self, port: u16) {
        self.port_conflict = Some(match discover::port_listener(port) {
            Some((pid, process)) => PortConflict {
                port,
                pid,
                process,
            },
            None => PortConflict {
                port,
                pid: 0,
                process: "неизвестно".into(),
            },
        });
    }

    fn fail_port(&mut self, port: u16) -> LaxError {
        self.remember_conflict(port);
        match &self.port_conflict {
            Some(c) if c.pid != 0 => LaxError::msg(format!(
                "порт {port} занят: {} (PID {})",
                c.process, c.pid
            )),
            _ => LaxError::msg(format!(
                "порт {port} не открылся. Смотри логи Apache/Nginx."
            )),
        }
    }

    pub fn prepare_sites(&mut self) -> LaxResult<()> {
        config::ensure_runtime_dirs(&self.paths, &self.config)?;
        crate::portable::rebase(&self.paths, &self.config)?;
        php::apply_php(&self.paths, &self.config)?;
        let projects = projects::list_projects(&self.paths, &self.config)?;
        vhosts::regenerate(&self.paths, &self.config, &projects)?;
        let _ = hosts::sync_hosts(&projects, &self.config.tld, false);
        self.last_message = None;
        Ok(())
    }

    pub fn start_all(&mut self) -> LaxResult<()> {
        self.prepare_sites()?;
        if self.config.mysql_enabled && !port_open(self.config.mysql_port) {
            services::start_mariadb(&mut self.procs, &self.paths, &self.config)?;
        }
        self.start_web()?;
        let _ = services::start_mailpit(&mut self.procs, &self.paths);
        self.port_conflict = None;
        Ok(())
    }

    pub fn stop_all(&mut self) {
        services::stop_nginx(&mut self.procs, &self.paths, &self.config);
        services::stop_apache(&mut self.procs);
        services::stop_php_cgi(&mut self.procs);
        services::stop_mariadb(&mut self.procs);
        services::stop_mailpit(&mut self.procs);
        self.procs.stop_all();
        self.port_conflict = None;
    }

    fn start_web(&mut self) -> LaxResult<()> {
        let port = self.web_port();
        let ours = self.procs.get("apache").is_some() || self.procs.get("nginx").is_some();
        if port_open(port) && !ours {
            return Err(self.fail_port(port));
        }
        if self.config.web_server == "nginx" {
            services::stop_apache(&mut self.procs);
            if !self.config.php_cgi_ports.iter().any(|p| port_open(*p)) {
                services::start_php_cgi(&mut self.procs, &self.paths, &self.config)?;
            }
            if !port_open(self.config.nginx_port) {
                services::start_nginx(&mut self.procs, &self.paths, &self.config)?;
                if wait_port(self.config.nginx_port, 30).is_err() {
                    return Err(self.fail_port(self.config.nginx_port));
                }
            }
        } else {
            services::stop_nginx(&mut self.procs, &self.paths, &self.config);
            services::stop_php_cgi(&mut self.procs);
            if !port_open(self.config.apache_port) {
                services::start_apache(&mut self.procs, &self.paths, &self.config)?;
                if wait_port(self.config.apache_port, 30).is_err() {
                    return Err(self.fail_port(self.config.apache_port));
                }
            }
        }
        Ok(())
    }

    pub fn start_service(&mut self, id: &str) -> LaxResult<()> {
        self.prepare_sites()?;
        match id {
            "apache" => {
                self.config.web_server = "apache".into();
                config::save_config(&self.paths, &self.config)?;
                self.start_web()?;
            }
            "nginx" => {
                self.config.web_server = "nginx".into();
                config::save_config(&self.paths, &self.config)?;
                self.start_web()?;
            }
            "mariadb" | "mysql" => {
                if !port_open(self.config.mysql_port) {
                    services::start_mariadb(&mut self.procs, &self.paths, &self.config)?;
                }
            }
            "php" => {
                if self.config.web_server == "nginx" {
                    services::start_php_cgi(&mut self.procs, &self.paths, &self.config)?;
                } else if !port_open(self.config.apache_port) {
                    self.start_web()?;
                }
            }
            "mailpit" => {
                if services::mailpit_bin(&self.paths).is_none() {
                    return Err(LaxError::msg(
                        "Mailpit не найден. Запусти scripts/fetch-tools.ps1 — бинарник появится в bin/mailpit",
                    ));
                }
                services::start_mailpit(&mut self.procs, &self.paths)?;
            }
            other => return Err(LaxError::msg(format!("unknown service {other}"))),
        }
        Ok(())
    }

    pub fn stop_service(&mut self, id: &str) -> LaxResult<()> {
        match id {
            "apache" => services::stop_apache(&mut self.procs),
            "nginx" => services::stop_nginx(&mut self.procs, &self.paths, &self.config),
            "mariadb" | "mysql" => services::stop_mariadb(&mut self.procs),
            "php" => {
                services::stop_php_cgi(&mut self.procs);
                if self.config.web_server == "apache" {
                    services::stop_apache(&mut self.procs);
                }
            }
            "mailpit" => services::stop_mailpit(&mut self.procs),
            other => return Err(LaxError::msg(format!("unknown service {other}"))),
        }
        Ok(())
    }

    pub fn switch_php(&mut self, version: &str) -> LaxResult<()> {
        if !php_versions(&self.paths.root).iter().any(|v| v == version) {
            return Err(LaxError::msg(format!("PHP version not found: {version}")));
        }
        self.config.php_version = version.to_string();
        config::save_config(&self.paths, &self.config)?;
        php::apply_php(&self.paths, &self.config)?;
        self.reload_web_if_running()
    }

    pub fn set_php_quick(&mut self, patch: php::PhpQuickPatch) -> LaxResult<php::PhpQuickSettings> {
        php::set_quick_settings(&self.paths, &self.config, &patch)?;
        self.reload_web_if_running()?;
        php::quick_settings(&self.paths, &self.config)
    }

    fn reload_web_if_running(&mut self) -> LaxResult<()> {
        let apache_up = port_open(self.config.apache_port) && self.config.web_server == "apache";
        let nginx_up = port_open(self.config.nginx_port) && self.config.web_server == "nginx";
        if apache_up {
            services::stop_apache(&mut self.procs);
            thread::sleep(Duration::from_millis(400));
            self.start_web()?;
        } else if nginx_up {
            services::stop_php_cgi(&mut self.procs);
            services::stop_nginx(&mut self.procs, &self.paths, &self.config);
            thread::sleep(Duration::from_millis(400));
            self.start_web()?;
        }
        Ok(())
    }

    pub fn switch_web_port(&mut self, port: u16) -> LaxResult<()> {
        if port == 0 {
            return Err(LaxError::msg("некорректный порт"));
        }
        self.config.apache_port = port;
        self.config.nginx_port = port;
        config::save_config(&self.paths, &self.config)?;
        self.port_conflict = None;
        self.start_all()
    }

    pub fn set_php_extension(&mut self, name: &str, enabled: bool) -> LaxResult<()> {
        php::set_extension(&self.paths, &self.config, name, enabled)
    }

    pub fn save(&mut self, cfg: LaxConfig) -> LaxResult<()> {
        self.config = cfg;
        config::save_config(&self.paths, &self.config)?;
        self.prepare_sites()?;
        Ok(())
    }

    pub fn create_project(&mut self, name: &str) -> LaxResult<ProjectInfo> {
        let p = projects::create_project(&self.paths, &self.config, name)?;
        self.prepare_sites()?;
        Ok(p)
    }
}

fn wait_port(port: u16, attempts: u32) -> LaxResult<()> {
    for _ in 0..attempts {
        if port_open(port) {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(120));
    }
    Err(LaxError::msg(format!(
        "web server did not open port {port}. Check logs."
    )))
}
