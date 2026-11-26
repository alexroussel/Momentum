-- Momentum schema v1. Timestamps are unix epoch milliseconds (UTC).
-- Syncable tables carry created_at / updated_at / deleted_at (soft delete) and UUID v7 ids.

CREATE TABLE category (
    id          TEXT PRIMARY KEY NOT NULL,
    title       TEXT NOT NULL,
    hex_color   TEXT NOT NULL CHECK (length(hex_color) = 7),
    status      TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'archived')),
    position    INTEGER NOT NULL DEFAULT 0,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    deleted_at  INTEGER
);
CREATE INDEX idx_category_position ON category(position) WHERE deleted_at IS NULL;

CREATE TABLE session (
    id               TEXT PRIMARY KEY NOT NULL,
    intent           TEXT NOT NULL DEFAULT '',
    category_id      TEXT REFERENCES category(id),
    kind             TEXT NOT NULL CHECK (kind IN ('focus', 'break', 'long_break')),
    result           TEXT CHECK (result IN ('fullFocus', 'partialFocus', 'endEarly', 'abandoned')),
    planned_sec      INTEGER NOT NULL,
    started_at       INTEGER NOT NULL,
    ended_at         INTEGER,
    focus_sec        INTEGER NOT NULL DEFAULT 0,
    pause_sec        INTEGER NOT NULL DEFAULT 0,
    overtime_sec     INTEGER NOT NULL DEFAULT 0,
    mood             TEXT CHECK (mood IN ('distracted', 'neutral', 'focused')),
    notes_md         TEXT NOT NULL DEFAULT '',
    source_task_ref  TEXT,
    created_at       INTEGER NOT NULL,
    updated_at       INTEGER NOT NULL,
    deleted_at       INTEGER
);
CREATE INDEX idx_session_started ON session(started_at) WHERE deleted_at IS NULL;
CREATE INDEX idx_session_category_started ON session(category_id, started_at) WHERE deleted_at IS NULL;

CREATE TABLE pause_interval (
    id          TEXT PRIMARY KEY NOT NULL,
    session_id  TEXT NOT NULL REFERENCES session(id) ON DELETE CASCADE,
    started_at  INTEGER NOT NULL,
    ended_at    INTEGER,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    deleted_at  INTEGER
);
CREATE INDEX idx_pause_session ON pause_interval(session_id);

-- Full-text search over intents and notes (used in phase 4; kept in sync by triggers).
CREATE VIRTUAL TABLE session_fts USING fts5(intent, notes_md, content='session', content_rowid='rowid');
CREATE TRIGGER session_fts_ai AFTER INSERT ON session BEGIN
    INSERT INTO session_fts(rowid, intent, notes_md) VALUES (new.rowid, new.intent, new.notes_md);
END;
CREATE TRIGGER session_fts_ad AFTER DELETE ON session BEGIN
    INSERT INTO session_fts(session_fts, rowid, intent, notes_md) VALUES ('delete', old.rowid, old.intent, old.notes_md);
END;
CREATE TRIGGER session_fts_au AFTER UPDATE OF intent, notes_md ON session BEGIN
    INSERT INTO session_fts(session_fts, rowid, intent, notes_md) VALUES ('delete', old.rowid, old.intent, old.notes_md);
    INSERT INTO session_fts(rowid, intent, notes_md) VALUES (new.rowid, new.intent, new.notes_md);
END;

-- Singleton row holding the live timer so it survives restarts (ADR-002).
CREATE TABLE active_timer (
    id          INTEGER PRIMARY KEY CHECK (id = 1),
    state_json  TEXT NOT NULL,
    updated_at  INTEGER NOT NULL
);

CREATE TABLE calendar_source (
    id              TEXT PRIMARY KEY NOT NULL,
    kind            TEXT NOT NULL CHECK (kind IN ('ics', 'google', 'outlook')),
    name            TEXT NOT NULL,
    color           TEXT,
    url_or_account  TEXT NOT NULL,
    enabled         INTEGER NOT NULL DEFAULT 1,
    write_enabled   INTEGER NOT NULL DEFAULT 0,
    category_id     TEXT REFERENCES category(id),
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL,
    deleted_at      INTEGER
);

CREATE TABLE blocklist_entry (
    id          TEXT PRIMARY KEY NOT NULL,
    list        TEXT NOT NULL CHECK (list IN ('focus', 'break')),
    kind        TEXT NOT NULL CHECK (kind IN ('site', 'app')),
    pattern     TEXT NOT NULL,
    mode        TEXT NOT NULL DEFAULT 'block' CHECK (mode IN ('block', 'allow')),
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    deleted_at  INTEGER
);

CREATE TABLE automation (
    id          TEXT PRIMARY KEY NOT NULL,
    event       TEXT NOT NULL,
    kind        TEXT NOT NULL CHECK (kind IN ('script', 'webhook')),
    target      TEXT NOT NULL,
    enabled     INTEGER NOT NULL DEFAULT 1,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    deleted_at  INTEGER
);

CREATE TABLE automation_run (
    id             TEXT PRIMARY KEY NOT NULL,
    automation_id  TEXT REFERENCES automation(id) ON DELETE SET NULL,
    event          TEXT NOT NULL,
    started_at     INTEGER NOT NULL,
    duration_ms    INTEGER,
    exit_code      INTEGER,
    output         TEXT NOT NULL DEFAULT ''
);
CREATE INDEX idx_automation_run_started ON automation_run(started_at);

CREATE TABLE integration_account (
    id           TEXT PRIMARY KEY NOT NULL,
    kind         TEXT NOT NULL CHECK (kind IN ('slack', 'todoist', 'linear', 'github', 'google', 'outlook')),
    label        TEXT NOT NULL,
    keyring_ref  TEXT NOT NULL,
    created_at   INTEGER NOT NULL,
    updated_at   INTEGER NOT NULL,
    deleted_at   INTEGER
);

CREATE TABLE extension_pairing (
    id            TEXT PRIMARY KEY NOT NULL,
    browser       TEXT NOT NULL,
    token_hash    TEXT NOT NULL,
    created_at    INTEGER NOT NULL,
    last_seen_at  INTEGER
);

CREATE TABLE window_tracker_rule (
    id           TEXT PRIMARY KEY NOT NULL,
    keyword      TEXT NOT NULL,
    category_id  TEXT REFERENCES category(id),
    enabled      INTEGER NOT NULL DEFAULT 1,
    created_at   INTEGER NOT NULL,
    updated_at   INTEGER NOT NULL,
    deleted_at   INTEGER
);

CREATE TABLE task_cache (
    id           TEXT PRIMARY KEY NOT NULL,
    source       TEXT NOT NULL,
    external_id  TEXT NOT NULL,
    title        TEXT NOT NULL,
    url          TEXT,
    done         INTEGER NOT NULL DEFAULT 0,
    fetched_at   INTEGER NOT NULL,
    UNIQUE (source, external_id)
);

CREATE TABLE setting (
    key         TEXT PRIMARY KEY NOT NULL,
    value_json  TEXT NOT NULL,
    updated_at  INTEGER NOT NULL
);

CREATE TABLE daily_goal (
    date             TEXT PRIMARY KEY NOT NULL, -- local calendar date, YYYY-MM-DD
    target_sessions  INTEGER NOT NULL CHECK (target_sessions >= 0)
);
