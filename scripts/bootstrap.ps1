#Requires -Version 5.1
<#
.SYNOPSIS
  Copy Apache/Nginx/MariaDB/PHP/composer/phpMyAdmin from Laragon into LaX
  and rewrite paths so the stack is portable under C:\Laragon\www\LaX.
#>
$ErrorActionPreference = "Stop"

$Src = "C:\Laragon"
$Dst = "C:\Laragon\www\LaX"
$UnixRoot = "C:/Laragon/www/LaX"
$WinRoot = "C:\Laragon\www\LaX"
$PhpDefault = "php-dlya-debilov"

function Invoke-Robo {
    param(
        [Parameter(Mandatory = $true)][string]$From,
        [Parameter(Mandatory = $true)][string]$To,
        [string[]]$ExcludeDirs = @()
    )
    if (-not (Test-Path $From)) {
        Write-Host "skip missing $From"
        return
    }
    New-Item -ItemType Directory -Force -Path $To | Out-Null
    $args = @($From, $To, "/E", "/NFL", "/NDL", "/NJH", "/NJS", "/NC", "/NS", "/NP", "/R:1", "/W:1")
    foreach ($d in $ExcludeDirs) {
        $args += @("/XD", $d)
    }
    & robocopy @args | Out-Null
    $code = $LASTEXITCODE
    if ($code -ge 8) {
        throw "robocopy failed ($code): $From -> $To"
    }
}

function Rewrite-TextFile {
    param([string]$Path)
    if (-not (Test-Path $Path)) { return }
    $bytes = [System.IO.File]::ReadAllBytes($Path)
    if ($bytes.Length -ge 3 -and $bytes[0] -eq 0xEF -and $bytes[1] -eq 0xBB -and $bytes[2] -eq 0xBF) {
        $text = [System.Text.Encoding]::UTF8.GetString($bytes, 3, $bytes.Length - 3)
        $bom = $true
    } else {
        $text = [System.Text.Encoding]::UTF8.GetString($bytes)
        $bom = $false
    }
    $orig = $text
    if ($text -match 'Laragon/www/LaX' -or $text -match 'Laragon\\www\\LaX') {
        return
    }
    $text = $text.Replace("C:/PortableLaragon", $UnixRoot)
    $text = $text.Replace("C:\PortableLaragon", $WinRoot)
    $text = $text.Replace("C:/Laragon", $UnixRoot)
    $text = $text.Replace("C:\Laragon", $WinRoot)
    if ($text -ne $orig) {
        $enc = New-Object System.Text.UTF8Encoding $bom
        [System.IO.File]::WriteAllText($Path, $text, $enc)
        Write-Host "rewrote $($Path.Replace($Dst, '.'))"
    }
}

function Rewrite-Tree {
    param([string]$Dir, [string[]]$Filters)
    if (-not (Test-Path $Dir)) { return }
    Get-ChildItem -Path $Dir -Recurse -File -Include $Filters -ErrorAction SilentlyContinue | ForEach-Object {
        Rewrite-TextFile $_.FullName
    }
}

Write-Host "==> Copying runtime from $Src to $Dst"

New-Item -ItemType Directory -Force -Path @(
    "$Dst\bin",
    "$Dst\etc\apache2\sites-enabled",
    "$Dst\etc\nginx\sites-enabled",
    "$Dst\etc\apps",
    "$Dst\usr\tpl",
    "$Dst\www",
    "$Dst\data\mariadb",
    "$Dst\tmp",
    "$Dst\logs"
) | Out-Null

Invoke-Robo "$Src\bin\apache" "$Dst\bin\apache" -ExcludeDirs @("logs")
Invoke-Robo "$Src\bin\nginx" "$Dst\bin\nginx" -ExcludeDirs @("logs")
Invoke-Robo "$Src\bin\mysql\mariadb-10.11.13" "$Dst\bin\mysql\mariadb-10.11.13" -ExcludeDirs @("data")
Invoke-Robo "$Src\bin\composer" "$Dst\bin\composer"
Invoke-Robo "$Src\bin\sendmail" "$Dst\bin\sendmail" -ExcludeDirs @("output")

$phpVersions = @(
    "php-dlya-debilov",
    "php-trash-8.2",
    "php-5.4.9-nts-Win32-VC9-x86"
)
foreach ($ver in $phpVersions) {
    Invoke-Robo "$Src\bin\php\$ver" "$Dst\bin\php\$ver"
}

Invoke-Robo "$Src\etc\apache2\modules" "$Dst\etc\apache2\modules"
Invoke-Robo "$Src\etc\apache2\alias" "$Dst\etc\apache2\alias"
foreach ($f in @("fcgid.conf", "mod_php.conf", "httpd-ssl.conf")) {
    $from = Join-Path "$Src\etc\apache2" $f
    if (Test-Path $from) {
        Copy-Item $from "$Dst\etc\apache2\$f" -Force
    }
}

Invoke-Robo "$Src\etc\nginx\alias" "$Dst\etc\nginx\alias"
if (Test-Path "$Src\etc\nginx\php_upstream.conf") {
    Copy-Item "$Src\etc\nginx\php_upstream.conf" "$Dst\etc\nginx\php_upstream.conf" -Force
}

# Do not exclude a directory named "twig": that also drops vendor/twig, which phpMyAdmin needs.
Invoke-Robo "$Src\etc\apps\phpMyAdmin" "$Dst\etc\apps\phpMyAdmin" -ExcludeDirs @("tmp")
New-Item -ItemType Directory -Force -Path "$Dst\etc\apps\phpMyAdmin\tmp" | Out-Null

if (Test-Path "$Src\etc\ssl\cacert.pem") {
    New-Item -ItemType Directory -Force -Path "$Dst\etc\ssl" | Out-Null
    Copy-Item "$Src\etc\ssl\cacert.pem" "$Dst\etc\ssl\cacert.pem" -Force
}

if (Test-Path "$Src\usr\tpl") {
    Copy-Item "$Src\usr\tpl\*" "$Dst\usr\tpl\" -Force
}

Write-Host "==> Rewriting config paths"

Rewrite-TextFile "$Dst\bin\apache\Apache24\conf\httpd.conf"
Rewrite-Tree "$Dst\etc\apache2" @("*.conf")
Rewrite-Tree "$Dst\etc\nginx" @("*.conf")
Rewrite-TextFile "$Dst\bin\nginx\nginx-1.14.0\conf\nginx.conf"
Rewrite-TextFile "$Dst\bin\mysql\mariadb-10.11.13\my.ini"
Get-ChildItem "$Dst\bin\php" -Directory | ForEach-Object {
    Rewrite-TextFile (Join-Path $_.FullName "php.ini")
}

# Point PHP default + Apache fcgid at php-dlya-debilov
$fcgid = "$Dst\etc\apache2\fcgid.conf"
if (Test-Path $fcgid) {
    $phpDir = "$UnixRoot/bin/php/$PhpDefault"
    $content = Get-Content $fcgid -Raw
    $content = [regex]::Replace($content, 'C:/Laragon(?:/www/LaX)?/bin/php/[^"\s;]+', "$phpDir")
    $content = $content.Replace("$phpDir`"", "$phpDir/php-cgi.exe`"")
    if ($content -notmatch 'php-cgi\.exe') {
        $content = $content.Replace($phpDir, "$phpDir/php-cgi.exe")
    }
    Set-Content -Path $fcgid -Value $content -Encoding UTF8
}

$modPhp = "$Dst\etc\apache2\mod_php.conf"
if (Test-Path $modPhp) {
    @"
# Disabled by LaX — PHP is served via FastCGI (fcgid.conf) so versions can switch.
# LoadModule php_module `"$UnixRoot/bin/php/$PhpDefault/php8apache2_4.dll`"
# PHPIniDir `"$UnixRoot/bin/php/$PhpDefault`"
"@ | Set-Content -Path $modPhp -Encoding UTF8
}

$httpd = "$Dst\bin\apache\Apache24\conf\httpd.conf"
if (Test-Path $httpd) {
    $h = Get-Content $httpd -Raw
    $h = $h -replace 'AllowOverride None', 'AllowOverride All'
    $h = $h -replace 'Include "C:/Laragon/www/LaX/etc/apache2/mod_php.conf"', "Include `"$UnixRoot/etc/apache2/fcgid.conf`"`r`nInclude `"$UnixRoot/etc/apache2/mod_php.conf`""
    if ($h -notmatch 'fcgid\.conf') {
        $h = $h -replace 'Include "C:/Laragon/etc/apache2/mod_php.conf"', "Include `"$UnixRoot/etc/apache2/fcgid.conf`"`r`nInclude `"$UnixRoot/etc/apache2/mod_php.conf`""
    }
    Set-Content -Path $httpd -Value $h -Encoding UTF8
}

$myIni = "$Dst\bin\mysql\mariadb-10.11.13\my.ini"
@"
[client]
port=3306
socket=/tmp/mysql.sock

[mysqld]
datadir="$UnixRoot/data/mariadb"
port=3306
socket=/tmp/mysql.sock
skip-external-locking
character-set-server=utf8mb4
collation-server=utf8mb4_general_ci
bind-address=127.0.0.1
innodb_buffer_pool_size=256M
max_allowed_packet=512M
key_buffer_size=32M
skip-log-bin
tmpdir="$UnixRoot/tmp"

[mysqldump]
quick
max_allowed_packet=512M
"@ | Set-Content -Path $myIni -Encoding ASCII

$upstream = @"
upstream php_upstream {
	server 127.0.0.1:9003 weight=1 max_fails=1 fail_timeout=1;
	server 127.0.0.1:9004 weight=1 max_fails=1 fail_timeout=1;
}
"@
Set-Content -Path "$Dst\etc\nginx\php_upstream.conf" -Value $upstream -Encoding ASCII

$nginxDefault = @"
server {
    listen 80 default_server;
    server_name localhost;
    root "$UnixRoot/www";
    index index.html index.htm index.php;
    client_max_body_size 2000M;
    include "$UnixRoot/etc/nginx/alias/*.conf";

    location / {
        try_files `$uri `$uri/ /index.php`$is_args`$args;
        autoindex on;
    }

    location ~ \.php`$ {
        include snippets/fastcgi-php.conf;
        fastcgi_pass php_upstream;
    }

    location = /favicon.ico { access_log off; log_not_found off; }
    location ~ /\.ht { deny all; }
}
"@
Set-Content -Path "$Dst\etc\nginx\sites-enabled\00-default.conf" -Value $nginxDefault -Encoding ASCII

$apacheDefault = @"
<VirtualHost *:80>
    DocumentRoot "$UnixRoot/www"
    ServerName localhost
    <Directory "$UnixRoot/www">
        Options Indexes FollowSymLinks Includes ExecCGI
        AllowOverride All
        Require all granted
    </Directory>
</VirtualHost>
"@
Set-Content -Path "$Dst\etc\apache2\sites-enabled\00-default.conf" -Value $apacheDefault -Encoding ASCII

# phpMyAdmin alias already copied; make sure tmp is writable
New-Item -ItemType Directory -Force -Path "$Dst\etc\apps\phpMyAdmin\tmp" | Out-Null

Write-Host "==> Initializing MariaDB datadir"
$mysqlBin = "$Dst\bin\mysql\mariadb-10.11.13\bin"
$dataDir = "$Dst\data\mariadb"
$ibdata = Join-Path $dataDir "ibdata1"
if (-not (Test-Path $ibdata)) {
    $installers = @(
        (Join-Path $mysqlBin "mariadb-install-db.exe"),
        (Join-Path $mysqlBin "mysql_install_db.exe")
    )
    $ok = $false
    foreach ($exe in $installers) {
        if (Test-Path $exe) {
            Write-Host "running $exe"
            & $exe --datadir="$dataDir" --password= 2>&1 | Write-Host
            if ($LASTEXITCODE -eq 0 -or (Test-Path $ibdata)) { $ok = $true; break }
        }
    }
    if (-not $ok) {
        $mysqld = Join-Path $mysqlBin "mysqld.exe"
        Write-Host "fallback mysqld --initialize-insecure"
        & $mysqld --datadir="$dataDir" --initialize-insecure --console 2>&1 | Write-Host
    }
} else {
    Write-Host "datadir already initialized"
}

New-Item -ItemType Directory -Force -Path "$Dst\bin\apache\Apache24\logs" | Out-Null
New-Item -ItemType Directory -Force -Path "$Dst\bin\nginx\nginx-1.14.0\logs" | Out-Null

Write-Host "==> Bootstrap complete"
Write-Host "PHP default: $PhpDefault"
Write-Host "Root: $Dst"
