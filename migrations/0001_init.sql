-- nrzPaper – Grundschema.
-- Alles liegt in dieser einen, mit SQLCipher verschlüsselten Datenbank.
-- Konventionen:
--   * IDs sind UUIDv7 als TEXT (zeitlich sortierbar).
--   * Zeitpunkte sind Unix-Sekunden (UTC) als INTEGER.
--   * STRICT-Tabellen: SQLite prüft die Spaltentypen.
--   * Aufzählungen sind TEXT mit CHECK – passend zu den Rust-Enums in src/db/models.rs.

-- ============================================================ Benutzer & Anmeldung

CREATE TABLE users (
    id            TEXT    PRIMARY KEY NOT NULL,
    username      TEXT    NOT NULL UNIQUE COLLATE NOCASE,
    email         TEXT    UNIQUE COLLATE NOCASE,
    display_name  TEXT    NOT NULL DEFAULT '',
    password_hash TEXT    NOT NULL,                  -- Argon2id (PHC-String)
    is_admin      INTEGER NOT NULL DEFAULT 0 CHECK (is_admin IN (0, 1)),
    created_at    INTEGER NOT NULL,
    updated_at    INTEGER NOT NULL
) STRICT;

-- Sitzungen: der Browser hält ein Zufalls-Token, hier liegt nur dessen SHA-256.
CREATE TABLE sessions (
    token_hash   TEXT    PRIMARY KEY NOT NULL,
    user_id      TEXT    NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    created_at   INTEGER NOT NULL,
    expires_at   INTEGER NOT NULL,
    last_seen_at INTEGER NOT NULL,
    user_agent   TEXT    NOT NULL DEFAULT ''
) STRICT;
CREATE INDEX sessions_user ON sessions (user_id);
CREATE INDEX sessions_expires ON sessions (expires_at);

-- API-Keys gehören einem Benutzer; gespeichert wird nur der SHA-256 des Keys.
CREATE TABLE api_keys (
    id           TEXT    PRIMARY KEY NOT NULL,
    user_id      TEXT    NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    name         TEXT    NOT NULL,
    prefix       TEXT    NOT NULL,                   -- erste Zeichen, zur Wiedererkennung
    key_hash     TEXT    NOT NULL UNIQUE,
    permissions  TEXT    NOT NULL DEFAULT '[]' CHECK (json_valid(permissions)),
    created_at   INTEGER NOT NULL,
    last_used_at INTEGER,
    expires_at   INTEGER
) STRICT;
CREATE INDEX api_keys_user ON api_keys (user_id);

-- ============================================================ Organisationen

CREATE TABLE organizations (
    id         TEXT    PRIMARY KEY NOT NULL,
    name       TEXT    NOT NULL CHECK (length(name) BETWEEN 1 AND 100),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    deleted_at INTEGER                                -- Papierkorb für ganze Organisationen
) STRICT;

CREATE TABLE organization_members (
    organization_id TEXT    NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    user_id         TEXT    NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    role            TEXT    NOT NULL CHECK (role IN ('owner', 'admin', 'member')),
    created_at      INTEGER NOT NULL,
    PRIMARY KEY (organization_id, user_id)
) STRICT;
CREATE INDEX organization_members_user ON organization_members (user_id);

CREATE TABLE organization_invitations (
    id              TEXT    PRIMARY KEY NOT NULL,
    organization_id TEXT    NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    invitee         TEXT    NOT NULL COLLATE NOCASE,  -- Benutzername oder E-Mail
    role            TEXT    NOT NULL CHECK (role IN ('admin', 'member')),
    invited_by      TEXT    REFERENCES users (id) ON DELETE SET NULL,
    created_at      INTEGER NOT NULL,
    expires_at      INTEGER NOT NULL,
    accepted_at     INTEGER,
    UNIQUE (organization_id, invitee)
) STRICT;

-- ============================================================ Dokumente

CREATE TABLE documents (
    -- Feste Zeilennummer für den Volltextindex. Ohne eigenen INTEGER PRIMARY KEY darf
    -- VACUUM die interne rowid neu vergeben – dann zeigte documents_fts ins Leere.
    num             INTEGER PRIMARY KEY,
    id              TEXT    NOT NULL UNIQUE,
    organization_id TEXT    NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    name            TEXT    NOT NULL,
    original_name   TEXT    NOT NULL,
    mime_type       TEXT    NOT NULL,
    size            INTEGER NOT NULL CHECK (size >= 0),
    sha256          TEXT    NOT NULL,
    document_date   INTEGER,                          -- Datum des Dokuments selbst
    content         TEXT    NOT NULL DEFAULT '',      -- erkannter bzw. bearbeiteter Text
    page_count      INTEGER,
    ocr_status      TEXT    NOT NULL DEFAULT 'pending'
                            CHECK (ocr_status IN ('pending', 'processing', 'done', 'failed', 'skipped')),
    ocr_error       TEXT,
    source          TEXT    NOT NULL DEFAULT 'web' CHECK (source IN ('web', 'api', 'ingestion')),
    created_by      TEXT    REFERENCES users (id) ON DELETE SET NULL,
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL,
    deleted_at      INTEGER,                          -- im Papierkorb seit
    deleted_by      TEXT    REFERENCES users (id) ON DELETE SET NULL,
    -- Dieselbe Datei gibt es je Organisation nur einmal (auch im Papierkorb –
    -- ein erneuter Upload holt sie dort heraus, wie bei Papra).
    UNIQUE (organization_id, sha256)
) STRICT;
CREATE INDEX documents_org_listing ON documents (organization_id, deleted_at, created_at DESC);
CREATE INDEX documents_ocr_queue ON documents (ocr_status) WHERE ocr_status IN ('pending', 'processing');
CREATE INDEX documents_trash ON documents (deleted_at) WHERE deleted_at IS NOT NULL;

-- Datei-Inhalt in Blöcken (1 MiB). Erlaubt Streaming und Range-Anfragen.
CREATE TABLE document_chunks (
    document_id TEXT    NOT NULL REFERENCES documents (id) ON DELETE CASCADE,
    seq         INTEGER NOT NULL CHECK (seq >= 0),
    data        BLOB    NOT NULL,
    PRIMARY KEY (document_id, seq)
) STRICT;

-- Volltextsuche über Name und Inhalt. External-Content-Tabelle: der Text liegt nur in
-- `documents`, der Index wird über Trigger nachgeführt.
CREATE VIRTUAL TABLE documents_fts USING fts5 (
    name,
    content,
    content = 'documents',
    content_rowid = 'num',
    tokenize = 'unicode61 remove_diacritics 2'
);

CREATE TRIGGER documents_fts_insert AFTER INSERT ON documents BEGIN
    INSERT INTO documents_fts (rowid, name, content) VALUES (new.num, new.name, new.content);
END;
CREATE TRIGGER documents_fts_delete AFTER DELETE ON documents BEGIN
    INSERT INTO documents_fts (documents_fts, rowid, name, content) VALUES ('delete', old.num, old.name, old.content);
END;
CREATE TRIGGER documents_fts_update AFTER UPDATE OF name, content ON documents BEGIN
    INSERT INTO documents_fts (documents_fts, rowid, name, content) VALUES ('delete', old.num, old.name, old.content);
    INSERT INTO documents_fts (rowid, name, content) VALUES (new.num, new.name, new.content);
END;

-- Verlauf je Dokument (erstellt, umbenannt, getaggt, gelöscht, wiederhergestellt …).
CREATE TABLE document_activity (
    id          TEXT    PRIMARY KEY NOT NULL,
    document_id TEXT    NOT NULL REFERENCES documents (id) ON DELETE CASCADE,
    event       TEXT    NOT NULL,
    user_id     TEXT    REFERENCES users (id) ON DELETE SET NULL,
    data        TEXT    NOT NULL DEFAULT '{}' CHECK (json_valid(data)),
    created_at  INTEGER NOT NULL
) STRICT;
CREATE INDEX document_activity_document ON document_activity (document_id, created_at DESC);

-- ============================================================ Tags & Regeln

CREATE TABLE tags (
    id              TEXT    PRIMARY KEY NOT NULL,
    organization_id TEXT    NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    name            TEXT    NOT NULL COLLATE NOCASE CHECK (length(name) BETWEEN 1 AND 64),
    color           TEXT    NOT NULL DEFAULT '#71717a' CHECK (color GLOB '#[0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f]'),
    description     TEXT    NOT NULL DEFAULT '',
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL,
    UNIQUE (organization_id, name)
) STRICT;

CREATE TABLE document_tags (
    document_id TEXT    NOT NULL REFERENCES documents (id) ON DELETE CASCADE,
    tag_id      TEXT    NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
    created_at  INTEGER NOT NULL,
    PRIMARY KEY (document_id, tag_id)
) STRICT;
CREATE INDEX document_tags_tag ON document_tags (tag_id);

-- Tagging-Regeln: alle bzw. eine der Bedingungen erfüllt → Tags setzen.
CREATE TABLE tagging_rules (
    id              TEXT    PRIMARY KEY NOT NULL,
    organization_id TEXT    NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    name            TEXT    NOT NULL,
    description     TEXT    NOT NULL DEFAULT '',
    enabled         INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
    match_mode      TEXT    NOT NULL DEFAULT 'all' CHECK (match_mode IN ('all', 'any')),
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL
) STRICT;
CREATE INDEX tagging_rules_org ON tagging_rules (organization_id);

CREATE TABLE tagging_rule_conditions (
    id             TEXT    PRIMARY KEY NOT NULL,
    rule_id        TEXT    NOT NULL REFERENCES tagging_rules (id) ON DELETE CASCADE,
    field          TEXT    NOT NULL CHECK (field IN ('name', 'content')),
    operator       TEXT    NOT NULL CHECK (operator IN ('contains', 'not_contains', 'equals', 'starts_with', 'ends_with', 'regex')),
    value          TEXT    NOT NULL,
    case_sensitive INTEGER NOT NULL DEFAULT 0 CHECK (case_sensitive IN (0, 1))
) STRICT;
CREATE INDEX tagging_rule_conditions_rule ON tagging_rule_conditions (rule_id);

CREATE TABLE tagging_rule_actions (
    rule_id TEXT NOT NULL REFERENCES tagging_rules (id) ON DELETE CASCADE,
    tag_id  TEXT NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
    PRIMARY KEY (rule_id, tag_id)
) STRICT;

-- ============================================================ Benutzerdefinierte Eigenschaften

CREATE TABLE custom_property_definitions (
    id              TEXT    PRIMARY KEY NOT NULL,
    organization_id TEXT    NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    name            TEXT    NOT NULL COLLATE NOCASE CHECK (length(name) BETWEEN 1 AND 64),
    description     TEXT    NOT NULL DEFAULT '',
    type            TEXT    NOT NULL CHECK (type IN ('text', 'number', 'date', 'boolean', 'select', 'multi_select', 'user_relation', 'document_relation')),
    display_order   INTEGER NOT NULL DEFAULT 0,
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL,
    UNIQUE (organization_id, name)
) STRICT;

CREATE TABLE custom_property_options (
    id            TEXT    PRIMARY KEY NOT NULL,
    definition_id TEXT    NOT NULL REFERENCES custom_property_definitions (id) ON DELETE CASCADE,
    name          TEXT    NOT NULL,
    display_order INTEGER NOT NULL DEFAULT 0,
    UNIQUE (definition_id, name)
) STRICT;

-- Ein Wert je Zeile; Mehrfachauswahl und Bezüge haben mehrere Zeilen je Eigenschaft.
-- Welche Spalte gefüllt ist, ergibt sich aus dem Typ der Definition.
CREATE TABLE document_property_values (
    id                  TEXT    PRIMARY KEY NOT NULL,
    document_id         TEXT    NOT NULL REFERENCES documents (id) ON DELETE CASCADE,
    definition_id       TEXT    NOT NULL REFERENCES custom_property_definitions (id) ON DELETE CASCADE,
    text_value          TEXT,
    number_value        REAL,
    date_value          INTEGER,
    boolean_value       INTEGER CHECK (boolean_value IN (0, 1)),
    option_id           TEXT    REFERENCES custom_property_options (id) ON DELETE CASCADE,
    user_id             TEXT    REFERENCES users (id) ON DELETE CASCADE,
    related_document_id TEXT    REFERENCES documents (id) ON DELETE CASCADE,
    CHECK (
        (text_value IS NOT NULL) + (number_value IS NOT NULL) + (date_value IS NOT NULL) + (boolean_value IS NOT NULL)
        + (option_id IS NOT NULL) + (user_id IS NOT NULL) + (related_document_id IS NOT NULL) = 1
    )
) STRICT;
CREATE INDEX document_property_values_document ON document_property_values (document_id, definition_id);
CREATE INDEX document_property_values_definition ON document_property_values (definition_id);

-- ============================================================ Freigaben & Webhooks

-- Öffentlicher Link auf ein Dokument; gespeichert wird nur der Hash des Tokens.
CREATE TABLE share_links (
    id          TEXT    PRIMARY KEY NOT NULL,
    document_id TEXT    NOT NULL REFERENCES documents (id) ON DELETE CASCADE,
    token_hash  TEXT    NOT NULL UNIQUE,
    created_by  TEXT    REFERENCES users (id) ON DELETE SET NULL,
    created_at  INTEGER NOT NULL,
    expires_at  INTEGER,
    revoked_at  INTEGER,
    view_count  INTEGER NOT NULL DEFAULT 0
) STRICT;
CREATE INDEX share_links_document ON share_links (document_id);

CREATE TABLE webhooks (
    id              TEXT    PRIMARY KEY NOT NULL,
    organization_id TEXT    NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    name            TEXT    NOT NULL,
    url             TEXT    NOT NULL CHECK (url LIKE 'http://%' OR url LIKE 'https://%'),
    secret          TEXT,                             -- signiert die Zustellungen
    enabled         INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
    created_by      TEXT    REFERENCES users (id) ON DELETE SET NULL,
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL
) STRICT;
CREATE INDEX webhooks_org ON webhooks (organization_id);

CREATE TABLE webhook_events (
    webhook_id TEXT NOT NULL REFERENCES webhooks (id) ON DELETE CASCADE,
    event      TEXT NOT NULL CHECK (event IN ('document:created', 'document:updated', 'document:deleted', 'document:restored', 'document:tag:added', 'document:tag:removed')),
    PRIMARY KEY (webhook_id, event)
) STRICT;

CREATE TABLE webhook_deliveries (
    id              TEXT    PRIMARY KEY NOT NULL,
    webhook_id      TEXT    NOT NULL REFERENCES webhooks (id) ON DELETE CASCADE,
    event           TEXT    NOT NULL,
    payload         TEXT    NOT NULL,
    response_status INTEGER,
    response_body   TEXT,                             -- gekürzt
    error           TEXT,
    created_at      INTEGER NOT NULL
) STRICT;
CREATE INDEX webhook_deliveries_webhook ON webhook_deliveries (webhook_id, created_at DESC);

-- ============================================================ Einstellungen

-- Instanzweite Einstellungen, die in der Oberfläche änderbar sind (z. B. Registrierung offen).
CREATE TABLE app_settings (
    key        TEXT    PRIMARY KEY NOT NULL,
    value      TEXT    NOT NULL CHECK (json_valid(value)),
    updated_at INTEGER NOT NULL
) STRICT;
