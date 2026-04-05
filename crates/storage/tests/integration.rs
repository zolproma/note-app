use note_core::model::*;
use note_core::service::NoteService;
use note_storage::SqliteStore;

fn fresh_service() -> NoteService<SqliteStore> {
    let store = SqliteStore::in_memory().expect("open in-memory db");
    let svc = NoteService::new(store);
    svc.init().expect("init");
    svc
}

// ===== Workspace initialization =====

#[test]
fn workspace_creates_inbox_and_notes_notebooks() {
    let svc = fresh_service();
    let ws = svc.create_workspace("Test").unwrap();
    let notebooks = svc.list_notebooks(ws.id).unwrap();

    assert!(notebooks.len() >= 2, "should have at least Inbox + Notes");
    assert!(notebooks.iter().any(|nb| nb.is_inbox), "should have inbox");
    assert!(
        notebooks
            .iter()
            .any(|nb| !nb.is_inbox && nb.name == "Notes"),
        "should have default Notes notebook"
    );
}

// ===== First note creation (no pre-existing notebook) =====

#[test]
fn create_note_in_fresh_workspace() {
    let svc = fresh_service();
    let ws = svc.create_workspace("Fresh").unwrap();
    let notebooks = svc.list_notebooks(ws.id).unwrap();
    let notes_nb = notebooks
        .iter()
        .find(|nb| !nb.is_inbox)
        .expect("default notebook");

    let note = svc.create_note(notes_nb.id, "First Note", None).unwrap();
    assert_eq!(note.title, "First Note");
    assert_eq!(note.lifecycle, NoteLifecycle::Active);
    assert!(!note.blocks.is_empty(), "should have at least one block");
}

// ===== Capture -> Inbox -> Triage flow =====

#[test]
fn capture_inbox_triage_flow() {
    let svc = fresh_service();
    let ws = svc.create_workspace("Flow").unwrap();

    // Capture to inbox
    let captured = svc.capture(ws.id, "Quick thought").unwrap();
    assert_eq!(captured.lifecycle, NoteLifecycle::Inbox);

    // List inbox
    let inbox_items = svc.list_inbox(ws.id).unwrap();
    assert_eq!(inbox_items.len(), 1);
    assert_eq!(inbox_items[0].id, captured.id);

    // Promote from inbox
    let notebooks = svc.list_notebooks(ws.id).unwrap();
    let target_nb = notebooks.iter().find(|nb| !nb.is_inbox).unwrap();
    let promoted = svc
        .promote_from_inbox(captured.id, Some(target_nb.id), Some("Promoted Note"))
        .unwrap();
    assert_eq!(promoted.lifecycle, NoteLifecycle::Active);
    assert_eq!(promoted.title, "Promoted Note");
    assert_eq!(promoted.notebook_id, target_nb.id);

    // Inbox should be empty
    let inbox_after = svc.list_inbox(ws.id).unwrap();
    assert!(inbox_after.is_empty());
}

// ===== Tag affects FTS =====

#[test]
fn tag_affects_fts_search() {
    let svc = fresh_service();
    let ws = svc.create_workspace("FTS").unwrap();
    let notebooks = svc.list_notebooks(ws.id).unwrap();
    let nb = notebooks.iter().find(|nb| !nb.is_inbox).unwrap();

    let note = svc.create_note(nb.id, "Plain Note", None).unwrap();

    // Search for "rustlang" should find nothing
    let before = svc.search(ws.id, "rustlang").unwrap();
    assert!(before.is_empty());

    // Add tag "rustlang"
    let tag = svc.create_tag(ws.id, "rustlang").unwrap();
    svc.tag_note(note.id, tag.id).unwrap();

    // Search for "rustlang" should now find the note
    let after = svc.search(ws.id, "rustlang").unwrap();
    assert_eq!(after.len(), 1);
    assert_eq!(after[0].id, note.id);

    // Remove tag
    svc.untag_note(note.id, tag.id).unwrap();

    // Search should find nothing again
    let removed = svc.search(ws.id, "rustlang").unwrap();
    assert!(removed.is_empty());
}

// ===== Alias affects FTS =====

#[test]
fn alias_affects_fts_search() {
    let svc = fresh_service();
    let ws = svc.create_workspace("Alias").unwrap();
    let notebooks = svc.list_notebooks(ws.id).unwrap();
    let nb = notebooks.iter().find(|nb| !nb.is_inbox).unwrap();

    let note = svc.create_note(nb.id, "My Note", None).unwrap();

    // Search for "zettelkasten" should find nothing
    let before = svc.search(ws.id, "zettelkasten").unwrap();
    assert!(before.is_empty());

    // Add alias "zettelkasten"
    let alias = svc.create_alias(note.id, "zettelkasten").unwrap();

    // Search should now find the note
    let after = svc.search(ws.id, "zettelkasten").unwrap();
    assert_eq!(after.len(), 1);
    assert_eq!(after[0].id, note.id);

    // Delete alias
    svc.delete_alias(alias.id).unwrap();

    // Search should find nothing again
    let removed = svc.search(ws.id, "zettelkasten").unwrap();
    assert!(removed.is_empty());
}

// ===== Block update atomicity =====

#[test]
fn block_update_preserves_content() {
    let svc = fresh_service();
    let ws = svc.create_workspace("Blocks").unwrap();
    let notebooks = svc.list_notebooks(ws.id).unwrap();
    let nb = notebooks.iter().find(|nb| !nb.is_inbox).unwrap();

    let note = svc.create_note(nb.id, "Block Test", None).unwrap();

    // Update blocks
    let new_blocks = vec![
        Block::new(note.id, BlockType::Heading, "Title".to_string()),
        Block::new(note.id, BlockType::Text, "Paragraph one".to_string()),
        Block::new(note.id, BlockType::Code, "fn main() {}".to_string()),
    ];
    let updated = svc.update_note_blocks(note.id, new_blocks).unwrap();
    assert_eq!(updated.blocks.len(), 3);
    assert_eq!(updated.blocks[0].block_type, BlockType::Heading);
    assert_eq!(updated.blocks[1].content, "Paragraph one");
    assert_eq!(updated.blocks[2].block_type, BlockType::Code);

    // Reload and verify persistence
    let reloaded = svc.get_note(note.id).unwrap();
    assert_eq!(reloaded.blocks.len(), 3);
    assert_eq!(reloaded.blocks[0].content, "Title");
    assert_eq!(reloaded.blocks[2].content, "fn main() {}");
}

// ===== Advanced search / saved search =====

#[test]
fn filtered_search_by_lifecycle() {
    let svc = fresh_service();
    let ws = svc.create_workspace("Search").unwrap();
    let notebooks = svc.list_notebooks(ws.id).unwrap();
    let nb = notebooks.iter().find(|nb| !nb.is_inbox).unwrap();

    svc.create_note(nb.id, "Active Note", None).unwrap();
    let archived = svc.create_note(nb.id, "To Archive", None).unwrap();
    svc.archive_note(archived.id).unwrap();

    let filter = SearchFilter {
        query: None,
        tags: vec![],
        notebook_id: None,
        lifecycle: Some("active".into()),
        pinned: None,
    };
    let results = svc.filtered_search(ws.id, &filter).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Active Note");
}

#[test]
fn saved_search_crud() {
    let svc = fresh_service();
    let ws = svc.create_workspace("Saved").unwrap();

    let filter = SearchFilter {
        query: Some("rust".into()),
        tags: vec![],
        notebook_id: None,
        lifecycle: None,
        pinned: None,
    };
    let ss = svc
        .create_saved_search(ws.id, "Rust notes", filter)
        .unwrap();
    assert_eq!(ss.name, "Rust notes");

    let list = svc.list_saved_searches(ws.id).unwrap();
    assert_eq!(list.len(), 1);

    svc.delete_saved_search(ss.id).unwrap();
    let empty = svc.list_saved_searches(ws.id).unwrap();
    assert!(empty.is_empty());
}

// ===== Template note creation =====

#[test]
fn create_note_with_template() {
    let svc = fresh_service();
    let ws = svc.create_workspace("Templates").unwrap();
    let notebooks = svc.list_notebooks(ws.id).unwrap();
    let nb = notebooks.iter().find(|nb| !nb.is_inbox).unwrap();

    let note = svc
        .create_note(nb.id, "Cornell Note", Some(TemplateKind::Cornell))
        .unwrap();
    assert!(
        note.blocks.len() >= 3,
        "Cornell template should have multiple blocks"
    );

    let has_cue = note
        .blocks
        .iter()
        .any(|b| b.block_type == BlockType::CornellCue);
    let has_summary = note
        .blocks
        .iter()
        .any(|b| b.block_type == BlockType::CornellSummary);
    assert!(has_cue, "Cornell should have a cue block");
    assert!(has_summary, "Cornell should have a summary block");
}

// ===== Archive and lifecycle =====

#[test]
fn archive_and_lifecycle_changes() {
    let svc = fresh_service();
    let ws = svc.create_workspace("Lifecycle").unwrap();
    let notebooks = svc.list_notebooks(ws.id).unwrap();
    let nb = notebooks.iter().find(|nb| !nb.is_inbox).unwrap();

    let note = svc.create_note(nb.id, "Test", None).unwrap();
    assert_eq!(note.lifecycle, NoteLifecycle::Active);

    let archived = svc.archive_note(note.id).unwrap();
    assert_eq!(archived.lifecycle, NoteLifecycle::Archived);

    let reactivated = svc
        .set_note_lifecycle(note.id, NoteLifecycle::Active)
        .unwrap();
    assert_eq!(reactivated.lifecycle, NoteLifecycle::Active);
}

// ===== Links and backlinks =====

#[test]
fn links_and_backlinks() {
    let svc = fresh_service();
    let ws = svc.create_workspace("Links").unwrap();
    let notebooks = svc.list_notebooks(ws.id).unwrap();
    let nb = notebooks.iter().find(|nb| !nb.is_inbox).unwrap();

    let a = svc.create_note(nb.id, "Note A", None).unwrap();
    let b = svc.create_note(nb.id, "Note B", None).unwrap();

    let link = svc.create_link(a.id, b.id, LinkType::WikiLink).unwrap();

    let from_a = svc.list_links_from(a.id).unwrap();
    assert_eq!(from_a.len(), 1);
    assert_eq!(from_a[0].target_note_id, b.id);

    let to_b = svc.list_backlinks(b.id).unwrap();
    assert_eq!(to_b.len(), 1);
    assert_eq!(to_b[0].source_note_id, a.id);

    svc.delete_link(link.id).unwrap();
    assert!(svc.list_links_from(a.id).unwrap().is_empty());
}
