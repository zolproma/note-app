use rusqlite::Connection;

const MIGRATIONS: &[&str] = &[MIGRATION_001, MIGRATION_002];

const MIGRATION_001: &str = r#"
-- Workspaces
CREATE TABLE IF NOT EXISTS workspaces (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    description     TEXT,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

-- Notebooks
CREATE TABLE IF NOT EXISTS notebooks (
    id              TEXT PRIMARY KEY,
    workspace_id    TEXT NOT NULL REFERENCES workspaces(id),
    name            TEXT NOT NULL,
    description     TEXT,
    is_inbox        INTEGER NOT NULL DEFAULT 0,
    sort_order      INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_notebooks_workspace ON notebooks(workspace_id);

-- Notes
CREATE TABLE IF NOT EXISTS notes (
    id              TEXT PRIMARY KEY,
    notebook_id     TEXT NOT NULL REFERENCES notebooks(id),
    title           TEXT NOT NULL,
    template_id     TEXT,
    lifecycle       TEXT NOT NULL DEFAULT 'inbox',
    visibility      TEXT NOT NULL DEFAULT 'normal',
    ai_policy       TEXT NOT NULL DEFAULT 'allowed',
    pinned          INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_notes_notebook ON notes(notebook_id);
CREATE INDEX IF NOT EXISTS idx_notes_lifecycle ON notes(lifecycle);
CREATE INDEX IF NOT EXISTS idx_notes_updated ON notes(updated_at DESC);

-- Note blocks (structured document model)
CREATE TABLE IF NOT EXISTS note_blocks (
    id              TEXT PRIMARY KEY,
    note_id         TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    block_type      TEXT NOT NULL,
    content         TEXT NOT NULL DEFAULT '',
    sort_order      INTEGER NOT NULL DEFAULT 0,
    metadata        TEXT,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_blocks_note ON note_blocks(note_id, sort_order);

-- Tags
CREATE TABLE IF NOT EXISTS tags (
    id              TEXT PRIMARY KEY,
    workspace_id    TEXT NOT NULL REFERENCES workspaces(id),
    name            TEXT NOT NULL,
    color           TEXT,
    created_at      TEXT NOT NULL,
    UNIQUE(workspace_id, name)
);

-- Note-tag junction
CREATE TABLE IF NOT EXISTS note_tags (
    note_id         TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    tag_id          TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    created_at      TEXT NOT NULL,
    PRIMARY KEY (note_id, tag_id)
);

-- Attachments
CREATE TABLE IF NOT EXISTS attachments (
    id              TEXT PRIMARY KEY,
    note_id         TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    filename        TEXT NOT NULL,
    media_type      TEXT NOT NULL,
    storage_path    TEXT NOT NULL,
    size_bytes      INTEGER NOT NULL DEFAULT 0,
    mime_type       TEXT,
    policy_flags    TEXT NOT NULL DEFAULT '{}',
    created_at      TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_attachments_note ON attachments(note_id);

-- Links (wiki-links, block refs, AI-suggested relations)
CREATE TABLE IF NOT EXISTS links (
    id              TEXT PRIMARY KEY,
    source_note_id  TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    target_note_id  TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    source_block_id TEXT,
    target_block_id TEXT,
    link_type       TEXT NOT NULL DEFAULT 'wiki_link',
    created_at      TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_links_source ON links(source_note_id);
CREATE INDEX IF NOT EXISTS idx_links_target ON links(target_note_id);

-- Aliases (alternative names for notes, used in search and link resolution)
CREATE TABLE IF NOT EXISTS aliases (
    id              TEXT PRIMARY KEY,
    note_id         TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    alias_text      TEXT NOT NULL,
    created_at      TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_aliases_note ON aliases(note_id);
CREATE INDEX IF NOT EXISTS idx_aliases_text ON aliases(alias_text);

-- Templates
CREATE TABLE IF NOT EXISTS templates (
    id              TEXT PRIMARY KEY,
    kind            TEXT NOT NULL UNIQUE,
    name            TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT ''
);

-- AI jobs
CREATE TABLE IF NOT EXISTS ai_jobs (
    id              TEXT PRIMARY KEY,
    workspace_id    TEXT NOT NULL,
    job_type        TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'pending',
    mode            TEXT NOT NULL DEFAULT 'blocked_remote',
    scope_json      TEXT NOT NULL DEFAULT '{}',
    result_json     TEXT,
    error_message   TEXT,
    created_at      TEXT NOT NULL,
    completed_at    TEXT
);

-- AI audit log (append-only)
CREATE TABLE IF NOT EXISTS ai_audit_logs (
    id              TEXT PRIMARY KEY,
    job_id          TEXT REFERENCES ai_jobs(id),
    action          TEXT NOT NULL,
    mode            TEXT NOT NULL,
    scope_snapshot  TEXT NOT NULL,
    policy_snapshot TEXT NOT NULL DEFAULT '{}',
    notes_accessed  TEXT NOT NULL DEFAULT '[]',
    network_targets TEXT NOT NULL DEFAULT '[]',
    diff_summary    TEXT,
    approval_state  TEXT NOT NULL DEFAULT 'pending',
    created_at      TEXT NOT NULL
);

-- Full-text search on note content
CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(
    note_id UNINDEXED,
    title,
    content,
    tags,
    aliases,
    content_rowid=rowid
);

-- Settings
CREATE TABLE IF NOT EXISTS settings (
    key             TEXT PRIMARY KEY,
    value           TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

-- Schema version
CREATE TABLE IF NOT EXISTS schema_version (
    version         INTEGER PRIMARY KEY
);
INSERT OR IGNORE INTO schema_version (version) VALUES (0);
"#;

const MIGRATION_002: &str = r#"
-- Saved searches
CREATE TABLE IF NOT EXISTS saved_searches (
    id              TEXT PRIMARY KEY,
    workspace_id    TEXT NOT NULL REFERENCES workspaces(id),
    name            TEXT NOT NULL,
    filter_json     TEXT NOT NULL DEFAULT '{}',
    created_at      TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_saved_searches_ws ON saved_searches(workspace_id);
"#;

pub fn run_migrations(conn: &Connection) -> Result<(), rusqlite::Error> {
    let current_version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    for (i, migration) in MIGRATIONS.iter().enumerate() {
        let version = (i + 1) as i32;
        if version > current_version {
            conn.execute_batch(migration)?;
            conn.execute(
                "INSERT OR REPLACE INTO schema_version (version) VALUES (?1)",
                [version],
            )?;
            tracing::info!("Applied migration v{version}");
        }
    }

    Ok(())
}
