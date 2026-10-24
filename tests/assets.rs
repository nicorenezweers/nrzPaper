//! Eingebettete Oberfläche: Tailwind-CSS ist aktuell, alles kommt vom eigenen Server.

use std::process::Command;

/// static/app.css muss zu Templates und Skripten passen. Prüft nur, wenn die CLI da ist
/// (scripts/tailwind.sh legt sie nach .tools/); der Docker-Build erzeugt die Datei ohnehin neu.
#[test]
fn tailwind_css_is_current() {
    let cli = std::path::Path::new(".tools/tailwindcss");
    if !cli.exists() {
        eprintln!("Tailwind-CLI fehlt – übersprungen (scripts/tailwind.sh lädt sie)");
        return;
    }
    let out = tempfile::NamedTempFile::new().unwrap();
    let status = Command::new(cli)
        .args([
            "-c",
            "tailwind.config.js",
            "-i",
            "assets/app.css",
            "--minify",
            "-o",
        ])
        .arg(out.path())
        .stderr(std::process::Stdio::null())
        .status()
        .unwrap();
    assert!(status.success());
    let fresh = std::fs::read(out.path()).unwrap();
    let committed = std::fs::read("static/app.css").unwrap();
    assert!(
        fresh == committed,
        "static/app.css ist veraltet – scripts/tailwind.sh ausführen"
    );
}

#[test]
fn css_uses_only_local_fonts() {
    let css = std::fs::read_to_string("static/app.css").unwrap();
    assert!(css.contains("/static/fonts/inter-latin.woff2"));
    assert!(
        !css.contains("url(http") && !css.contains("@import"),
        "keine fremden Quellen"
    );
}
