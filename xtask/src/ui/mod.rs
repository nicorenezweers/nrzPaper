//! Oberfläche (Dioxus, ui/) bauen und nach ui/dist/public legen – build.rs bettet sie ins
//! Programm ein. Vor jedem Release bzw. Docker-Build ausführen.
//! Braucht: rustup mit Ziel wasm32-unknown-unknown und die Dioxus-CLI (`dx`, Version 0.7).
//!
//! `dx bundle` räumt seinen Zwischenordner (target/dx/…/public) nicht auf und kopiert alles
//! daraus mit; darum den vorher leeren, in einen frischen Ordner bauen und den Zielordner
//! danach komplett ersetzen (sonst landen alte WASM-Dateien mit im Programm).

mod index;
mod tailwind;

use std::path::Path;
use std::process::{Command, ExitCode};
use std::time::{SystemTime, UNIX_EPOCH};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// ui/index.html und ui/tailwind.css schreiben – beide liest `dx` vor dem Übersetzen.
pub fn write_files() -> Result<()> {
    let ui = crate::root().join("ui");
    std::fs::write(ui.join("index.html"), index::INDEX_HTML)?;
    std::fs::write(ui.join("tailwind.css"), tailwind::TAILWIND_CSS)?;
    Ok(())
}

pub fn build() -> Result<ExitCode> {
    write_files()?;
    let ui = crate::root().join("ui");

    let home = std::env::var("HOME").unwrap_or_default();
    let mut path = std::env::var_os("PATH").unwrap_or_default();
    for extra in [format!("{home}/.cargo/bin"), format!("{home}/.rustup/toolchains/stable-aarch64-apple-darwin/bin")] {
        if Path::new(&extra).is_dir() {
            let mut p = std::ffi::OsString::from(extra);
            p.push(":");
            p.push(&path);
            path = p;
        }
    }

    let stale = ui.join("target/dx/nrzpaper-ui/release/web/public");
    if stale.exists() {
        std::fs::remove_dir_all(&stale)?;
    }
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let out = format!("target/ui-bundle-{stamp}");
    let status = Command::new("dx")
        .args(["bundle", "--web", "--release", "--out-dir", &out])
        .current_dir(&ui)
        .env("PATH", path)
        .status()?;
    if !status.success() {
        return Err(format!("dx bundle fehlgeschlagen ({status})").into());
    }

    let dest = ui.join("dist/public");
    if dest.exists() {
        std::fs::remove_dir_all(&dest)?;
    }
    copy_dir(&ui.join(&out).join("public"), &dest)?;
    println!("Oberfläche gebaut: ui/dist/public ({} KB)", dir_size(&dest)? / 1024);
    Ok(ExitCode::SUCCESS)
}

fn copy_dir(from: &Path, to: &Path) -> Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

fn dir_size(dir: &Path) -> Result<u64> {
    let mut sum = 0;
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        sum += if entry.file_type()?.is_dir() { dir_size(&entry.path())? } else { entry.metadata()?.len() };
    }
    Ok(sum)
}
