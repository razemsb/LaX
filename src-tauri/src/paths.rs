use std::path::{Path, PathBuf};

use dunce::canonicalize;

pub fn unix(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub fn detect_root() -> PathBuf {
    if let Ok(v) = std::env::var("LAX_ROOT") {
        return PathBuf::from(v);
    }

    // AppImage: the .AppImage file lives next to bin/, not the mounted squashfs exe.
    if let Ok(appimage) = std::env::var("APPIMAGE") {
        if let Some(dir) = Path::new(&appimage).parent() {
            if looks_like_root(dir) {
                return canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
            }
        }
    }

    // Portable layout: lax.exe sits next to bin/, usr/, www/ (like laragon.exe).
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let candidates = [
                dir.to_path_buf(),
                dir.join(".."),
                dir.join("../.."),
                dir.join("../../.."),
                dir.join("../../../.."),
            ];
            for c in candidates {
                if looks_like_root(&c) {
                    return canonicalize(&c).unwrap_or(c);
                }
            }
        }
    }

    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let from_manifest = manifest.join("..");
    if looks_like_root(&from_manifest) {
        return canonicalize(&from_manifest).unwrap_or(from_manifest);
    }

    canonicalize(&from_manifest).unwrap_or(from_manifest)
}

fn looks_like_root(p: &Path) -> bool {
    p.join("usr").join("lax.toml").exists()
        || p.join("bin").join("apache").exists()
        || p.join("bin").join("nginx").exists()
        || p.join("bin").join("php").exists()
        || p.join("bin").join("mysql").exists()
}

pub fn join_unix(root: &Path, rel: &str) -> PathBuf {
    let mut p = root.to_path_buf();
    for part in rel.split(['/', '\\']) {
        if !part.is_empty() {
            p.push(part);
        }
    }
    p
}
