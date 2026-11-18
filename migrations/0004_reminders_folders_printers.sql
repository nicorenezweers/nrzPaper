-- Erinnerungen an Dokumente, Herkunft der Inbox-Einträge, Eingangsordner zur Prüfung, Drucker.

-- „Erinnere mich in 10 Tagen“: je Benutzer und Dokument, erledigt oder offen.
CREATE TABLE reminders (
    id              TEXT    PRIMARY KEY NOT NULL,
    organization_id TEXT    NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    document_id     TEXT    NOT NULL REFERENCES documents (id) ON DELETE CASCADE,
    user_id         TEXT    NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    due_at          INTEGER NOT NULL,
    note            TEXT    NOT NULL DEFAULT '' CHECK (length(note) <= 500),
    done_at         INTEGER,
    created_at      INTEGER NOT NULL
) STRICT;
CREATE INDEX reminders_user ON reminders (user_id, done_at, due_at);
CREATE INDEX reminders_document ON reminders (document_id);

-- Woher genau ein Inbox-Eintrag kam: ID des Postfachs ("mail:<id>") oder des Ordners
-- ("folder:<id>"); leer bei Scan und Upload.
ALTER TABLE inbox_items ADD COLUMN source_ref TEXT NOT NULL DEFAULT '';
CREATE INDEX inbox_items_source ON inbox_items (organization_id, source, source_ref);

-- Ordner auf dem Server, deren Dateien (PDFs, Bilder) zur Prüfung in die Inbox kommen –
-- z. B. der Scan-Ordner eines Multifunktionsgeräts auf dem NAS.
CREATE TABLE watch_folders (
    id              TEXT    PRIMARY KEY NOT NULL,
    organization_id TEXT    NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    created_by      TEXT    REFERENCES users (id) ON DELETE SET NULL,
    name            TEXT    NOT NULL CHECK (length(name) BETWEEN 1 AND 100),
    path            TEXT    NOT NULL,
    tags            TEXT    NOT NULL DEFAULT '[]' CHECK (json_valid(tags)),
    -- danach löschen (1) oder in den Unterordner "nrzpaper-importiert" verschieben (0)
    delete_after    INTEGER NOT NULL DEFAULT 0 CHECK (delete_after IN (0, 1)),
    enabled         INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
    last_check_at   INTEGER,
    last_error      TEXT    NOT NULL DEFAULT '',
    imported_count  INTEGER NOT NULL DEFAULT 0,
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL
) STRICT;

-- Netzwerkdrucker (IPP / AirPrint / IPP Everywhere) der Instanz.
CREATE TABLE printers (
    id          TEXT    PRIMARY KEY NOT NULL,
    name        TEXT    NOT NULL CHECK (length(name) BETWEEN 1 AND 100),
    url         TEXT    NOT NULL UNIQUE,
    -- vom Drucker gemeldete Formate, z. B. ["application/pdf","image/jpeg"]
    formats     TEXT    NOT NULL DEFAULT '[]' CHECK (json_valid(formats)),
    make        TEXT    NOT NULL DEFAULT '',
    created_at  INTEGER NOT NULL
) STRICT;
