use std::path::PathBuf;
use std::sync::Mutex;

use note_ai_gateway::provider::OpenAiCompatProvider;
use note_ai_gateway::service::{AiGateway, AiSuggestion, SuggestionStatus};
use note_core::ai_policy::AiMode;
use note_core::model::{
    Block, BlockType, LinkType, Note, NoteLifecycle, SearchFilter, TemplateKind,
};
use note_core::service::NoteService;
use note_storage::SqliteStore;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

struct AppState {
    svc: Mutex<NoteService<SqliteStore>>,
    workspace_id: Mutex<Option<Uuid>>,
}

#[derive(Serialize)]
struct NoteItem {
    id: String,
    title: String,
    lifecycle: String,
    notebook_id: String,
    pinned: bool,
    created_at: String,
    updated_at: String,
}

impl From<&Note> for NoteItem {
    fn from(n: &Note) -> Self {
        Self {
            id: n.id.to_string(),
            title: n.title.clone(),
            lifecycle: n.lifecycle.to_string(),
            notebook_id: n.notebook_id.to_string(),
            pinned: n.pinned,
            created_at: n.created_at.to_rfc3339(),
            updated_at: n.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Serialize)]
struct NoteDetail {
    id: String,
    title: String,
    lifecycle: String,
    notebook_id: String,
    blocks: Vec<BlockItem>,
    tags: Vec<TagItem>,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize)]
struct BlockItem {
    id: String,
    block_type: String,
    content: String,
    sort_order: i32,
}

#[derive(Serialize)]
struct TagItem {
    id: String,
    name: String,
}

#[derive(Serialize)]
struct NotebookItem {
    id: String,
    name: String,
    is_inbox: bool,
}

#[derive(Serialize)]
struct LinkItem {
    id: String,
    source_note_id: String,
    target_note_id: String,
    target_title: String,
    link_type: String,
}

fn ensure_workspace(state: &AppState) -> Result<Uuid, String> {
    {
        let guard = state.workspace_id.lock().unwrap();
        if let Some(id) = *guard {
            return Ok(id);
        }
    }
    let svc = state.svc.lock().unwrap();
    let workspaces = svc.list_workspaces().map_err(|e| e.to_string())?;
    if let Some(ws) = workspaces.first() {
        let mut guard = state.workspace_id.lock().unwrap();
        *guard = Some(ws.id);
        Ok(ws.id)
    } else {
        let ws = svc.create_workspace("Default").map_err(|e| e.to_string())?;
        let mut guard = state.workspace_id.lock().unwrap();
        *guard = Some(ws.id);
        Ok(ws.id)
    }
}

// === Read commands ===

#[tauri::command]
fn list_inbox(state: State<'_, AppState>) -> Result<Vec<NoteItem>, String> {
    let ws_id = ensure_workspace(&state)?;
    let svc = state.svc.lock().unwrap();
    let notes = svc.list_inbox(ws_id).map_err(|e| e.to_string())?;
    Ok(notes.iter().map(NoteItem::from).collect())
}

#[tauri::command]
fn list_all_notes(state: State<'_, AppState>) -> Result<Vec<NoteItem>, String> {
    let ws_id = ensure_workspace(&state)?;
    let svc = state.svc.lock().unwrap();
    let notebooks = svc.list_notebooks(ws_id).map_err(|e| e.to_string())?;
    let mut all = Vec::new();
    for nb in &notebooks {
        let notes = svc.list_notes(nb.id).map_err(|e| e.to_string())?;
        all.extend(notes);
    }
    all.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(all.iter().map(NoteItem::from).collect())
}

#[tauri::command]
fn list_notebooks(state: State<'_, AppState>) -> Result<Vec<NotebookItem>, String> {
    let ws_id = ensure_workspace(&state)?;
    let svc = state.svc.lock().unwrap();
    let notebooks = svc.list_notebooks(ws_id).map_err(|e| e.to_string())?;
    Ok(notebooks
        .iter()
        .map(|nb| NotebookItem {
            id: nb.id.to_string(),
            name: nb.name.clone(),
            is_inbox: nb.is_inbox,
        })
        .collect())
}

#[tauri::command]
fn search_notes(state: State<'_, AppState>, query: String) -> Result<Vec<NoteItem>, String> {
    let ws_id = ensure_workspace(&state)?;
    let svc = state.svc.lock().unwrap();
    let notes = svc.search(ws_id, &query).map_err(|e| e.to_string())?;
    Ok(notes.iter().map(NoteItem::from).collect())
}

#[tauri::command]
fn get_note(state: State<'_, AppState>, id: String) -> Result<NoteDetail, String> {
    let svc = state.svc.lock().unwrap();
    let uuid: Uuid = id.parse().map_err(|_| "invalid UUID".to_string())?;
    let note = svc.get_note(uuid).map_err(|e| e.to_string())?;
    let tags = svc.get_note_tags(uuid).map_err(|e| e.to_string())?;
    Ok(NoteDetail {
        id: note.id.to_string(),
        title: note.title,
        lifecycle: note.lifecycle.to_string(),
        notebook_id: note.notebook_id.to_string(),
        blocks: note
            .blocks
            .iter()
            .map(|b| BlockItem {
                id: b.id.to_string(),
                block_type: b.block_type.to_string(),
                content: b.content.clone(),
                sort_order: b.sort_order,
            })
            .collect(),
        tags: tags
            .iter()
            .map(|t| TagItem {
                id: t.id.to_string(),
                name: t.name.clone(),
            })
            .collect(),
        created_at: note.created_at.to_rfc3339(),
        updated_at: note.updated_at.to_rfc3339(),
    })
}

#[tauri::command]
fn get_backlinks(state: State<'_, AppState>, id: String) -> Result<Vec<LinkItem>, String> {
    let svc = state.svc.lock().unwrap();
    let uuid: Uuid = id.parse().map_err(|_| "invalid UUID".to_string())?;
    let backlinks = svc.list_backlinks(uuid).map_err(|e| e.to_string())?;
    let mut items = Vec::new();
    for link in &backlinks {
        let title = svc
            .get_note(link.source_note_id)
            .map(|n| n.title)
            .unwrap_or_else(|_| "Unknown".into());
        items.push(LinkItem {
            id: link.id.to_string(),
            source_note_id: link.source_note_id.to_string(),
            target_note_id: link.target_note_id.to_string(),
            target_title: title,
            link_type: format!("{:?}", link.link_type),
        });
    }
    Ok(items)
}

// === Write commands ===

#[tauri::command]
fn create_note(
    state: State<'_, AppState>,
    title: String,
    notebook_id: Option<String>,
    template: Option<String>,
) -> Result<NoteItem, String> {
    let ws_id = ensure_workspace(&state)?;
    let svc = state.svc.lock().unwrap();
    let nb_id = if let Some(id) = notebook_id {
        id.parse().map_err(|_| "invalid notebook ID".to_string())?
    } else {
        let notebooks = svc.list_notebooks(ws_id).map_err(|e| e.to_string())?;
        notebooks
            .iter()
            .find(|nb| !nb.is_inbox)
            .map(|nb| nb.id)
            .ok_or_else(|| "No notebook found".to_string())?
    };
    let tmpl = template
        .as_deref()
        .map(|t| t.parse::<TemplateKind>())
        .transpose()
        .map_err(|e| e.to_string())?;
    let note = svc
        .create_note(nb_id, &title, tmpl)
        .map_err(|e| e.to_string())?;
    Ok(NoteItem::from(&note))
}

#[tauri::command]
fn capture_note(state: State<'_, AppState>, content: String) -> Result<NoteItem, String> {
    let ws_id = ensure_workspace(&state)?;
    let svc = state.svc.lock().unwrap();
    let note = svc.capture(ws_id, &content).map_err(|e| e.to_string())?;
    Ok(NoteItem::from(&note))
}

#[tauri::command]
fn update_note_title(
    state: State<'_, AppState>,
    id: String,
    title: String,
) -> Result<NoteItem, String> {
    let svc = state.svc.lock().unwrap();
    let uuid: Uuid = id.parse().map_err(|_| "invalid UUID".to_string())?;
    let note = svc
        .update_note_title(uuid, &title)
        .map_err(|e| e.to_string())?;
    Ok(NoteItem::from(&note))
}

#[tauri::command]
fn update_note_blocks(
    state: State<'_, AppState>,
    id: String,
    blocks: Vec<BlockInput>,
) -> Result<(), String> {
    let svc = state.svc.lock().unwrap();
    let uuid: Uuid = id.parse().map_err(|_| "invalid UUID".to_string())?;
    let _note = svc.get_note(uuid).map_err(|e| e.to_string())?;
    let new_blocks: Vec<Block> = blocks
        .into_iter()
        .enumerate()
        .map(|(i, b)| {
            let bt: BlockType = serde_json::from_value(serde_json::Value::String(b.block_type))
                .unwrap_or(BlockType::Text);
            let mut block = Block::new(uuid, bt, b.content);
            if let Some(bid) = b.id
                && let Ok(parsed) = bid.parse()
            {
                block.id = parsed;
            }
            block.sort_order = i as i32;
            block
        })
        .collect();
    svc.update_note_blocks(uuid, new_blocks)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Deserialize)]
struct BlockInput {
    id: Option<String>,
    block_type: String,
    content: String,
}

#[tauri::command]
fn promote_inbox(
    state: State<'_, AppState>,
    id: String,
    notebook_id: Option<String>,
    new_title: Option<String>,
) -> Result<NoteItem, String> {
    let svc = state.svc.lock().unwrap();
    let uuid: Uuid = id.parse().map_err(|_| "invalid UUID".to_string())?;
    let nb_id = notebook_id
        .as_deref()
        .map(|s| s.parse::<Uuid>())
        .transpose()
        .map_err(|_| "invalid notebook ID".to_string())?;
    let note = svc
        .promote_from_inbox(uuid, nb_id, new_title.as_deref())
        .map_err(|e| e.to_string())?;
    Ok(NoteItem::from(&note))
}

#[tauri::command]
fn delete_note(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let svc = state.svc.lock().unwrap();
    let uuid: Uuid = id.parse().map_err(|_| "invalid UUID".to_string())?;
    svc.delete_note(uuid).map_err(|e| e.to_string())
}

#[tauri::command]
fn move_note_to_notebook(
    state: State<'_, AppState>,
    id: String,
    notebook_id: String,
) -> Result<NoteItem, String> {
    let svc = state.svc.lock().unwrap();
    let uuid: Uuid = id.parse().map_err(|_| "invalid UUID".to_string())?;
    let nb_id: Uuid = notebook_id
        .parse()
        .map_err(|_| "invalid notebook ID".to_string())?;
    let note = svc.move_note(uuid, nb_id).map_err(|e| e.to_string())?;
    Ok(NoteItem::from(&note))
}

#[tauri::command]
fn archive_note(state: State<'_, AppState>, id: String) -> Result<NoteItem, String> {
    let svc = state.svc.lock().unwrap();
    let uuid: Uuid = id.parse().map_err(|_| "invalid UUID".to_string())?;
    let note = svc.archive_note(uuid).map_err(|e| e.to_string())?;
    Ok(NoteItem::from(&note))
}

#[tauri::command]
fn set_lifecycle(
    state: State<'_, AppState>,
    id: String,
    lifecycle: String,
) -> Result<NoteItem, String> {
    let svc = state.svc.lock().unwrap();
    let uuid: Uuid = id.parse().map_err(|_| "invalid UUID".to_string())?;
    let lc: NoteLifecycle = lifecycle.parse().map_err(|e: String| e)?;
    let note = svc
        .set_note_lifecycle(uuid, lc)
        .map_err(|e| e.to_string())?;
    Ok(NoteItem::from(&note))
}

// === Tag commands ===

#[tauri::command]
fn list_all_tags(state: State<'_, AppState>) -> Result<Vec<TagItem>, String> {
    let ws_id = ensure_workspace(&state)?;
    let svc = state.svc.lock().unwrap();
    let tags = svc.list_tags(ws_id).map_err(|e| e.to_string())?;
    Ok(tags
        .iter()
        .map(|t| TagItem {
            id: t.id.to_string(),
            name: t.name.clone(),
        })
        .collect())
}

#[tauri::command]
fn add_tag(
    state: State<'_, AppState>,
    note_id: String,
    tag_name: String,
) -> Result<TagItem, String> {
    let ws_id = ensure_workspace(&state)?;
    let svc = state.svc.lock().unwrap();
    let nid: Uuid = note_id.parse().map_err(|_| "invalid UUID".to_string())?;

    // Find or create the tag
    let tag = if let Some(existing) = svc
        .find_tag_by_name(ws_id, &tag_name)
        .map_err(|e| e.to_string())?
    {
        existing
    } else {
        svc.create_tag(ws_id, &tag_name)
            .map_err(|e| e.to_string())?
    };
    svc.tag_note(nid, tag.id).map_err(|e| e.to_string())?;
    Ok(TagItem {
        id: tag.id.to_string(),
        name: tag.name,
    })
}

#[tauri::command]
fn remove_tag(state: State<'_, AppState>, note_id: String, tag_id: String) -> Result<(), String> {
    let svc = state.svc.lock().unwrap();
    let nid: Uuid = note_id
        .parse()
        .map_err(|_| "invalid note UUID".to_string())?;
    let tid: Uuid = tag_id.parse().map_err(|_| "invalid tag UUID".to_string())?;
    svc.untag_note(nid, tid).map_err(|e| e.to_string())
}

// === Link commands ===

#[tauri::command]
fn list_links_from(state: State<'_, AppState>, id: String) -> Result<Vec<LinkItem>, String> {
    let svc = state.svc.lock().unwrap();
    let uuid: Uuid = id.parse().map_err(|_| "invalid UUID".to_string())?;
    let links = svc.list_links_from(uuid).map_err(|e| e.to_string())?;
    let mut items = Vec::new();
    for link in &links {
        let title = svc
            .get_note(link.target_note_id)
            .map(|n| n.title)
            .unwrap_or_else(|_| "Unknown".into());
        items.push(LinkItem {
            id: link.id.to_string(),
            source_note_id: link.source_note_id.to_string(),
            target_note_id: link.target_note_id.to_string(),
            target_title: title,
            link_type: format!("{:?}", link.link_type),
        });
    }
    Ok(items)
}

#[tauri::command]
fn resolve_wiki_link(state: State<'_, AppState>, text: String) -> Result<Option<String>, String> {
    let ws_id = ensure_workspace(&state)?;
    let svc = state.svc.lock().unwrap();
    let result = svc.resolve_link(ws_id, &text).map_err(|e| e.to_string())?;
    Ok(result.map(|id| id.to_string()))
}

#[tauri::command]
fn create_wiki_link(
    state: State<'_, AppState>,
    source_id: String,
    target_id: String,
) -> Result<LinkItem, String> {
    let svc = state.svc.lock().unwrap();
    let src: Uuid = source_id
        .parse()
        .map_err(|_| "invalid source UUID".to_string())?;
    let tgt: Uuid = target_id
        .parse()
        .map_err(|_| "invalid target UUID".to_string())?;
    let link = svc
        .create_link(src, tgt, LinkType::WikiLink)
        .map_err(|e| e.to_string())?;
    let title = svc
        .get_note(tgt)
        .map(|n| n.title)
        .unwrap_or_else(|_| "Unknown".into());
    Ok(LinkItem {
        id: link.id.to_string(),
        source_note_id: link.source_note_id.to_string(),
        target_note_id: link.target_note_id.to_string(),
        target_title: title,
        link_type: format!("{:?}", link.link_type),
    })
}

#[tauri::command]
fn delete_link(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let svc = state.svc.lock().unwrap();
    let uuid: Uuid = id.parse().map_err(|_| "invalid UUID".to_string())?;
    svc.delete_link(uuid).map_err(|e| e.to_string())
}

// === Phase 4: Advanced Search, Graph, Related ===

#[derive(Serialize)]
struct SearchResultItem {
    note_id: String,
    title: String,
    lifecycle: String,
    notebook_id: String,
    snippet: String,
    pinned: bool,
    updated_at: String,
}

#[derive(Deserialize)]
struct SearchFilterInput {
    query: Option<String>,
    tags: Option<Vec<String>>,
    notebook_id: Option<String>,
    lifecycle: Option<String>,
    pinned: Option<bool>,
}

#[tauri::command]
fn filtered_search(
    state: State<'_, AppState>,
    filter: SearchFilterInput,
) -> Result<Vec<SearchResultItem>, String> {
    let ws_id = ensure_workspace(&state)?;
    let svc = state.svc.lock().unwrap();
    let f = SearchFilter {
        query: filter.query,
        tags: filter.tags.unwrap_or_default(),
        notebook_id: filter
            .notebook_id
            .map(|s| s.parse::<Uuid>())
            .transpose()
            .map_err(|_| "invalid notebook ID".to_string())?,
        lifecycle: filter.lifecycle,
        pinned: filter.pinned,
    };
    let results = svc.filtered_search(ws_id, &f).map_err(|e| e.to_string())?;
    Ok(results
        .into_iter()
        .map(|r| SearchResultItem {
            note_id: r.note_id.to_string(),
            title: r.title,
            lifecycle: r.lifecycle,
            notebook_id: r.notebook_id.to_string(),
            snippet: r.snippet,
            pinned: r.pinned,
            updated_at: r.updated_at,
        })
        .collect())
}

#[derive(Serialize)]
struct SavedSearchItem {
    id: String,
    name: String,
    filter_json: String,
}

#[tauri::command]
fn list_saved_searches(state: State<'_, AppState>) -> Result<Vec<SavedSearchItem>, String> {
    let ws_id = ensure_workspace(&state)?;
    let svc = state.svc.lock().unwrap();
    let searches = svc.list_saved_searches(ws_id).map_err(|e| e.to_string())?;
    Ok(searches
        .into_iter()
        .map(|s| SavedSearchItem {
            id: s.id.to_string(),
            name: s.name,
            filter_json: serde_json::to_string(&s.filter).unwrap_or_default(),
        })
        .collect())
}

#[tauri::command]
fn save_search(
    state: State<'_, AppState>,
    name: String,
    filter: SearchFilterInput,
) -> Result<SavedSearchItem, String> {
    let ws_id = ensure_workspace(&state)?;
    let svc = state.svc.lock().unwrap();
    let f = SearchFilter {
        query: filter.query,
        tags: filter.tags.unwrap_or_default(),
        notebook_id: filter
            .notebook_id
            .map(|s| s.parse::<Uuid>())
            .transpose()
            .map_err(|_| "invalid notebook ID".to_string())?,
        lifecycle: filter.lifecycle,
        pinned: filter.pinned,
    };
    let ss = svc
        .create_saved_search(ws_id, &name, f)
        .map_err(|e| e.to_string())?;
    Ok(SavedSearchItem {
        id: ss.id.to_string(),
        name: ss.name,
        filter_json: serde_json::to_string(&ss.filter).unwrap_or_default(),
    })
}

#[tauri::command]
fn delete_saved_search(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let svc = state.svc.lock().unwrap();
    let uuid: Uuid = id.parse().map_err(|_| "invalid UUID".to_string())?;
    svc.delete_saved_search(uuid).map_err(|e| e.to_string())
}

#[derive(Serialize)]
struct GraphNode {
    id: String,
    title: String,
    lifecycle: String,
    link_count: usize,
}

#[derive(Serialize)]
struct GraphEdge {
    source: String,
    target: String,
}

#[derive(Serialize)]
struct GraphData {
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
}

#[tauri::command]
fn get_graph_data(state: State<'_, AppState>) -> Result<GraphData, String> {
    let ws_id = ensure_workspace(&state)?;
    let svc = state.svc.lock().unwrap();
    let data = svc.get_graph_data(ws_id).map_err(|e| e.to_string())?;
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    for (note, links) in &data {
        nodes.push(GraphNode {
            id: note.id.to_string(),
            title: note.title.clone(),
            lifecycle: note.lifecycle.to_string(),
            link_count: links.len(),
        });
        for link in links {
            edges.push(GraphEdge {
                source: link.source_note_id.to_string(),
                target: link.target_note_id.to_string(),
            });
        }
    }
    Ok(GraphData { nodes, edges })
}

#[tauri::command]
fn get_related_notes(state: State<'_, AppState>, id: String) -> Result<Vec<NoteItem>, String> {
    let svc = state.svc.lock().unwrap();
    let uuid: Uuid = id.parse().map_err(|_| "invalid UUID".to_string())?;
    let notes = svc
        .find_related_notes(uuid, 10)
        .map_err(|e| e.to_string())?;
    Ok(notes.iter().map(NoteItem::from).collect())
}

// === Phase 5: Attachments ===

#[derive(Serialize)]
struct AttachmentItem {
    id: String,
    note_id: String,
    filename: String,
    media_type: String,
    storage_path: String,
    size_bytes: u64,
    created_at: String,
}

fn attachments_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("notes")
        .join("attachments")
}

#[tauri::command]
fn list_note_attachments(
    state: State<'_, AppState>,
    note_id: String,
) -> Result<Vec<AttachmentItem>, String> {
    let svc = state.svc.lock().unwrap();
    let nid: Uuid = note_id.parse().map_err(|_| "invalid UUID".to_string())?;
    let atts = svc.list_attachments(nid).map_err(|e| e.to_string())?;
    Ok(atts
        .iter()
        .map(|a| AttachmentItem {
            id: a.id.to_string(),
            note_id: a.note_id.to_string(),
            filename: a.filename.clone(),
            media_type: serde_json::to_value(a.media_type)
                .ok()
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_else(|| "other".into()),
            storage_path: a.storage_path.clone(),
            size_bytes: a.size_bytes,
            created_at: a.created_at.to_rfc3339(),
        })
        .collect())
}

#[tauri::command]
fn upload_attachment(
    state: State<'_, AppState>,
    note_id: String,
    file_path: String,
) -> Result<AttachmentItem, String> {
    let svc = state.svc.lock().unwrap();
    let nid: Uuid = note_id.parse().map_err(|_| "invalid UUID".to_string())?;
    let src = PathBuf::from(&file_path);
    let att_dir = attachments_dir();
    let att = svc
        .attach_file(nid, &src, &att_dir)
        .map_err(|e| e.to_string())?;
    Ok(AttachmentItem {
        id: att.id.to_string(),
        note_id: att.note_id.to_string(),
        filename: att.filename,
        media_type: serde_json::to_value(att.media_type)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| "other".into()),
        storage_path: att.storage_path,
        size_bytes: att.size_bytes,
        created_at: att.created_at.to_rfc3339(),
    })
}

#[tauri::command]
fn delete_attachment(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let svc = state.svc.lock().unwrap();
    let uuid: Uuid = id.parse().map_err(|_| "invalid UUID".to_string())?;
    let att_dir = attachments_dir();
    svc.delete_attachment(uuid, &att_dir)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_attachment_path(state: State<'_, AppState>, id: String) -> Result<String, String> {
    let svc = state.svc.lock().unwrap();
    let uuid: Uuid = id.parse().map_err(|_| "invalid UUID".to_string())?;
    let att = svc.get_attachment(uuid).map_err(|e| e.to_string())?;
    let full_path = attachments_dir().join(&att.storage_path);
    Ok(full_path.to_string_lossy().to_string())
}

// === Phase 6: AI Commands ===

#[derive(Serialize)]
struct AiSuggestionItem {
    id: String,
    job_type: String,
    note_id: String,
    content: String,
    status: String,
    model: String,
    created_at: String,
}

impl From<&AiSuggestion> for AiSuggestionItem {
    fn from(s: &AiSuggestion) -> Self {
        Self {
            id: s.id.to_string(),
            job_type: s.job_type.clone(),
            note_id: s.note_id.to_string(),
            content: s.content.clone(),
            status: match s.status {
                SuggestionStatus::Pending => "pending".into(),
                SuggestionStatus::Accepted => "accepted".into(),
                SuggestionStatus::Rejected => "rejected".into(),
            },
            model: s.model.clone(),
            created_at: s.created_at.clone(),
        }
    }
}

#[derive(Deserialize)]
struct AiConfig {
    provider: String,
    model: String,
    api_key: Option<String>,
    mode: String,
}

fn build_ai_gateway(config: &AiConfig) -> Result<AiGateway<OpenAiCompatProvider>, String> {
    let mode = match config.mode.as_str() {
        "local_only" => AiMode::LocalOnly,
        "private_api" => AiMode::PrivateApi,
        _ => AiMode::BlockedRemote,
    };
    let provider = match config.provider.as_str() {
        "openai" => {
            let key = config
                .api_key
                .clone()
                .ok_or("API key required for openai")?;
            OpenAiCompatProvider::openai(key, &config.model)
        }
        _ => OpenAiCompatProvider::ollama(&config.model),
    };
    AiGateway::new(provider, mode).map_err(|e| e.to_string())
}

#[tauri::command]
async fn ai_suggest_tags(
    state: State<'_, AppState>,
    note_id: String,
    config: AiConfig,
) -> Result<AiSuggestionItem, String> {
    let gateway = build_ai_gateway(&config)?;
    let nid: Uuid = note_id.parse().map_err(|_| "invalid UUID".to_string())?;
    let prepared = {
        let svc = state.svc.lock().unwrap();
        gateway
            .prepare_suggest_tags(&*svc, nid)
            .map_err(|e| e.to_string())?
    };
    let suggestion = gateway.execute(prepared).await.map_err(|e| e.to_string())?;
    Ok(AiSuggestionItem::from(&suggestion))
}

#[tauri::command]
async fn ai_summarize(
    state: State<'_, AppState>,
    note_id: String,
    config: AiConfig,
) -> Result<AiSuggestionItem, String> {
    let gateway = build_ai_gateway(&config)?;
    let nid: Uuid = note_id.parse().map_err(|_| "invalid UUID".to_string())?;
    let prepared = {
        let svc = state.svc.lock().unwrap();
        gateway
            .prepare_summarize(&*svc, nid)
            .map_err(|e| e.to_string())?
    };
    let suggestion = gateway.execute(prepared).await.map_err(|e| e.to_string())?;
    Ok(AiSuggestionItem::from(&suggestion))
}

#[tauri::command]
async fn ai_classify(
    state: State<'_, AppState>,
    note_id: String,
    config: AiConfig,
) -> Result<AiSuggestionItem, String> {
    let ws_id = ensure_workspace(&state)?;
    let gateway = build_ai_gateway(&config)?;
    let nid: Uuid = note_id.parse().map_err(|_| "invalid UUID".to_string())?;
    let prepared = {
        let svc = state.svc.lock().unwrap();
        gateway
            .prepare_classify(&*svc, nid, ws_id)
            .map_err(|e| e.to_string())?
    };
    let suggestion = gateway.execute(prepared).await.map_err(|e| e.to_string())?;
    Ok(AiSuggestionItem::from(&suggestion))
}

#[tauri::command]
async fn ai_suggest_links(
    state: State<'_, AppState>,
    note_id: String,
    config: AiConfig,
) -> Result<AiSuggestionItem, String> {
    let ws_id = ensure_workspace(&state)?;
    let gateway = build_ai_gateway(&config)?;
    let nid: Uuid = note_id.parse().map_err(|_| "invalid UUID".to_string())?;
    let prepared = {
        let svc = state.svc.lock().unwrap();
        gateway
            .prepare_suggest_links(&*svc, nid, ws_id)
            .map_err(|e| e.to_string())?
    };
    let suggestion = gateway.execute(prepared).await.map_err(|e| e.to_string())?;
    Ok(AiSuggestionItem::from(&suggestion))
}

fn db_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("notes")
        .join("notes.db")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let path = db_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let store = SqliteStore::open(&path).expect("Failed to open database");
    let svc = NoteService::new(store);
    svc.init().expect("Failed to initialize database");

    let app_state = AppState {
        svc: Mutex::new(svc),
        workspace_id: Mutex::new(None),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            list_inbox,
            list_all_notes,
            list_notebooks,
            search_notes,
            get_note,
            get_backlinks,
            create_note,
            capture_note,
            update_note_title,
            update_note_blocks,
            promote_inbox,
            delete_note,
            move_note_to_notebook,
            archive_note,
            set_lifecycle,
            list_all_tags,
            add_tag,
            remove_tag,
            list_links_from,
            resolve_wiki_link,
            create_wiki_link,
            delete_link,
            filtered_search,
            list_saved_searches,
            save_search,
            delete_saved_search,
            get_graph_data,
            get_related_notes,
            list_note_attachments,
            upload_attachment,
            delete_attachment,
            get_attachment_path,
            ai_suggest_tags,
            ai_summarize,
            ai_classify,
            ai_suggest_links,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
