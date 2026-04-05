import { useState, useEffect } from "react";
import { invoke, type NotebookItem } from "../tauri";

interface CreateNoteDialogProps {
  onClose: () => void;
  onCreated: () => void;
}

interface TemplateInfo {
  value: string;
  label: string;
  description: string;
  blocks: string[];
}

const TEMPLATES: TemplateInfo[] = [
  {
    value: "",
    label: "Blank",
    description: "A simple note with a text block.",
    blocks: ["Text"],
  },
  {
    value: "cornell",
    label: "Cornell",
    description: "Structured for active recall: cue column, notes area, and summary.",
    blocks: ["Cue", "Text (Notes)", "Summary"],
  },
  {
    value: "zettelkasten",
    label: "Zettelkasten",
    description: "Atomic idea cards for long-term knowledge building.",
    blocks: ["Atomic Idea", "Source", "Text (Context)"],
  },
  {
    value: "feedback",
    label: "Feedback Analysis",
    description: "Structured comparison of expected vs actual outcomes.",
    blocks: ["Expected", "Actual", "Deviation", "Cause", "Action"],
  },
  {
    value: "daily",
    label: "Daily Log",
    description: "Simple daily record with heading and free text.",
    blocks: ["Heading: Today", "Text"],
  },
  {
    value: "retrospective",
    label: "Retrospective",
    description: "Project or sprint retrospective template.",
    blocks: ["What went well", "What could be improved", "Action items"],
  },
  {
    value: "meeting",
    label: "Meeting Minutes",
    description: "Structured meeting notes with attendees and decisions.",
    blocks: ["Attendees", "Agenda", "Decisions", "Action Items"],
  },
];

function CreateNoteDialog({ onClose, onCreated }: CreateNoteDialogProps) {
  const [title, setTitle] = useState("");
  const [template, setTemplate] = useState("");
  const [notebookId, setNotebookId] = useState("");
  const [notebooks, setNotebooks] = useState<NotebookItem[]>([]);
  const [creating, setCreating] = useState(false);

  useEffect(() => {
    invoke<NotebookItem[]>("list_notebooks").then((nbs) => {
      setNotebooks(nbs);
      const first = nbs.find((nb) => !nb.is_inbox);
      if (first) setNotebookId(first.id);
    }).catch(console.error);
  }, []);

  const selectedTemplate = TEMPLATES.find((t) => t.value === template) || TEMPLATES[0];

  async function handleCreate() {
    if (!title.trim()) return;
    setCreating(true);
    try {
      await invoke("create_note", {
        title,
        notebookId: notebookId || null,
        template: template || null,
      });
      onCreated();
    } catch (e) {
      console.error("Create failed:", e);
    }
    setCreating(false);
  }

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog-card" style={{ width: 540 }} onClick={(e) => e.stopPropagation()}>
        <div className="dialog-title">Create Note</div>

        {/* Title input */}
        <input
          type="text"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          onKeyDown={(e) => { if (e.key === "Enter") handleCreate(); }}
          placeholder="Note title..."
          autoFocus
          style={{
            width: "100%",
            border: "1px solid var(--line)",
            borderRadius: "var(--radius-sm)",
            padding: "10px 14px",
            fontSize: 14,
            fontFamily: "var(--font-sans)",
            outline: "none",
            marginBottom: 12,
          }}
        />

        {/* Notebook selector */}
        <div style={{ display: "flex", gap: 8, marginBottom: 16 }}>
          <select
            value={notebookId}
            onChange={(e) => setNotebookId(e.target.value)}
            className="create-dialog-select"
          >
            {notebooks.filter((nb) => !nb.is_inbox).map((nb) => (
              <option key={nb.id} value={nb.id}>{nb.name}</option>
            ))}
          </select>
        </div>

        {/* Template selector with preview */}
        <div className="template-grid">
          {TEMPLATES.map((tmpl) => (
            <div
              key={tmpl.value}
              className={`template-card ${template === tmpl.value ? "selected" : ""}`}
              onClick={() => setTemplate(tmpl.value)}
            >
              <div className="template-card-name">{tmpl.label}</div>
              <div className="template-card-desc">{tmpl.description}</div>
            </div>
          ))}
        </div>

        {/* Template preview */}
        <div className="template-preview">
          <div className="template-preview-title">Structure Preview: {selectedTemplate.label}</div>
          <div className="template-preview-blocks">
            {selectedTemplate.blocks.map((block, i) => (
              <div key={i} className="template-preview-block">
                <span className="template-preview-index">{i + 1}</span>
                {block}
              </div>
            ))}
          </div>
        </div>

        {/* Actions */}
        <div style={{ display: "flex", justifyContent: "flex-end", gap: 8, marginTop: 16 }}>
          <button className="btn btn-ghost" onClick={onClose}>Cancel</button>
          <button
            className="btn btn-primary"
            onClick={handleCreate}
            disabled={!title.trim() || creating}
          >
            {creating ? "Creating..." : "Create Note"}
          </button>
        </div>
      </div>
    </div>
  );
}

export default CreateNoteDialog;
