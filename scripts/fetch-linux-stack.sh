#!/usr/bin/env bash
# Download a portable Linux stack into bin/ — same binaries on Ubuntu, Fedora,
# Debian, openSUSE (glibc). Alpine is not supported (MariaDB tarball is glibc).
#
# Apache is not vendored: there is no portable httpd build. Linux defaults to Nginx.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TMP="$ROOT/tmp/fetch-linux"
SKIP_MARIADB=0

PHP_VER="${LAX_PHP_VER:-8.4.23}"
NGINX_VER="${LAX_NGINX_VER:-1.28.3}"
MARIADB_VER="${LAX_MARIADB_VER:-11.4.12}"
NODE_VER="${LAX_NODE_VER:-22.23.2}"

usage() {
  echo "Usage: $0 [--skip-mariadb]"
  echo "  Fetches Node, PHP (cli+fpm), Nginx, Mailpit, Composer, MariaDB into bin/"
}

for arg in "$@"; do
  case "$arg" in
    --skip-mariadb) SKIP_MARIADB=1 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown arg: $arg" >&2; usage; exit 1 ;;
  esac
done

need() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "need $1 (Ubuntu: sudo apt install curl tar gzip; Fedora: sudo dnf install curl tar gzip)" >&2
    exit 1
  }
}
need curl
need tar
need gzip

ARCH="$(uname -m)"
case "$ARCH" in
  x86_64|amd64)
    ARCH=x86_64
    NODE_ARCH=x64
    MAILPIT_ARCH=amd64
    MARIADB_ARCH=x86_64
    ;;
  aarch64|arm64)
    ARCH=aarch64
    NODE_ARCH=arm64
    MAILPIT_ARCH=arm64
    MARIADB_ARCH=aarch64
    ;;
  *)
    echo "unsupported arch: $ARCH (need x86_64 or aarch64)" >&2
    exit 1
    ;;
esac

mkdir -p "$TMP" "$ROOT/bin" "$ROOT/usr" "$ROOT/etc/nginx/sites-enabled" \
  "$ROOT/etc/apache2/sites-enabled" "$ROOT/www" "$ROOT/data/mariadb" "$ROOT/logs" "$ROOT/tmp"

get() {
  local url="$1" out="$2"
  echo "GET $url"
  curl -fL --retry 3 --retry-delay 2 -o "$out" "$url"
}

extract_first_bin() {
  local archive="$1" dest="$2" want="$3"
  local unpack="$TMP/unpack-$$"
  rm -rf "$unpack"
  mkdir -p "$unpack"
  tar -xzf "$archive" -C "$unpack"
  local found
  found="$(find "$unpack" -type f \( -name "$want" -o -name "${want}-*" \) | head -n 1 || true)"
  if [[ -z "$found" ]]; then
    found="$(find "$unpack" -type f -executable | head -n 1 || true)"
  fi
  if [[ -z "$found" ]]; then
    echo "no binary named $want in $archive" >&2
    find "$unpack" -type f | head
    exit 1
  fi
  mkdir -p "$(dirname "$dest")"
  cp -f "$found" "$dest"
  chmod +x "$dest"
  rm -rf "$unpack"
}

# --- PHP (static musl: Ubuntu + Fedora + Debian) ---
PHP_MAJOR_MINOR="$(echo "$PHP_VER" | awk -F. '{print $1"."$2}')"
PHP_DIR="$ROOT/bin/php/php-${PHP_MAJOR_MINOR}"
if [[ ! -x "$PHP_DIR/php" || ! -x "$PHP_DIR/php-fpm" ]]; then
  echo "==> PHP ${PHP_VER} (${ARCH})"
  mkdir -p "$PHP_DIR"
  local_cli="$TMP/php-cli.tar.gz"
  local_fpm="$TMP/php-fpm.tar.gz"
  base="https://dl.static-php.dev/static-php-cli/common"
  get "$base/php-${PHP_VER}-cli-linux-${ARCH}.tar.gz" "$local_cli"
  get "$base/php-${PHP_VER}-fpm-linux-${ARCH}.tar.gz" "$local_fpm"
  extract_first_bin "$local_cli" "$PHP_DIR/php" "php"
  extract_first_bin "$local_fpm" "$PHP_DIR/php-fpm" "php-fpm"
  if [[ ! -f "$PHP_DIR/php.ini" ]]; then
    cat > "$PHP_DIR/php.ini" <<'INI'
[PHP]
display_errors = On
display_startup_errors = On
error_reporting = E_ALL
memory_limit = 256M
upload_max_filesize = 128M
post_max_size = 128M
max_execution_time = 120
date.timezone = UTC
cgi.fix_pathinfo = 1

[mail function]
SMTP = 127.0.0.1
smtp_port = 1025
sendmail_from = lax@localhost

[mysqli]
mysqli.default_host = 127.0.0.1
mysqli.default_port = 3306

[Pdo_mysql]
pdo_mysql.default_socket =
INI
  fi
  echo "ok $PHP_DIR/php"
else
  echo "skip PHP (already in $PHP_DIR)"
fi

# --- Nginx (static musl from jirutka/nginx-binaries) ---
NGINX_DIR="$ROOT/bin/nginx/nginx-${NGINX_VER}"
if [[ ! -x "$NGINX_DIR/nginx" ]]; then
  echo "==> Nginx ${NGINX_VER} (${ARCH})"
  mkdir -p "$NGINX_DIR/conf" "$NGINX_DIR/logs" "$NGINX_DIR/html"
  get "https://jirutka.github.io/nginx-binaries/nginx-${NGINX_VER}-${ARCH}-linux" "$NGINX_DIR/nginx"
  chmod +x "$NGINX_DIR/nginx"
  echo "<html><body>LaX nginx</body></html>" > "$NGINX_DIR/html/index.html"
  echo "ok $NGINX_DIR/nginx"
else
  echo "skip Nginx (already in $NGINX_DIR)"
fi

# --- Node ---
NODE_DIR="$ROOT/bin/node"
if [[ ! -x "$NODE_DIR/bin/node" && ! -x "$NODE_DIR/node" ]]; then
  echo "==> Node ${NODE_VER} (${NODE_ARCH})"
  zip="$TMP/node.tar.gz"
  get "https://nodejs.org/dist/v${NODE_VER}/node-v${NODE_VER}-linux-${NODE_ARCH}.tar.gz" "$zip"
  unpack="$TMP/node"
  rm -rf "$unpack"
  mkdir -p "$unpack"
  tar -xzf "$zip" -C "$unpack"
  inner="$(find "$unpack" -mindepth 1 -maxdepth 1 -type d | head -n 1)"
  rm -rf "$NODE_DIR"
  mv "$inner" "$NODE_DIR"
  chmod +x "$NODE_DIR/bin/node"
  echo "ok $NODE_DIR/bin/node"
else
  echo "skip Node (already in $NODE_DIR)"
fi

# --- Mailpit ---
MAIL_DIR="$ROOT/bin/mailpit"
if [[ ! -x "$MAIL_DIR/mailpit" ]]; then
  echo "==> Mailpit (${MAILPIT_ARCH})"
  mkdir -p "$MAIL_DIR"
  zip="$TMP/mailpit.tar.gz"
  get "https://github.com/axllent/mailpit/releases/latest/download/mailpit-linux-${MAILPIT_ARCH}.tar.gz" "$zip"
  tar -xzf "$zip" -C "$MAIL_DIR"
  chmod +x "$MAIL_DIR/mailpit"
  echo "ok $MAIL_DIR/mailpit"
else
  echo "skip Mailpit (already in $MAIL_DIR)"
fi

# --- DbGate web ---
DBGATE_DIR="$ROOT/usr/apps/dbgate"
DBGATE_JS="$DBGATE_DIR/node_modules/dbgate-serve/bin/dbgate-serve.js"
if [[ ! -f "$DBGATE_JS" ]]; then
  echo "==> DbGate (dbgate-serve)"
  mkdir -p "$DBGATE_DIR"
  if [[ ! -f "$DBGATE_DIR/package.json" ]]; then
    cat > "$DBGATE_DIR/package.json" <<'EOF'
{
  "name": "lax-dbgate",
  "private": true,
  "dependencies": {
    "dbgate-serve": "7.2.5"
  }
}
EOF
  fi
  NODE_BIN="$NODE_DIR/bin/node"
  NPM_BIN="$NODE_DIR/bin/npm"
  [[ -x "$NODE_DIR/node" ]] && NODE_BIN="$NODE_DIR/node"
  [[ -x "$NODE_DIR/npm" ]] && NPM_BIN="$NODE_DIR/npm"
  if [[ ! -x "$NODE_BIN" ]]; then
    echo "Node missing — skip DbGate" >&2
  else
    (cd "$DBGATE_DIR" && PATH="$(dirname "$NODE_BIN"):$PATH" "$NPM_BIN" install --omit=dev --no-fund --no-audit)
    echo "ok $DBGATE_JS"
  fi
else
  echo "skip DbGate (already in $DBGATE_DIR)"
fi

# --- Composer ---
COMP_DIR="$ROOT/bin/composer"
if [[ ! -f "$COMP_DIR/composer.phar" ]]; then
  echo "==> Composer"
  mkdir -p "$COMP_DIR"
  get "https://getcomposer.org/download/latest-stable/composer.phar" "$COMP_DIR/composer.phar"
  cat > "$COMP_DIR/composer" <<'SH'
#!/bin/sh
DIR=$(CDPATH= cd -- "$(dirname "$0")" && pwd)
PHP=$(command -v php)
if [ -z "$PHP" ]; then
  echo "php not on PATH" >&2
  exit 1
fi
exec "$PHP" "$DIR/composer.phar" "$@"
SH
  chmod +x "$COMP_DIR/composer"
  echo "ok $COMP_DIR/composer.phar"
else
  echo "skip Composer (already in $COMP_DIR)"
fi

# --- MariaDB generic glibc tarball ---
MYSQL_DIR="$ROOT/bin/mysql/mariadb-${MARIADB_VER}"
if [[ "$SKIP_MARIADB" -eq 1 ]]; then
  echo "skip MariaDB (--skip-mariadb)"
elif [[ -x "$MYSQL_DIR/bin/mariadbd" || -x "$MYSQL_DIR/bin/mysqld" ]]; then
  echo "skip MariaDB (already in $MYSQL_DIR)"
else
  echo "==> MariaDB ${MARIADB_VER} (${MARIADB_ARCH}) — ~360MB"
  zip="$TMP/mariadb.tar.gz"
  name="mariadb-${MARIADB_VER}-linux-systemd-${MARIADB_ARCH}"
  url="https://archive.mariadb.org/mariadb-${MARIADB_VER}/bintar-linux-systemd-${MARIADB_ARCH}/${name}.tar.gz"
  get "$url" "$zip"
  unpack="$TMP/mariadb"
  rm -rf "$unpack"
  mkdir -p "$unpack"
  tar -xzf "$zip" -C "$unpack"
  inner="$(find "$unpack" -mindepth 1 -maxdepth 1 -type d | head -n 1)"
  mkdir -p "$ROOT/bin/mysql"
  rm -rf "$MYSQL_DIR"
  mv "$inner" "$MYSQL_DIR"
  echo "ok $MYSQL_DIR"
fi

# --- lax.toml defaults for Linux (do not clobber an existing file) ---
TOML="$ROOT/usr/lax.toml"
if [[ ! -f "$TOML" ]]; then
  cat > "$TOML" <<EOF
documentRoot = "www"
tld = "localhost"
autoVhost = false
webServer = "nginx"
apachePort = 8080
nginxPort = 8080
mysqlPort = 3306
phpVersion = "php-${PHP_MAJOR_MINOR}"
mysqlVersion = "mariadb-${MARIADB_VER}"
nginxVersion = "nginx-${NGINX_VER}"
apacheVersion = "Apache24"
phpCgiPorts = [9003]
autoStart = false
mysqlEnabled = true
theme = "noir"
dbAdmin = "phpmyadmin"
EOF
  echo "wrote $TOML"
fi

echo
echo "done. layout:"
echo "  PHP     $PHP_DIR"
echo "  Nginx   $NGINX_DIR"
echo "  Node    $NODE_DIR"
echo "  Mailpit $MAIL_DIR"
echo "  DbGate  $DBGATE_DIR"
echo "  Composer $COMP_DIR"
[[ "$SKIP_MARIADB" -eq 0 ]] && echo "  MariaDB $MYSQL_DIR"
echo
echo "Ubuntu/Fedora: unpack LaX next to these folders, run ./lax or the AppImage."
echo "Port 80 needs root/capabilities — default is 8080 → http://localhost:8080/"
