<p align="center">
  <img src="assets/logo.png" alt="ono" width="128" height="128">
</p>

<h1 align="center">ono</h1>

<p align="center">
  Local-first, cross-platform note-taking app built with Tauri 2 + React + Rust.
</p>

<p align="center">
  <a href="https://github.com/zolproma/note-app/actions"><img src="https://github.com/zolproma/note-app/actions/workflows/build.yml/badge.svg" alt="Build"></a>
  <img src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux%20%7C%20Android%20%7C%20iOS-blue" alt="Platforms">
  <img src="https://img.shields.io/badge/license-MIT-green" alt="License">
</p>

---

## Features

- **Local-first** — All data stored locally in SQLite with WAL mode
- **Block-based editor** — Headings, text, code blocks, Cornell notes
- **Full-text search** — Powered by SQLite FTS5
- **Knowledge graph** — Visualize note connections via wiki-links
- **Quick capture** — `Ctrl+Shift+N` to capture thoughts instantly
- **Inbox workflow** — Capture → Inbox → Triage → Organize
- **Notebooks & tags** — Organize notes your way
- **AI-ready** — Optional AI features with strict privacy controls (off by default)
- **Cross-platform** — Windows, macOS, Linux, Android, iOS

## Screenshots

> Coming soon

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) 22+
- [pnpm](https://pnpm.io/) 10+
- Platform-specific dependencies (see below)

### Linux dependencies

```bash
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev \
  patchelf libgtk-3-dev libsoup-3.0-dev libjavascriptcoregtk-4.1-dev
```

### Build & Run

```bash
# Install frontend dependencies
pnpm install

# Run in development mode
cd apps/gui && cargo tauri dev

# Build for production
cd apps/gui && cargo tauri build
```

### CLI

```bash
cargo run --bin notes -- --help
```

## Project Structure

```
ono/
├── apps/
│   ├── cli/              # Rust CLI binary
│   └── gui/              # Tauri 2 + React frontend
│       ├── src/           # React components
│       └── src-tauri/     # Tauri Rust backend
├── crates/
│   ├── core/             # Domain models, service layer
│   ├── storage/          # SQLite + FTS5 implementation
│   └── ai-gateway/       # AI provider abstraction
├── packages/
│   └── design-tokens/    # Shared design tokens
└── docs/                 # Architecture, PRD, AI policy
```

## Architecture

- **Monorepo**: Cargo workspace (Rust) + pnpm workspace (TypeScript)
- **Shared core**: CLI and GUI share the `note-core` service layer
- **Storage**: SQLite with bundled build, WAL mode, FTS5 for search
- **AI policy**: Off by default, scoped access, no-remote option per note

## License

MIT
