mod commands;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

use note_core::service::NoteService;
use note_storage::SqliteStore;

#[derive(Parser)]
#[command(name = "notes", about = "Local-first AI note-taking CLI", version)]
struct Cli {
    /// Path to the notes database
    #[arg(long, env = "NOTES_DB", default_value = "~/.local/share/notes/notes.db")]
    db: String,

    /// Output format
    #[arg(long, default_value = "table")]
    format: OutputFormat,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Copy, clap::ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new workspace
    Init {
        #[arg(default_value = "Default")]
        name: String,
    },
    /// Create a new note
    New {
        title: String,
        #[arg(short, long)]
        notebook: Option<String>,
        #[arg(short, long)]
        template: Option<String>,
        #[arg(short, long)]
        workspace: Option<String>,
    },
    /// List notes
    List {
        #[arg(short, long)]
        notebook: Option<String>,
        #[arg(short, long)]
        workspace: Option<String>,
    },
    /// Search notes
    Search {
        query: String,
        #[arg(short, long)]
        workspace: Option<String>,
    },
    /// Quick capture to inbox
    Capture {
        content: String,
        #[arg(short, long)]
        workspace: Option<String>,
    },
    /// List inbox items
    Inbox {
        #[command(subcommand)]
        action: Option<InboxAction>,
        #[arg(short, long)]
        workspace: Option<String>,
    },
    /// Manage tags
    Tag {
        #[command(subcommand)]
        action: TagAction,
    },
    /// Show note details
    Show {
        id: String,
    },
    /// Edit a note
    Edit {
        /// Note ID
        id: String,
        /// New title
        #[arg(long)]
        title: Option<String>,
        /// Set lifecycle: inbox, active, archived, trashed
        #[arg(long)]
        lifecycle: Option<String>,
    },
    /// Move a note to a different notebook
    Move {
        /// Note ID
        id: String,
        /// Target notebook name
        notebook: String,
        #[arg(short, long)]
        workspace: Option<String>,
    },
    /// Attach a file to a note
    Attach {
        /// Note ID
        id: String,
        /// File path
        file: String,
    },
    /// Manage wiki-links
    Link {
        #[command(subcommand)]
        action: LinkAction,
    },
    /// Manage aliases
    Alias {
        #[command(subcommand)]
        action: AliasAction,
    },
    /// Create a new notebook
    Notebook {
        #[command(subcommand)]
        action: NotebookAction,
    },
    /// Export a note to stdout
    Export {
        id: String,
    },
    /// AI-assisted operations (suggest tags, summarize, classify, suggest links)
    Ai {
        #[command(subcommand)]
        action: AiAction,
        /// AI mode: local_only or private_api
        #[arg(long, default_value = "local_only")]
        mode: String,
        /// Provider: ollama or openai
        #[arg(long, default_value = "ollama")]
        provider: String,
        /// Model name (e.g. llama3, gpt-4o-mini)
        #[arg(long, default_value = "llama3")]
        model: String,
        /// API key for remote providers
        #[arg(long, env = "OPENAI_API_KEY")]
        api_key: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum InboxAction {
    /// Promote inbox item to active note
    Triage {
        /// Note ID
        id: String,
        /// New title (optional)
        #[arg(long)]
        title: Option<String>,
        /// Target notebook name
        #[arg(short, long)]
        notebook: Option<String>,
        #[arg(short, long)]
        workspace: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum TagAction {
    Create {
        name: String,
        #[arg(short, long)]
        workspace: Option<String>,
    },
    List {
        #[arg(short, long)]
        workspace: Option<String>,
    },
    Add {
        note_id: String,
        tag: String,
    },
    Remove {
        note_id: String,
        tag: String,
    },
}

#[derive(Subcommand)]
pub enum LinkAction {
    /// Create a wiki-link between two notes
    Create {
        /// Source note ID
        from: String,
        /// Target note ID or title
        to: String,
        #[arg(short, long)]
        workspace: Option<String>,
    },
    /// List outgoing links from a note
    From {
        id: String,
    },
    /// List backlinks to a note
    To {
        id: String,
    },
}

#[derive(Subcommand)]
pub enum AliasAction {
    /// Add an alias to a note
    Add {
        id: String,
        alias: String,
    },
    /// List aliases for a note
    List {
        id: String,
    },
}

#[derive(Subcommand)]
pub enum AiAction {
    /// Suggest tags for a note
    SuggestTags {
        /// Note ID
        id: String,
    },
    /// Generate a summary for a note
    Summarize {
        /// Note ID
        id: String,
    },
    /// Classify a note into a notebook
    Classify {
        /// Note ID
        id: String,
        #[arg(short, long)]
        workspace: Option<String>,
    },
    /// Suggest related notes to link
    SuggestLinks {
        /// Note ID
        id: String,
        #[arg(short, long)]
        workspace: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum NotebookAction {
    /// Create a new notebook
    Create {
        name: String,
        #[arg(short, long)]
        workspace: Option<String>,
    },
    /// List all notebooks
    List {
        #[arg(short, long)]
        workspace: Option<String>,
    },
}

fn expand_path(path: &str) -> PathBuf {
    if let Some(stripped) = path.strip_prefix("~/")
        && let Some(home) = dirs_home()
    {
        return home.join(stripped);
    }
    PathBuf::from(path)
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

pub fn data_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".local").join("share").join("notes")
}

fn open_store(db_path: &str) -> Result<NoteService<SqliteStore>> {
    let path = expand_path(db_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let store = SqliteStore::open(&path)?;
    let svc = NoteService::new(store);
    svc.init()?;
    Ok(svc)
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    let svc = open_store(&cli.db)?;

    match cli.command {
        Commands::Init { name } => commands::init(&svc, &name, cli.format),
        Commands::New { title, notebook, template, workspace } => {
            commands::new_note(&svc, &title, notebook.as_deref(), template.as_deref(), workspace.as_deref(), cli.format)
        }
        Commands::List { notebook, workspace } => {
            commands::list_notes(&svc, notebook.as_deref(), workspace.as_deref(), cli.format)
        }
        Commands::Search { query, workspace } => {
            commands::search(&svc, &query, workspace.as_deref(), cli.format)
        }
        Commands::Capture { content, workspace } => {
            commands::capture(&svc, &content, workspace.as_deref(), cli.format)
        }
        Commands::Inbox { action, workspace } => {
            commands::inbox_cmd(&svc, action, workspace.as_deref(), cli.format)
        }
        Commands::Tag { action } => commands::tag(&svc, action, cli.format),
        Commands::Show { id } => commands::show(&svc, &id, cli.format),
        Commands::Edit { id, title, lifecycle } => {
            commands::edit(&svc, &id, title.as_deref(), lifecycle.as_deref(), cli.format)
        }
        Commands::Move { id, notebook, workspace } => {
            commands::move_note(&svc, &id, &notebook, workspace.as_deref(), cli.format)
        }
        Commands::Attach { id, file } => commands::attach(&svc, &id, &file, cli.format),
        Commands::Link { action } => commands::link(&svc, action, cli.format),
        Commands::Alias { action } => commands::alias(&svc, action, cli.format),
        Commands::Notebook { action } => commands::notebook(&svc, action, cli.format),
        Commands::Export { id } => commands::export(&svc, &id),
        Commands::Ai { action, mode, provider, model, api_key } => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(commands::ai_cmd(&svc, action, &mode, &provider, &model, api_key.as_deref(), cli.format))
        }
    }
}
