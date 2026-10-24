# nrzPaper

Selbst gehostete Dokumentenverwaltung im Funktionsumfang von [Papra](https://papra.app) – komplett in Rust.
Alle Daten, auch die Dokumente selbst, liegen in **einer einzigen, mit SQLCipher verschlüsselten SQLite-Datei**.

> Stand: Schritt 5 von 7 – Oberfläche mit Live-Suche, PDF-Viewer, Tags und Papierkorb; Upload, Texterkennung, Eingangsordner, Dokument-API. Aufbau und Plan: [ARCHITECTURE.md](ARCHITECTURE.md).

## Starten (Entwicklung)

```bash
cargo run
# http://localhost:8320 – beim ersten Aufruf das Administrator-Konto anlegen
```

Beim ersten Start entsteht `data/nrzpaper.db` (verschlüsselt) und `data/nrzpaper.key`.
**Den Schlüssel sichern** – ohne ihn sind die Daten nicht mehr lesbar. Alternativ den Schlüssel
über `NRZPAPER_DB_KEY` vorgeben (Passphrase oder 64 Hex-Zeichen).

| Variable | Standard | Bedeutung |
|----------|----------|-----------|
| `NRZPAPER_PORT` | `8320` | HTTP-Port |
| `NRZPAPER_DATA_DIR` | `./data` | Datenbank, Schlüsseldatei, Eingangsordner |
| `NRZPAPER_DB_KEY` | – | Datenbank-Schlüssel; fehlt er, wird `nrzpaper.key` verwendet bzw. erzeugt |
| `NRZPAPER_OCR_LANGUAGES` | `deu+eng` | Sprachen für Tesseract (fehlende werden weggelassen) |
| `NRZPAPER_OCR_WORKERS` | `2` | Gleichzeitige Texterkennungen |
| `NRZPAPER_TRASH_RETENTION_DAYS` | `30` | Aufbewahrung im Papierkorb |
| `NRZPAPER_REGISTRATION_OPEN` | `false` | Selbst-Registrierung nach dem ersten Konto |
| `NRZPAPER_COOKIE_SECURE` | `false` | Cookies nur über HTTPS (hinter HTTPS einschalten) |
| `NRZPAPER_LOG` | `info` | Log-Filter (`tracing`) |

### Texterkennung

Braucht Tesseract und Poppler (im Docker-Image enthalten). Auf dem Mac:

```bash
brew install tesseract poppler
# Deutsch: deu.traineddata nach $(brew --prefix)/share/tessdata (tessdata_fast) oder brew install tesseract-lang
```

Ohne Tesseract werden Dokumente trotzdem gespeichert, nur ohne erkannten Text.

### Eingangsordner

Dateien nach `data/ingestion/<Organisations-ID>/` legen – sie werden übernommen und danach gelöscht.

### Suche

`rechnung zürich` · `"mobile abo"` · `name:swisscom` · `content:iban` · `tag:steuern` · `-tag:privat` · `has:tags` ·
`type:pdf` · `created:2026` · `created:>2026-06-01` · `status:failed`

## Tests

```bash
cargo test
```

- `tests/sqlcipher.rs` – Grundannahme: verschlüsselte Datei ohne lesbaren Inhalt, FTS5 mit Umlauten, falscher Schlüssel wird abgewiesen.
- `tests/ui.rs` – Suchsyntax gegen die echte Datenbank (inkl. böser Eingaben), Liste und Live-Suche, Detailseite, alle Formular-Aktionen, Tags, Papierkorb mit Rechten, eingebettete Dateien.
- `tests/documents.rs` – Aufnahme und Typen, Duplikate, Papierkorb, echte Texterkennung (Textebene, Scan, Bild – übersprungen ohne Tesseract), Warteschlange mit Volltextsuche, Eingangsordner, Upload mit Range, Dokument-API mit Berechtigungen.
- `tests/web.rs` – Ersteinrichtung, Anmeldung, Sitzungen (auch nach Neustart), fremde Organisationen, CSRF, offene Redirects, Sicherheits-Header, Fehler als HTML/JSON, API-Keys mit Berechtigungen und Ablauf.
- `tests/schema.rs` – Migrationen, Volltextindex (auch nach `VACUUM`), Löschketten, Eindeutigkeit, Prüfbedingungen, Modelle, verschlüsselte Dokument-Blöcke.
