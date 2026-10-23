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

## Datenbank-Schema

`migrations/0001_init.sql`, beim Start automatisch angewendet (eingebettet mit `sqlx::migrate!`).

| Bereich | Tabellen |
|---------|----------|
| Anmeldung | `users`, `sessions` (nur Token-Hash), `api_keys` (nur Key-Hash, Berechtigungen als JSON) |
| Organisationen | `organizations`, `organization_members` (owner/admin/member), `organization_invitations` |
| Dokumente | `documents`, `document_chunks` (Inhalt, 1 MiB je Block), `documents_fts` (FTS5), `document_activity` |
| Ordnung | `tags`, `document_tags`, `tagging_rules`, `tagging_rule_conditions`, `tagging_rule_actions` |
| Eigenschaften | `custom_property_definitions`, `custom_property_options`, `document_property_values` |
| Teilen & Anbindung | `share_links`, `webhooks`, `webhook_events`, `webhook_deliveries` |
| Instanz | `app_settings` |

Regeln, die im Schema stecken:

- **STRICT-Tabellen** (SQLite 3.50): falsche Typen werden abgewiesen. Aufzählungen sind TEXT mit `CHECK`, passend zu den Rust-Enums.
- **IDs** sind UUIDv7, **Zeitpunkte** Unix-Sekunden. `documents.num` ist eine feste Zeilennummer nur für den Volltextindex – ohne sie könnte `VACUUM` die interne rowid neu vergeben und `documents_fts` zeigte ins Leere.
- **Volltextindex** als External-Content-FTS5, nachgeführt über Trigger bei Anlegen, Ändern von Name/Inhalt und Löschen.
- **Duplikate**: `UNIQUE (organization_id, sha256)` – auch für Dokumente im Papierkorb (ein erneuter Upload holt sie zurück).
- **Löschketten**: Organisation löschen entfernt Dokumente, Blöcke, Tags, Regeln usw.; Benutzer löschen lässt Dokumente stehen (`created_by` wird leer).
- **Eigenschaftswerte**: genau eine Wert-Spalte je Zeile (`CHECK`), Mehrfachwerte als mehrere Zeilen.
- **Abfragen zur Laufzeit** (`query_as` + `FromRow`): die `query!`-Makros bräuchten beim Übersetzen eine unverschlüsselte Datenbank. Dynamisches SQL muss in sqlx 0.9 ausdrücklich als `AssertSqlSafe` markiert werden – nur für feste Bausteine, Werte immer über `bind`.

## Server (Schritt 3)

**Start** (`lib.rs::build_with`): Konfiguration lesen → Datenbank öffnen und migrieren → Cookie-Schlüssel aus der
Datenbank (beim ersten Start erzeugt, liegt verschlüsselt in `app_settings`; ein gesetzter `secret_key` hat Vorrang)
→ Fairings, Routen, Catcher.

**Anmeldung**

- Passwörter: Argon2id auf einem Blocking-Thread. Unbekannte Benutzernamen prüfen einen Dummy-Hash, damit die
  Antwortzeit nichts verrät; Fehlversuche warten zusätzlich 400 ms.
- Sitzungen: Zufalls-Token (256 Bit) im privaten Cookie `nrzpaper_session` (von Rocket verschlüsselt und signiert,
  `HttpOnly`, `SameSite=Lax`, optional `Secure`). In der Datenbank liegt nur der SHA-256. 30 Tage, verlängert sich
  bei Benutzung (höchstens einmal pro Stunde ein Schreibzugriff), abgelaufene löscht ein Hintergrund-Job stündlich.
- Das erste Konto wird Administrator. Danach ist die Registrierung zu (`registration_open` bzw. Einstellung in der DB).
- API-Keys: `np_` + 240 Bit Zufall, gespeichert als SHA-256 plus Präfix zur Anzeige, mit Berechtigungen und Ablauf.

**Request Guards** (`src/auth/guards.rs`)

| Guard | Bedeutung |
|-------|-----------|
| `CurrentUser` | Sitzung gültig; ändernde Anfragen mit fremdem `Origin` → 403 |
| `MaybeUser` | Sitzung optional (Anmeldeseite, Start) |
| `AdminUser` | zusätzlich Administrator |
| `ApiUser` | `Authorization: Bearer np_…` oder Sitzung; `require(Permission)` prüft den Key |
| `UserAgent` | Browser-Kennung für die Sitzungsliste |

Organisationen prüft `services::organizations::membership` – wer kein Mitglied ist, bekommt 404, nicht 403.

**CSRF**: `SameSite=Lax` verhindert, dass fremde Seiten POST-Anfragen mit Sitzung auslösen. Zusätzlich lehnen die
Guards ändernde Anfragen ab, deren `Origin` nicht zu `Host`/`X-Forwarded-Host` passt. Rücksprünge nach der Anmeldung
(`next`) sind nur auf eigene Pfade erlaubt.

**Fairings**: `SecurityHeaders` (CSP mit genau den beiden CDNs, `X-Frame-Options`, `nosniff`, Referrer- und
Permissions-Policy), `RequestLog` (Methode, Pfad, Status, Dauer über `tracing`), `Background` (Liftoff: Aufräum-Jobs,
beendet sich mit dem Server).

**Fehler**: `AppError` ist ein Responder – API-Pfade und `Accept: application/json` bekommen
`{"error":{"code","message"}}`, die Oberfläche eine Fehlerseite. Interne Fehler werden protokolliert, nach aussen steht
nur „Interner Fehler“. Catcher: 401 leitet in der Oberfläche zur Anmeldung (mit Rücksprung), sonst Seite bzw. JSON.

**Routen bisher**

| Pfad | Zweck |
|------|-------|
| `GET /` | weiter zur ersten Organisation, zur Anmeldung oder zur Ersteinrichtung |
| `GET/POST /login`, `POST /logout`, `GET/POST /register` | Anmeldung |
| `GET/POST /organizations/new`, `GET /organizations/<id>` | Organisation anlegen, öffnen |
| `GET /api/v1/health` | Lebenszeichen |
| `GET /api/v1/api-keys/current` | eigener Key und Rechte |
| `GET/POST /api/v1/organizations`, `GET /api/v1/organizations/<id>` | Organisationen |

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
2. **Datenbank-Schema & Migrationen** – `migrations/0001_init.sql`, Modelle in `src/db/models.rs`, Tests in `tests/schema.rs`. ✅
3. **Rocket-Server** – Fairings, Request Guards, Anmeldung, Routing, Fehlerbehandlung, erste Seiten, `tests/web.rs`. ✅
4. Upload-, Speicher- und OCR-Pipeline, Eingangsordner.
5. SSR-Oberfläche (Askama + Tailwind, Grautöne), PDF-Vorschau.
6. REST-API, Webhooks, Freigabelinks.
7. Dockerfile (Multi-Stage, `debian:bookworm-slim` mit Tesseract und Poppler).
