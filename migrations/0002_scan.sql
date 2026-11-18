-- Scannen und Prüfen (übernommen aus nrzDocsScanner): Scanner-Einstellungen je Benutzer,
-- laufende Scan-Sitzung je Benutzer und Organisation, Prüfliste vor der Ablage.
-- Seiten und PDFs liegen als BLOB in der (verschlüsselten) Datenbank – keine Dateien daneben.

CREATE TABLE scan_settings (
    user_id     TEXT    PRIMARY KEY NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    scanner_url TEXT    NOT NULL DEFAULT '',
    color       INTEGER NOT NULL DEFAULT 1 CHECK (color IN (0, 1)),
    dpi         INTEGER NOT NULL DEFAULT 300 CHECK (dpi IN (150, 300, 600)),
    updated_at  INTEGER NOT NULL
) STRICT;

-- Name und vorgegebene Tags des Scans, der gerade entsteht.
CREATE TABLE scan_sessions (
    user_id         TEXT    NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    organization_id TEXT    NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    name            TEXT    NOT NULL DEFAULT '',
    hint_tags       TEXT    NOT NULL DEFAULT '[]' CHECK (json_valid(hint_tags)),
    updated_at      INTEGER NOT NULL,
    PRIMARY KEY (user_id, organization_id)
) STRICT;

-- Gescannte Seiten (JPEG) in ihrer Reihenfolge.
CREATE TABLE scan_pages (
    id              TEXT    PRIMARY KEY NOT NULL,
    user_id         TEXT    NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    organization_id TEXT    NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    position        INTEGER NOT NULL,
    data            BLOB    NOT NULL,
    created_at      INTEGER NOT NULL
) STRICT;
CREATE INDEX scan_pages_session ON scan_pages (user_id, organization_id, position);

-- Prüfliste: was gescannt oder importiert wurde, wartet hier auf die KI und die Freigabe.
-- analyse → pruefen → (import →) fertig; fehler bei gescheiterter Ablage.
CREATE TABLE inbox_items (
    id              TEXT    PRIMARY KEY NOT NULL,
    organization_id TEXT    NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    user_id         TEXT    REFERENCES users (id) ON DELETE SET NULL,
    name            TEXT    NOT NULL,
    auto_name       INTEGER NOT NULL DEFAULT 0 CHECK (auto_name IN (0, 1)),
    status          TEXT    NOT NULL DEFAULT 'analyse' CHECK (status IN ('analyse', 'pruefen', 'import', 'fertig', 'fehler')),
    step            TEXT    NOT NULL DEFAULT '',
    error           TEXT    NOT NULL DEFAULT '',
    warning         TEXT    NOT NULL DEFAULT '',
    tags            TEXT    NOT NULL DEFAULT '[]' CHECK (json_valid(tags)),
    metadata        TEXT    NOT NULL DEFAULT '{}' CHECK (json_valid(metadata)),
    summary         TEXT    NOT NULL DEFAULT '',
    model           TEXT    NOT NULL DEFAULT '',
    page_count      INTEGER NOT NULL DEFAULT 0,
    document_id     TEXT    REFERENCES documents (id) ON DELETE SET NULL,
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL
) STRICT;
CREATE INDEX inbox_items_organization ON inbox_items (organization_id, status, created_at DESC);

-- PDF (position 0) und Seitenbilder (1, 2, …) eines Eintrags; nach der Ablage gelöscht.
CREATE TABLE inbox_files (
    inbox_id TEXT    NOT NULL REFERENCES inbox_items (id) ON DELETE CASCADE,
    kind     TEXT    NOT NULL CHECK (kind IN ('pdf', 'page')),
    position INTEGER NOT NULL,
    data     BLOB    NOT NULL,
    PRIMARY KEY (inbox_id, kind, position)
) STRICT;
