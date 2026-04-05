# Architecture

## System Overview

```
┌─────────────────────────────────────────────────────────┐
│                    Entry Points                          │
│  ┌──────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │ Rust CLI  │  │ Tauri 2 GUI  │  │ Mobile (Tauri 2) │  │
│  └─────┬────┘  └──────┬───────┘  └────────┬─────────┘  │
│        │               │                   │             │
│        └───────────────┼───────────────────┘             │
│                        │                                 │
│  ┌─────────────────────▼───────────────────────────┐    │
│  │              Core Service Layer                   │    │
│  │  (note-core: domain models, business logic,       │    │
│  │   permission checks, AI policy enforcement)       │    │
│  └──────┬──────────────┬──────────────┬─────────┘    │
│         │              │              │               │
│  ┌──────▼──────┐ ┌─────▼─────┐ ┌─────▼─────────┐   │
│  │  Storage    │ │ AI Gateway│ │ Agent Runtime  │   │
│  │  (SQLite    │ │ (local/   │ │ (orchestration │   │
│  │   + FTS5)   │ │  remote)  │ │  + audit)      │   │
│  └─────────────┘ └───────────┘ └────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

## Crate Map

| Crate             | Role                                          |
| ----------------- | --------------------------------------------- |
| `note-core`       | Domain models, service trait, AI policy        |
| `note-storage`    | SQLite store, FTS5, migrations                 |
| `notes-cli`       | CLI binary (apps/cli)                          |
| `notes-gui`       | Tauri backend (apps/gui/src-tauri)             |
| `ai-gateway`      | Provider abstraction (planned)                 |
| `agent-runtime`   | Multi-agent orchestration (planned)            |
| `search`          | Ranking, semantic retrieval (planned)          |

## Key Design Rules

1. **GUI never touches the database directly.** All operations go through `NoteService`.
2. **CLI never bypasses core logic.** Same service layer as GUI.
3. **AI gateway is isolated.** Frontend cannot call model providers directly.
4. **Permissions enforced in core**, not just at the UI layer.
5. **Multi-agent orchestration runs in core**, not assembled in frontend.

## Data Flow

### Write Path
```
User action → Entry point (CLI/GUI) → NoteService → NoteStore (SQLite)
                                                   → FTS index update
```

### AI Path
```
User triggers AI → NoteService.ai_* → Policy check → AI Gateway
                                    → Scope enforcement
                                    → Result as suggestion (not direct write)
                                    → User approval → Write-back → Audit log
```

### Search Path
```
Query → NoteService.search → FTS5 MATCH → Filter by lifecycle/visibility
                           → (future: semantic re-rank)
                           → Results
```

## Document Model

Notes use a structured block model:

```
Note
├── id, title, lifecycle, visibility, ai_policy
├── Block[] (ordered by sort_order)
│   ├── id, block_type, content, metadata
│   └── Types: text, heading, code, quote, list, image, embed,
│              cornell_cue, cornell_summary, zettel_atom, etc.
├── Tags[]
├── Links[] (wiki-links, block refs)
└── Aliases[]
```

Content is stored as structured JSON blocks in SQLite. Export produces Markdown.

## Storage

- **SQLite** with WAL mode for concurrent reads
- **FTS5** virtual table for full-text search
- All timestamps in RFC 3339 UTC
- UUIDs stored as TEXT
- Soft-delete via lifecycle = "trashed"
