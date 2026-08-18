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
