#!/usr/bin/env bash
# Pack a portable Linux tree (GUI binary + stack) into LaX-<ver>-linux-x86_64.tar.gz
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VER="$(node -p "require('$ROOT/package.json').version")"
ARCH="$(uname -m)"
case "$ARCH" in
  x86_64|amd64) ARCH=x86_64 ;;
  aarch64|arm64) ARCH=aarch64 ;;
esac
DEST="$ROOT/pack-linux"
BIN="$ROOT/src-tauri/target/release/lax"
if [[ ! -x "$BIN" ]]; then
  echo "missing $BIN — build the Linux GUI first (npx tauri build --bundles none)" >&2
  exit 1
fi
if [[ ! -x "$ROOT/bin/php/php-8.4/php" && ! -d "$ROOT/bin/php" ]]; then
  echo "run scripts/fetch-linux-stack.sh first" >&2
  exit 1
fi
rm -rf "$DEST"
mkdir -p "$DEST/tmp" "$DEST/logs" "$DEST/usr" "$DEST/www"
cp -a "$BIN" "$DEST/lax"
chmod +x "$DEST/lax"
cp -a "$ROOT/bin" "$DEST/bin"
cp -a "$ROOT/usr/." "$DEST/usr/"
rm -rf "$DEST/usr/apps/dbgate/node_modules"
if [[ -d "$ROOT/etc" ]]; then
  mkdir -p "$DEST/etc"
  cp -a "$ROOT/etc/." "$DEST/etc/"
fi
# phpMyAdmin LaX theme (icons from pmahomme)
PMA="$DEST/etc/apps/phpMyAdmin"
THEME_SRC="$ROOT/usr/themes/phpmyadmin/lax"
if [[ -f "$PMA/index.php" && -f "$THEME_SRC/theme.json" ]]; then
  HOMME="$PMA/themes/pmahomme"
  TDEST="$PMA/themes/lax"
  mkdir -p "$TDEST/css" "$TDEST/jquery" "$TDEST/fonts" "$TDEST/img"
  cp -a "$THEME_SRC/theme.json" "$TDEST/theme.json"
  if [[ -d "$HOMME/img" ]]; then cp -a "$HOMME/img/." "$TDEST/img/"; fi
  [[ -f "$THEME_SRC/img/logo.svg" ]] && cp -a "$THEME_SRC/img/logo.svg" "$TDEST/img/logo.svg"
  [[ -d "$THEME_SRC/fonts" ]] && cp -a "$THEME_SRC/fonts/." "$TDEST/fonts/"
  [[ -f "$HOMME/screen.png" ]] && cp -a "$HOMME/screen.png" "$TDEST/screen.png"
  OVER="$(cat "$THEME_SRC/css/lax.css")"
  if [[ -f "$HOMME/css/theme.css" ]]; then
    { cat "$HOMME/css/theme.css"; printf '\n\n/* ---- LaX ---- */\n%s\n' "$OVER"; } > "$TDEST/css/theme.css"
  fi
  if [[ -f "$HOMME/css/theme.rtl.css" ]]; then
    { cat "$HOMME/css/theme.rtl.css"; printf '\n\n/* ---- LaX ---- */\n%s\n' "$OVER"; } > "$TDEST/css/theme.rtl.css"
  fi
  if [[ -f "$HOMME/jquery/jquery-ui.css" ]]; then
    { cat "$HOMME/jquery/jquery-ui.css"; printf '\n\n/* ---- LaX ---- */\n%s\n' "$OVER"; } > "$TDEST/jquery/jquery-ui.css"
  fi
  CFG="$PMA/config.inc.php"
  if [[ -f "$CFG" ]]; then
    python3 - <<PY
from pathlib import Path
p = Path(r"$CFG")
t = p.read_text(encoding="utf-8", errors="replace")
mark = "\n/* LaX theme */\n"
i = t.find(mark)
if i >= 0:
    t = t[:i]
t += mark + "\$cfg['ThemeDefault'] = 'lax';\n\$cfg['NavigationDisplayLogo'] = true;\n\$cfg['NavigationWidth'] = 268;\n"
p.write_text(t, encoding="utf-8")
PY
  fi
fi
if [[ -f "$ROOT/logo.svg" ]]; then
  cp "$ROOT/logo.svg" "$DEST/logo.svg"
fi
if [[ -f "$ROOT/realese (1).zip" ]]; then
  python3 - <<PY || unzip -qo "$ROOT/realese (1).zip" -d "$DEST/www"
import zipfile
zipfile.ZipFile(r"$ROOT/realese (1).zip").extractall(r"$DEST/www")
PY
fi
echo "C:/Laragon/www/LaX" > "$DEST/usr/.install-root"
OUT="$ROOT/LaX-${VER}-linux-${ARCH}.tar.gz"
tar -C "$DEST" -czf "$OUT" .
echo "PACK=$OUT"
ls -lh "$OUT"
