#!/bin/sh
# static/app.css aus assets/app.css erzeugen – mit der eigenständigen Tailwind-CLI (kein Node nötig).
# Die CLI wird beim ersten Aufruf nach .tools/ geladen und gegen die Prüfsumme geprüft.
#   scripts/tailwind.sh          einmal bauen (minifiziert)
#   scripts/tailwind.sh --watch  bei Änderungen neu bauen
set -eu
cd "$(dirname "$0")/.."

VERSION=3.4.17
case "$(uname -s)-$(uname -m)" in
  Darwin-arm64) ASSET=macos-arm64; SHA=a1d0c7985759accca0bf12e51ac1dcbf0f6cf2fffb62e6e0f62d091c477a10a3 ;;
  Linux-x86_64) ASSET=linux-x64;   SHA=7d24f7fa191d2193b78cd5f5a42a6093e14409521908529f42d80b11fde1f1d4 ;;
  Linux-aarch64|Linux-arm64) ASSET=linux-arm64; SHA=69b1378b8133192d7d2feb12a116fa12d035594f58db3eff215879e4ad8cf39b ;;
  *) echo "Keine Tailwind-CLI für $(uname -s)-$(uname -m) hinterlegt." >&2; exit 1 ;;
esac

CLI=.tools/tailwindcss
if [ ! -x "$CLI" ] || [ "$(cat .tools/tailwindcss.sha 2>/dev/null)" != "$SHA" ]; then
  mkdir -p .tools
  echo "Lade Tailwind $VERSION ($ASSET) …" >&2
  curl -sSfL -o "$CLI.tmp" "https://github.com/tailwindlabs/tailwindcss/releases/download/v$VERSION/tailwindcss-$ASSET"
  echo "$SHA  $CLI.tmp" | shasum -a 256 -c - >/dev/null 2>&1 || echo "$SHA  $CLI.tmp" | sha256sum -c - >/dev/null
  chmod +x "$CLI.tmp" && mv "$CLI.tmp" "$CLI" && echo "$SHA" > .tools/tailwindcss.sha
fi

exec "$CLI" -c tailwind.config.js -i assets/app.css -o static/app.css --minify "$@"
