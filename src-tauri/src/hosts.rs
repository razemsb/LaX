use std::fs;
use std::path::Path;

use crate::error::LaxResult;
use crate::platform;
use crate::projects::ProjectInfo;

const BEGIN: &str = "# LaX begin";
const END: &str = "# LaX end";

pub fn hosts_path() -> &'static str {
    platform::hosts_path()
}

pub fn writable() -> bool {
    let path = hosts_path();
    match fs::OpenOptions::new().append(true).open(path) {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn sync_hosts(projects: &[ProjectInfo], tld: &str, enabled: bool) -> LaxResult<bool> {
    let path = Path::new(hosts_path());
    let Ok(raw) = fs::read_to_string(path) else {
        return Ok(false);
    };
    let stripped = strip_block(&raw);
    let mut next = stripped.trim_end().to_string();
    next.push('\n');
    if enabled {
        next.push_str(BEGIN);
        next.push('\n');
        for p in projects {
            next.push_str(&format!("127.0.0.1 {0}.{1}\n", p.name, tld));
        }
        next.push_str(END);
        next.push('\n');
    }
    match fs::write(path, next) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

fn strip_block(raw: &str) -> String {
    let mut out = String::new();
    let mut skip = false;
    for line in raw.lines() {
        if line.trim() == BEGIN {
            skip = true;
            continue;
        }
        if line.trim() == END {
            skip = false;
            continue;
        }
        if !skip {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}
