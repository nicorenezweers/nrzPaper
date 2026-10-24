# nrzPaper

Selbst gehostete Dokumentenverwaltung im Funktionsumfang von [Papra](https://papra.app) – komplett in Rust.
Alle Daten, auch die Dokumente selbst, liegen in **einer einzigen, mit SQLCipher verschlüsselten SQLite-Datei**.

> Stand: alle 7 Schritte – Oberfläche mit Live-Suche, PDF-Viewer, Tags und Papierkorb; Upload, Texterkennung, Eingangsordner; vollständige REST-API, API-Keys und Passwort in der Oberfläche, signierte Webhooks, öffentliche Freigabelinks; Docker-Image mit Tesseract, Tailwind beim Build. Aufbau und Plan: [ARCHITECTURE.md](ARCHITECTURE.md).

## Starten mit Docker

```bash
docker compose up -d --build
# http://localhost:8320 – beim ersten Aufruf das Administrator-Konto anlegen
```

oder ohne Compose:

```bash
docker build -t nrzpaper .
docker run -d --name nrzpaper -p 8320:8320 -v nrzpaper-data:/data nrzpaper
```

Fertiges Image (amd64 und arm64) aus der Forgejo-Registry:

```bash
docker run -d --name nrzpaper -p 8320:8320 -v nrzpaper-data:/data 100.78.107.162:8101/nicorenezweers/nrzpaper:1.0.0
```

Auf Umbrel: App **nrzPaper** im nrzStore (bringt Programm und Texterkennung selbst mit, braucht keine Registry).

Das Image enthält Tesseract (Deutsch, Englisch) und Poppler. Alle Daten liegen im Volume `/data`
(`nrzpaper.db`, `nrzpaper.key`, `ingestion/`). Den Schlüssel am besten über `NRZPAPER_DB_KEY` setzen und
getrennt vom Volume aufbewahren – wer Volume *und* Schlüsseldatei hat, kann alles lesen.

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
| `NRZPAPER_LOG` | `info` (Rocket-Details gedämpft) | Log-Filter (`tracing`), z. B. `debug` |

### Oberfläche (CSS)

Das CSS wird aus den Templates erzeugt. Nach Änderungen an Klassen in `templates/` oder `static/*.js`:

```bash
scripts/tailwind.sh          # einmal (lädt beim ersten Mal die Tailwind-CLI nach .tools/)
scripts/tailwind.sh --watch  # während der Arbeit
```

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

### API

API-Key unter *Benutzername → Konto* anlegen (er wird nur einmal angezeigt), dann:

```bash
curl -H "Authorization: Bearer np_…" http://localhost:8320/api/v1/organizations
curl -H "Authorization: Bearer np_…" -F file=@rechnung.pdf http://localhost:8320/api/v1/organizations/<org>/documents
curl -H "Authorization: Bearer np_…" "http://localhost:8320/api/v1/organizations/<org>/documents?searchQuery=tag:steuern"
```

Alle Endpunkte: [ARCHITECTURE.md](ARCHITECTURE.md#api-konto-webhooks-freigaben-schritt-6).

### Webhooks

Unter *Einstellungen* (Besitzer/Admin). Prüfen beim Empfänger:
`webhook-signature == "v1," + base64(hmac_sha256(secret, webhook-id + "." + webhook-timestamp + "." + body))`.

## Tests

```bash
cargo test
```

- `tests/sqlcipher.rs` – Grundannahme: verschlüsselte Datei ohne lesbaren Inhalt, FTS5 mit Umlauten, falscher Schlüssel wird abgewiesen.
- `tests/ui.rs` – Suchsyntax gegen die echte Datenbank (inkl. böser Eingaben), Liste und Live-Suche, Detailseite, alle Formular-Aktionen, Tags, Papierkorb mit Rechten, eingebettete Dateien.
- `tests/documents.rs` – Aufnahme und Typen, Duplikate, Papierkorb, echte Texterkennung (Textebene, Scan, Bild – übersprungen ohne Tesseract), Warteschlange mit Volltextsuche, Eingangsordner, Upload mit Range, Dokument-API mit Berechtigungen.
- `tests/web.rs` – Ersteinrichtung, Anmeldung, Sitzungen (auch nach Neustart), fremde Organisationen, CSRF, offene Redirects, Sicherheits-Header, Fehler als HTML/JSON, API-Keys mit Berechtigungen und Ablauf.
- `tests/integrations.rs` – API (Tags, Suche, Statistik, Verlauf, Organisation), Rechte der Keys, Webhook an einen echten HTTP-Empfänger mit Signaturprüfung, Freigabelinks (öffentlich, Range, Ablauf, Papierkorb, Widerruf), Passwortwechsel.
- `tests/assets.rs` – `static/app.css` passt zu den Templates, nur lokale Schriften.
- `tests/schema.rs` – Migrationen, Volltextindex (auch nach `VACUUM`), Löschketten, Eindeutigkeit, Prüfbedingungen, Modelle, verschlüsselte Dokument-Blöcke.
