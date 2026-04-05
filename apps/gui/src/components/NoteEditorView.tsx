import { useEffect, useState, useCallback, useRef, useMemo } from "react";
import { invoke, type NoteDetail, type NoteItem, type BlockItem, type TagItem, type LinkItem, type NotebookItem } from "../tauri";
import AttachmentPanel from "./AttachmentPanel";
import AiPanel from "./AiPanel";

interface NoteEditorViewProps {
  noteId: string;
  onBack: () => void;
  onOpenNote: (id: string) => void;
}

const BLOCK_TYPES = [
  { value: "text", label: "Text" },
  { value: "heading", label: "Heading" },
  { value: "code", label: "Code" },
  { value: "quote", label: "Quote" },
  { value: "cornell_cue", label: "Cue" },
  { value: "cornell_summary", label: "Summary" },
  { value: "zettel_atom", label: "Atomic Idea" },
  { value: "zettel_source", label: "Source" },
  { value: "feedback_expected", label: "Expected" },
  { value: "feedback_actual", label: "Actual" },
  { value: "feedback_deviation", label: "Deviation" },
  { value: "feedback_cause", label: "Cause" },
  { value: "feedback_action", label: "Action" },
  { value: "divider", label: "Divider" },
];

const blockTypeLabels: Record<string, string> = Object.fromEntries(
  BLOCK_TYPES.map((bt) => [bt.value, bt.label])
);

// Color accents for specialized block types
const blockTypeAccent: Record<string, string> = {
  cornell_cue: "var(--warn)",
  cornell_summary: "var(--green)",
  zettel_atom: "var(--primary)",
  zettel_source: "var(--accent)",
  feedback_expected: "#6b8fad",
  feedback_actual: "#ad6b8f",
  feedback_deviation: "#ad8f6b",
  feedback_cause: "#8f6bad",
  feedback_action: "var(--green)",
  heading: "var(--primary-deep)",
  code: "var(--secondary)",
  quote: "var(--muted)",
};

function NoteEditorView({ noteId, onBack, onOpenNote }: NoteEditorViewProps) {
  const [note, setNote] = useState<NoteDetail | null>(null);
  const [title, setTitle] = useState("");
  const [blocks, setBlocks] = useState<BlockItem[]>([]);
  const [saving, setSaving] = useState(false);
  const [dirty, setDirty] = useState(false);

  // Panels
  const [backlinks, setBacklinks] = useState<LinkItem[]>([]);
  const [forwardLinks, setForwardLinks] = useState<LinkItem[]>([]);
  const [relatedNotes, setRelatedNotes] = useState<NoteItem[]>([]);
  const [showLinkPanel, setShowLinkPanel] = useState(false);
  const [showAiPanel, setShowAiPanel] = useState(false);

  // Tags
  const [tagInput, setTagInput] = useState("");
  const [showTagInput, setShowTagInput] = useState(false);

  // Notebooks (for move dialog)
  const [notebooks, setNotebooks] = useState<NotebookItem[]>([]);
  const [showMoveDialog, setShowMoveDialog] = useState(false);

  // Block focus tracking
  const blockRefs = useRef<(HTMLTextAreaElement | null)[]>([]);

  // Wiki-link detection: extract all [[...]] from blocks
  const detectedWikiLinks = useMemo(() => {
    const links: string[] = [];
    for (const block of blocks) {
      const matches = block.content.matchAll(/\[\[(.+?)\]\]/g);
      for (const m of matches) {
        if (!links.includes(m[1])) links.push(m[1]);
      }
    }
    return links;
  }, [blocks]);

  // Resolved wiki-links: map link text -> note ID
  const [resolvedLinks, setResolvedLinks] = useState<Record<string, string | null>>({});

  // Resolve detected wiki-links
  useEffect(() => {
    let cancelled = false;
    async function resolve() {
      const newResolved: Record<string, string | null> = {};
      for (const text of detectedWikiLinks) {
        if (resolvedLinks[text] !== undefined) {
          newResolved[text] = resolvedLinks[text];
          continue;
        }
        try {
          const id = await invoke<string | null>("resolve_wiki_link", { text });
          if (cancelled) return;
          newResolved[text] = id;
        } catch {
          newResolved[text] = null;
        }
      }
      if (!cancelled) setResolvedLinks(newResolved);
    }
    if (detectedWikiLinks.length > 0) resolve();
    return () => { cancelled = true; };
  }, [detectedWikiLinks]);

  // Load note
  useEffect(() => {
    invoke<NoteDetail>("get_note", { id: noteId }).then((n) => {
      setNote(n);
      setTitle(n.title);
      setBlocks(n.blocks);
    }).catch(console.error);

    // Load links
    invoke<LinkItem[]>("get_backlinks", { id: noteId }).then(setBacklinks).catch(console.error);
    invoke<LinkItem[]>("list_links_from", { id: noteId }).then(setForwardLinks).catch(console.error);
    invoke<NoteItem[]>("get_related_notes", { id: noteId }).then(setRelatedNotes).catch(console.error);
    invoke<NotebookItem[]>("list_notebooks").then(setNotebooks).catch(console.error);
  }, [noteId]);

  // Keyboard shortcut: Ctrl+S to save
  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      if ((e.metaKey || e.ctrlKey) && e.key === "s") {
        e.preventDefault();
        save();
      }
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [dirty, title, blocks, note]);

  const save = useCallback(async () => {
    if (!dirty) return;
    setSaving(true);
    try {
      if (title !== note?.title) {
        await invoke("update_note_title", { id: noteId, title });
      }
      await invoke("update_note_blocks", {
        id: noteId,
        blocks: blocks.map((b) => ({
          id: b.id,
          block_type: b.block_type,
          content: b.content,
        })),
      });
      setDirty(false);
    } catch (e) {
      console.error("Save failed:", e);
    }
    setSaving(false);
  }, [noteId, title, blocks, note, dirty]);

  function updateBlock(index: number, content: string) {
    setBlocks((prev) => prev.map((b, i) => (i === index ? { ...b, content } : b)));
    setDirty(true);
  }

  function changeBlockType(index: number, blockType: string) {
    setBlocks((prev) =>
      prev.map((b, i) => (i === index ? { ...b, block_type: blockType } : b))
    );
    setDirty(true);
  }

  function addBlockAfter(index: number) {
    setBlocks((prev) => {
      const newBlock: BlockItem = {
        id: "",
        block_type: "text",
        content: "",
        sort_order: index + 1,
      };
      const result = [...prev];
      result.splice(index + 1, 0, newBlock);
      return result.map((b, i) => ({ ...b, sort_order: i }));
    });
    setDirty(true);
    // Focus the new block
    setTimeout(() => {
      blockRefs.current[index + 1]?.focus();
    }, 50);
  }

  function deleteBlock(index: number) {
    if (blocks.length <= 1) return; // Keep at least one block
    setBlocks((prev) => prev.filter((_, i) => i !== index).map((b, i) => ({ ...b, sort_order: i })));
    setDirty(true);
  }

  function moveBlock(index: number, direction: "up" | "down") {
    const targetIndex = direction === "up" ? index - 1 : index + 1;
    if (targetIndex < 0 || targetIndex >= blocks.length) return;
    setBlocks((prev) => {
      const result = [...prev];
      [result[index], result[targetIndex]] = [result[targetIndex], result[index]];
      return result.map((b, i) => ({ ...b, sort_order: i }));
    });
    setDirty(true);
  }

  // Block keydown handler
  function handleBlockKeyDown(e: React.KeyboardEvent<HTMLTextAreaElement>, index: number) {
    if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      addBlockAfter(index);
    }
    if (e.key === "Backspace" && blocks[index].content === "" && blocks.length > 1) {
      e.preventDefault();
      deleteBlock(index);
      // Focus previous block
      setTimeout(() => {
        const prev = Math.max(0, index - 1);
        blockRefs.current[prev]?.focus();
      }, 50);
    }
    // Move between blocks with arrow keys at boundaries
    if (e.key === "ArrowUp" && e.altKey) {
      e.preventDefault();
      moveBlock(index, "up");
    }
    if (e.key === "ArrowDown" && e.altKey) {
      e.preventDefault();
      moveBlock(index, "down");
    }
  }

  // Tag management
  async function handleAddTag() {
    const name = tagInput.trim();
    if (!name) return;
    try {
      const tag = await invoke<TagItem>("add_tag", { noteId, tagName: name });
      setNote((prev) => prev ? { ...prev, tags: [...prev.tags, tag] } : prev);
      setTagInput("");
      setShowTagInput(false);
    } catch (e) {
      console.error("Add tag failed:", e);
    }
  }

  async function handleRemoveTag(tagId: string) {
    try {
      await invoke("remove_tag", { noteId, tagId });
      setNote((prev) =>
        prev ? { ...prev, tags: prev.tags.filter((t) => t.id !== tagId) } : prev
      );
    } catch (e) {
      console.error("Remove tag failed:", e);
    }
  }

  // Note actions
  async function handleArchive() {
    try {
      await invoke("archive_note", { id: noteId });
      onBack();
    } catch (e) {
      console.error("Archive failed:", e);
    }
  }

  async function handleMove(notebookId: string) {
    try {
      await invoke("move_note_to_notebook", { id: noteId, notebookId });
      setShowMoveDialog(false);
    } catch (e) {
      console.error("Move failed:", e);
    }
  }

  if (!note) return <div className="empty-state"><div className="empty-state-desc">Loading...</div></div>;

  return (
    <div className="editor-layout">
      <div className="editor-main">
        {/* Title */}
        <input
          type="text"
          value={title}
          onChange={(e) => { setTitle(e.target.value); setDirty(true); }}
          className="editor-title"
          placeholder="Untitled"
        />

        {/* Meta row */}
        <div className="editor-meta">
          <span className={`note-lifecycle ${note.lifecycle}`}>{note.lifecycle}</span>
          {note.tags.map((t) => (
            <span key={t.id} className="tag-badge tag-removable" onClick={() => handleRemoveTag(t.id)}>
              {t.name} <span className="tag-remove">&times;</span>
            </span>
          ))}
          {showTagInput ? (
            <span className="tag-input-inline">
              <input
                type="text"
                value={tagInput}
                onChange={(e) => setTagInput(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") handleAddTag();
                  if (e.key === "Escape") { setShowTagInput(false); setTagInput(""); }
                }}
                placeholder="tag name"
                autoFocus
              />
            </span>
          ) : (
            <button className="btn-tag-add" onClick={() => setShowTagInput(true)}>+ tag</button>
          )}
          <span className="editor-meta-time">{new Date(note.updated_at).toLocaleString("zh-CN")}</span>
        </div>

        {/* Blocks */}
        <div className="editor-blocks">
          {blocks.map((block, i) => {
            const accent = blockTypeAccent[block.block_type];
            return (
              <div
                key={block.id || `new-${i}`}
                className={`editor-block ${block.block_type === "divider" ? "editor-block-divider" : ""}`}
                style={accent ? { borderLeftColor: accent } : undefined}
              >
                <div className="editor-block-toolbar">
                  <select
                    value={block.block_type}
                    onChange={(e) => changeBlockType(i, e.target.value)}
                    className="block-type-select"
                  >
                    {BLOCK_TYPES.map((bt) => (
                      <option key={bt.value} value={bt.value}>{bt.label}</option>
                    ))}
                  </select>
                  <div className="block-actions">
                    <button
                      className="block-action-btn"
                      onClick={() => moveBlock(i, "up")}
                      disabled={i === 0}
                      title="Move up (Alt+Up)"
                    >&#x25B2;</button>
                    <button
                      className="block-action-btn"
                      onClick={() => moveBlock(i, "down")}
                      disabled={i === blocks.length - 1}
                      title="Move down (Alt+Down)"
                    >&#x25BC;</button>
                    <button
                      className="block-action-btn block-action-delete"
                      onClick={() => deleteBlock(i)}
                      disabled={blocks.length <= 1}
                      title="Delete block"
                    >&#x2715;</button>
                  </div>
                </div>
                {block.block_type === "divider" ? (
                  <hr className="block-divider" />
                ) : (
                  <textarea
                    ref={(el) => { blockRefs.current[i] = el; }}
                    value={block.content}
                    onChange={(e) => updateBlock(i, e.target.value)}
                    onKeyDown={(e) => handleBlockKeyDown(e, i)}
                    rows={Math.max(block.block_type === "code" ? 3 : 1, block.content.split("\n").length)}
                    placeholder={`${blockTypeLabels[block.block_type] || block.block_type}...`}
                    className={`editor-textarea ${block.block_type === "code" ? "editor-textarea-code" : ""} ${block.block_type === "heading" ? "editor-textarea-heading" : ""} ${block.block_type === "quote" ? "editor-textarea-quote" : ""}`}
                  />
                )}
              </div>
            );
          })}
        </div>

        {/* Attachments */}
        <AttachmentPanel noteId={noteId} />

        {/* Bottom actions */}
        <div className="editor-bottom-bar">
          <div className="editor-bottom-left">
            <button className="btn btn-ghost" onClick={() => addBlockAfter(blocks.length - 1)}>
              + Add Block
            </button>
            <span className="editor-hint">Ctrl+Enter: new block | Ctrl+S: save | Alt+Arrow: reorder</span>
          </div>
          <div className="editor-bottom-right">
            <button className="btn btn-ghost" onClick={() => { setShowAiPanel(!showAiPanel); if (!showAiPanel) setShowLinkPanel(false); }}>
              AI
            </button>
            <button className="btn btn-ghost" onClick={() => { setShowLinkPanel(!showLinkPanel); if (!showLinkPanel) setShowAiPanel(false); }}>
              Links {backlinks.length + forwardLinks.length > 0 ? `(${backlinks.length + forwardLinks.length})` : ""}
            </button>
            <button className="btn btn-ghost" onClick={() => setShowMoveDialog(true)}>Move</button>
            <button className="btn btn-ghost" style={{ color: "var(--warn)" }} onClick={handleArchive}>Archive</button>
            <button
              className="btn btn-primary"
              onClick={save}
              disabled={!dirty || saving}
            >
              {saving ? "Saving..." : dirty ? "Save" : "Saved"}
            </button>
          </div>
        </div>
      </div>

      {/* Side panel: Links */}
      {showLinkPanel && (
        <div className="editor-side-panel">
          <div className="side-panel-section">
            <div className="side-panel-title">Backlinks ({backlinks.length})</div>
            {backlinks.length === 0 ? (
              <div className="side-panel-empty">No backlinks yet</div>
            ) : (
              backlinks.map((link) => (
                <div
                  key={link.id}
                  className="side-panel-link"
                  onClick={() => onOpenNote(link.source_note_id)}
                >
                  {link.target_title}
                </div>
              ))
            )}
          </div>
          <div className="side-panel-section">
            <div className="side-panel-title">Forward Links ({forwardLinks.length})</div>
            {forwardLinks.length === 0 ? (
              <div className="side-panel-empty">No links yet</div>
            ) : (
              forwardLinks.map((link) => (
                <div
                  key={link.id}
                  className="side-panel-link"
                  onClick={() => onOpenNote(link.target_note_id)}
                >
                  {link.target_title}
                </div>
              ))
            )}
          </div>
          <div className="side-panel-section">
            <div className="side-panel-title">Detected Wiki-links ({detectedWikiLinks.length})</div>
            {detectedWikiLinks.length === 0 ? (
              <div className="side-panel-empty" style={{ fontSize: 11 }}>
                Type <code>[[note title]]</code> in any block to create links
              </div>
            ) : (
              detectedWikiLinks.map((text) => {
                const targetId = resolvedLinks[text];
                return (
                  <div
                    key={text}
                    className={`side-panel-wikilink ${targetId ? "resolved" : "unresolved"}`}
                    onClick={() => targetId && onOpenNote(targetId)}
                  >
                    <span className="wikilink-icon">{targetId ? "~" : "?"}</span>
                    <span>{text}</span>
                  </div>
                );
              })
            )}
          </div>
          {relatedNotes.length > 0 && (
            <div className="side-panel-section">
              <div className="side-panel-title">Related Notes ({relatedNotes.length})</div>
              {relatedNotes.map((n) => (
                <div
                  key={n.id}
                  className="side-panel-link"
                  onClick={() => onOpenNote(n.id)}
                >
                  {n.title}
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* AI panel */}
      {showAiPanel && (
        <div className="editor-side-panel">
          <AiPanel
            noteId={noteId}
            onApplyTags={async (tags) => {
              for (const tag of tags) {
                try {
                  await invoke("add_tag", { noteId, tagName: tag });
                } catch { /* ignore duplicates */ }
              }
              const n = await invoke<import("../tauri").NoteDetail>("get_note", { id: noteId });
              setNote(n);
            }}
            onNavigate={onOpenNote}
          />
        </div>
      )}

      {/* Move dialog */}
      {showMoveDialog && (
        <div className="dialog-overlay" onClick={() => setShowMoveDialog(false)}>
          <div className="dialog-card" onClick={(e) => e.stopPropagation()}>
            <div className="dialog-title">Move to Notebook</div>
            <div className="dialog-body">
              {notebooks.filter((nb) => !nb.is_inbox).map((nb) => (
                <button
                  key={nb.id}
                  className={`dialog-option ${nb.id === note.notebook_id ? "active" : ""}`}
                  onClick={() => handleMove(nb.id)}
                >
                  {nb.name} {nb.id === note.notebook_id ? "(current)" : ""}
                </button>
              ))}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default NoteEditorView;
