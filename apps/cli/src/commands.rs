use std::path::{Path, PathBuf};

use anyhow::Result;
use uuid::Uuid;

use note_ai_gateway::provider::OpenAiCompatProvider;
use note_ai_gateway::service::AiGateway;
use note_core::ai_policy::AiMode;
use note_core::model::{LinkType, NoteLifecycle, TemplateKind};
use note_core::service::{NoteService, NoteStore};

use crate::{
    AiAction, AliasAction, InboxAction, LinkAction, NotebookAction, OutputFormat, TagAction,
};

fn get_default_workspace<S: NoteStore>(svc: &NoteService<S>) -> Result<Uuid> {
    let workspaces = svc.list_workspaces()?;
    workspaces
        .first()
        .map(|w| w.id)
        .ok_or_else(|| anyhow::anyhow!("No workspace found. Run `notes init` first."))
}

fn resolve_workspace<S: NoteStore>(svc: &NoteService<S>, name: Option<&str>) -> Result<Uuid> {
    if let Some(name) = name {
        let workspaces = svc.list_workspaces()?;
        workspaces
            .iter()
            .find(|w| w.name.eq_ignore_ascii_case(name))
            .map(|w| w.id)
            .ok_or_else(|| anyhow::anyhow!("Workspace '{}' not found", name))
    } else {
        get_default_workspace(svc)
    }
}

fn find_note_by_prefix<S: NoteStore>(svc: &NoteService<S>, prefix: &str) -> Result<Uuid> {
    if let Ok(id) = prefix.parse::<Uuid>() {
        return Ok(id);
    }
    let workspaces = svc.list_workspaces()?;
    for ws in &workspaces {
        let notebooks = svc.list_notebooks(ws.id)?;
        for nb in &notebooks {
            let notes = svc.list_notes(nb.id)?;
            for note in &notes {
                if note.id.to_string().starts_with(prefix) {
                    return Ok(note.id);
                }
            }
        }
    }
    Err(anyhow::anyhow!(
        "Note with ID prefix '{}' not found",
        prefix
    ))
}

fn resolve_notebook<S: NoteStore>(svc: &NoteService<S>, ws_id: Uuid, name: &str) -> Result<Uuid> {
    let notebooks = svc.list_notebooks(ws_id)?;
    notebooks
        .iter()
        .find(|nb| nb.name.eq_ignore_ascii_case(name))
        .map(|nb| nb.id)
        .ok_or_else(|| anyhow::anyhow!("Notebook '{}' not found", name))
}

fn attachments_dir() -> PathBuf {
    crate::data_dir().join("attachments")
}

// === Init ===

pub fn init<S: NoteStore>(svc: &NoteService<S>, name: &str, fmt: OutputFormat) -> Result<()> {
    let ws = svc.create_workspace(name)?;
    match fmt {
        OutputFormat::Table => {
            println!("Workspace '{}' created.", ws.name);
            println!("ID: {}", ws.id);
            println!("Inbox notebook auto-created.");
        }
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&ws)?),
    }
    Ok(())
}

// === New Note ===

pub fn new_note<S: NoteStore>(
    svc: &NoteService<S>,
    title: &str,
    notebook_name: Option<&str>,
    template: Option<&str>,
    workspace: Option<&str>,
    fmt: OutputFormat,
) -> Result<()> {
    let ws_id = resolve_workspace(svc, workspace)?;
    let notebook_id = if let Some(name) = notebook_name {
        resolve_notebook(svc, ws_id, name)?
    } else {
        let notebooks = svc.list_notebooks(ws_id)?;
        notebooks
            .iter()
            .find(|nb| !nb.is_inbox)
            .map(|nb| nb.id)
            .unwrap_or_else(|| {
                svc.create_notebook(ws_id, "Notes")
                    .expect("failed to create default notebook")
                    .id
            })
    };
    let template_kind = template
        .map(|t| t.parse::<TemplateKind>())
        .transpose()
        .map_err(|e| anyhow::anyhow!(e))?;
    let note = svc.create_note(notebook_id, title, template_kind)?;
    match fmt {
        OutputFormat::Table => {
            println!("Note created: {}", note.title);
            println!("ID: {}", note.id);
            if let Some(kind) = template_kind {
                println!("Template: {kind}");
            }
        }
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&note)?),
    }
    Ok(())
}

// === List ===

pub fn list_notes<S: NoteStore>(
    svc: &NoteService<S>,
    notebook_name: Option<&str>,
    workspace: Option<&str>,
    fmt: OutputFormat,
) -> Result<()> {
    let ws_id = resolve_workspace(svc, workspace)?;
    let notes = if let Some(name) = notebook_name {
        let nb_id = resolve_notebook(svc, ws_id, name)?;
        svc.list_notes(nb_id)?
    } else {
        let notebooks = svc.list_notebooks(ws_id)?;
        let mut all = Vec::new();
        for nb in &notebooks {
            all.extend(svc.list_notes(nb.id)?);
        }
        all.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        all
    };
    match fmt {
        OutputFormat::Table => {
            if notes.is_empty() {
                println!("No notes found.");
                return Ok(());
            }
            println!(
                "{:<8}  {:<30}  {:<10}  {:<20}",
                "ID", "Title", "Status", "Updated"
            );
            println!("{}", "-".repeat(72));
            for note in &notes {
                let short_id = &note.id.to_string()[..8];
                let title: String = note.title.chars().take(30).collect();
                let updated = note.updated_at.format("%Y-%m-%d %H:%M");
                println!(
                    "{:<8}  {:<30}  {:<10}  {:<20}",
                    short_id, title, note.lifecycle, updated
                );
            }
            println!("\n{} note(s)", notes.len());
        }
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&notes)?),
    }
    Ok(())
}

// === Search ===

pub fn search<S: NoteStore>(
    svc: &NoteService<S>,
    query: &str,
    workspace: Option<&str>,
    fmt: OutputFormat,
) -> Result<()> {
    let ws_id = resolve_workspace(svc, workspace)?;
    let notes = svc.search(ws_id, query)?;
    match fmt {
        OutputFormat::Table => {
            if notes.is_empty() {
                println!("No results for '{query}'.");
                return Ok(());
            }
            println!(
                "{:<8}  {:<30}  {:<10}  {:<20}",
                "ID", "Title", "Status", "Updated"
            );
            println!("{}", "-".repeat(72));
            for note in &notes {
                let short_id = &note.id.to_string()[..8];
                let title: String = note.title.chars().take(30).collect();
                let updated = note.updated_at.format("%Y-%m-%d %H:%M");
                println!(
                    "{:<8}  {:<30}  {:<10}  {:<20}",
                    short_id, title, note.lifecycle, updated
                );
            }
            println!("\n{} result(s)", notes.len());
        }
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&notes)?),
    }
    Ok(())
}

// === Capture ===

pub fn capture<S: NoteStore>(
    svc: &NoteService<S>,
    content: &str,
    workspace: Option<&str>,
    fmt: OutputFormat,
) -> Result<()> {
    let ws_id = resolve_workspace(svc, workspace)?;
    let note = svc.capture(ws_id, content)?;
    match fmt {
        OutputFormat::Table => {
            println!("Captured to Inbox.");
            println!("ID: {}", note.id);
            println!("Title: {}", note.title);
        }
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&note)?),
    }
    Ok(())
}

// === Inbox ===

pub fn inbox_cmd<S: NoteStore>(
    svc: &NoteService<S>,
    action: Option<InboxAction>,
    workspace: Option<&str>,
    fmt: OutputFormat,
) -> Result<()> {
    match action {
        None => {
            let ws_id = resolve_workspace(svc, workspace)?;
            let notes = svc.list_inbox(ws_id)?;
            match fmt {
                OutputFormat::Table => {
                    if notes.is_empty() {
                        println!("Inbox is empty.");
                        return Ok(());
                    }
                    println!("{:<8}  {:<40}  {:<20}", "ID", "Title", "Captured");
                    println!("{}", "-".repeat(72));
                    for note in &notes {
                        let short_id = &note.id.to_string()[..8];
                        let title: String = note.title.chars().take(40).collect();
                        let created = note.created_at.format("%Y-%m-%d %H:%M");
                        println!("{:<8}  {:<40}  {:<20}", short_id, title, created);
                    }
                    println!("\n{} item(s) in inbox", notes.len());
                }
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&notes)?),
            }
        }
        Some(InboxAction::Triage {
            id,
            title,
            notebook,
            workspace: ws,
        }) => {
            let ws_id = resolve_workspace(svc, ws.as_deref().or(workspace))?;
            let note_id = find_note_by_prefix(svc, &id)?;
            let nb_id = notebook
                .as_deref()
                .map(|name| resolve_notebook(svc, ws_id, name))
                .transpose()?;
            let note = svc.promote_from_inbox(note_id, nb_id, title.as_deref())?;
            match fmt {
                OutputFormat::Table => {
                    println!("Promoted to active: {}", note.title);
                    println!("ID: {}", note.id);
                    if let Some(nb) = nb_id {
                        let notebook = svc.get_notebook(nb)?;
                        println!("Notebook: {}", notebook.name);
                    }
                }
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&note)?),
            }
        }
    }
    Ok(())
}

// === Edit ===

pub fn edit<S: NoteStore>(
    svc: &NoteService<S>,
    id: &str,
    title: Option<&str>,
    lifecycle: Option<&str>,
    fmt: OutputFormat,
) -> Result<()> {
    let note_id = find_note_by_prefix(svc, id)?;
    if let Some(new_title) = title {
        let note = svc.update_note_title(note_id, new_title)?;
        match fmt {
            OutputFormat::Table => println!("Title updated: {}", note.title),
            OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&note)?),
        }
    }
    if let Some(lc) = lifecycle {
        let lifecycle: NoteLifecycle = lc.parse().map_err(|e: String| anyhow::anyhow!(e))?;
        let note = svc.set_note_lifecycle(note_id, lifecycle)?;
        match fmt {
            OutputFormat::Table => println!("Lifecycle set to: {}", note.lifecycle),
            OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&note)?),
        }
    }
    if title.is_none() && lifecycle.is_none() {
        println!("Nothing to update. Use --title or --lifecycle.");
    }
    Ok(())
}

// === Move ===

pub fn move_note<S: NoteStore>(
    svc: &NoteService<S>,
    id: &str,
    notebook_name: &str,
    workspace: Option<&str>,
    fmt: OutputFormat,
) -> Result<()> {
    let ws_id = resolve_workspace(svc, workspace)?;
    let note_id = find_note_by_prefix(svc, id)?;
    let nb_id = resolve_notebook(svc, ws_id, notebook_name)?;
    let note = svc.move_note(note_id, nb_id)?;
    match fmt {
        OutputFormat::Table => println!("Moved '{}' to '{}'.", note.title, notebook_name),
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&note)?),
    }
    Ok(())
}

// === Attach ===

pub fn attach<S: NoteStore>(
    svc: &NoteService<S>,
    id: &str,
    file: &str,
    fmt: OutputFormat,
) -> Result<()> {
    let note_id = find_note_by_prefix(svc, id)?;
    let src = Path::new(file);
    if !src.exists() {
        return Err(anyhow::anyhow!("File not found: {}", file));
    }
    let att = svc.attach_file(note_id, src, &attachments_dir())?;
    match fmt {
        OutputFormat::Table => {
            println!("Attached: {}", att.filename);
            println!("ID: {}", att.id);
            println!("Type: {:?}", att.media_type);
            println!("Size: {} bytes", att.size_bytes);
        }
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&att)?),
    }
    Ok(())
}

// === Tag ===

pub fn tag<S: NoteStore>(svc: &NoteService<S>, action: TagAction, fmt: OutputFormat) -> Result<()> {
    match action {
        TagAction::Create { name, workspace } => {
            let ws_id = resolve_workspace(svc, workspace.as_deref())?;
            let tag = svc.create_tag(ws_id, &name)?;
            match fmt {
                OutputFormat::Table => println!("Tag '{}' created. ID: {}", tag.name, tag.id),
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&tag)?),
            }
        }
        TagAction::List { workspace } => {
            let ws_id = resolve_workspace(svc, workspace.as_deref())?;
            let tags = svc.list_tags(ws_id)?;
            match fmt {
                OutputFormat::Table => {
                    if tags.is_empty() {
                        println!("No tags.");
                        return Ok(());
                    }
                    for tag in &tags {
                        println!("  {} ({})", tag.name, &tag.id.to_string()[..8]);
                    }
                }
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&tags)?),
            }
        }
        TagAction::Add { note_id, tag } => {
            let note_uuid = find_note_by_prefix(svc, &note_id)?;
            let ws_id = resolve_workspace(svc, None)?;
            let tag_obj = svc
                .find_tag_by_name(ws_id, &tag)?
                .ok_or_else(|| anyhow::anyhow!("Tag '{}' not found. Create it first.", tag))?;
            svc.tag_note(note_uuid, tag_obj.id)?;
            let note = svc.get_note(note_uuid)?;
            println!("Tagged '{}' with '{}'.", note.title, tag_obj.name);
        }
        TagAction::Remove { note_id, tag } => {
            let note_uuid = find_note_by_prefix(svc, &note_id)?;
            let ws_id = resolve_workspace(svc, None)?;
            let tag_obj = svc
                .find_tag_by_name(ws_id, &tag)?
                .ok_or_else(|| anyhow::anyhow!("Tag '{}' not found.", tag))?;
            svc.untag_note(note_uuid, tag_obj.id)?;
            println!("Removed tag '{}'.", tag_obj.name);
        }
    }
    Ok(())
}

// === Show ===

pub fn show<S: NoteStore>(svc: &NoteService<S>, id: &str, fmt: OutputFormat) -> Result<()> {
    let note_id = find_note_by_prefix(svc, id)?;
    let note = svc.get_note(note_id)?;
    let tags = svc.get_note_tags(note_id)?;
    let attachments = svc.list_attachments(note_id)?;
    let links = svc.list_links_from(note_id)?;
    let backlinks = svc.list_backlinks(note_id)?;
    let aliases = svc.list_aliases(note_id)?;

    match fmt {
        OutputFormat::Table => {
            println!("Title:      {}", note.title);
            println!("ID:         {}", note.id);
            println!("Status:     {}", note.lifecycle);
            println!("Visibility: {:?}", note.visibility);
            println!("AI Policy:  {:?}", note.ai_policy);
            println!(
                "Created:    {}",
                note.created_at.format("%Y-%m-%d %H:%M:%S")
            );
            println!(
                "Updated:    {}",
                note.updated_at.format("%Y-%m-%d %H:%M:%S")
            );
            if !tags.is_empty() {
                let tag_names: Vec<_> = tags.iter().map(|t| t.name.as_str()).collect();
                println!("Tags:       {}", tag_names.join(", "));
            }
            if !aliases.is_empty() {
                let alias_names: Vec<_> = aliases.iter().map(|a| a.alias_text.as_str()).collect();
                println!("Aliases:    {}", alias_names.join(", "));
            }
            if !attachments.is_empty() {
                println!("Attachments:");
                for att in &attachments {
                    println!(
                        "  {} ({:?}, {} bytes)",
                        att.filename, att.media_type, att.size_bytes
                    );
                }
            }
            if !links.is_empty() {
                println!("Links out:  {}", links.len());
            }
            if !backlinks.is_empty() {
                println!("Backlinks:  {}", backlinks.len());
            }
            println!();
            for block in &note.blocks {
                if !block.content.is_empty() {
                    println!("[{}] {}", block.block_type, block.content);
                }
            }
        }
        OutputFormat::Json => {
            let mut val = serde_json::to_value(&note)?;
            val["tags"] = serde_json::to_value(&tags)?;
            val["attachments"] = serde_json::to_value(&attachments)?;
            val["aliases"] = serde_json::to_value(&aliases)?;
            val["links_out"] = serde_json::to_value(&links)?;
            val["backlinks"] = serde_json::to_value(&backlinks)?;
            println!("{}", serde_json::to_string_pretty(&val)?);
        }
    }
    Ok(())
}

// === Link ===

pub fn link<S: NoteStore>(
    svc: &NoteService<S>,
    action: LinkAction,
    fmt: OutputFormat,
) -> Result<()> {
    match action {
        LinkAction::Create {
            from,
            to,
            workspace,
        } => {
            let source_id = find_note_by_prefix(svc, &from)?;
            // Try to resolve 'to' as note ID prefix first, then as title/alias
            let target_id = if let Ok(id) = find_note_by_prefix(svc, &to) {
                id
            } else {
                let ws_id = resolve_workspace(svc, workspace.as_deref())?;
                svc.resolve_link(ws_id, &to)?
                    .ok_or_else(|| anyhow::anyhow!("Could not resolve link target: '{}'", to))?
            };
            let link = svc.create_link(source_id, target_id, LinkType::WikiLink)?;
            let source = svc.get_note(source_id)?;
            let target = svc.get_note(target_id)?;
            match fmt {
                OutputFormat::Table => {
                    println!("Linked: '{}' -> '{}'", source.title, target.title);
                }
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&link)?),
            }
        }
        LinkAction::From { id } => {
            let note_id = find_note_by_prefix(svc, &id)?;
            let links = svc.list_links_from(note_id)?;
            if links.is_empty() {
                println!("No outgoing links.");
                return Ok(());
            }
            for link in &links {
                if let Ok(target) = svc.get_note(link.target_note_id) {
                    println!(
                        "  -> {} ({})",
                        target.title,
                        &link.target_note_id.to_string()[..8]
                    );
                }
            }
        }
        LinkAction::To { id } => {
            let note_id = find_note_by_prefix(svc, &id)?;
            let backlinks = svc.list_backlinks(note_id)?;
            if backlinks.is_empty() {
                println!("No backlinks.");
                return Ok(());
            }
            for link in &backlinks {
                if let Ok(source) = svc.get_note(link.source_note_id) {
                    println!(
                        "  <- {} ({})",
                        source.title,
                        &link.source_note_id.to_string()[..8]
                    );
                }
            }
        }
    }
    Ok(())
}

// === Alias ===

pub fn alias<S: NoteStore>(
    svc: &NoteService<S>,
    action: AliasAction,
    fmt: OutputFormat,
) -> Result<()> {
    match action {
        AliasAction::Add { id, alias } => {
            let note_id = find_note_by_prefix(svc, &id)?;
            let a = svc.create_alias(note_id, &alias)?;
            match fmt {
                OutputFormat::Table => println!("Alias '{}' added.", a.alias_text),
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&a)?),
            }
        }
        AliasAction::List { id } => {
            let note_id = find_note_by_prefix(svc, &id)?;
            let aliases = svc.list_aliases(note_id)?;
            if aliases.is_empty() {
                println!("No aliases.");
                return Ok(());
            }
            for a in &aliases {
                println!("  {} ({})", a.alias_text, &a.id.to_string()[..8]);
            }
        }
    }
    Ok(())
}

// === Notebook ===

pub fn notebook<S: NoteStore>(
    svc: &NoteService<S>,
    action: NotebookAction,
    fmt: OutputFormat,
) -> Result<()> {
    match action {
        NotebookAction::Create { name, workspace } => {
            let ws_id = resolve_workspace(svc, workspace.as_deref())?;
            let nb = svc.create_notebook(ws_id, &name)?;
            match fmt {
                OutputFormat::Table => println!("Notebook '{}' created. ID: {}", nb.name, nb.id),
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&nb)?),
            }
        }
        NotebookAction::List { workspace } => {
            let ws_id = resolve_workspace(svc, workspace.as_deref())?;
            let notebooks = svc.list_notebooks(ws_id)?;
            match fmt {
                OutputFormat::Table => {
                    for nb in &notebooks {
                        let inbox_mark = if nb.is_inbox { " [inbox]" } else { "" };
                        println!("  {}{} ({})", nb.name, inbox_mark, &nb.id.to_string()[..8]);
                    }
                }
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&notebooks)?),
            }
        }
    }
    Ok(())
}

// === AI ===

fn build_provider(
    provider: &str,
    model: &str,
    api_key: Option<&str>,
) -> Result<OpenAiCompatProvider> {
    match provider {
        "ollama" => Ok(OpenAiCompatProvider::ollama(model)),
        "openai" => {
            let key = api_key.ok_or_else(|| {
                anyhow::anyhow!("--api-key or OPENAI_API_KEY required for openai provider")
            })?;
            Ok(OpenAiCompatProvider::openai(key, model))
        }
        other => Err(anyhow::anyhow!(
            "Unknown provider '{}'. Use 'ollama' or 'openai'.",
            other
        )),
    }
}

fn parse_ai_mode(mode: &str) -> Result<AiMode> {
    match mode {
        "local_only" => Ok(AiMode::LocalOnly),
        "private_api" => Ok(AiMode::PrivateApi),
        "blocked" => Ok(AiMode::BlockedRemote),
        other => Err(anyhow::anyhow!(
            "Unknown AI mode '{}'. Use 'local_only', 'private_api', or 'blocked'.",
            other
        )),
    }
}

pub async fn ai_cmd<S: NoteStore>(
    svc: &NoteService<S>,
    action: AiAction,
    mode: &str,
    provider: &str,
    model: &str,
    api_key: Option<&str>,
    fmt: OutputFormat,
) -> Result<()> {
    let ai_mode = parse_ai_mode(mode)?;
    let prov = build_provider(provider, model, api_key)?;
    let gateway = AiGateway::new(prov, ai_mode).map_err(|e| anyhow::anyhow!("{e}"))?;

    match action {
        AiAction::SuggestTags { id } => {
            let note_id = find_note_by_prefix(svc, &id)?;
            let suggestion = gateway
                .suggest_tags(svc, note_id)
                .await
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            match fmt {
                OutputFormat::Table => {
                    println!("Suggested tags for note {}:", &note_id.to_string()[..8]);
                    println!("{}", suggestion.content);
                    println!(
                        "\n(model: {}, status: pending — use 'notes tag add' to apply)",
                        suggestion.model
                    );
                }
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&suggestion)?),
            }
        }
        AiAction::Summarize { id } => {
            let note_id = find_note_by_prefix(svc, &id)?;
            let suggestion = gateway
                .summarize(svc, note_id)
                .await
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            match fmt {
                OutputFormat::Table => {
                    println!("Summary for note {}:", &note_id.to_string()[..8]);
                    println!("{}", suggestion.content);
                    println!("\n(model: {})", suggestion.model);
                }
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&suggestion)?),
            }
        }
        AiAction::Classify { id, workspace } => {
            let ws_id = resolve_workspace(svc, workspace.as_deref())?;
            let note_id = find_note_by_prefix(svc, &id)?;
            let suggestion = gateway
                .classify(svc, note_id, ws_id)
                .await
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            match fmt {
                OutputFormat::Table => {
                    println!("Classification for note {}:", &note_id.to_string()[..8]);
                    println!("{}", suggestion.content);
                    println!(
                        "\n(model: {}, status: pending — use 'notes move' to apply)",
                        suggestion.model
                    );
                }
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&suggestion)?),
            }
        }
        AiAction::SuggestLinks { id, workspace } => {
            let ws_id = resolve_workspace(svc, workspace.as_deref())?;
            let note_id = find_note_by_prefix(svc, &id)?;
            let suggestion = gateway
                .suggest_links(svc, note_id, ws_id)
                .await
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            match fmt {
                OutputFormat::Table => {
                    println!("Suggested links for note {}:", &note_id.to_string()[..8]);
                    println!("{}", suggestion.content);
                    println!(
                        "\n(model: {}, status: pending — use 'notes link create' to apply)",
                        suggestion.model
                    );
                }
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&suggestion)?),
            }
        }
    }
    Ok(())
}

// === Export ===

pub fn export<S: NoteStore>(svc: &NoteService<S>, id: &str) -> Result<()> {
    let note_id = find_note_by_prefix(svc, id)?;
    let note = svc.get_note(note_id)?;
    let tags = svc.get_note_tags(note_id)?;

    // Export as Markdown
    println!("# {}", note.title);
    println!();
    if !tags.is_empty() {
        let tag_str: Vec<_> = tags.iter().map(|t| format!("#{}", t.name)).collect();
        println!("{}", tag_str.join(" "));
        println!();
    }
    for block in &note.blocks {
        match block.block_type {
            note_core::model::BlockType::Heading => println!("## {}", block.content),
            note_core::model::BlockType::Code => {
                println!("```");
                println!("{}", block.content);
                println!("```");
            }
            note_core::model::BlockType::Quote => println!("> {}", block.content),
            note_core::model::BlockType::CornellCue => println!("**Cue:** {}", block.content),
            note_core::model::BlockType::CornellSummary => {
                println!("**Summary:** {}", block.content)
            }
            note_core::model::BlockType::ZettelAtom => println!("**Idea:** {}", block.content),
            note_core::model::BlockType::ZettelSource => println!("**Source:** {}", block.content),
            note_core::model::BlockType::FeedbackExpected => {
                println!("**Expected:** {}", block.content)
            }
            note_core::model::BlockType::FeedbackActual => {
                println!("**Actual:** {}", block.content)
            }
            note_core::model::BlockType::FeedbackDeviation => {
                println!("**Deviation:** {}", block.content)
            }
            note_core::model::BlockType::FeedbackCause => println!("**Cause:** {}", block.content),
            note_core::model::BlockType::FeedbackAction => {
                println!("**Action:** {}", block.content)
            }
            note_core::model::BlockType::Divider => println!("---"),
            _ => {
                if !block.content.is_empty() {
                    println!("{}", block.content);
                }
            }
        }
        println!();
    }
    Ok(())
}
