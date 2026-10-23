# nrzPaper

Selbst gehostete Dokumentenverwaltung im Funktionsumfang von [Papra](https://papra.app) – komplett in Rust.
Alle Daten, auch die Dokumente selbst, liegen in **einer einzigen, mit SQLCipher verschlüsselten SQLite-Datei**.

> Stand: Schritt 1 von 7 – Architektur, Abhängigkeiten, Projektgerüst. Aufbau und Plan: [ARCHITECTURE.md](ARCHITECTURE.md).

## Starten (Entwicklung)

```bash
cargo run
# http://localhost:8320/api/v1/health
```

Beim ersten Start entsteht `data/nrzpaper.db` (verschlüsselt) und `data/nrzpaper.key`.
**Den Schlüssel sichern** – ohne ihn sind die Daten nicht mehr lesbar. Alternativ den Schlüssel
über `NRZPAPER_DB_KEY` vorgeben (Passphrase oder 64 Hex-Zeichen).

| Variable | Standard | Bedeutung |
|----------|----------|-----------|
| `NRZPAPER_PORT` | `8320` | HTTP-Port |
| `NRZPAPER_DATA_DIR` | `./data` | Datenbank, Schlüsseldatei, Eingangsordner |
| `NRZPAPER_DB_KEY` | – | Datenbank-Schlüssel; fehlt er, wird `nrzpaper.key` verwendet bzw. erzeugt |
| `NRZPAPER_OCR_LANGUAGES` | `deu+eng` | Sprachen für Tesseract |
| `NRZPAPER_TRASH_RETENTION_DAYS` | `30` | Aufbewahrung im Papierkorb |
| `NRZPAPER_LOG` | `info` | Log-Filter (`tracing`) |

## Tests

```bash
cargo test
```

`tests/sqlcipher.rs` prüft die Grundannahme: verschlüsselte Datei ohne lesbaren Inhalt, FTS5-Volltextsuche
mit Umlaut-Normalisierung, falscher Schlüssel wird abgewiesen.
