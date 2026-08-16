use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use crate::error::{LaxError, LaxResult};

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GITHUB_REPO: &str = "razemsb/LaX";
pub const REPO_URL: &str = "https://github.com/razemsb/LaX";
pub const ISSUES_URL: &str = "https://github.com/razemsb/LaX/issues";
pub const FEEDBACK_URL: &str = "https://github.com/razemsb/LaX/issues/new";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub version: String,
    pub tag: String,
    pub url: String,
    pub notes: String,
    pub download_url: Option<String>,
    pub download_name: Option<String>,
    pub size: Option<u64>,
}

#[derive(Deserialize)]
struct GhRelease {
    tag_name: String,
    html_url: String,
    body: Option<String>,
    assets: Vec<GhAsset>,
}

#[derive(Deserialize)]
struct GhAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

pub fn is_newer(latest: &str, current: &str) -> bool {
    cmp_semver(latest) > cmp_semver(current)
}

fn cmp_semver(v: &str) -> (u64, u64, u64) {
    let v = v.trim().trim_start_matches('v');
    let mut parts = v.split(|c: char| !c.is_ascii_digit());
    let major = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    (major, minor, patch)
}

pub fn fetch_latest() -> LaxResult<UpdateInfo> {
    let url = format!("https://api.github.com/repos/{GITHUB_REPO}/releases/latest");
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(15))
        .timeout(Duration::from_secs(30))
        .user_agent(&format!("LaX/{APP_VERSION} (+{REPO_URL})"))
        .build();
    let rel: GhRelease = agent
        .get(&url)
        .set("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| LaxError::msg(format!("GitHub: {e}")))?
        .into_json()
        .map_err(|e| LaxError::msg(format!("GitHub JSON: {e}")))?;

    let version = rel.tag_name.trim().trim_start_matches('v').to_string();
    let asset = pick_asset(&rel.assets);
    let notes = rel
        .body
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .take(8)
        .collect::<Vec<_>>()
        .join("\n");

    Ok(UpdateInfo {
        version,
        tag: rel.tag_name,
        url: rel.html_url,
        notes,
        download_url: asset.map(|a| a.browser_download_url.clone()),
        download_name: asset.map(|a| a.name.clone()),
        size: asset.map(|a| a.size),
    })
}

fn pick_asset(assets: &[GhAsset]) -> Option<&GhAsset> {
    let want_zip = |n: &str| {
        let n = n.to_ascii_lowercase();
        n.ends_with(".zip") && n.contains("lax") && !n.contains("source") && !n.contains("linux")
    };
    let want_appimage = |n: &str| n.to_ascii_lowercase().ends_with(".appimage");
    if cfg!(windows) {
        assets.iter().find(|a| want_zip(&a.name))
    } else {
        assets
            .iter()
            .find(|a| want_appimage(&a.name))
            .or_else(|| assets.iter().find(|a| want_zip(&a.name)))
    }
}

pub fn emit_progress(app: &AppHandle, msg: &str) {
    let _ = app.emit("update-progress", msg);
}

pub fn download_asset(
    root: &Path,
    info: &UpdateInfo,
    mut progress: impl FnMut(&str),
) -> LaxResult<PathBuf> {
    let url = info
        .download_url
        .as_deref()
        .ok_or_else(|| LaxError::msg("в релизе нет файла для этой ОС — открой страницу релиза"))?;
    let name = info
        .download_name
        .as_deref()
        .unwrap_or("LaX-update.bin");
    fs::create_dir_all(root.join("tmp"))?;
    let dest = root.join("tmp").join(name);
    progress("скачиваю обновление…");
    download(url, &dest, &mut progress)?;
    progress("скачано, готовлю файлы…");
    Ok(dest)
}

pub fn install_asset(
    root: &Path,
    dest: &Path,
    mut progress: impl FnMut(&str),
) -> LaxResult<()> {
    let name = dest
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if name.ends_with(".appimage") {
        progress("ставлю AppImage…");
        install_appimage(dest)?;
        return Ok(());
    }
    if name.ends_with(".zip") {
        install_zip(root, dest, &mut progress)?;
        let _ = fs::remove_file(dest);
        return Ok(());
    }
    Err(LaxError::msg("неизвестный формат обновления"))
}

fn download(url: &str, dest: &Path, progress: &mut impl FnMut(&str)) -> LaxResult<()> {
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(20))
        .timeout(Duration::from_secs(600))
        .user_agent(&format!("LaX/{APP_VERSION} (+{REPO_URL})"))
        .build();
    let resp = agent
        .get(url)
        .call()
        .map_err(|e| LaxError::msg(format!("скачивание: {e}")))?;
    let total = resp
        .header("Content-Length")
        .and_then(|s| s.parse::<u64>().ok())
        .filter(|n| *n > 0);
    let mut reader = resp.into_reader();
    let tmp = dest.with_extension("part");
    let mut file = fs::File::create(&tmp)?;
    let mut buf = [0u8; 64 * 1024];
    let mut copied: u64 = 0;
    let mut last_pct = 101u64;
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])?;
        copied += n as u64;
        if let Some(total) = total {
            let pct = copied.saturating_mul(100) / total;
            if pct != last_pct {
                last_pct = pct;
                progress(&format!("скачиваю {pct}%"));
            }
        }
    }
    file.flush()?;
    drop(file);
    if dest.exists() {
        let _ = fs::remove_file(dest);
    }
    fs::rename(&tmp, dest)?;
    Ok(())
}

fn install_zip(root: &Path, zip_path: &Path, progress: &mut impl FnMut(&str)) -> LaxResult<()> {
    progress("распаковываю…");
    let file = fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| LaxError::msg(e.to_string()))?;
    let prefix = zip_prefix(&mut archive);
    let total = archive.len().max(1);
    let mut written = 0u32;
    let mut skipped = 0u32;
    for i in 0..archive.len() {
        if i % 80 == 0 {
            progress(&format!("раскладываю файлы… {}%", i * 100 / total));
        }
        let mut entry = archive
            .by_index(i)
            .map_err(|e| LaxError::msg(e.to_string()))?;
        if entry.is_dir() {
            continue;
        }
        let Some(rel) = zip_rel(entry.name(), prefix.as_deref()) else {
            continue;
        };
        if skip_rel(&rel) {
            continue;
        }
        let dest = if is_self_exe(&rel) {
            root.join(if cfg!(windows) {
                "lax.exe.new"
            } else {
                "lax.new"
            })
        } else {
            root.join(&rel)
        };
        if !is_self_exe(&rel) && dest.is_file() {
            if let Ok(meta) = dest.metadata() {
                if meta.len() == entry.size() {
                    skipped += 1;
                    continue;
                }
            }
        }
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut out = fs::File::create(&dest)?;
        io::copy(&mut entry, &mut out)?;
        written += 1;
    }
    progress(&format!("готово · новых файлов {written}, без изменений {skipped}"));
    Ok(())
}

fn zip_prefix(archive: &mut zip::ZipArchive<fs::File>) -> Option<String> {
    let mut prefix: Option<String> = None;
    let n = archive.len().min(24);
    for i in 0..n {
        let Ok(entry) = archive.by_index(i) else {
            continue;
        };
        let name = entry.name().replace('\\', "/");
        let first = name.split('/').next().unwrap_or("");
        if first.is_empty() {
            return None;
        }
        let lower = first.to_ascii_lowercase();
        if matches!(
            lower.as_str(),
            "lax.exe" | "lax" | "bin" | "www" | "etc" | "usr" | "data" | "tmp" | "logs"
        ) {
            return None;
        }
        match &prefix {
            None => prefix = Some(first.to_string()),
            Some(p) if p != first => return None,
            _ => {}
        }
    }
    prefix
}

fn zip_rel(name: &str, prefix: Option<&str>) -> Option<PathBuf> {
    let name = name.replace('\\', "/").trim_matches('/').to_string();
    if name.is_empty() {
        return None;
    }
    let rest = if let Some(prefix) = prefix {
        name.strip_prefix(&format!("{prefix}/"))
            .unwrap_or(&name)
            .to_string()
    } else {
        name
    };
    if rest.is_empty() {
        return None;
    }
    let path = PathBuf::from(&rest);
    if path
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir | std::path::Component::Prefix(_)))
    {
        return None;
    }
    Some(path)
}

fn skip_rel(rel: &Path) -> bool {
    if rel.as_os_str().is_empty() {
        return false;
    }
    let mut comps = rel.components();
    let Some(std::path::Component::Normal(first)) = comps.next() else {
        return false;
    };
    let first = first.to_string_lossy().to_ascii_lowercase();
    if matches!(
        first.as_str(),
        "www" | "data" | "logs" | "tmp" | ".git" | "src" | "src-tauri" | "node_modules" | "pack"
    ) {
        return true;
    }
    if first == "usr" {
        let rest = rel
            .components()
            .skip(1)
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join("/");
        if rest == "lax.toml" || rest == ".install-root" {
            return true;
        }
    }
    false
}

fn is_self_exe(rel: &Path) -> bool {
    let name = rel
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    name == "lax.exe" || name == "lax"
}

fn appimage_path() -> Option<PathBuf> {
    std::env::var_os("APPIMAGE").map(PathBuf::from)
}

fn install_appimage(downloaded: &Path) -> LaxResult<()> {
    let Some(current) = appimage_path() else {
        return Err(LaxError::msg(
            "это не AppImage — скачай файл вручную со страницы релиза",
        ));
    };
    let staged = PathBuf::from(format!("{}.new", current.display()));
    fs::copy(downloaded, &staged)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&staged)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&staged, perms)?;
    }
    let _ = fs::remove_file(downloaded);
    Ok(())
}

pub fn has_staged_exe(root: &Path) -> bool {
    if root.join("lax.exe.new").exists() || root.join("lax.new").exists() {
        return true;
    }
    appimage_path()
        .map(|p| PathBuf::from(format!("{}.new", p.display())).exists())
        .unwrap_or(false)
}

pub fn spawn_relaunch(root: &Path) -> LaxResult<()> {
    #[cfg(windows)]
    {
        return spawn_relaunch_windows(root);
    }
    #[cfg(not(windows))]
    {
        spawn_relaunch_unix(root)
    }
}

#[cfg(windows)]
fn spawn_relaunch_windows(root: &Path) -> LaxResult<()> {
    let exe = root.join("lax.exe");
    let staged = root.join("lax.exe.new");
    if !staged.exists() {
        return Ok(());
    }
    let script = root.join("tmp").join("lax-relaunch.cmd");
    let body = format!(
        "@echo off\r\n\
set n=0\r\n\
:wait\r\n\
if %n% geq 15 goto kill\r\n\
ping -n 2 127.0.0.1 >nul\r\n\
tasklist /FI \"IMAGENAME eq lax.exe\" | find /I \"lax.exe\" >nul\r\n\
if not errorlevel 1 (\r\n\
  set /a n+=1\r\n\
  goto wait\r\n\
)\r\n\
goto copy\r\n\
:kill\r\n\
taskkill /F /IM lax.exe /T >nul 2>&1\r\n\
ping -n 2 127.0.0.1 >nul\r\n\
:copy\r\n\
copy /Y \"{staged}\" \"{exe}\" >nul\r\n\
del \"{staged}\" >nul 2>&1\r\n\
start \"\" \"{exe}\"\r\n\
del \"%~f0\"\r\n",
        staged = staged.display(),
        exe = exe.display()
    );
    fs::write(&script, body)?;
    Command::new("cmd.exe")
        .args(["/C", "start", "", "/min", "cmd.exe", "/C", &script.to_string_lossy()])
        .spawn()
        .map_err(|e| LaxError::msg(format!("relaunch: {e}")))?;
    Ok(())
}

#[cfg(not(windows))]
fn spawn_relaunch_unix(root: &Path) -> LaxResult<()> {
    let (exe, staged) = if let Some(img) = appimage_path() {
        let staged = PathBuf::from(format!("{}.new", img.display()));
        (img, staged)
    } else {
        (root.join("lax"), root.join("lax.new"))
    };
    if !staged.exists() {
        return Ok(());
    }
    let script = root.join("tmp").join("lax-relaunch.sh");
    let body = format!(
        "#!/bin/sh\n\
sleep 1\n\
while pgrep -x '{name}' >/dev/null 2>&1; do sleep 1; done\n\
mv -f '{staged}' '{exe}'\n\
chmod +x '{exe}'\n\
'{exe}' &\n\
rm -f \"$0\"\n",
        name = exe.file_name().and_then(|s| s.to_str()).unwrap_or("lax"),
        staged = staged.display(),
        exe = exe.display()
    );
    fs::write(&script, body)?;
    let _ = Command::new("chmod").args(["+x", &script.to_string_lossy()]).status();
    Command::new("sh")
        .arg(&script)
        .spawn()
        .map_err(|e| LaxError::msg(format!("relaunch: {e}")))?;
    Ok(())
}
