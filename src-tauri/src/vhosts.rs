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
    write_defaults(paths, cfg)?;
    write_phpmyadmin(paths)?;
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
        include snippets/fastcgi-php.conf;
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

fn write_phpmyadmin(paths: &Paths) -> LaxResult<()> {
    let root = unix(&paths.root);
    let pma = format!("{root}/etc/apps/phpMyAdmin");
    fs::create_dir_all(paths.root.join("etc/apps/phpMyAdmin/tmp"))?;
    fs::create_dir_all(paths.root.join("etc/apache2/alias"))?;
    fs::create_dir_all(paths.root.join("etc/nginx/alias"))?;

    let apache = format!(
        r#"Alias /phpmyadmin "{pma}/"
Alias /phpMyAdmin "{pma}/"

<Directory "{pma}/">
    Options Indexes FollowSymLinks ExecCGI
    AllowOverride All
    Require all granted
    DirectoryIndex index.php
</Directory>
"#
    );
    write_file(
        &paths.root.join("etc/apache2/alias/phpmyadmin.conf"),
        &apache,
    )?;

    let nginx = format!(
        r#"location /phpmyadmin {{
    alias {pma}/;
    index index.php;
}}

location ~ ^/phpmyadmin/(.+\.php)$ {{
    alias {pma}/$1;
    fastcgi_pass php_upstream;
    fastcgi_index index.php;
    fastcgi_param SCRIPT_FILENAME $request_filename;
    include fastcgi_params;
}}
"#
    );
    write_file(&paths.root.join("etc/nginx/alias/phpmyadmin.conf"), &nginx)?;
    Ok(())
}
