use std::fs;
use std::path::Path;

use crate::config::{LaxConfig, Paths};
use crate::error::LaxResult;
use crate::paths::unix;
use crate::projects::ProjectInfo;
use crate::process::write_file;

pub fn regenerate(paths: &Paths, cfg: &LaxConfig, _projects: &[ProjectInfo]) -> LaxResult<()> {
    let apache_dir = paths.root.join("etc/apache2/sites-enabled");
    let nginx_dir = paths.root.join("etc/nginx/sites-enabled");
    fs::create_dir_all(&apache_dir)?;
    fs::create_dir_all(&nginx_dir)?;
    clear_auto(&apache_dir)?;
    clear_auto(&nginx_dir)?;
    write_nginx_support(paths)?;
    write_defaults(paths, cfg)?;
    write_db_apps(paths, cfg)?;
    patch_apache_listen(paths, cfg)?;
    Ok(())
}

fn patch_apache_listen(paths: &Paths, cfg: &LaxConfig) -> LaxResult<()> {
    let httpd = paths.apache_dir(cfg).join("conf").join("httpd.conf");
    if !httpd.exists() {
        return Ok(());
    }
    let body = fs::read_to_string(&httpd)?;
    let mut out = String::new();
    let mut done = false;
    for line in body.lines() {
        let t = line.trim_start();
        if t.starts_with("Listen ") && !t.contains("443") {
            if !done {
                out.push_str(&format!("Listen {}\n", cfg.apache_port));
                done = true;
            }
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    if !done {
        out.push_str(&format!("\nListen {}\n", cfg.apache_port));
    }
    fs::write(httpd, out)?;
    Ok(())
}

fn clear_auto(dir: &Path) -> LaxResult<()> {
    if let Ok(rd) = fs::read_dir(dir) {
        for ent in rd.flatten() {
            let name = ent.file_name().to_string_lossy().into_owned();
            if name.starts_with("auto.") && name.ends_with(".conf") {
                let _ = fs::remove_file(ent.path());
            }
        }
    }
    Ok(())
}

fn write_defaults(paths: &Paths, cfg: &LaxConfig) -> LaxResult<()> {
    let www = unix(&paths.www(cfg));
    let root = unix(&paths.root);
    let apache = format!(
        r#"<VirtualHost *:{port}>
    DocumentRoot "{www}"
    ServerName localhost
    <Directory "{www}">
        Options Indexes FollowSymLinks Includes ExecCGI
        AllowOverride All
        Require all granted
    </Directory>
</VirtualHost>
"#,
        port = cfg.apache_port,
        www = www
    );
    write_file(
        &paths.root.join("etc/apache2/sites-enabled/00-default.conf"),
        &apache,
    )?;

    let nginx = format!(
        r#"server {{
    listen {port} default_server;
    server_name localhost;
    root "{www}";
    index index.html index.htm index.php;
    client_max_body_size 2000M;
    include "{root}/etc/nginx/alias/*.conf";

    location / {{
        try_files $uri $uri/ /index.php$is_args$args;
        autoindex on;
    }}

    location ~ \.php$ {{
        include "{root}/etc/nginx/fastcgi_params";
        fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;
        fastcgi_index index.php;
        fastcgi_pass php_upstream;
    }}

    location = /favicon.ico {{ access_log off; log_not_found off; }}
    location ~ /\.ht {{ deny all; }}
}}
"#,
        port = cfg.nginx_port,
        www = www,
        root = root
    );
    write_file(
        &paths.root.join("etc/nginx/sites-enabled/00-default.conf"),
        &nginx,
    )?;
    Ok(())
}

fn write_db_apps(paths: &Paths, _cfg: &LaxConfig) -> LaxResult<()> {
    let root = unix(&paths.root);
    let pma = format!("{root}/etc/apps/phpMyAdmin");
    fs::create_dir_all(paths.root.join("etc/apps/phpMyAdmin/tmp"))?;
    fs::create_dir_all(paths.root.join("usr/apps/dbgate"))?;
    fs::create_dir_all(paths.root.join("etc/apache2/alias"))?;
    fs::create_dir_all(paths.root.join("etc/nginx/alias"))?;

    write_file(
        &paths.root.join("etc/apache2/alias/phpmyadmin.conf"),
        &format!(
            r#"Alias /phpmyadmin "{pma}/"
Alias /phpMyAdmin "{pma}/"

<Directory "{pma}/">
    Options Indexes FollowSymLinks ExecCGI
    AllowOverride All
    Require all granted
    DirectoryIndex index.php
</Directory>
"#
        ),
    )?;

    write_file(
        &paths.root.join("etc/nginx/alias/phpmyadmin.conf"),
        &nginx_php_alias("/phpmyadmin", &pma, &root),
    )?;
    let _ = fs::remove_file(paths.root.join("etc/apache2/alias/adminer.conf"));
    let _ = fs::remove_file(paths.root.join("etc/nginx/alias/adminer.conf"));
    install_pma_theme(paths);
    Ok(())
}

/// Ship `usr/themes/phpmyadmin/lax` into phpMyAdmin as a real theme (icons from pmahomme).
pub fn install_pma_theme(paths: &Paths) {
    if let Err(e) = install_pma_theme_inner(paths) {
        tracing::warn!("phpMyAdmin LaX theme: {e}");
    }
}

fn install_pma_theme_inner(paths: &Paths) -> LaxResult<()> {
    let pma = paths.root.join("etc/apps/phpMyAdmin");
    if !pma.join("index.php").is_file() {
        return Ok(());
    }
    let src = paths.root.join("usr/themes/phpmyadmin/lax");
    if !src.join("theme.json").is_file() {
        return Ok(());
    }
    let dest = pma.join("themes/lax");
    let homme = pma.join("themes/pmahomme");
    fs::create_dir_all(dest.join("css"))?;
    fs::create_dir_all(dest.join("jquery"))?;
    fs::create_dir_all(dest.join("fonts"))?;
    fs::create_dir_all(dest.join("img"))?;

    fs::copy(src.join("theme.json"), dest.join("theme.json"))?;
    copy_if(&src.join("img/logo.svg"), &dest.join("img/logo.svg"));
    copy_dir_if_missing(&homme.join("img"), &dest.join("img"));
    copy_if(&src.join("img/logo.svg"), &dest.join("img/logo.svg"));
    copy_if(
        &src.join("fonts/JetBrainsMono-Variable.ttf"),
        &dest.join("fonts/JetBrainsMono-Variable.ttf"),
    );
    copy_if(&src.join("fonts/OFL.txt"), &dest.join("fonts/OFL.txt"));
    copy_if(&homme.join("screen.png"), &dest.join("screen.png"));

    let base_css = fs::read_to_string(homme.join("css/theme.css")).unwrap_or_default();
    let overlay = fs::read_to_string(src.join("css/lax.css")).unwrap_or_default();
    fs::write(
        dest.join("css/theme.css"),
        format!("{base_css}\n\n/* ---- LaX ---- */\n{overlay}"),
    )?;
    if homme.join("css/theme.rtl.css").is_file() {
        let rtl = fs::read_to_string(homme.join("css/theme.rtl.css")).unwrap_or_default();
        fs::write(
            dest.join("css/theme.rtl.css"),
            format!("{rtl}\n\n/* ---- LaX ---- */\n{overlay}"),
        )?;
    }
    let mut ui = fs::read_to_string(homme.join("jquery/jquery-ui.css")).unwrap_or_default();
    ui.push_str("\n\n/* ---- LaX ---- */\n");
    ui.push_str(&overlay);
    fs::write(dest.join("jquery/jquery-ui.css"), ui)?;

    patch_pma_config(&pma.join("config.inc.php"))?;
    patch_pma_css_cache_bust(&pma.join("templates/header.twig"))?;
    Ok(())
}

fn copy_if(from: &Path, to: &Path) {
    if from.is_file() {
        let _ = fs::copy(from, to);
    }
}

fn copy_dir_if_missing(from: &Path, to: &Path) {
    if !from.is_dir() {
        return;
    }
    let marker = to.join("b_home.png");
    if marker.is_file() {
        return;
    }
    let _ = copy_dir(from, to);
}

fn copy_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    fs::create_dir_all(to)?;
    for ent in fs::read_dir(from)? {
        let ent = ent?;
        let dest = to.join(ent.file_name());
        if ent.path().is_dir() {
            copy_dir(&ent.path(), &dest)?;
        } else if !dest.is_file() {
            fs::copy(ent.path(), dest)?;
        }
    }
    Ok(())
}

fn patch_pma_css_cache_bust(path: &Path) -> LaxResult<()> {
    if !path.is_file() {
        return Ok(());
    }
    let mut body = fs::read_to_string(path)?;
    body = body.replace(
        "href=\"{{ theme_path }}/jquery/jquery-ui.css\"",
        "href=\"{{ theme_path }}/jquery/jquery-ui.css?lax=17\"",
    );
    body = body.replace(
        "css/theme{{ text_dir == 'rtl' ? '.rtl' }}.css?{{ version }}\"",
        "css/theme{{ text_dir == 'rtl' ? '.rtl' }}.css?{{ version }}&lax=17\"",
    );
    body = body.replace("jquery-ui.css?lax=12", "jquery-ui.css?lax=17");
    body = body.replace("jquery-ui.css?lax=13", "jquery-ui.css?lax=17");
    body = body.replace("jquery-ui.css?lax=14", "jquery-ui.css?lax=17");
    body = body.replace("&lax=12", "&lax=17");
    body = body.replace("&lax=13", "&lax=17");
    body = body.replace("&lax=14", "&lax=17");
    fs::write(path, body)?;
    Ok(())
}

fn patch_pma_config(path: &Path) -> LaxResult<()> {
    if !path.is_file() {
        return Ok(());
    }
    let mut body = fs::read_to_string(path)?;
    const MARK: &str = "\n/* LaX theme */\n";
    if let Some(i) = body.find(MARK) {
        body.truncate(i);
    }
    body.push_str(MARK);
    body.push_str("$cfg['ThemeDefault'] = 'lax';\n");
    body.push_str("$cfg['NavigationDisplayLogo'] = true;\n");
    body.push_str("$cfg['NavigationWidth'] = 268;\n");
    fs::write(path, body)?;
    Ok(())
}

fn nginx_php_alias(url: &str, dir: &str, root: &str) -> String {
    format!(
        r#"location {url} {{
    alias {dir}/;
    index index.php;
}}

location ~ ^{url}/(.+\.php)$ {{
    alias {dir}/$1;
    include {root}/etc/nginx/fastcgi_params;
    fastcgi_param SCRIPT_FILENAME $request_filename;
    fastcgi_index index.php;
    fastcgi_pass php_upstream;
}}
"#
    )
}

fn write_nginx_support(paths: &Paths) -> LaxResult<()> {
    let params = r#"fastcgi_param  QUERY_STRING       $query_string;
fastcgi_param  REQUEST_METHOD     $request_method;
fastcgi_param  CONTENT_TYPE       $content_type;
fastcgi_param  CONTENT_LENGTH     $content_length;
fastcgi_param  SCRIPT_NAME        $fastcgi_script_name;
fastcgi_param  REQUEST_URI        $request_uri;
fastcgi_param  DOCUMENT_URI       $document_uri;
fastcgi_param  DOCUMENT_ROOT      $document_root;
fastcgi_param  SERVER_PROTOCOL    $server_protocol;
fastcgi_param  REQUEST_SCHEME     $scheme;
fastcgi_param  HTTPS              $https if_not_empty;
fastcgi_param  GATEWAY_INTERFACE  CGI/1.1;
fastcgi_param  SERVER_SOFTWARE    nginx;
fastcgi_param  REMOTE_ADDR        $remote_addr;
fastcgi_param  REMOTE_PORT        $remote_port;
fastcgi_param  SERVER_ADDR        $server_addr;
fastcgi_param  SERVER_PORT        $server_port;
fastcgi_param  SERVER_NAME        $server_name;
fastcgi_param  REDIRECT_STATUS    200;
"#;
    write_file(&paths.root.join("etc/nginx/fastcgi_params"), params)?;

    let mime = r#"types {
    text/html                             html htm shtml;
    text/css                              css;
    text/xml                              xml;
    image/gif                             gif;
    image/jpeg                            jpeg jpg;
    application/javascript                js mjs;
    application/atom+xml                  atom;
    application/rss+xml                   rss;
    text/plain                            txt;
    image/png                             png;
    image/svg+xml                         svg svgz;
    image/webp                            webp;
    image/x-icon                          ico;
    application/json                      json;
    application/wasm                      wasm;
    application/pdf                       pdf;
    application/zip                       zip;
    application/gzip                      gz;
    font/woff                             woff;
    font/woff2                            woff2;
    application/vnd.ms-fontobject         eot;
    font/ttf                              ttf;
    audio/mpeg                            mp3;
    video/mp4                             mp4;
    application/octet-stream              bin exe dll;
}
"#;
    write_file(&paths.root.join("etc/nginx/mime.types"), mime)?;
    Ok(())
}
