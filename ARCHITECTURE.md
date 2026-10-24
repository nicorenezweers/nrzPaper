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
| Oberfläche | Server-Side Rendering mit **Askama** (Templates zur Übersetzungszeit geprüft) + Tailwind (beim Build erzeugt, eingebettet) | Vorgabe. Eigenes Design in Grautönen; kein CDN, läuft offline. |
| PDF-Vorschau | PDF.js (pdfjs-dist 4.10.38) **ins Programm eingebettet**, Daten per Range-Anfrage aus der verschlüsselten DB | Browser führen kein Rust aus; die Vorschau ist der einzige grössere Teil mit JavaScript. Eingebettet statt CDN, damit sie auch ohne Internet (z. B. auf Umbrel) läuft. |
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

**Fairings**: `SecurityHeaders` (CSP nur für die eigene Seite, `X-Frame-Options`, `nosniff`, Referrer- und
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

## Dokumente, Texterkennung, Eingangsordner (Schritt 4)

**Aufnahme** (`services::documents::ingest`) – gleich für Web, API und Eingangsordner:

1. Typ aus dem Inhalt (`infer`), nicht aus dem Dateinamen; Text erkennt man an gültigem UTF-8.
2. SHA-256 → gibt es die Datei in der Organisation schon? Aktiv: Konflikt (409). Im Papierkorb: wird wiederhergestellt.
3. Dokument und Blöcke in **einer Transaktion** schreiben, Verlauf „created“.
4. PDF/Bild → `pending` und OCR-Worker wecken; Text-Datei → sofort durchsuchbar; andere Typen → `skipped`.

**Upload ohne Klartext auf der Platte**: Rockets `TempFile` würde die Datei zuerst in ein temporäres
Verzeichnis schreiben. `routes::upload::UploadedFile` liest das Formularfeld stattdessen in den Speicher
(Grenze: `limits.file`, Standard 200 MiB).

**Speicher** (`storage`): 1-MiB-Blöcke; `read_range` liest nur die Blöcke eines `Range`-Bereichs.
`FileResponse` antwortet mit `206 Partial Content`, `Accept-Ranges`, `Content-Range`, `416` für unerfüllbare
Bereiche, `Content-Disposition` inline oder als Anhang (UTF-8-Namen nach RFC 5987) und `Cache-Control: private, no-store`.

**Texterkennung** (`ocr::engine`): beim Start wird geprüft, was installiert ist (`tesseract --list-langs`,
`pdftotext`, `pdftocairo`); gewünschte, aber fehlende Sprachen werden gemeldet und weggelassen.

| Datei | Weg (nur Pipes: stdin → stdout, `fd://0`) |
|-------|------------------------------------------|
| PDF mit Textebene | `pdfinfo` (Seitenzahl) → `pdftotext -layout` – genügt, wenn ≥ 25 Zeichen je Seite |
| Gescanntes PDF | je Seite `pdftocairo -png -singlefile -r 300 -gray` → `tesseract stdin stdout` (höchstens 150 Seiten) |
| Bild (PNG, JPEG, TIFF, WebP, BMP, GIF) | `tesseract stdin stdout` |

Jeder Aufruf hat ein Zeitlimit von 5 Minuten; Schreiben nach stdin und Lesen von stdout laufen gleichzeitig,
damit volle Pipes nicht blockieren.

**Warteschlange** (`ocr::queue`): Zustand in der Datenbank. Ein Worker übernimmt das nächste Dokument mit
*einer* Anweisung (`UPDATE … WHERE num = (SELECT … LIMIT 1) RETURNING id`), deshalb greifen mehrere Worker
(`ocr_workers`, Standard 2) nie dasselbe. Ein `Notify` weckt sie bei neuen Dokumenten, sonst schauen sie alle
30 s nach. Beim Start wird `processing` wieder zu `pending` (Absturz-Wiederaufnahme). Erkannter Text landet in
`documents.content` und damit über die Trigger im Volltextindex.

**Eingangsordner** (`ingestion`): `data/ingestion/<Organisations-ID>/`, für neue Organisationen sofort angelegt.
Eine Datei wird übernommen, wenn sie zwei Runden lang unverändert und mindestens 2 s alt ist (also fertig
geschrieben), danach **gelöscht**. Duplikate werden ebenfalls gelöscht, Fehlerhaftes landet in
`<Organisation>/fehler/`. Versteckte und halbe Dateien (`.part`, `.tmp`, `.crdownload` …) bleiben liegen.
`notify` meldet Änderungen sofort; zusätzlich alle 20 s eine Runde, weil Datei-Ereignisse in Docker-Bind-Mounts
nicht immer ankommen.

**Papierkorb**: `DELETE` legt Dokumente in den Papierkorb; erneuter Upload oder `restore` holt sie zurück; endgültig
löschen nur aus dem Papierkorb. Ein Hintergrund-Job leert ihn täglich nach `trash_retention_days`.

**Pools immer schliessen, nie fallen lassen**: SQLCipher räumt beim Prozessende mit einem eigenen
`atexit`-Handler (`sqlcipher_atexit`) auf. Wird ein sqlx-Pool nur fallen gelassen, schliessen seine
Worker-Threads die Verbindungen nebenher – fällt das mit dem Prozessende zusammen, stürzt `sqlcipher_free`
ab (SIGSEGV, gefunden über die macOS-Absturzberichte). Deshalb:

- `CloseDatabase` schliesst den Pool beim Herunterfahren des Servers,
- `db::open`/`db::connect` schliessen den Pool, bevor sie einen Fehler zurückgeben (falscher Schlüssel, Migration),
- Tests schliessen ihre Datenbank bzw. beenden den Test-Server mit `terminate()`.

**Neue Routen**

| Pfad | Zweck |
|------|-------|
| `POST /organizations/<org>/documents` | Upload aus der Oberfläche (mehrere Dateien, Antwort JSON je Datei) |
| `GET /organizations/<org>/documents/<id>/file[?download=true]` | Datei für Vorschau/Download, mit Range |
| `POST/GET /api/v1/organizations/<org>/documents` | Upload (multipart `file`) und Liste (`pageIndex`, `pageSize`) |
| `GET /api/v1/organizations/<org>/documents/deleted` | Papierkorb |
| `GET/PATCH/DELETE /api/v1/organizations/<org>/documents/<id>` | lesen, Name/Inhalt ändern, in den Papierkorb |
| `GET /api/v1/organizations/<org>/documents/<id>/file` | Datei (Range) |
| `POST /api/v1/organizations/<org>/documents/<id>/restore` · `/reprocess` | wiederherstellen, Texterkennung neu |

## Oberfläche (Schritt 5)

**Seiten** (`src/routes/web/pages.rs`, Templates unter `templates/`) – alles server-seitig gerendert, Formulare
nach Post/Redirect/Get:

| Pfad | Seite |
|------|-------|
| `/organizations/<org>?q=&page=` | Dokumentliste: Suche, Tag-Filter mit Anzahl, Upload per Drag & Drop, 25 je Seite |
| `…?partial=true` | nur die Ergebnisliste (für die Live-Suche) |
| `/organizations/<org>/documents/<id>?find=` | Detail: Vorschau, Name, Tags, Angaben, erkannter Text (bearbeitbar), Verlauf |
| `/organizations/<org>/tags` | Tags anlegen, bearbeiten, löschen |
| `/organizations/<org>/trash` | Papierkorb: wiederherstellen, endgültig löschen, leeren (nur Besitzer/Admin) |

**Suche** (`src/search`): eigene Syntax wie bei Papra – freie Wörter (Präfix, Umlaute egal), `"Wortfolge"`,
`name:`, `content:`, `tag:`/`-tag:`, `has:tags`, `type:pdf|image|text`, `created:2026`, `created:>2026-06-01`,
`status:failed`. Daraus wird eine Abfrage mit `QueryBuilder` und gebundenen Werten; der FTS5-Ausdruck besteht nur
aus vollständig gequoteten Begriffen, Eingaben können also keine FTS-Syntax einschleusen. Mit Text-Suche wird nach
`bm25` sortiert (Name 10-fach gewichtet), sonst nach Datum.

**PDF-Viewer** (`static/viewer.js`): fortlaufendes Scrollen, Seiten werden erst beim Sichtbarwerden gerendert
(IntersectionObserver), scharf auf Retina; Miniaturen-Leiste; Seitenwahl; Zoom (Seitenbreite, ganze Seite,
50–300 %, ±); Drehen; Suche im PDF mit Treffer-Navigation (Umlaute egal) über die Text-Ebene; markierbarer Text;
Vollbild; Tastatur (←/→, +/−, R, ⌘F, Esc). Der Suchbegriff aus der Liste wird gleich markiert (`?find=`).
Bilder: Zoom, Drehen, Vollbild. Text-Dateien: als Text. Eingescannte PDFs ohne Textebene lassen sich im Viewer
nicht durchsuchen – ihr erkannter Text steht rechts daneben und ist in der Dokumentsuche enthalten.

**Eigene Dateien** (`src/routes/assets.rs`): `app.css`, `app.js`, `viewer.js`, `viewer.css`, die Schrift Inter
und PDF.js sind mit `include_bytes!` eingebettet. Die Links tragen `?v=` mit einer Prüfsumme der Dateien, damit
Browser nach jeder Änderung neu laden; PDF.js und die Schrift werden ein Jahr lang gecacht (`immutable`).
Die Content-Security-Policy erlaubt nur die eigene Seite (`script-src 'self'`, keine Inline-Skripte).

**Design**: nur Grautöne (Tailwind `zinc`), Tags dürfen eine Farbe tragen. Auf kleinen Bildschirmen ersetzt eine
Leiste oben die Seitenleiste.

## API, Konto, Webhooks, Freigaben (Schritt 6)

**REST-API** (`src/routes/api/`) – JSON in camelCase wie Papra; jeder Endpunkt prüft Mitgliedschaft (sonst 404)
und die Berechtigung des Keys (sonst 403). Mit Sitzung statt Key gelten alle Rechte des Kontos.

| Endpunkt | Zweck | Recht |
|----------|-------|-------|
| `GET …/documents?pageIndex&pageSize&searchQuery` | Liste oder Suche (gleiche Syntax wie die Oberfläche), jeweils mit `tags` | `documents:read` |
| `GET …/documents/statistics` | Anzahl, Grösse, Papierkorb, laufende Erkennung | `documents:read` |
| `GET …/documents/<id>/activity` | Verlauf mit Benutzer und Daten | `documents:read` |
| `POST …/documents/<id>/tags` `{tagId}` · `DELETE …/tags/<tag>` | Tag anhängen (201, schon da: 204) / entfernen | `documents:update` |
| `GET/POST …/tags` · `PUT/DELETE …/tags/<id>` | Tags verwalten (mit `documentsCount`) | `tags:*` |
| `GET/POST …/documents/<id>/share-links` · `DELETE …/<link>` | Freigabelinks; Token nur in der Antwort auf POST | `documents:read`/`update` |
| `GET/POST …/webhooks` · `PATCH/DELETE …/<id>` · `GET …/<id>/deliveries` | Webhooks; nur Besitzer/Admin | `organizations:update` |
| `PUT /organizations/<id>` · `DELETE` | umbenennen (Besitzer/Admin) / löschen (nur Besitzer) | `organizations:update`/`delete` |

**Konto** (`/organizations/<org>/account`, Link über den Benutzernamen): API-Keys mit Rechten (lesende
vorausgewählt) und Ablauf anlegen und löschen; Passwort ändern beendet alle Sitzungen, die aktuelle bekommt
sofort eine neue. **Einstellungen** (`/organizations/<org>/settings`): Name, Mitglieder, Webhooks mit den letzten
Zustellungen, Organisation löschen (Besitzer). Neue Geheimnisse (Key, Webhook-Secret, Freigabelink) stehen nur in
der Antwort auf das Formular – nie in einer URL oder einem Cookie; danach setzt `app.js` die Adresse zurück,
damit Neuladen nichts erneut absendet.

**Webhooks** (`src/services/webhooks.rs`): Ereignisse `document:created|updated|deleted|restored` und
`document:tag:added|removed` kommen aus den Diensten (Aufnahme, Umbenennen, Text, Papierkorb, Tags) und aus der
Texterkennung (fertig → `document:updated`). Zustellung im Hintergrund nach „Standard Webhooks“: `webhook-id`
(`msg_…`), `webhook-timestamp`, `webhook-signature: v1,Base64(HMAC-SHA256(secret, "id.timestamp.body"))`, Body
`{type, timestamp, data}`. Drei Versuche (sofort, 5 s, 30 s), keine Weiterleitungen, 15 s Zeitlimit; jeder
Versuch landet in `webhook_deliveries`. Lokale Ziele sind erlaubt (Selbst-Hosting im Heimnetz).

**Freigabelinks** (`src/services/share_links.rs`): Token `sh_` + 32 Zeichen, in der DB nur als SHA-256.
`/share/<token>` zeigt Name, Angaben und denselben Viewer wie die Detailseite (`templates/documents/_viewer.html`),
`/share/<token>/file` liefert die Datei mit Range. Ungültig (→ 404) nach Ablauf, Widerruf, im Papierkorb oder
wenn die Organisation gelöscht ist. Seitenaufrufe werden gezählt; `noindex`, `Referrer-Policy: same-origin`.

## Build & Betrieb (Schritt 7)

**CSS**: `assets/app.css` (Tailwind-Direktiven, `@font-face` für Inter, Grundstile) → `scripts/tailwind.sh` →
`static/app.css` (minifiziert, ~20 KB). Das Skript lädt die eigenständige Tailwind-CLI 3.4.17 (kein Node) nach
`.tools/` und prüft ihre SHA-256-Prüfsumme. `static/app.css` liegt im Repository, damit `cargo build` ohne Werkzeug
geht; `tests/assets.rs` meldet, wenn die Datei nicht mehr zu den Templates passt. Der Docker-Build erzeugt sie immer
frisch.

**Docker** (`Dockerfile`): Stufe 1 `rust:1.98-bookworm` (gepinnt) baut CSS und Programm, für die fremde Architektur
quer-kompiliert statt emuliert (`--locked`, SQLCipher/OpenSSL statisch mitgebaut, Cargo-Cache über BuildKit).
Stufe `ocr` installiert Tesseract und Poppler und legt sie zusätzlich als eigenständiges Paket ab (Programme,
Sprachdaten, nur die Bibliotheken, die im Basis-Image fehlen); Stufe `umbrel` exportiert Programm und Paket für den
nrzStore (`--target umbrel --output type=local,dest=dist`), dort laufen sie im gepinnten `debian:bookworm-slim`. Stufe 2 `debian:bookworm-slim` mit `tesseract-ocr` (+ `deu`, `eng`),
`poppler-utils` und CA-Zertifikaten (für Webhooks über HTTPS). Läuft als Benutzer `nrzpaper` (UID 1000), Daten in
`/data` (Volume), Port 8320. `HEALTHCHECK` ruft `nrzpaper healthcheck` auf (fragt `/api/v1/health`, kein curl im
Image). `docker stop` schickt SIGTERM: Rocket schliesst laufende Anfragen ab (5 s), dann wird die Datenbank
geschlossen.

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
  → Webhooks „document:updated“ (angelegt: „document:created“ schon beim Upload)
```

## Scannen & Prüfen (aus nrzDocsScanner)

nrzDocsScanner (eigene App: scannen → KI → Papra) ist in nrzPaper aufgegangen; Ziel ist jetzt nrzPaper selbst,
alles Papra-Spezifische (Instanzen, Papra-Keys, Ablageziele) entfällt.

- **Schema** `migrations/0002_scan.sql`: `scan_settings` (Scanner, Farbe, dpi je Benutzer), `scan_sessions` und
  `scan_pages` (laufender Scan je Benutzer und Organisation, Seiten als JPEG-BLOB), `inbox_items` und
  `inbox_files` (Prüfliste mit PDF und Seitenbildern). Alles in der verschlüsselten Datenbank, keine Dateien.
- **Scanner** `src/scan/`: eSCL (`ScannerStatus`, `ScanJobs`, `NextDocument` mit Warten auf 503), Suche aller
  Adressen eines /24-Netzes nach `ScannerCapabilities`, PDF aus JPEG-Seiten ohne Neukodierung (DCTDecode).
  Ein globaler Lock lässt Scans nacheinander laufen.
- **KI** `src/ai.rs`: OpenRouter Chat Completions, Seiten als Bilder (bis 4) oder das PDF als Datei; Antwort als
  JSON (title, tags nur aus der Tag-Liste der Organisation, metadata, summary). Bei `openrouter/free` wird das
  freie Routing zweimal und danach eine Reihe kostenloser Bild-Modelle probiert.
- **Dienste** `services::scan` (Sitzung), `services::inbox` (anlegen → KI im Hintergrund → prüfen → freigeben;
  nach Neustart werden hängende Einträge zurückgesetzt, alte stündlich gelöscht), `services::properties`
  (Metadaten als benutzerdefinierte Eigenschaften: Datum als Datum, Betrag als Zahl, Rest als Text).
- **Oberfläche** `templates/scan/`, `static/scan.js`: Seiten „Scannen“ und „Prüfen“ (Zähler in der Navigation),
  Scanner und KI-Test im Konto, Eigenschaften auf der Dokumentseite (änderbar).

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
4. **Upload, Speicher, OCR, Eingangsordner** – `storage`, `ocr`, `ingestion`, `services::documents`, `tests/documents.rs`. ✅
5. **Oberfläche** – Dokumentliste mit Live-Suche, Detailseite mit PDF-Viewer, Tags, Papierkorb, `tests/ui.rs`. ✅
6. **API, Konto, Webhooks, Freigaben** – restliche Endpunkte, API-Keys und Passwort in der Oberfläche, Einstellungen, signierte Webhooks, öffentliche Links, `tests/integrations.rs`. ✅
7. **Docker & Tailwind-Build** – Multi-Stage-`Dockerfile` (`debian:bookworm-slim` mit Tesseract und Poppler), Tailwind-CSS beim Build, Schrift eingebettet, `tests/assets.rs`. ✅
8. **Scannen & Prüfen** – nrzDocsScanner integriert: eSCL-Scanner, KI-Vorschläge, Prüfliste, Eigenschaften, `tests/scanning.rs`. ✅
