# Code Review For Claude

日期: 2026-04-04 (已修复)
仓库: `/home/lsq/Documents/note/note-app`
目的: 记录代码审查发现的问题及修复状态

## 1. 当前结论

所有审查问题已修复。仓库处于可继续开发状态。

当前状态:

- `cargo build` 通过 (0 warnings)
- `cargo test` 通过 (11 tests)
- `pnpm --dir apps/gui build` 通过
- 36 个 Tauri IPC 命令已注册
- Phase 1-6 功能已实现 (含 AI Gateway)

## 2. 已修复问题清单

### [已修复] P0: Rust 工作区编译失败

- 问题: `NoteStore` trait 新增方法未在 `SqliteStore` 实现
- 修复: 全部 6 个方法已实现 (`filtered_search`, `create_saved_search`, `list_saved_searches`, `delete_saved_search`, `get_graph_data`, `find_related_notes`)
- 文件: `crates/storage/src/sqlite.rs`

### [已修复] P0: 新工作区首次创建普通笔记失败

- 问题: `create_workspace` 只建 Inbox，无普通 notebook
- 修复: `create_workspace` 现在同时创建 `Inbox` 和 `Notes` notebook
- ��件: `crates/core/src/service.rs` — `create_workspace` 方法
- 测试: `workspace_creates_inbox_and_notes_notebooks`, `create_note_in_fresh_workspace`

### [已修复] P1: FTS 索引与标签/别名不一致

- 问题: `update_fts` 写空字符串给 tags/aliases; tag/alias 操作不触发 reindex
- 修复:
  - `update_fts` 现在查询并聚合真实的 tags 和 aliases 写入 FTS
  - 新增 `reindex_fts_by_id` 内部方法
  - `tag_note`, `untag_note`, `create_alias`, `delete_alias` ��自动触发 FTS 重建
- 文件: `crates/storage/src/sqlite.rs`
- 测试: `tag_affects_fts_search`, `alias_affects_fts_search`

### [已修复] P1: 块写入是破坏式且非事��性的

- 问题: create_note + save_blocks, update_note + save_blocks + update_fts 无事务包裹
- 修复:
  - 新增 `NoteStore` trait 方法: `create_note_atomic`, `update_note_atomic`
  - `SqliteStore` 实现用 `BEGIN/COMMIT/ROLLBACK` 包裹整个操作
  - 服务层 `create_note`, `capture`, `update_note_title`, `update_note_blocks` 均改用原子方法
- 文件: `crates/core/src/service.rs`, `crates/storage/src/sqlite.rs`
- 测试: `block_update_preserves_content`

### [已修复] 测试缺口

- 问题: 0 tests
- 修复: 新增 11 个集成测试
- 文件: `crates/storage/tests/integration.rs`
- 覆盖:
  - workspace 初始化 (含默认 notebook)
  - 首次创建普通 note
  - capture -> inbox -> triage 流程
  - tag 影响 FTS
  - alias 影响 FTS
  - block 更新持久化
  - filtered search (lifecycle 过滤)
  - saved search CRUD
  - 模板 note 创建
  - archive / lifecycle 变更
  - links / backlinks

## 3. 后续可增强项

- 事务覆盖可扩展到更多复合操作 (如 delete_note 级联清理)
- FTS 可增加权重 (title 优先于 content)
- 测试可扩展: AI gateway mock 测试、前端 e2e 测试
- `save_blocks` 可优化为 diff-based 而非 delete-all + insert-all
