#!/bin/sh
# Oberfläche (Dioxus, ui/) bauen und nach ui/dist/public legen – build.rs bettet sie ins
# Programm ein. Vor jedem Release bzw. Docker-Build ausführen.
# Braucht: rustup mit Ziel wasm32-unknown-unknown und die Dioxus-CLI (`dx`, Version 0.7).
# `dx bundle` räumt seinen Zwischenordner (target/dx/…/public) nicht auf und kopiert alles
# daraus mit; darum den vorher leeren, in einen frischen Ordner bauen und mit rsync --delete
# übernehmen (sonst landen alte WASM-Dateien mit im Programm).
set -eu
cd "$(dirname "$0")/../ui"

if [ -d "$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin" ]; then
  PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
fi
PATH="$HOME/.cargo/bin:$PATH"
export PATH

rm -rf target/dx/nrzpaper-ui/release/web/public
out="target/ui-bundle-$(date +%s)"
dx bundle --web --release --out-dir "$out"
mkdir -p dist/public
rsync -a --delete "$out/public/" dist/public/
echo "Oberfläche gebaut: ui/dist/public ($(du -sh dist/public | cut -f1))"
