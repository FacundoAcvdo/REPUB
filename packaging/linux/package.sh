#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:-v0.1.0}"
ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
DIST_DIR="$ROOT_DIR/dist"
TAR_STAGE="$DIST_DIR/repub-${VERSION}-linux-x86_64"
APPDIR="$DIST_DIR/REPUB.AppDir"

mkdir -p "$DIST_DIR"
rm -rf "$TAR_STAGE" "$APPDIR"

if [[ ! -f "$ROOT_DIR/target/release/repub" ]]; then
  echo "Compilando binario release..."
  (cd "$ROOT_DIR" && cargo build --release --bin repub)
fi

# 1. Paquete portable .tar.gz
mkdir -p "$TAR_STAGE"
cp "$ROOT_DIR/target/release/repub" "$TAR_STAGE/repub"
cp "$ROOT_DIR/assets/repub.desktop" "$TAR_STAGE/repub.desktop"
cp "$ROOT_DIR/assets/app_icon.png" "$TAR_STAGE/repub.png"
cp "$ROOT_DIR/README.md" "$TAR_STAGE/README.md"
find "$ROOT_DIR/target/release" -maxdepth 1 -name "*.so*" -exec cp {} "$TAR_STAGE/" \; || true

(cd "$DIST_DIR" && tar -czf "repub-${VERSION}-linux-x86_64.tar.gz" "repub-${VERSION}-linux-x86_64")

# 2. AppImage (si appimagetool está disponible o se descarga en CI)
mkdir -p "$APPDIR/usr/bin" "$APPDIR/usr/share/icons/hicolor/512x512/apps" "$APPDIR/usr/share/applications"
cp "$ROOT_DIR/target/release/repub" "$APPDIR/usr/bin/repub"
find "$ROOT_DIR/target/release" -maxdepth 1 -name "*.so*" -exec cp {} "$APPDIR/usr/bin/" \; || true
cp "$ROOT_DIR/assets/app_icon.png" "$APPDIR/repub.png"
cp "$ROOT_DIR/assets/app_icon.png" "$APPDIR/usr/share/icons/hicolor/512x512/apps/repub.png"
cp "$ROOT_DIR/assets/repub.desktop" "$APPDIR/repub.desktop"
cp "$ROOT_DIR/assets/repub.desktop" "$APPDIR/usr/share/applications/repub.desktop"

cat > "$APPDIR/AppRun" <<'EOF'
#!/usr/bin/env bash
HERE="$(dirname "$(readlink -f "$0")")"
export LD_LIBRARY_PATH="$HERE/usr/bin:${LD_LIBRARY_PATH:-}"
exec "$HERE/usr/bin/repub" "$@"
EOF
chmod +x "$APPDIR/AppRun"

if ! command -v appimagetool >/dev/null 2>&1; then
  if [[ "${CI:-false}" == "true" ]]; then
    wget -q -O /tmp/appimagetool "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage"
    chmod +x /tmp/appimagetool
    APPIMAGETOOL="/tmp/appimagetool"
  else
    APPIMAGETOOL=""
  fi
else
  APPIMAGETOOL="appimagetool"
fi

if [[ -n "$APPIMAGETOOL" ]]; then
  ARCH=x86_64 "$APPIMAGETOOL" --appimage-extract-and-run "$APPDIR" "$DIST_DIR/repub-${VERSION}-x86_64.AppImage" || \
    ARCH=x86_64 "$APPIMAGETOOL" "$APPDIR" "$DIST_DIR/repub-${VERSION}-x86_64.AppImage"
fi

echo "Artefactos generados en $DIST_DIR"
