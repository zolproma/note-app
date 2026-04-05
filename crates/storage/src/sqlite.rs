use std::path::Path;

use chrono::Utc;
use rusqlite::{params, Connection};
use uuid::Uuid;

use note_core::error::CoreError;
use note_core::model::*;
use note_core::service::NoteStore;

use crate::migration;

pub struct SqliteStore {
    conn: Connection,
}

impl SqliteStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, CoreError> {
        let conn =
            Connection::open(path).map_err(|e| CoreError::Storage(e.to_string()))?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(Self { conn })
    }

    pub fn in_memory() -> Result<Self, CoreError> {
        let conn =
            Connection::open_in_memory().map_err(|e| CoreError::Storage(e.to_string()))?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(Self { conn })
    }

    fn map_err(e: rusqlite::Error) -> CoreError {
        CoreError::Storage(e.to_string())
    }

    fn row_to_link(row: &rusqlite::Row<'_>) -> Result<Link, rusqlite::Error> {
        let lt_str: String = row.get(5)?;
        Ok(Link {
            id: row.get::<_, String>(0)?.parse().unwrap(),
            source_note_id: row.get::<_, String>(1)?.parse().unwrap(),
            target_note_id: row.get::<_, String>(2)?.parse().unwrap(),
            source_block_id: row.get::<_, Option<String>>(3)?.map(|s| s.parse().unwrap()),
            target_block_id: row.get::<_, Option<String>>(4)?.map(|s| s.parse().unwrap()),
            link_type: serde_json::from_value(serde_json::Value::String(lt_str))
                .unwrap_or(LinkType::WikiLink),
            created_at: row.get::<_, String>(6)?.parse().unwrap(),
        })
    }

    fn update_fts(&self, note: &Note) -> Result<(), CoreError> {
        let content = note.plain_text();

        // Aggregate tags for this note
        let tags: String = {
            let mut stmt = self.conn
                .prepare("SELECT t.name FROM tags t JOIN note_tags nt ON t.id = nt.tag_id WHERE nt.note_id = ?1")
                .map_err(Self::map_err)?;
            let names: Vec<String> = stmt
                .query_map(params![note.id.to_string()], |row| row.get(0))
                .map_err(Self::map_err)?
                .filter_map(|r| r.ok())
                .collect();
            names.join(" ")
        };

        // Aggregate aliases for this note
        let aliases: String = {
            let mut stmt = self.conn
                .prepare("SELECT alias_text FROM aliases WHERE note_id = ?1")
                .map_err(Self::map_err)?;
            let names: Vec<String> = stmt
                .query_map(params![note.id.to_string()], |row| row.get(0))
                .map_err(Self::map_err)?
                .filter_map(|r| r.ok())
                .collect();
            names.join(" ")
        };

        // Delete old FTS entry
        self.conn
            .execute(
                "DELETE FROM notes_fts WHERE note_id = ?1",
                params![note.id.to_string()],
            )
            .map_err(Self::map_err)?;
        // Insert new FTS entry with real tags and aliases
        self.conn
            .execute(
                "INSERT INTO notes_fts (note_id, title, content, tags, aliases) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![note.id.to_string(), note.title, content, tags, aliases],
            )
            .map_err(Self::map_err)?;
        Ok(())
    }

    /// Reindex FTS for a note by ID (loads the note internally)
    fn reindex_fts_by_id(&self, note_id: Uuid) -> Result<(), CoreError> {
        let note = NoteStore::get_note(self, note_id)?;
        let mut note_with_blocks = note;
        note_with_blocks.blocks = NoteStore::get_blocks(self, note_id)?;
        self.update_fts(&note_with_blocks)
    }
}

impl NoteStore for SqliteStore {
    fn init(&self) -> Result<(), CoreError> {
        migration::run_migrations(&self.conn).map_err(Self::map_err)
    }

    // === Workspace ===

    fn create_workspace(&self, ws: &Workspace) -> Result<(), CoreError> {
        self.conn
            .execute(
                "INSERT INTO workspaces (id, name, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    ws.id.to_string(),
                    ws.name,
                    ws.description,
                    ws.created_at.to_rfc3339(),
                    ws.updated_at.to_rfc3339(),
                ],
            )
            .map_err(Self::map_err)?;
        Ok(())
    }

    fn get_workspace(&self, id: Uuid) -> Result<Workspace, CoreError> {
        self.conn
            .query_row(
                "SELECT id, name, description, created_at, updated_at FROM workspaces WHERE id = ?1",
                params![id.to_string()],
                |row| {
                    Ok(Workspace {
                        id: row.get::<_, String>(0)?.parse().unwrap(),
                        name: row.get(1)?,
                        description: row.get(2)?,
                        created_at: row.get::<_, String>(3)?.parse().unwrap(),
                        updated_at: row.get::<_, String>(4)?.parse().unwrap(),
                    })
                },
            )
            .map_err(|_| CoreError::WorkspaceNotFound(id.to_string()))
    }

    fn list_workspaces(&self) -> Result<Vec<Workspace>, CoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, description, created_at, updated_at FROM workspaces ORDER BY name")
            .map_err(Self::map_err)?;
        let rows = stmt
            .query_map([], |row| {
                Ok(Workspace {
                    id: row.get::<_, String>(0)?.parse().unwrap(),
                    name: row.get(1)?,
                    description: row.get(2)?,
                    created_at: row.get::<_, String>(3)?.parse().unwrap(),
                    updated_at: row.get::<_, String>(4)?.parse().unwrap(),
                })
            })
            .map_err(Self::map_err)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Self::map_err)
    }

    // === Notebook ===

    fn create_notebook(&self, nb: &Notebook) -> Result<(), CoreError> {
        self.conn
            .execute(
                "INSERT INTO notebooks (id, workspace_id, name, description, is_inbox, sort_order, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    nb.id.to_string(),
                    nb.workspace_id.to_string(),
                    nb.name,
                    nb.description,
                    nb.is_inbox as i32,
                    nb.sort_order,
                    nb.created_at.to_rfc3339(),
                    nb.updated_at.to_rfc3339(),
                ],
            )
            .map_err(Self::map_err)?;
        Ok(())
    }

    fn get_notebook(&self, id: Uuid) -> Result<Notebook, CoreError> {
        self.conn
            .query_row(
                "SELECT id, workspace_id, name, description, is_inbox, sort_order, created_at, updated_at FROM notebooks WHERE id = ?1",
                params![id.to_string()],
                |row| {
                    Ok(Notebook {
                        id: row.get::<_, String>(0)?.parse().unwrap(),
                        workspace_id: row.get::<_, String>(1)?.parse().unwrap(),
                        name: row.get(2)?,
                        description: row.get(3)?,
                        is_inbox: row.get::<_, i32>(4)? != 0,
                        sort_order: row.get(5)?,
                        created_at: row.get::<_, String>(6)?.parse().unwrap(),
                        updated_at: row.get::<_, String>(7)?.parse().unwrap(),
                    })
                },
            )
            .map_err(|_| CoreError::NotebookNotFound(id.to_string()))
    }

    fn list_notebooks(&self, workspace_id: Uuid) -> Result<Vec<Notebook>, CoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, workspace_id, name, description, is_inbox, sort_order, created_at, updated_at FROM notebooks WHERE workspace_id = ?1 ORDER BY is_inbox DESC, sort_order, name")
            .map_err(Self::map_err)?;
        let rows = stmt
            .query_map(params![workspace_id.to_string()], |row| {
                Ok(Notebook {
                    id: row.get::<_, String>(0)?.parse().unwrap(),
                    workspace_id: row.get::<_, String>(1)?.parse().unwrap(),
                    name: row.get(2)?,
                    description: row.get(3)?,
                    is_inbox: row.get::<_, i32>(4)? != 0,
                    sort_order: row.get(5)?,
                    created_at: row.get::<_, String>(6)?.parse().unwrap(),
                    updated_at: row.get::<_, String>(7)?.parse().unwrap(),
                })
            })
            .map_err(Self::map_err)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Self::map_err)
    }

    fn get_inbox(&self, workspace_id: Uuid) -> Result<Notebook, CoreError> {
        self.conn
            .query_row(
                "SELECT id, workspace_id, name, description, is_inbox, sort_order, created_at, updated_at FROM notebooks WHERE workspace_id = ?1 AND is_inbox = 1",
                params![workspace_id.to_string()],
                |row| {
                    Ok(Notebook {
                        id: row.get::<_, String>(0)?.parse().unwrap(),
                        workspace_id: row.get::<_, String>(1)?.parse().unwrap(),
                        name: row.get(2)?,
                        description: row.get(3)?,
                        is_inbox: row.get::<_, i32>(4)? != 0,
                        sort_order: row.get(5)?,
                        created_at: row.get::<_, String>(6)?.parse().unwrap(),
                        updated_at: row.get::<_, String>(7)?.parse().unwrap(),
                    })
                },
            )
            .map_err(|_| CoreError::NotebookNotFound("inbox".into()))
    }

    // === Note ===

    fn create_note(&self, note: &Note) -> Result<(), CoreError> {
        self.conn
            .execute(
                "INSERT INTO notes (id, notebook_id, title, template_id, lifecycle, visibility, ai_policy, pinned, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    note.id.to_string(),
                    note.notebook_id.to_string(),
                    note.title,
                    note.template_id.map(|id| id.to_string()),
                    note.lifecycle.to_string(),
                    serde_json::to_value(note.visibility).unwrap().as_str().unwrap(),
                    serde_json::to_value(note.ai_policy).unwrap().as_str().unwrap(),
                    note.pinned as i32,
                    note.created_at.to_rfc3339(),
                    note.updated_at.to_rfc3339(),
                ],
            )
            .map_err(Self::map_err)?;

        if !note.blocks.is_empty() {
            self.update_fts(note)?;
        }

        Ok(())
    }

    fn get_note(&self, id: Uuid) -> Result<Note, CoreError> {
        self.conn
            .query_row(
                "SELECT id, notebook_id, title, template_id, lifecycle, visibility, ai_policy, pinned, created_at, updated_at FROM notes WHERE id = ?1",
                params![id.to_string()],
                |row| {
                    let lifecycle_str: String = row.get(4)?;
                    let visibility_str: String = row.get(5)?;
                    let ai_policy_str: String = row.get(6)?;
                    Ok(Note {
                        id: row.get::<_, String>(0)?.parse().unwrap(),
                        notebook_id: row.get::<_, String>(1)?.parse().unwrap(),
                        title: row.get(2)?,
                        template_id: row.get::<_, Option<String>>(3)?.map(|s| s.parse().unwrap()),
                        lifecycle: lifecycle_str.parse().unwrap_or_default(),
                        visibility: serde_json::from_value(serde_json::Value::String(visibility_str)).unwrap_or_default(),
                        ai_policy: serde_json::from_value(serde_json::Value::String(ai_policy_str)).unwrap_or_default(),
                        blocks: Vec::new(),
                        pinned: row.get::<_, i32>(7)? != 0,
                        created_at: row.get::<_, String>(8)?.parse().unwrap(),
                        updated_at: row.get::<_, String>(9)?.parse().unwrap(),
                    })
                },
            )
            .map_err(|_| CoreError::NoteNotFound(id.to_string()))
    }

    fn update_note(&self, note: &Note) -> Result<(), CoreError> {
        let now = Utc::now();
        self.conn
            .execute(
                "UPDATE notes SET title = ?1, lifecycle = ?2, visibility = ?3, ai_policy = ?4, pinned = ?5, updated_at = ?6 WHERE id = ?7",
                params![
                    note.title,
                    note.lifecycle.to_string(),
                    serde_json::to_value(note.visibility).unwrap().as_str().unwrap(),
                    serde_json::to_value(note.ai_policy).unwrap().as_str().unwrap(),
                    note.pinned as i32,
                    now.to_rfc3339(),
                    note.id.to_string(),
                ],
            )
            .map_err(Self::map_err)?;
        Ok(())
    }

    fn delete_note(&self, id: Uuid) -> Result<(), CoreError> {
        // Soft delete: move to trashed
        let now = Utc::now();
        self.conn
            .execute(
                "UPDATE notes SET lifecycle = 'trashed', updated_at = ?1 WHERE id = ?2",
                params![now.to_rfc3339(), id.to_string()],
            )
            .map_err(Self::map_err)?;
        // Remove from FTS
        self.conn
            .execute(
                "DELETE FROM notes_fts WHERE note_id = ?1",
                params![id.to_string()],
            )
            .map_err(Self::map_err)?;
        Ok(())
    }

    fn list_notes(&self, notebook_id: Uuid) -> Result<Vec<Note>, CoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, notebook_id, title, template_id, lifecycle, visibility, ai_policy, pinned, created_at, updated_at FROM notes WHERE notebook_id = ?1 AND lifecycle != 'trashed' ORDER BY pinned DESC, updated_at DESC")
            .map_err(Self::map_err)?;
        let rows = stmt
            .query_map(params![notebook_id.to_string()], |row| {
                let lifecycle_str: String = row.get(4)?;
                let visibility_str: String = row.get(5)?;
                let ai_policy_str: String = row.get(6)?;
                Ok(Note {
                    id: row.get::<_, String>(0)?.parse().unwrap(),
                    notebook_id: row.get::<_, String>(1)?.parse().unwrap(),
                    title: row.get(2)?,
                    template_id: row.get::<_, Option<String>>(3)?.map(|s| s.parse().unwrap()),
                    lifecycle: lifecycle_str.parse().unwrap_or_default(),
                    visibility: serde_json::from_value(serde_json::Value::String(visibility_str)).unwrap_or_default(),
                    ai_policy: serde_json::from_value(serde_json::Value::String(ai_policy_str)).unwrap_or_default(),
                    blocks: Vec::new(),
                    pinned: row.get::<_, i32>(7)? != 0,
                    created_at: row.get::<_, String>(8)?.parse().unwrap(),
                    updated_at: row.get::<_, String>(9)?.parse().unwrap(),
                })
            })
            .map_err(Self::map_err)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Self::map_err)
    }

    fn list_notes_by_lifecycle(
        &self,
        workspace_id: Uuid,
        lifecycle: NoteLifecycle,
    ) -> Result<Vec<Note>, CoreError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT n.id, n.notebook_id, n.title, n.template_id, n.lifecycle, n.visibility, n.ai_policy, n.pinned, n.created_at, n.updated_at FROM notes n JOIN notebooks nb ON n.notebook_id = nb.id WHERE nb.workspace_id = ?1 AND n.lifecycle = ?2 ORDER BY n.updated_at DESC",
            )
            .map_err(Self::map_err)?;
        let rows = stmt
            .query_map(params![workspace_id.to_string(), lifecycle.to_string()], |row| {
                let lifecycle_str: String = row.get(4)?;
                let visibility_str: String = row.get(5)?;
                let ai_policy_str: String = row.get(6)?;
                Ok(Note {
                    id: row.get::<_, String>(0)?.parse().unwrap(),
                    notebook_id: row.get::<_, String>(1)?.parse().unwrap(),
                    title: row.get(2)?,
                    template_id: row.get::<_, Option<String>>(3)?.map(|s| s.parse().unwrap()),
                    lifecycle: lifecycle_str.parse().unwrap_or_default(),
                    visibility: serde_json::from_value(serde_json::Value::String(visibility_str)).unwrap_or_default(),
                    ai_policy: serde_json::from_value(serde_json::Value::String(ai_policy_str)).unwrap_or_default(),
                    blocks: Vec::new(),
                    pinned: row.get::<_, i32>(7)? != 0,
                    created_at: row.get::<_, String>(8)?.parse().unwrap(),
                    updated_at: row.get::<_, String>(9)?.parse().unwrap(),
                })
            })
            .map_err(Self::map_err)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Self::map_err)
    }

    fn search_notes(&self, workspace_id: Uuid, query: &str) -> Result<Vec<Note>, CoreError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT n.id, n.notebook_id, n.title, n.template_id, n.lifecycle, n.visibility, n.ai_policy, n.pinned, n.created_at, n.updated_at FROM notes n JOIN notebooks nb ON n.notebook_id = nb.id WHERE nb.workspace_id = ?1 AND n.id IN (SELECT note_id FROM notes_fts WHERE notes_fts MATCH ?2) AND n.lifecycle != 'trashed' ORDER BY n.updated_at DESC",
            )
            .map_err(Self::map_err)?;
        let rows = stmt
            .query_map(params![workspace_id.to_string(), query], |row| {
                let lifecycle_str: String = row.get(4)?;
                let visibility_str: String = row.get(5)?;
                let ai_policy_str: String = row.get(6)?;
                Ok(Note {
                    id: row.get::<_, String>(0)?.parse().unwrap(),
                    notebook_id: row.get::<_, String>(1)?.parse().unwrap(),
                    title: row.get(2)?,
                    template_id: row.get::<_, Option<String>>(3)?.map(|s| s.parse().unwrap()),
                    lifecycle: lifecycle_str.parse().unwrap_or_default(),
                    visibility: serde_json::from_value(serde_json::Value::String(visibility_str)).unwrap_or_default(),
                    ai_policy: serde_json::from_value(serde_json::Value::String(ai_policy_str)).unwrap_or_default(),
                    blocks: Vec::new(),
                    pinned: row.get::<_, i32>(7)? != 0,
                    created_at: row.get::<_, String>(8)?.parse().unwrap(),
                    updated_at: row.get::<_, String>(9)?.parse().unwrap(),
                })
            })
            .map_err(Self::map_err)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Self::map_err)
    }

    // === Block ===

    fn save_blocks(&self, note_id: Uuid, blocks: &[Block]) -> Result<(), CoreError> {
        // Delete existing blocks
        self.conn
            .execute(
                "DELETE FROM note_blocks WHERE note_id = ?1",
                params![note_id.to_string()],
            )
            .map_err(Self::map_err)?;
        // Insert new blocks
        for block in blocks {
            self.conn
                .execute(
                    "INSERT INTO note_blocks (id, note_id, block_type, content, sort_order, metadata, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![
                        block.id.to_string(),
                        note_id.to_string(),
                        block.block_type.to_string(),
                        block.content,
                        block.sort_order,
                        block.metadata.as_ref().map(|m| m.to_string()),
                        block.created_at.to_rfc3339(),
                        block.updated_at.to_rfc3339(),
                    ],
                )
                .map_err(Self::map_err)?;
        }
        Ok(())
    }

    fn get_blocks(&self, note_id: Uuid) -> Result<Vec<Block>, CoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, note_id, block_type, content, sort_order, metadata, created_at, updated_at FROM note_blocks WHERE note_id = ?1 ORDER BY sort_order")
            .map_err(Self::map_err)?;
        let rows = stmt
            .query_map(params![note_id.to_string()], |row| {
                let bt_str: String = row.get(2)?;
                let metadata_str: Option<String> = row.get(5)?;
                Ok(Block {
                    id: row.get::<_, String>(0)?.parse().unwrap(),
                    note_id: row.get::<_, String>(1)?.parse().unwrap(),
                    block_type: serde_json::from_value(serde_json::Value::String(bt_str))
                        .unwrap_or(BlockType::Text),
                    content: row.get(3)?,
                    sort_order: row.get(4)?,
                    metadata: metadata_str
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    created_at: row.get::<_, String>(6)?.parse().unwrap(),
                    updated_at: row.get::<_, String>(7)?.parse().unwrap(),
                })
            })
            .map_err(Self::map_err)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Self::map_err)
    }

    // === Tag ===

    fn create_tag(&self, tag: &Tag) -> Result<(), CoreError> {
        self.conn
            .execute(
                "INSERT INTO tags (id, workspace_id, name, color, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    tag.id.to_string(),
                    tag.workspace_id.to_string(),
                    tag.name,
                    tag.color,
                    tag.created_at.to_rfc3339(),
                ],
            )
            .map_err(Self::map_err)?;
        Ok(())
    }

    fn list_tags(&self, workspace_id: Uuid) -> Result<Vec<Tag>, CoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, workspace_id, name, color, created_at FROM tags WHERE workspace_id = ?1 ORDER BY name")
            .map_err(Self::map_err)?;
        let rows = stmt
            .query_map(params![workspace_id.to_string()], |row| {
                Ok(Tag {
                    id: row.get::<_, String>(0)?.parse().unwrap(),
                    workspace_id: row.get::<_, String>(1)?.parse().unwrap(),
                    name: row.get(2)?,
                    color: row.get(3)?,
                    created_at: row.get::<_, String>(4)?.parse().unwrap(),
                })
            })
            .map_err(Self::map_err)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Self::map_err)
    }

    fn tag_note(&self, note_id: Uuid, tag_id: Uuid) -> Result<(), CoreError> {
        let now = Utc::now();
        self.conn
            .execute(
                "INSERT OR IGNORE INTO note_tags (note_id, tag_id, created_at) VALUES (?1, ?2, ?3)",
                params![note_id.to_string(), tag_id.to_string(), now.to_rfc3339()],
            )
            .map_err(Self::map_err)?;
        self.reindex_fts_by_id(note_id)?;
        Ok(())
    }

    fn untag_note(&self, note_id: Uuid, tag_id: Uuid) -> Result<(), CoreError> {
        self.conn
            .execute(
                "DELETE FROM note_tags WHERE note_id = ?1 AND tag_id = ?2",
                params![note_id.to_string(), tag_id.to_string()],
            )
            .map_err(Self::map_err)?;
        self.reindex_fts_by_id(note_id)?;
        Ok(())
    }

    fn get_note_tags(&self, note_id: Uuid) -> Result<Vec<Tag>, CoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT t.id, t.workspace_id, t.name, t.color, t.created_at FROM tags t JOIN note_tags nt ON t.id = nt.tag_id WHERE nt.note_id = ?1 ORDER BY t.name")
            .map_err(Self::map_err)?;
        let rows = stmt
            .query_map(params![note_id.to_string()], |row| {
                Ok(Tag {
                    id: row.get::<_, String>(0)?.parse().unwrap(),
                    workspace_id: row.get::<_, String>(1)?.parse().unwrap(),
                    name: row.get(2)?,
                    color: row.get(3)?,
                    created_at: row.get::<_, String>(4)?.parse().unwrap(),
                })
            })
            .map_err(Self::map_err)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Self::map_err)
    }

    fn find_tag_by_name(&self, workspace_id: Uuid, name: &str) -> Result<Option<Tag>, CoreError> {
        self.conn
            .query_row(
                "SELECT id, workspace_id, name, color, created_at FROM tags WHERE workspace_id = ?1 AND LOWER(name) = LOWER(?2)",
                params![workspace_id.to_string(), name],
                |row| {
                    Ok(Tag {
                        id: row.get::<_, String>(0)?.parse().unwrap(),
                        workspace_id: row.get::<_, String>(1)?.parse().unwrap(),
                        name: row.get(2)?,
                        color: row.get(3)?,
                        created_at: row.get::<_, String>(4)?.parse().unwrap(),
                    })
                },
            )
            .map(Some)
            .or_else(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                _ => Err(Self::map_err(e)),
            })
    }

    // === Note move ===

    fn move_note(&self, note_id: Uuid, notebook_id: Uuid) -> Result<(), CoreError> {
        let now = Utc::now();
        self.conn
            .execute(
                "UPDATE notes SET notebook_id = ?1, updated_at = ?2 WHERE id = ?3",
                params![notebook_id.to_string(), now.to_rfc3339(), note_id.to_string()],
            )
            .map_err(Self::map_err)?;
        Ok(())
    }

    fn update_fts_for_note(&self, note: &Note) -> Result<(), CoreError> {
        self.update_fts(note)
    }

    fn create_note_atomic(&self, note: &Note) -> Result<(), CoreError> {
        self.conn.execute_batch("BEGIN").map_err(Self::map_err)?;
        let result = (|| -> Result<(), CoreError> {
            NoteStore::create_note(self, note)?;
            NoteStore::save_blocks(self, note.id, &note.blocks)?;
            self.update_fts(note)?;
            Ok(())
        })();
        match result {
            Ok(()) => {
                self.conn.execute_batch("COMMIT").map_err(Self::map_err)?;
                Ok(())
            }
            Err(e) => {
                self.conn.execute_batch("ROLLBACK").ok();
                Err(e)
            }
        }
    }

    fn update_note_atomic(&self, note: &Note) -> Result<(), CoreError> {
        self.conn.execute_batch("BEGIN").map_err(Self::map_err)?;
        let result = (|| -> Result<(), CoreError> {
            NoteStore::update_note(self, note)?;
            NoteStore::save_blocks(self, note.id, &note.blocks)?;
            self.update_fts(note)?;
            Ok(())
        })();
        match result {
            Ok(()) => {
                self.conn.execute_batch("COMMIT").map_err(Self::map_err)?;
                Ok(())
            }
            Err(e) => {
                self.conn.execute_batch("ROLLBACK").ok();
                Err(e)
            }
        }
    }

    // === Attachment ===

    fn create_attachment(&self, att: &Attachment) -> Result<(), CoreError> {
        self.conn
            .execute(
                "INSERT INTO attachments (id, note_id, filename, media_type, storage_path, size_bytes, mime_type, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    att.id.to_string(),
                    att.note_id.to_string(),
                    att.filename,
                    serde_json::to_value(att.media_type).unwrap().as_str().unwrap(),
                    att.storage_path,
                    att.size_bytes as i64,
                    att.mime_type,
                    att.created_at.to_rfc3339(),
                ],
            )
            .map_err(Self::map_err)?;
        Ok(())
    }

    fn list_attachments(&self, note_id: Uuid) -> Result<Vec<Attachment>, CoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, note_id, filename, media_type, storage_path, size_bytes, mime_type, created_at FROM attachments WHERE note_id = ?1 ORDER BY created_at")
            .map_err(Self::map_err)?;
        let rows = stmt
            .query_map(params![note_id.to_string()], |row| {
                let mt_str: String = row.get(3)?;
                Ok(Attachment {
                    id: row.get::<_, String>(0)?.parse().unwrap(),
                    note_id: row.get::<_, String>(1)?.parse().unwrap(),
                    filename: row.get(2)?,
                    media_type: serde_json::from_value(serde_json::Value::String(mt_str))
                        .unwrap_or(MediaType::Other),
                    storage_path: row.get(4)?,
                    size_bytes: row.get::<_, i64>(5)? as u64,
                    mime_type: row.get(6)?,
                    created_at: row.get::<_, String>(7)?.parse().unwrap(),
                })
            })
            .map_err(Self::map_err)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Self::map_err)
    }

    fn get_attachment(&self, id: Uuid) -> Result<Attachment, CoreError> {
        self.conn
            .query_row(
                "SELECT id, note_id, filename, media_type, storage_path, size_bytes, mime_type, created_at FROM attachments WHERE id = ?1",
                params![id.to_string()],
                |row| {
                    let mt_str: String = row.get(3)?;
                    Ok(Attachment {
                        id: row.get::<_, String>(0)?.parse().unwrap(),
                        note_id: row.get::<_, String>(1)?.parse().unwrap(),
                        filename: row.get(2)?,
                        media_type: serde_json::from_value(serde_json::Value::String(mt_str))
                            .unwrap_or(MediaType::Other),
                        storage_path: row.get(4)?,
                        size_bytes: row.get::<_, i64>(5)? as u64,
                        mime_type: row.get(6)?,
                        created_at: row.get::<_, String>(7)?.parse().unwrap(),
                    })
                },
            )
            .map_err(|_| CoreError::Storage(format!("attachment not found: {id}")))
    }

    fn delete_attachment(&self, id: Uuid) -> Result<(), CoreError> {
        self.conn
            .execute("DELETE FROM attachments WHERE id = ?1", params![id.to_string()])
            .map_err(Self::map_err)?;
        Ok(())
    }

    // === Link ===

    fn create_link(&self, link: &Link) -> Result<(), CoreError> {
        self.conn
            .execute(
                "INSERT INTO links (id, source_note_id, target_note_id, source_block_id, target_block_id, link_type, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    link.id.to_string(),
                    link.source_note_id.to_string(),
                    link.target_note_id.to_string(),
                    link.source_block_id.map(|id| id.to_string()),
                    link.target_block_id.map(|id| id.to_string()),
                    serde_json::to_value(link.link_type).unwrap().as_str().unwrap(),
                    link.created_at.to_rfc3339(),
                ],
            )
            .map_err(Self::map_err)?;
        Ok(())
    }

    fn list_links_from(&self, note_id: Uuid) -> Result<Vec<Link>, CoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, source_note_id, target_note_id, source_block_id, target_block_id, link_type, created_at FROM links WHERE source_note_id = ?1")
            .map_err(Self::map_err)?;
        let rows = stmt
            .query_map(params![note_id.to_string()], Self::row_to_link)
            .map_err(Self::map_err)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Self::map_err)
    }

    fn list_backlinks(&self, note_id: Uuid) -> Result<Vec<Link>, CoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, source_note_id, target_note_id, source_block_id, target_block_id, link_type, created_at FROM links WHERE target_note_id = ?1")
            .map_err(Self::map_err)?;
        let rows = stmt
            .query_map(params![note_id.to_string()], Self::row_to_link)
            .map_err(Self::map_err)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Self::map_err)
    }

    fn delete_link(&self, id: Uuid) -> Result<(), CoreError> {
        self.conn
            .execute("DELETE FROM links WHERE id = ?1", params![id.to_string()])
            .map_err(Self::map_err)?;
        Ok(())
    }

    // === Alias ===

    fn create_alias(&self, alias: &Alias) -> Result<(), CoreError> {
        self.conn
            .execute(
                "INSERT INTO aliases (id, note_id, alias_text, created_at) VALUES (?1, ?2, ?3, ?4)",
                params![
                    alias.id.to_string(),
                    alias.note_id.to_string(),
                    alias.alias_text,
                    alias.created_at.to_rfc3339(),
                ],
            )
            .map_err(Self::map_err)?;
        self.reindex_fts_by_id(alias.note_id)?;
        Ok(())
    }

    fn list_aliases(&self, note_id: Uuid) -> Result<Vec<Alias>, CoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, note_id, alias_text, created_at FROM aliases WHERE note_id = ?1 ORDER BY alias_text")
            .map_err(Self::map_err)?;
        let rows = stmt
            .query_map(params![note_id.to_string()], |row| {
                Ok(Alias {
                    id: row.get::<_, String>(0)?.parse().unwrap(),
                    note_id: row.get::<_, String>(1)?.parse().unwrap(),
                    alias_text: row.get(2)?,
                    created_at: row.get::<_, String>(3)?.parse().unwrap(),
                })
            })
            .map_err(Self::map_err)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Self::map_err)
    }

    fn resolve_alias(&self, workspace_id: Uuid, text: &str) -> Result<Option<Uuid>, CoreError> {
        self.conn
            .query_row(
                "SELECT a.note_id FROM aliases a JOIN notes n ON a.note_id = n.id JOIN notebooks nb ON n.notebook_id = nb.id WHERE nb.workspace_id = ?1 AND LOWER(a.alias_text) = LOWER(?2) AND n.lifecycle != 'trashed' LIMIT 1",
                params![workspace_id.to_string(), text],
                |row| {
                    let id_str: String = row.get(0)?;
                    Ok(id_str.parse::<Uuid>().unwrap())
                },
            )
            .map(Some)
            .or_else(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                _ => Err(Self::map_err(e)),
            })
    }

    fn delete_alias(&self, id: Uuid) -> Result<(), CoreError> {
        // Look up note_id before deleting so we can reindex FTS
        let note_id: Option<Uuid> = self.conn
            .query_row(
                "SELECT note_id FROM aliases WHERE id = ?1",
                params![id.to_string()],
                |row| {
                    let s: String = row.get(0)?;
                    Ok(s.parse::<Uuid>().unwrap())
                },
            )
            .ok();
        self.conn
            .execute("DELETE FROM aliases WHERE id = ?1", params![id.to_string()])
            .map_err(Self::map_err)?;
        if let Some(nid) = note_id {
            self.reindex_fts_by_id(nid)?;
        }
        Ok(())
    }

    // === Advanced Search ===

    fn filtered_search(
        &self,
        workspace_id: Uuid,
        filter: &SearchFilter,
    ) -> Result<Vec<SearchResult>, CoreError> {
        // Build dynamic SQL
        let mut conditions = vec![
            "nb.workspace_id = ?1".to_string(),
            "n.lifecycle != 'trashed'".to_string(),
        ];
        let mut param_values: Vec<String> = vec![workspace_id.to_string()];
        let mut idx = 2;

        // FTS query
        if let Some(ref q) = filter.query
            && !q.is_empty()
        {
            conditions.push(format!(
                "n.id IN (SELECT note_id FROM notes_fts WHERE notes_fts MATCH ?{idx})"
            ));
            param_values.push(q.clone());
            idx += 1;
        }

        // Lifecycle filter
        if let Some(ref lc) = filter.lifecycle {
            conditions.push(format!("n.lifecycle = ?{idx}"));
            param_values.push(lc.clone());
            idx += 1;
        }

        // Notebook filter
        if let Some(nb_id) = filter.notebook_id {
            conditions.push(format!("n.notebook_id = ?{idx}"));
            param_values.push(nb_id.to_string());
            idx += 1;
        }

        // Pinned filter
        if let Some(pinned) = filter.pinned {
            conditions.push(format!("n.pinned = ?{idx}"));
            param_values.push(if pinned { "1".into() } else { "0".into() });
            idx += 1;
        }

        // Tag filter — notes must have ALL specified tags
        for tag_name in &filter.tags {
            conditions.push(format!(
                "n.id IN (SELECT nt.note_id FROM note_tags nt JOIN tags t ON nt.tag_id = t.id WHERE LOWER(t.name) = LOWER(?{idx}))"
            ));
            param_values.push(tag_name.clone());
            idx += 1;
        }
        let _ = idx; // suppress unused warning

        let where_clause = conditions.join(" AND ");
        let sql = format!(
            "SELECT n.id, n.title, n.lifecycle, n.notebook_id, n.pinned, n.updated_at, \
             COALESCE((SELECT SUBSTR(nb2.content, 1, 200) FROM note_blocks nb2 WHERE nb2.note_id = n.id ORDER BY nb2.sort_order LIMIT 1), '') as snippet \
             FROM notes n JOIN notebooks nb ON n.notebook_id = nb.id \
             WHERE {where_clause} ORDER BY n.pinned DESC, n.updated_at DESC LIMIT 200"
        );

        let mut stmt = self.conn.prepare(&sql).map_err(Self::map_err)?;
        let params: Vec<&dyn rusqlite::types::ToSql> = param_values
            .iter()
            .map(|s| s as &dyn rusqlite::types::ToSql)
            .collect();
        let rows = stmt
            .query_map(params.as_slice(), |row| {
                Ok(SearchResult {
                    note_id: row.get::<_, String>(0)?.parse().unwrap(),
                    title: row.get(1)?,
                    lifecycle: row.get(2)?,
                    notebook_id: row.get::<_, String>(3)?.parse().unwrap(),
                    pinned: row.get::<_, i32>(4)? != 0,
                    updated_at: row.get(5)?,
                    snippet: row.get(6)?,
                })
            })
            .map_err(Self::map_err)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Self::map_err)
    }

    // === Saved Searches ===

    fn create_saved_search(&self, ss: &SavedSearch) -> Result<(), CoreError> {
        let filter_json =
            serde_json::to_string(&ss.filter).map_err(|e| CoreError::Serialization(e.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO saved_searches (id, workspace_id, name, filter_json, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    ss.id.to_string(),
                    ss.workspace_id.to_string(),
                    ss.name,
                    filter_json,
                    ss.created_at,
                ],
            )
            .map_err(Self::map_err)?;
        Ok(())
    }

    fn list_saved_searches(&self, workspace_id: Uuid) -> Result<Vec<SavedSearch>, CoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, workspace_id, name, filter_json, created_at FROM saved_searches WHERE workspace_id = ?1 ORDER BY name")
            .map_err(Self::map_err)?;
        let rows = stmt
            .query_map(params![workspace_id.to_string()], |row| {
                let filter_str: String = row.get(3)?;
                Ok(SavedSearch {
                    id: row.get::<_, String>(0)?.parse().unwrap(),
                    workspace_id: row.get::<_, String>(1)?.parse().unwrap(),
                    name: row.get(2)?,
                    filter: serde_json::from_str(&filter_str).unwrap_or_default(),
                    created_at: row.get(4)?,
                })
            })
            .map_err(Self::map_err)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Self::map_err)
    }

    fn delete_saved_search(&self, id: Uuid) -> Result<(), CoreError> {
        self.conn
            .execute(
                "DELETE FROM saved_searches WHERE id = ?1",
                params![id.to_string()],
            )
            .map_err(Self::map_err)?;
        Ok(())
    }

    // === Graph Data ===

    fn get_graph_data(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<(Note, Vec<Link>)>, CoreError> {
        // Get all active notes in workspace
        let mut stmt = self
            .conn
            .prepare(
                "SELECT n.id, n.notebook_id, n.title, n.template_id, n.lifecycle, n.visibility, n.ai_policy, n.pinned, n.created_at, n.updated_at \
                 FROM notes n JOIN notebooks nb ON n.notebook_id = nb.id \
                 WHERE nb.workspace_id = ?1 AND n.lifecycle != 'trashed' ORDER BY n.title",
            )
            .map_err(Self::map_err)?;
        let notes: Vec<Note> = stmt
            .query_map(params![workspace_id.to_string()], |row| {
                let lifecycle_str: String = row.get(4)?;
                let visibility_str: String = row.get(5)?;
                let ai_policy_str: String = row.get(6)?;
                Ok(Note {
                    id: row.get::<_, String>(0)?.parse().unwrap(),
                    notebook_id: row.get::<_, String>(1)?.parse().unwrap(),
                    title: row.get(2)?,
                    template_id: row.get::<_, Option<String>>(3)?.map(|s| s.parse().unwrap()),
                    lifecycle: lifecycle_str.parse().unwrap_or_default(),
                    visibility: serde_json::from_value(serde_json::Value::String(visibility_str))
                        .unwrap_or_default(),
                    ai_policy: serde_json::from_value(serde_json::Value::String(ai_policy_str))
                        .unwrap_or_default(),
                    blocks: Vec::new(),
                    pinned: row.get::<_, i32>(7)? != 0,
                    created_at: row.get::<_, String>(8)?.parse().unwrap(),
                    updated_at: row.get::<_, String>(9)?.parse().unwrap(),
                })
            })
            .map_err(Self::map_err)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(Self::map_err)?;

        let mut result = Vec::new();
        for note in notes {
            let links = self.list_links_from(note.id)?;
            result.push((note, links));
        }
        Ok(result)
    }

    // === Related Notes ===

    fn find_related_notes(
        &self,
        note_id: Uuid,
        limit: usize,
    ) -> Result<Vec<Note>, CoreError> {
        // Find notes that share tags or have direct links with the given note
        let sql = format!(
            "SELECT DISTINCT n.id, n.notebook_id, n.title, n.template_id, n.lifecycle, n.visibility, n.ai_policy, n.pinned, n.created_at, n.updated_at \
             FROM notes n \
             WHERE n.id != ?1 AND n.lifecycle != 'trashed' AND (\
               n.id IN (SELECT nt2.note_id FROM note_tags nt1 JOIN note_tags nt2 ON nt1.tag_id = nt2.tag_id WHERE nt1.note_id = ?1 AND nt2.note_id != ?1) \
               OR n.id IN (SELECT target_note_id FROM links WHERE source_note_id = ?1) \
               OR n.id IN (SELECT source_note_id FROM links WHERE target_note_id = ?1) \
             ) ORDER BY n.updated_at DESC LIMIT {limit}"
        );
        let mut stmt = self.conn.prepare(&sql).map_err(Self::map_err)?;
        let rows = stmt
            .query_map(params![note_id.to_string()], |row| {
                let lifecycle_str: String = row.get(4)?;
                let visibility_str: String = row.get(5)?;
                let ai_policy_str: String = row.get(6)?;
                Ok(Note {
                    id: row.get::<_, String>(0)?.parse().unwrap(),
                    notebook_id: row.get::<_, String>(1)?.parse().unwrap(),
                    title: row.get(2)?,
                    template_id: row.get::<_, Option<String>>(3)?.map(|s| s.parse().unwrap()),
                    lifecycle: lifecycle_str.parse().unwrap_or_default(),
                    visibility: serde_json::from_value(serde_json::Value::String(visibility_str))
                        .unwrap_or_default(),
                    ai_policy: serde_json::from_value(serde_json::Value::String(ai_policy_str))
                        .unwrap_or_default(),
                    blocks: Vec::new(),
                    pinned: row.get::<_, i32>(7)? != 0,
                    created_at: row.get::<_, String>(8)?.parse().unwrap(),
                    updated_at: row.get::<_, String>(9)?.parse().unwrap(),
                })
            })
            .map_err(Self::map_err)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Self::map_err)
    }
}
