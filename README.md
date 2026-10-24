# nrzPaper

Selbst gehostete Dokumentenverwaltung im Funktionsumfang von [Papra](https://papra.app) – komplett in Rust.
Alle Daten, auch die Dokumente selbst, liegen in **einer einzigen, mit SQLCipher verschlüsselten SQLite-Datei**.

> Stand: alle 7 Schritte – Oberfläche mit Live-Suche, PDF-Viewer, Tags und Papierkorb; Upload, Texterkennung, Eingangsordner; vollständige REST-API, API-Keys und Passwort in der Oberfläche, signierte Webhooks, öffentliche Freigabelinks; **Scannen mit KI** (aus nrzDocsScanner: Netzwerk-Scanner, KI-Vorschläge, Prüfen vor dem Ablegen); **E-Mail-Eingang** (IMAP), **Ordner** aus Tags, Kachelansicht mit Vorschaubildern; Docker-Image mit Tesseract, Tailwind beim Build. Aufbau und Plan: [ARCHITECTURE.md](ARCHITECTURE.md).

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
| `NRZPAPER_SCANNER_URL` | – | Voreingestellter Scanner (eSCL/AirScan) für alle, die keinen eigenen eintragen |
| `NRZPAPER_SCAN_SUBNETS` | `192.168.0.0/24,192.168.1.0/24` | Netze für „Scanner suchen“ |
| `NRZPAPER_OPENROUTER_KEY` | eingebaut, falls beim Bauen gesetzt | Schlüssel für die KI (OpenRouter) |
| `NRZPAPER_OPENROUTER_MODEL` | `openrouter/free` | Modell; bei `openrouter/free` werden kostenlose Bild-Modelle als Ersatz probiert |
| `NRZPAPER_INBOX_KEEP_DAYS` | `7` | So lange bleiben nicht freigegebene Scans in der Prüfliste |

### Oberfläche (CSS)

Das CSS wird aus den Templates erzeugt. Nach Änderungen an Klassen in `templates/` oder `static/*.js`:

```bash
scripts/tailwind.sh          # einmal (lädt beim ersten Mal die Tailwind-CLI nach .tools/)
scripts/tailwind.sh --watch  # während der Arbeit
```

### Scannen und Prüfen

Übernommen aus nrzDocsScanner, jetzt direkt in nrzPaper:

1. **Scannen** – Seiten vom Netzwerk-Scanner (eSCL/AirScan, z. B. Canon TS4100i) oder als Fotos, Lupe,
   Reihenfolge per Drag & Drop, Tags vorgeben. Auch PDFs lassen sich zur Prüfung importieren.
2. **KI** – liest Name („JJJJ-MM-TT Absender Dokumentart Thema“), passende Tags der Organisation und
   Metadaten (Absender, Empfänger, Datum, Betrag, Währung, Referenz, Kundennummer, Fälligkeit, IBAN).
3. **Prüfen** – alles ändern, neue Tags eintippen, in eine andere Organisation verschieben, neu analysieren
   oder verwerfen. Erst **Freigeben** legt das Dokument an: mit Tags (fehlende werden angelegt), den Metadaten
   als Eigenschaften und Texterkennung. Seiten und PDF liegen bis dahin verschlüsselt in der Datenbank.

Scanner unter *Benutzername → Konto → Scanner* eintragen oder im Netz suchen. Die KI braucht einen
OpenRouter-Schlüssel – beim Docker-Build als Secret eingebaut (`--secret id=openrouter,env=NRZ_OPENROUTER_KEY`)
oder über `NRZPAPER_OPENROUTER_KEY`.

### Inbox, E-Mail-Eingang und Ordner

- **Inbox** – alles, was noch abgenommen werden muss: Scans, importierte PDFs und E-Mail-Anhänge.
- **E-Mail-Eingang** – unter *Benutzername → Konto → E-Mail-Eingang* Postfächer eintragen (nur Empfang, IMAP mit
  SSL/TLS, STARTTLS oder ohne; Vorlagen für Gmail, Outlook, iCloud, GMX, WEB.DE, Bluewin, IONOS). Neue Mails werden
  regelmässig abgeholt (alles ab dem Einrichten, nichts doppelt, nie gelöscht); PDFs und Bilder aus den Anhängen
  kommen mit den vorgegebenen Tags und den KI-Vorschlägen in die Inbox. Bei Gmail, iCloud und Outlook braucht es ein
  App-Passwort.
- **Ordner** – unter *Ordner → Verwalten* (auch verschachtelt). Ein Ordner fasst mehrere Tags zusammen und zeigt
  jedes Dokument mit mindestens einem davon, Unterordner eingeschlossen.
- **Kacheln** – Dokumente als Karten mit Vorschaubild der ersten Seite, Tags und Datum; umschaltbar auf Liste.
  Das Auge öffnet eine grosse Vorschau mit Zoom (+/−, Strg/⌘ + Mausrad).

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
- `tests/scanning.rs` – Scannen gegen einen Schein-Scanner (eSCL), KI gegen ein Schein-OpenRouter, Prüfen, Verschieben, Freigeben mit Tags und Eigenschaften, Duplikate, Fehlerfälle.
- `tests/mail.rs` – E-Mail-Eingang gegen einen Schein-IMAP-Server (Anmeldung, Abruf, keine Duplikate, gelesen markieren, Abnahme mit Tags), Ordner, Vorschaubilder, Kachelansicht.
- `tests/assets.rs` – `static/app.css` passt zu den Templates, nur lokale Schriften.
- `tests/schema.rs` – Migrationen, Volltextindex (auch nach `VACUUM`), Löschketten, Eindeutigkeit, Prüfbedingungen, Modelle, verschlüsselte Dokument-Blöcke.
