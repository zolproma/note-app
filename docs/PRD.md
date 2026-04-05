# Notes — Product Requirements Document

Version: 0.1.0
Date: 2026-04-04

## Product Definition

Notes is a local-first, cross-platform note-taking application for long-term knowledge accumulation and large-scale retrieval. It is NOT a learning app — no FSRS, no spaced repetition, no flashcard training.

## Target Users

- Long-term writers and knowledge workers
- Project-based professionals
- Researchers and material organizers
- CLI-first power users who value local-first and privacy

## Core Workflow

```
Capture → Inbox → Structure → Connect → Retrieve → Archive
```

| Stage     | Description                                                   |
| --------- | ------------------------------------------------------------- |
| Capture   | Quick-record ideas, snippets, images, audio, web clips        |
| Inbox     | All unprocessed content lands here                            |
| Structure | Organize with templates, tags, notebooks, structured fields   |
| Connect   | Build wiki-links, block refs, aliases, topic relationships    |
| Retrieve  | Full-text, tag, semantic, graph-based, AI-assisted retrieval  |
| Archive   | Settled notes go to archive, still searchable and linkable    |

## Platform Matrix

| Capability            | Desktop (macOS/Linux/Win) | Mobile (iOS/Android) | CLI |
| --------------------- | :-: | :-: | :-: |
| Full editing          | ✓ | Lite | — |
| FTS search            | ✓ | ✓ | ✓ |
| Quick capture         | ✓ | ✓ | ✓ |
| Inbox triage          | ✓ | ✓ | ✓ |
| Template creation     | ✓ | — | ✓ |
| Wiki-links & graph    | ✓ | View only | — |
| Attachments           | Drag & drop | Camera/mic | Path |
| AI classify/tag       | ✓ | ✓ | ✓ |
| AI audit log          | ✓ | ✓ | ✓ |
| Batch operations      | ✓ | — | ✓ |

## MVP Scope

### Included

- Local workspace, notebooks, tags, inbox
- Markdown + structured block editing
- Images, attachments, audio memos
- Full-text search (FTS5) and filtering
- Wiki-links (bidirectional)
- Aliases and redirects
- Cornell, Zettelkasten, Feedback Analysis, Daily, Capture templates
- AI classify, tag suggestions, summaries, link suggestions
- Approval-based AI write-back
- CLI: init, new, list, search, capture, inbox, tag, show, export

### Excluded from MVP

- Spaced repetition / FSRS / flashcards
- Multi-agent automatic cooperation
- Complex whiteboard/canvas
- Full mind-map editor
- Real-time team collaboration
- Cloud sync
- Complex recommendation algorithms

## Design Principles

- Local-first: all data on-device by default
- Single codebase: five platforms, one Rust core
- CLI first-class: all core data ops available via CLI
- AI under control: off by default, read-only by default, scoped access only
- Retrieval-first: every feature serves "find it fast at scale"
- Templates, not subsystems: methodologies expressed as templates
- Offline-capable: edit, search, browse without network
- Portable: stable export formats

## Technology Stack

| Layer         | Technology                      |
| ------------- | ------------------------------- |
| GUI framework | Tauri 2                         |
| Frontend      | React + TypeScript              |
| Core logic    | Rust                            |
| CLI           | Rust (shared core)              |
| Database      | SQLite + FTS5                   |
| AI gateway    | Rust provider gateway           |
| Package mgmt  | Cargo workspace + pnpm          |
