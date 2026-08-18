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
if [[ -d "$ROOT/etc" ]]; then
  mkdir -p "$DEST/etc"
  cp -a "$ROOT/etc/." "$DEST/etc/"
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
