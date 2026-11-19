//! Sammelt alle deutschen Texte, die übersetzt werden: tr("…")/trf("…") der Oberfläche,
//! feste Meldungen des Servers (AppError::bad/forbidden/Conflict, Err("…")) und die Felder der
//! KI. Schreibt locales/strings.json (sortiert) und meldet je Sprache, was fehlt oder veraltet ist.

use regex::Regex;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const LIT: &str = r#""((?:[^"\\]|\\.)*)""#;

fn re(pattern: &str) -> Regex {
    Regex::new(pattern).unwrap()
}

/// Inhalt eines Rust-Literals wie ein JSON-String lesen (\" \\ \n …).
fn unescape(s: &str) -> String {
    serde_json::from_str(&format!("\"{s}\"")).unwrap_or_else(|e| panic!("Literal {s:?}: {e}"))
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.file_name().is_some_and(|n| n.to_string_lossy().starts_with('.')) {
            continue;
        }
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// Die letzten `n` Zeichen (nicht Bytes) von `s`.
fn tail(s: &str, n: usize) -> &str {
    match s.char_indices().rev().nth(n - 1) {
        Some((i, _)) => &s[i..],
        None => s,
    }
}

fn ui_strings(root: &Path, out: &mut BTreeSet<String>) {
    let call = re(&format!(r"\btrf?\(\s*{LIT}"));
    let table = re(&format!(r#"\("[^"]*",\s*{LIT}\)"#));
    let table_name = re("(OPERATORS|PROPERTY_TYPES|WEBHOOK_EVENTS|PERMISSIONS|ITEMS)");
    let field = re(&format!(r"(?:label|group):\s*{LIT}"));
    let mut files = Vec::new();
    rust_files(&root.join("ui/src"), &mut files);
    for path in files {
        let src = read(&path);
        for c in call.captures_iter(&src) {
            out.insert(unescape(&c[1]));
        }
        // Tabellen, die erst beim Anzeigen übersetzt werden
        for c in table.captures_iter(&src) {
            let start = c.get(0).unwrap().start();
            if table_name.is_match(tail(&src[..start], 3000)) {
                out.insert(unescape(&c[1]));
            }
        }
        for c in field.captures_iter(&src) {
            out.insert(unescape(&c[1]));
        }
    }
    // match-Arme in Beschriftungs-Funktionen (tr(match … { "x" => "Text" }))
    let models = read(&root.join("ui/src/models.rs"));
    let arm = re(&format!(r"=>\s*{LIT}"));
    for block in re(r"(?s)crate::i18n::tr\(match[^}]*\}").find_iter(&models) {
        for c in arm.captures_iter(block.as_str()) {
            out.insert(unescape(&c[1]));
        }
    }
    let admin = read(&root.join("ui/src/pages/settings/admin.rs"));
    for c in re(r#"=>\s*"(Lokal \(eigener Rechner\)|Eigene Adresse|Cloud)""#).captures_iter(&admin) {
        out.insert(c[1].to_string());
    }
}

fn server_strings(root: &Path, out: &mut BTreeSet<String>) {
    let letter = re("[A-Za-zÄÖÜäöü]");
    let base: Vec<Regex> = [
        format!(r"AppError::(?:bad|forbidden)\(\s*{LIT}"),
        format!(r"AppError::Conflict\(\s*{LIT}"),
        format!(r"AppError::unique\([^,]+,\s*{LIT}"),
        format!(r"Err\(\s*{LIT}\s*\.into\(\)"),
        format!(r"Some\(\s*{LIT}\s*\.into\(\)\)"),
    ]
    .iter()
    .map(|p| re(p))
    .collect();
    let catchers = re(&format!(r"\b(?:400|401|403|404|413|422|_) => {LIT}"));
    let mut files = Vec::new();
    rust_files(&root.join("src"), &mut files);
    for path in files {
        let src = read(&path);
        let extra = path.to_string_lossy().ends_with("catchers.rs").then_some(&catchers);
        for p in base.iter().chain(extra) {
            for c in p.captures_iter(&src) {
                let s = unescape(&c[1]);
                if !s.contains('{') && letter.is_match(&s) {
                    out.insert(s);
                }
            }
        }
    }
    let ai = read(&root.join("src/ai.rs"));
    let meta = &ai[ai.find("META_FIELDS").expect("META_FIELDS fehlt in src/ai.rs")..];
    for c in re(&format!(r#"\("\w+",\s*{LIT}\)"#)).captures_iter(meta) {
        out.insert(unescape(&c[1]));
    }
    for c in re(r#"(?:name|"[a-z]+", ")([^"]+\(lokal\)|Eigene OpenAI-kompatible API)""#).captures_iter(&ai) {
        out.insert(c[1].to_string());
    }
    for c in re(r#""([^"]+\(lokal\))""#).captures_iter(&ai) {
        out.insert(c[1].to_string());
    }
}

fn placeholders(s: &str) -> Vec<String> {
    let mut v: Vec<String> = re(r"\{\d+\}").find_iter(s).map(|m| m.as_str().to_string()).collect();
    v.sort();
    v
}

pub fn run(check: bool) -> Result<ExitCode> {
    let root = crate::root();
    let mut set = BTreeSet::new();
    ui_strings(&root, &mut set);
    server_strings(&root, &mut set);
    let strings: Vec<String> = set.into_iter().collect();

    let locales = root.join("locales");
    if !check {
        std::fs::create_dir_all(&locales)?;
        // Ein Text pro Zeile, ohne Einrückung – kleine Diffs beim Hinzufügen
        let items: Vec<String> = strings.iter().map(|s| serde_json::to_string(s).unwrap()).collect();
        let body = if items.is_empty() { "[]\n".to_string() } else { format!("[\n{}\n]\n", items.join(",\n")) };
        std::fs::write(locales.join("strings.json"), body)?;
    }
    println!("{} Texte", strings.len());

    let mut files: Vec<PathBuf> = std::fs::read_dir(&locales)?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    files.sort();
    let mut incomplete = false;
    for path in files {
        let code = path.file_stem().unwrap().to_string_lossy().to_string();
        if code == "strings" {
            continue;
        }
        let cat: serde_json::Map<String, Value> = serde_json::from_str(&read(&path))?;
        let filled = |v: Option<&Value>| match v {
            None | Some(Value::Null) | Some(Value::Bool(false)) => false,
            Some(Value::String(s)) => !s.is_empty(),
            Some(Value::Array(a)) => !a.is_empty(),
            Some(Value::Object(o)) => !o.is_empty(),
            Some(Value::Number(n)) => n.as_f64() != Some(0.0),
            Some(Value::Bool(true)) => true,
        };
        let missing = strings.iter().filter(|s| !filled(cat.get(*s))).count();
        let stale = cat.keys().filter(|k| strings.binary_search(k).is_err()).count();
        let bad: Vec<&String> = cat
            .iter()
            .filter(|(k, v)| filled(Some(v)) && v.as_str().is_some_and(|v| placeholders(k) != placeholders(v)))
            .map(|(k, _)| k)
            .collect();
        if missing > 0 || !bad.is_empty() {
            incomplete = true;
        }
        let mut line = format!("  {code}: {}/{}", strings.len() - missing, strings.len());
        if stale > 0 {
            line += &format!(", {stale} veraltet");
        }
        if !bad.is_empty() {
            line += &format!(", {} mit falschen Platzhaltern: {:?}", bad.len(), &bad[..bad.len().min(3)]);
        }
        println!("{line}");
    }
    Ok(if check && incomplete { ExitCode::FAILURE } else { ExitCode::SUCCESS })
}
