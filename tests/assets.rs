//! Oberfläche (Dioxus): nur eigene Quellen – Schrift und Stile kommen vom eigenen Server.

/// Die Stil-Quelle der App bindet nur die eingebettete Schrift ein, nichts von außen.
#[test]
fn styles_use_only_local_fonts() {
    let css = std::fs::read_to_string("ui/tailwind.css").unwrap();
    assert!(css.contains("/static/fonts/inter-latin.woff2"));
    assert!(!css.contains("url(http"), "keine fremden Quellen");
    for line in css.lines().filter(|l| l.trim_start().starts_with("@import")) {
        assert!(line.contains("\"tailwindcss\""), "nur Tailwind selbst: {line}");
    }
}

/// Die App lädt keine Skripte oder Schriften aus dem Netz (kein CDN).
#[test]
fn app_has_no_remote_sources() {
    for entry in walk("ui/src") {
        let code = std::fs::read_to_string(&entry).unwrap();
        for bad in ["https://cdn", "fonts.googleapis", "unpkg.com", "jsdelivr"] {
            assert!(!code.contains(bad), "{} lädt {bad}", entry.display());
        }
    }
}

fn walk(dir: &str) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![std::path::PathBuf::from(dir)];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "rs") {
                out.push(p);
            }
        }
    }
    out
}
