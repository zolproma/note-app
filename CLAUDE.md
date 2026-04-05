# Notes App — Development Guide

## Project Structure

Monorepo: Cargo workspace (Rust) + pnpm workspace (JS/TS)

- `apps/cli` — Rust CLI binary (`notes`)
- `apps/gui` — Tauri 2 + React frontend
- `apps/gui/src-tauri` — Tauri Rust backend
- `crates/core` — Domain models, service layer, AI policy
- `crates/storage` — SQLite + FTS5 implementation
- `docs/` — PRD, Architecture, AI Policy, Document Model
- `packages/` — Shared frontend packages (design tokens)

## Build Commands

```bash
# Build CLI
cargo build -p notes-cli

# Run CLI
cargo run --bin notes -- --help

# Build frontend
cd apps/gui && pnpm build

# Build Tauri GUI (requires GTK dev libs)
cargo build -p notes-gui

# Run Tauri dev
cd apps/gui && cargo tauri dev
```

## Key Principles

- CLI and GUI share `note-core` service layer
- All data operations go through `NoteService` trait
- SQLite with WAL mode, FTS5 for search
- AI is off by default, scoped access only
- Templates are structural scaffolding, not subsystems
- Block-level document model for all content

## Database

Default location: `~/.local/share/notes/notes.db`
Override with: `--db <path>` or `NOTES_DB` env var
