# nrzPaper – Architektur

nrzPaper ist eine selbst gehostete Dokumentenverwaltung im Funktionsumfang von Papra, komplett in Rust.
Ein Prozess, eine Datei: Server, Hintergrundarbeiten und Oberfläche laufen in einem Programm, alle
Daten liegen in **einer einzigen, vollständig verschlüsselten SQLite-Datenbank**.

## Leitentscheidungen

| Thema | Entscheidung | Warum |
|-------|--------------|-------|
| Web | Rocket **0.5.1** (fest, inkl. `rocket_http`/`rocket_codegen` =0.5.1) | Vorgabe. Ohne feste Unterversionen zieht Cargo `rocket_http` 0.5.2 (hyper 1.x), das mit Rocket 0.5.1 nicht übersetzt. |
| Datenbank | SQLite über `sqlx` 0.9, **SQLCipher** (`libsqlite3-sys` 0.37, `bundled-sqlcipher-vendored-openssl`) | Eine Datei, AES-256 verschlüsselt auf Seitenebene – auch Indizes, FTS5 und Datei-Inhalte. Getestet in `tests/sqlcipher.rs`. |
| Dokumente | **Als BLOB in der Datenbank**, in Blöcken zu 1 MiB (`document_chunks`) | „Eine einzige Datenbank, alles verschlüsselt in sich“. Blöcke erlauben Streaming und HTTP-Range-Anfragen für die PDF-Vorschau, ohne ganze Dateien in den Speicher zu laden. Abweichung von „Dateisystem `./data/documents`“ der ursprünglichen Vorgabe. |
| Schlüssel | `NRZPAPER_DB_KEY` (Umgebung/Docker-Secret); fehlt er, wird beim ersten Start `data/nrzpaper.key` erzeugt | Der Schlüssel darf nicht in der Datenbank liegen. Ohne Schlüssel sind die Daten verloren – im Betrieb sichern. |
| Volltextsuche | FTS5 in derselben Datenbank, `unicode61 remove_diacritics 2` | „Zürich“ findet auch „Zurich“. Präfix-Suche, Ranking über `bm25`. |
| OCR | Tesseract + Poppler als Systemprogramme, **nur über Pipes** (stdin/stdout, `fd://0`) | Kein Klartext auf der Platte, auch nicht kurzzeitig. Textebene eines PDFs zuerst (`pdftotext`), sonst Seiten rendern (`pdftoppm`) → `tesseract`. |
| Oberfläche | Server-Side Rendering mit **Askama** (Templates zur Übersetzungszeit geprüft) + Tailwind (CDN) | Vorgabe. Eigenes Design in Grautönen. |
| PDF-Vorschau | PDF.js im Browser, Daten per Range-Anfrage aus der verschlüsselten DB | Browser führen kein Rust aus; die Vorschau (Miniaturen, Zoom, Drehen, Suche im PDF, Text-Ebene) ist der einzige Teil mit JavaScript. Alles andere rendert der Server. |
| Hintergrund | Tokio-Aufgaben, gestartet in einem Liftoff-Fairing | OCR-Warteschlange, Eingangsordner-Beobachter (`notify`), Papierkorb-Leerung, Webhook-Zustellung. |

## Datenfluss

```
Upload (Web, API, Eingangsordner)
  → Typ prüfen (infer), SHA-256 (Duplikate je Organisation)
  → documents + document_chunks (verschlüsselt)  ── Status: wartet
  → OCR-Warteschlange (Tokio-Kanal + DB-Status, übersteht Neustarts)
      PDF:  pdftotext fd://0  → genug Text? fertig
            sonst je Seite: pdftoppm fd://0 → PNG → tesseract stdin → Text
      Bild: tesseract stdin
  → documents.content + documents_fts aktualisieren  ── Status: fertig
  → Tagging-Regeln anwenden → Webhooks „document:created“
```

## Funktionsumfang (wie Papra)

- **Konten & Anmeldung** – Argon2id, Sitzungen in der DB (Cookie mit Zufalls-Token, privat signiert), CSRF-Schutz für Formulare.
- **Organisationen** – mehrere je Benutzer, Rollen Besitzer/Admin/Mitglied, Einladungen.
- **Dokumente** – Upload per Drag & Drop, Eingangsordner, API; umbenennen, Dokumentdatum, Inhalt bearbeiten, Papierkorb mit Aufbewahrungsfrist und Wiederherstellen, Duplikaterkennung, Aktivitätsprotokoll.
- **Suche** – Volltext mit Filtern wie in Papra: `tag:rechnung`, `name:swisscom`, `created:>2026-01-01`, `has:tags`, Freitext.
- **Tags** – Farbe, Beschreibung; Filtern nach Tags.
- **Tagging-Regeln** – Bedingungen auf Name/Inhalt (enthält, beginnt mit, Regex) → Tags; auf bestehende Dokumente anwendbar.
- **Benutzerdefinierte Eigenschaften** – Text, Zahl, Datum, Ja/Nein, Auswahl, Mehrfachauswahl, Benutzer- und Dokument-Bezug.
- **Freigabelinks** – öffentlicher Link auf ein Dokument, optional mit Ablauf.
- **API** – REST unter `/api/v1`, API-Keys mit Berechtigungen (`documents:read`, `documents:create`, `tags:*` …), gehasht gespeichert.
- **Webhooks** – ausgehend, signiert nach „Standard Webhooks“ (`webhook-id`, `webhook-timestamp`, `webhook-signature`).

## Projektstruktur

```
nrzPaper/
├── Cargo.toml
├── Rocket.toml               Standardwerte (Port, Limits); alles per NRZPAPER_* überschreibbar
├── migrations/               sqlx-Migrationen (Schritt 2)
├── templates/                Askama-Templates (Schritt 4)
├── static/                   CSS-Ergänzungen, PDF-Viewer (Schritt 4)
├── tests/                    Integrationstests (u. a. SQLCipher)
└── src/
    ├── main.rs               Start: Konfiguration, DB, Fairings, Routen
    ├── config.rs             Einstellungen (Rocket/Figment, NRZPAPER_*)
    ├── error.rs              Ein Fehlertyp → HTML- oder JSON-Antwort
    ├── db/                   Pool mit SQLCipher-Schlüssel, Migrationen, Modelle
    ├── auth/                 Passwörter, Sitzungen, Request Guards, API-Keys
    ├── storage/              Dokument-Blöcke in der DB, Streaming, Range
    ├── ocr/                  Tesseract/Poppler-Pipeline und Warteschlange
    ├── ingestion/            Eingangsordner-Beobachter
    ├── search/               Suchsyntax → FTS5-Abfrage
    ├── services/             Fachlogik: Dokumente, Tags, Regeln, Eigenschaften, Webhooks
    ├── fairings/             Sicherheits-Header, Hintergrundarbeiten
    └── routes/               web/ (SSR-Seiten) und api/ (REST)
```

## Umsetzung in Schritten

1. **Architektur & Projektstruktur** – diese Datei, `Cargo.toml`, Modulgerüst, SQLCipher-Test. ✅
2. Datenbank-Schema & Migrationen (`sqlx`), Modelle, Schlüsselverwaltung.
3. Rocket-Server: Konfiguration, Fairings, Request Guards, Routing, Fehlerbehandlung.
4. Upload-, Speicher- und OCR-Pipeline, Eingangsordner.
5. SSR-Oberfläche (Askama + Tailwind, Grautöne), PDF-Vorschau.
6. REST-API, Webhooks, Freigabelinks.
7. Dockerfile (Multi-Stage, `debian:bookworm-slim` mit Tesseract und Poppler).
