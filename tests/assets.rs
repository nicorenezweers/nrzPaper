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

/// Jede angebotene Sprache hat einen vollständigen Katalog mit denselben Platzhaltern.
#[test]
fn translations_are_complete() {
    let strings: Vec<String> = serde_json::from_str(&std::fs::read_to_string("locales/strings.json").unwrap()).unwrap();
    let ui = std::fs::read_to_string("ui/src/i18n.rs").unwrap();
    let start = ui.find("pub const LANGUAGES").unwrap();
    let block = &ui[start..start + ui[start..].find("];").unwrap()];
    let codes: Vec<&str> = block.split("(\"").skip(1).map(|p| &p[..p.find('"').unwrap()]).collect();
    assert!(codes.len() >= 7, "{codes:?}");
    let holes = |s: &str| {
        let mut v: Vec<String> = s.match_indices('{').filter_map(|(i, _)| {
            let rest = &s[i + 1..];
            let end = rest.find('}')?;
            rest[..end].chars().all(|c| c.is_ascii_digit()).then(|| rest[..end].to_string())
        }).collect();
        v.sort();
        v
    };
    for code in codes.iter().filter(|c| **c != "de") {
        let cat: std::collections::HashMap<String, String> =
            serde_json::from_str(&std::fs::read_to_string(format!("locales/{code}.json")).unwrap()).unwrap();
        for s in &strings {
            let t = cat.get(s).unwrap_or_else(|| panic!("{code}: fehlt „{s}“ – scripts/i18n_extract.py"));
            assert!(!t.trim().is_empty() || s.trim().is_empty(), "{code}: leer „{s}“");
            assert_eq!(holes(s), holes(t), "{code}: Platzhalter in „{s}“");
        }
    }
}

/// Die Liste der Texte passt zum Quelltext (sonst: scripts/i18n_extract.py ausführen).
#[test]
fn string_list_is_current() {
    let out = std::process::Command::new("python3").args(["scripts/i18n_extract.py", "--check"]).output();
    let Ok(out) = out else { return }; // ohne Python übersprungen
    let strings: Vec<String> = serde_json::from_str(&std::fs::read_to_string("locales/strings.json").unwrap()).unwrap();
    let report = String::from_utf8_lossy(&out.stdout);
    assert!(report.starts_with(&format!("{} Texte", strings.len())), "locales/strings.json veraltet: {report}");
}
