// Migrationen sind ins Programm eingebettet (sqlx::migrate!) – bei Änderungen neu übersetzen.
//
// Die Oberfläche (Dioxus, `ui/`) wird mit `scripts/ui.sh` nach `ui/dist/public` gebaut und hier
// ins Programm eingebettet: Für jede Datei entsteht ein Eintrag in `$OUT_DIR/spa.rs`, Text und
// WebAssembly zusätzlich gzip-komprimiert. Fehlt der Ordner, bleibt die Liste leer und der Server
// zeigt einen Hinweis statt der Oberfläche.
use std::io::Write;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=migrations");
    println!("cargo:rerun-if-changed=ui/dist/public");
    println!("cargo:rerun-if-changed=locales");

    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("ui/dist/public");
    let mut files = Vec::new();
    if root.join("index.html").is_file() {
        collect(&root, &root, &mut files);
    }
    files.sort();

    let mut code = String::from("pub static FILES: &[SpaFile] = &[\n");
    for (i, rel) in files.iter().enumerate() {
        let src = root.join(rel);
        println!("cargo:rerun-if-changed={}", src.display());
        let data = std::fs::read(&src).unwrap();
        let gz = if compressible(rel) {
            let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::best());
            enc.write_all(&data).unwrap();
            let gz = enc.finish().unwrap();
            let path = out.join(format!("spa-{i}.gz"));
            std::fs::write(&path, gz).unwrap();
            format!("Some(include_bytes!({:?}))", path.display().to_string())
        } else {
            "None".into()
        };
        code += &format!(
            "    SpaFile {{ path: {rel:?}, body: include_bytes!({:?}), gzip: {gz} }},\n",
            src.display().to_string()
        );
    }
    code += "];\n";
    std::fs::write(out.join("spa.rs"), code).unwrap();

    // Übersetzungen (locales/<sprache>.json, deutsch → Sprache) für die Oberfläche
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("locales");
    let mut locales: Vec<(String, PathBuf)> = std::fs::read_dir(&dir)
        .map(|r| r.flatten().map(|e| e.path()).collect::<Vec<_>>())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|p| {
            let stem = p.file_stem()?.to_str()?.to_string();
            (p.extension()? == "json" && stem.len() == 2).then_some((stem, p))
        })
        .collect();
    locales.sort();
    let mut code = String::from("pub static LOCALES: &[(&str, &[u8])] = &[\n");
    for (lang, path) in &locales {
        println!("cargo:rerun-if-changed={}", path.display());
        code += &format!("    ({lang:?}, include_bytes!({:?})),\n", path.display().to_string());
    }
    code += "];\n";
    std::fs::write(out.join("locales.rs"), code).unwrap();
}

fn collect(root: &Path, dir: &Path, files: &mut Vec<String>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect(root, &path, files);
        } else {
            let rel = path.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/");
            files.push(rel);
        }
    }
}

fn compressible(path: &str) -> bool {
    [".html", ".js", ".css", ".wasm", ".svg", ".json"]
        .iter()
        .any(|e| path.ends_with(e))
}
