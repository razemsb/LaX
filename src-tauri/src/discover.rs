use std::fs;
use std::net::{SocketAddr, TcpStream};
use std::path::Path;
use std::time::Duration;

pub fn list_subdirs(dir: &Path) -> Vec<String> {
    let mut names = Vec::new();
    let Ok(rd) = fs::read_dir(dir) else {
        return names;
    };
    for ent in rd.flatten() {
        if ent.path().is_dir() {
            names.push(ent.file_name().to_string_lossy().into_owned());
        }
    }
    names.sort();
    names
}

pub fn php_versions(root: &Path) -> Vec<String> {
    list_subdirs(&root.join("bin").join("php"))
        .into_iter()
        .filter(|n| root.join("bin").join("php").join(n).join("php.exe").exists()
            || root.join("bin").join("php").join(n).join("php-cgi.exe").exists())
        .collect()
}

pub fn mysql_versions(root: &Path) -> Vec<String> {
    list_subdirs(&root.join("bin").join("mysql"))
}

pub fn nginx_versions(root: &Path) -> Vec<String> {
    list_subdirs(&root.join("bin").join("nginx"))
}

pub fn apache_versions(root: &Path) -> Vec<String> {
    list_subdirs(&root.join("bin").join("apache"))
}

pub fn port_open(port: u16) -> bool {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    TcpStream::connect_timeout(&addr, Duration::from_millis(40)).is_ok()
}
