-- Vorschaubilder, Ordner aus Tags und E-Mail-Eingang.

-- Kleines JPEG der ersten Seite (Kachelansicht); entsteht beim ersten Abruf.
CREATE TABLE document_thumbnails (
    document_id TEXT    PRIMARY KEY NOT NULL REFERENCES documents (id) ON DELETE CASCADE,
    data        BLOB    NOT NULL,
    created_at  INTEGER NOT NULL
) STRICT;

-- Ordner einer Organisation (auch verschachtelt). Ein Ordner zeigt alle Dokumente, die
-- mindestens einen seiner Tags tragen.
CREATE TABLE folders (
    id              TEXT    PRIMARY KEY NOT NULL,
    organization_id TEXT    NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    parent_id       TEXT    REFERENCES folders (id) ON DELETE CASCADE,
    name            TEXT    NOT NULL CHECK (length(name) BETWEEN 1 AND 100),
    position        INTEGER NOT NULL DEFAULT 0,
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL
) STRICT;
CREATE INDEX folders_organization ON folders (organization_id, parent_id, position);

CREATE TABLE folder_tags (
    folder_id TEXT NOT NULL REFERENCES folders (id) ON DELETE CASCADE,
    tag_id    TEXT NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
    PRIMARY KEY (folder_id, tag_id)
) STRICT;

-- E-Mail-Postfächer (nur Empfang, IMAP). PDFs und Bilder aus Anhängen landen in der Prüfliste.
-- Das Passwort liegt in der (verschlüsselten) Datenbank und verlässt sie nie Richtung Oberfläche.
CREATE TABLE mail_accounts (
    id              TEXT    PRIMARY KEY NOT NULL,
    user_id         TEXT    NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    organization_id TEXT    NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    name            TEXT    NOT NULL,
    host            TEXT    NOT NULL,
    port            INTEGER NOT NULL CHECK (port BETWEEN 1 AND 65535),
    security        TEXT    NOT NULL DEFAULT 'tls' CHECK (security IN ('tls', 'starttls', 'none')),
    username        TEXT    NOT NULL,
    password        TEXT    NOT NULL,
    folder          TEXT    NOT NULL DEFAULT 'INBOX',
    mark_seen       INTEGER NOT NULL DEFAULT 1 CHECK (mark_seen IN (0, 1)),
    tags            TEXT    NOT NULL DEFAULT '[]' CHECK (json_valid(tags)),
    enabled         INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
    interval_minutes INTEGER NOT NULL DEFAULT 5 CHECK (interval_minutes BETWEEN 1 AND 1440),
    last_check_at   INTEGER,
    last_error      TEXT    NOT NULL DEFAULT '',
    imported_count  INTEGER NOT NULL DEFAULT 0,
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL
) STRICT;

-- Schon verarbeitete Nachrichten (UIDVALIDITY:UID), damit nichts doppelt kommt.
CREATE TABLE mail_seen (
    account_id  TEXT    NOT NULL REFERENCES mail_accounts (id) ON DELETE CASCADE,
    message_key TEXT    NOT NULL,
    created_at  INTEGER NOT NULL,
    PRIMARY KEY (account_id, message_key)
) STRICT;

-- Herkunft eines Eintrags der Prüfliste (Scan, Import, E-Mail) für die Anzeige.
ALTER TABLE inbox_items ADD COLUMN source TEXT NOT NULL DEFAULT 'scan' CHECK (source IN ('scan', 'import', 'mail'));
ALTER TABLE inbox_items ADD COLUMN source_info TEXT NOT NULL DEFAULT '';
