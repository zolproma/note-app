use std::path::Path;

use chrono::Utc;
use uuid::Uuid;

use crate::error::CoreError;
use crate::model::*;

/// Trait for storage backend - implemented by crates/storage
pub trait NoteStore {
    fn init(&self) -> Result<(), CoreError>;

    // Workspace
    fn create_workspace(&self, ws: &Workspace) -> Result<(), CoreError>;
    fn get_workspace(&self, id: Uuid) -> Result<Workspace, CoreError>;
    fn list_workspaces(&self) -> Result<Vec<Workspace>, CoreError>;

    // Notebook
    fn create_notebook(&self, nb: &Notebook) -> Result<(), CoreError>;
    fn get_notebook(&self, id: Uuid) -> Result<Notebook, CoreError>;
    fn list_notebooks(&self, workspace_id: Uuid) -> Result<Vec<Notebook>, CoreError>;
    fn get_inbox(&self, workspace_id: Uuid) -> Result<Notebook, CoreError>;

    // Note
    fn create_note(&self, note: &Note) -> Result<(), CoreError>;
    fn get_note(&self, id: Uuid) -> Result<Note, CoreError>;
    fn update_note(&self, note: &Note) -> Result<(), CoreError>;
    fn delete_note(&self, id: Uuid) -> Result<(), CoreError>;
    fn move_note(&self, note_id: Uuid, notebook_id: Uuid) -> Result<(), CoreError>;
    fn list_notes(&self, notebook_id: Uuid) -> Result<Vec<Note>, CoreError>;
    fn list_notes_by_lifecycle(
        &self,
        workspace_id: Uuid,
        lifecycle: NoteLifecycle,
    ) -> Result<Vec<Note>, CoreError>;
    fn search_notes(&self, workspace_id: Uuid, query: &str) -> Result<Vec<Note>, CoreError>;
    fn update_fts_for_note(&self, note: &Note) -> Result<(), CoreError>;

    // Block
    fn save_blocks(&self, note_id: Uuid, blocks: &[Block]) -> Result<(), CoreError>;
    fn get_blocks(&self, note_id: Uuid) -> Result<Vec<Block>, CoreError>;

    /// Atomically create a note with its blocks and FTS entry
    fn create_note_atomic(&self, note: &Note) -> Result<(), CoreError>;
    /// Atomically update a note, its blocks, and FTS entry
    fn update_note_atomic(&self, note: &Note) -> Result<(), CoreError>;

    // Tag
    fn create_tag(&self, tag: &Tag) -> Result<(), CoreError>;
    fn list_tags(&self, workspace_id: Uuid) -> Result<Vec<Tag>, CoreError>;
    fn find_tag_by_name(&self, workspace_id: Uuid, name: &str) -> Result<Option<Tag>, CoreError>;
    fn tag_note(&self, note_id: Uuid, tag_id: Uuid) -> Result<(), CoreError>;
    fn untag_note(&self, note_id: Uuid, tag_id: Uuid) -> Result<(), CoreError>;
    fn get_note_tags(&self, note_id: Uuid) -> Result<Vec<Tag>, CoreError>;

    // Attachment
    fn create_attachment(&self, att: &Attachment) -> Result<(), CoreError>;
    fn list_attachments(&self, note_id: Uuid) -> Result<Vec<Attachment>, CoreError>;
    fn get_attachment(&self, id: Uuid) -> Result<Attachment, CoreError>;
    fn delete_attachment(&self, id: Uuid) -> Result<(), CoreError>;

    // Link
    fn create_link(&self, link: &Link) -> Result<(), CoreError>;
    fn list_links_from(&self, note_id: Uuid) -> Result<Vec<Link>, CoreError>;
    fn list_backlinks(&self, note_id: Uuid) -> Result<Vec<Link>, CoreError>;
    fn delete_link(&self, id: Uuid) -> Result<(), CoreError>;

    // Alias
    fn create_alias(&self, alias: &Alias) -> Result<(), CoreError>;
    fn list_aliases(&self, note_id: Uuid) -> Result<Vec<Alias>, CoreError>;
    fn resolve_alias(&self, workspace_id: Uuid, text: &str) -> Result<Option<Uuid>, CoreError>;
    fn delete_alias(&self, id: Uuid) -> Result<(), CoreError>;

    // Advanced search
    fn filtered_search(
        &self,
        workspace_id: Uuid,
        filter: &SearchFilter,
    ) -> Result<Vec<SearchResult>, CoreError>;

    // Saved searches
    fn create_saved_search(&self, ss: &SavedSearch) -> Result<(), CoreError>;
    fn list_saved_searches(&self, workspace_id: Uuid) -> Result<Vec<SavedSearch>, CoreError>;
    fn delete_saved_search(&self, id: Uuid) -> Result<(), CoreError>;

    // Graph data: get all notes with their link counts for graph visualization
    fn get_graph_data(&self, workspace_id: Uuid) -> Result<Vec<(Note, Vec<Link>)>, CoreError>;

    // Related notes: find notes sharing tags or links with the given note
    fn find_related_notes(&self, note_id: Uuid, limit: usize) -> Result<Vec<Note>, CoreError>;
}

/// Core service that all entry points (CLI, GUI) go through
pub struct NoteService<S: NoteStore> {
    pub store: S,
}

impl<S: NoteStore> NoteService<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub fn init(&self) -> Result<(), CoreError> {
        self.store.init()
    }

    // === Workspace ===

    pub fn create_workspace(&self, name: &str) -> Result<Workspace, CoreError> {
        let ws = Workspace::new(name);
        self.store.create_workspace(&ws)?;
        let inbox = Notebook::inbox(ws.id);
        self.store.create_notebook(&inbox)?;
        let default_nb = Notebook::new(ws.id, "Notes");
        self.store.create_notebook(&default_nb)?;
        Ok(ws)
    }

    pub fn get_workspace(&self, id: Uuid) -> Result<Workspace, CoreError> {
        self.store.get_workspace(id)
    }

    pub fn list_workspaces(&self) -> Result<Vec<Workspace>, CoreError> {
        self.store.list_workspaces()
    }

    // === Notebook ===

    pub fn create_notebook(&self, workspace_id: Uuid, name: &str) -> Result<Notebook, CoreError> {
        let nb = Notebook::new(workspace_id, name);
        self.store.create_notebook(&nb)?;
        Ok(nb)
    }

    pub fn list_notebooks(&self, workspace_id: Uuid) -> Result<Vec<Notebook>, CoreError> {
        self.store.list_notebooks(workspace_id)
    }

    pub fn get_notebook(&self, id: Uuid) -> Result<Notebook, CoreError> {
        self.store.get_notebook(id)
    }

    // === Note CRUD ===

    pub fn create_note(
        &self,
        notebook_id: Uuid,
        title: &str,
        template: Option<TemplateKind>,
    ) -> Result<Note, CoreError> {
        let mut note = Note::new(notebook_id, title);
        note.lifecycle = NoteLifecycle::Active;
        if let Some(kind) = template {
            note.blocks = Template::generate_blocks(kind, note.id);
        } else {
            note.blocks.push(Block::text(note.id, ""));
        }
        for (i, block) in note.blocks.iter_mut().enumerate() {
            block.sort_order = i as i32;
        }
        self.store.create_note_atomic(&note)?;
        Ok(note)
    }

    pub fn capture(&self, workspace_id: Uuid, content: &str) -> Result<Note, CoreError> {
        let inbox = self.store.get_inbox(workspace_id)?;
        let note = Note::capture(inbox.id, content);
        self.store.create_note_atomic(&note)?;
        Ok(note)
    }

    pub fn get_note(&self, id: Uuid) -> Result<Note, CoreError> {
        let mut note = self.store.get_note(id)?;
        note.blocks = self.store.get_blocks(id)?;
        Ok(note)
    }

    pub fn update_note_title(&self, id: Uuid, title: &str) -> Result<Note, CoreError> {
        let mut note = self.store.get_note(id)?;
        note.title = title.to_string();
        note.updated_at = Utc::now();
        note.blocks = self.store.get_blocks(id)?;
        self.store.update_note_atomic(&note)?;
        Ok(note)
    }

    pub fn update_note_blocks(&self, id: Uuid, blocks: Vec<Block>) -> Result<Note, CoreError> {
        let mut note = self.store.get_note(id)?;
        note.blocks = blocks;
        for (i, block) in note.blocks.iter_mut().enumerate() {
            block.sort_order = i as i32;
        }
        note.updated_at = Utc::now();
        self.store.update_note_atomic(&note)?;
        Ok(note)
    }

    pub fn set_note_lifecycle(
        &self,
        id: Uuid,
        lifecycle: NoteLifecycle,
    ) -> Result<Note, CoreError> {
        let mut note = self.store.get_note(id)?;
        note.lifecycle = lifecycle;
        note.updated_at = Utc::now();
        self.store.update_note(&note)?;
        Ok(note)
    }

    pub fn list_notes(&self, notebook_id: Uuid) -> Result<Vec<Note>, CoreError> {
        self.store.list_notes(notebook_id)
    }

    pub fn list_inbox(&self, workspace_id: Uuid) -> Result<Vec<Note>, CoreError> {
        self.store
            .list_notes_by_lifecycle(workspace_id, NoteLifecycle::Inbox)
    }

    pub fn search(&self, workspace_id: Uuid, query: &str) -> Result<Vec<Note>, CoreError> {
        self.store.search_notes(workspace_id, query)
    }

    pub fn delete_note(&self, id: Uuid) -> Result<(), CoreError> {
        self.store.delete_note(id)
    }

    // === Inbox Triage ===

    /// Promote a note from Inbox to Active, optionally moving to a different notebook
    pub fn promote_from_inbox(
        &self,
        note_id: Uuid,
        target_notebook_id: Option<Uuid>,
        new_title: Option<&str>,
    ) -> Result<Note, CoreError> {
        let mut note = self.store.get_note(note_id)?;
        if note.lifecycle != NoteLifecycle::Inbox {
            return Err(CoreError::InvalidModel(format!(
                "Note is not in inbox (current: {})",
                note.lifecycle
            )));
        }
        note.lifecycle = NoteLifecycle::Active;
        if let Some(title) = new_title {
            note.title = title.to_string();
        }
        note.updated_at = Utc::now();
        self.store.update_note(&note)?;
        if let Some(nb_id) = target_notebook_id {
            self.store.move_note(note_id, nb_id)?;
            note.notebook_id = nb_id;
        }
        note.blocks = self.store.get_blocks(note_id)?;
        self.store.update_fts_for_note(&note)?;
        Ok(note)
    }

    /// Move a note to a different notebook
    pub fn move_note(&self, note_id: Uuid, notebook_id: Uuid) -> Result<Note, CoreError> {
        self.store.move_note(note_id, notebook_id)?;
        let mut note = self.store.get_note(note_id)?;
        note.blocks = self.store.get_blocks(note_id)?;
        Ok(note)
    }

    /// Archive a note
    pub fn archive_note(&self, id: Uuid) -> Result<Note, CoreError> {
        self.set_note_lifecycle(id, NoteLifecycle::Archived)
    }

    // === Tags ===

    pub fn create_tag(&self, workspace_id: Uuid, name: &str) -> Result<Tag, CoreError> {
        let tag = Tag::new(workspace_id, name);
        self.store.create_tag(&tag)?;
        Ok(tag)
    }

    pub fn tag_note(&self, note_id: Uuid, tag_id: Uuid) -> Result<(), CoreError> {
        self.store.tag_note(note_id, tag_id)
    }

    pub fn untag_note(&self, note_id: Uuid, tag_id: Uuid) -> Result<(), CoreError> {
        self.store.untag_note(note_id, tag_id)
    }

    pub fn list_tags(&self, workspace_id: Uuid) -> Result<Vec<Tag>, CoreError> {
        self.store.list_tags(workspace_id)
    }

    pub fn find_tag_by_name(
        &self,
        workspace_id: Uuid,
        name: &str,
    ) -> Result<Option<Tag>, CoreError> {
        self.store.find_tag_by_name(workspace_id, name)
    }

    pub fn get_note_tags(&self, note_id: Uuid) -> Result<Vec<Tag>, CoreError> {
        self.store.get_note_tags(note_id)
    }

    // === Attachments ===

    pub fn attach_file(
        &self,
        note_id: Uuid,
        src_path: &Path,
        attachments_dir: &Path,
    ) -> Result<Attachment, CoreError> {
        let filename = src_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| CoreError::InvalidModel("invalid filename".into()))?
            .to_string();

        let media_type = MediaType::from_filename(&filename);
        let ext = src_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let storage_name = format!("{}.{}", Uuid::new_v4(), ext);
        let dest = attachments_dir.join(&storage_name);

        std::fs::create_dir_all(attachments_dir)
            .map_err(|e| CoreError::Storage(format!("create attachments dir: {e}")))?;
        std::fs::copy(src_path, &dest)
            .map_err(|e| CoreError::Storage(format!("copy file: {e}")))?;

        let size = std::fs::metadata(&dest).map(|m| m.len()).unwrap_or(0);

        let att = Attachment::new(note_id, filename, media_type, storage_name, size);
        self.store.create_attachment(&att)?;
        Ok(att)
    }

    pub fn list_attachments(&self, note_id: Uuid) -> Result<Vec<Attachment>, CoreError> {
        self.store.list_attachments(note_id)
    }

    pub fn get_attachment(&self, id: Uuid) -> Result<Attachment, CoreError> {
        self.store.get_attachment(id)
    }

    pub fn delete_attachment(&self, id: Uuid, attachments_dir: &Path) -> Result<(), CoreError> {
        let att = self.store.get_attachment(id)?;
        let path = attachments_dir.join(&att.storage_path);
        if path.exists() {
            std::fs::remove_file(&path).ok();
        }
        self.store.delete_attachment(id)
    }

    // === Links ===

    pub fn create_link(
        &self,
        source_id: Uuid,
        target_id: Uuid,
        link_type: LinkType,
    ) -> Result<Link, CoreError> {
        let mut link = Link::wiki_link(source_id, target_id);
        link.link_type = link_type;
        self.store.create_link(&link)?;
        Ok(link)
    }

    pub fn list_links_from(&self, note_id: Uuid) -> Result<Vec<Link>, CoreError> {
        self.store.list_links_from(note_id)
    }

    pub fn list_backlinks(&self, note_id: Uuid) -> Result<Vec<Link>, CoreError> {
        self.store.list_backlinks(note_id)
    }

    pub fn delete_link(&self, id: Uuid) -> Result<(), CoreError> {
        self.store.delete_link(id)
    }

    /// Resolve a [[wiki-link]] text to a note ID by searching titles and aliases
    pub fn resolve_link(&self, workspace_id: Uuid, text: &str) -> Result<Option<Uuid>, CoreError> {
        // First try alias resolution
        if let Some(id) = self.store.resolve_alias(workspace_id, text)? {
            return Ok(Some(id));
        }
        // Then try title match across all notebooks in workspace
        let notebooks = self.store.list_notebooks(workspace_id)?;
        for nb in &notebooks {
            let notes = self.store.list_notes(nb.id)?;
            for note in &notes {
                if note.title.eq_ignore_ascii_case(text) {
                    return Ok(Some(note.id));
                }
            }
        }
        Ok(None)
    }

    // === Aliases ===

    pub fn create_alias(&self, note_id: Uuid, text: &str) -> Result<Alias, CoreError> {
        let alias = Alias::new(note_id, text);
        self.store.create_alias(&alias)?;
        Ok(alias)
    }

    pub fn list_aliases(&self, note_id: Uuid) -> Result<Vec<Alias>, CoreError> {
        self.store.list_aliases(note_id)
    }

    pub fn delete_alias(&self, id: Uuid) -> Result<(), CoreError> {
        self.store.delete_alias(id)
    }

    // === Advanced Search ===

    pub fn filtered_search(
        &self,
        workspace_id: Uuid,
        filter: &SearchFilter,
    ) -> Result<Vec<SearchResult>, CoreError> {
        self.store.filtered_search(workspace_id, filter)
    }

    // === Saved Searches ===

    pub fn create_saved_search(
        &self,
        workspace_id: Uuid,
        name: &str,
        filter: SearchFilter,
    ) -> Result<SavedSearch, CoreError> {
        let ss = SavedSearch::new(workspace_id, name, filter);
        self.store.create_saved_search(&ss)?;
        Ok(ss)
    }

    pub fn list_saved_searches(&self, workspace_id: Uuid) -> Result<Vec<SavedSearch>, CoreError> {
        self.store.list_saved_searches(workspace_id)
    }

    pub fn delete_saved_search(&self, id: Uuid) -> Result<(), CoreError> {
        self.store.delete_saved_search(id)
    }

    // === Graph ===

    pub fn get_graph_data(&self, workspace_id: Uuid) -> Result<Vec<(Note, Vec<Link>)>, CoreError> {
        self.store.get_graph_data(workspace_id)
    }

    // === Related Notes ===

    pub fn find_related_notes(&self, note_id: Uuid, limit: usize) -> Result<Vec<Note>, CoreError> {
        self.store.find_related_notes(note_id, limit)
    }
}
