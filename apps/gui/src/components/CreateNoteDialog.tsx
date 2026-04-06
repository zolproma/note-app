import { useState, useEffect } from "react";
import { invoke, type NotebookItem } from "../tauri";
import { useI18n } from "../i18n";

interface CreateNoteDialogProps {
  onClose: () => void;
  onCreated: () => void;
}

interface TemplateInfo {
  value: string;
  labelKey: string;
  descKey: string;
  blocks: string[];
}

const TEMPLATE_KEYS: TemplateInfo[] = [
  {
    value: "",
    labelKey: "templateBlank",
    descKey: "templateBlankDesc",
    blocks: ["blockText"],
  },
  {
    value: "cornell",
    labelKey: "templateCornell",
    descKey: "templateCornellDesc",
    blocks: ["blockCue", "blockText", "blockSummary"],
  },
  {
    value: "zettelkasten",
    labelKey: "templateZettelkasten",
    descKey: "templateZettelkastenDesc",
    blocks: ["blockAtomicIdea", "blockSource", "blockText"],
  },
  {
    value: "feedback",
    labelKey: "templateFeedback",
    descKey: "templateFeedbackDesc",
    blocks: ["blockExpected", "blockActual", "blockDeviation", "blockCause", "blockAction"],
  },
  {
    value: "daily",
    labelKey: "templateDaily",
    descKey: "templateDailyDesc",
    blocks: ["blockHeading", "blockText"],
  },
  {
    value: "retrospective",
    labelKey: "templateRetrospective",
    descKey: "templateRetrospectiveDesc",
    blocks: ["blockText", "blockText", "blockText"],
  },
  {
    value: "meeting",
    labelKey: "templateMeeting",
    descKey: "templateMeetingDesc",
    blocks: ["blockText", "blockText", "blockText", "blockText"],
  },
];

function CreateNoteDialog({ onClose, onCreated }: CreateNoteDialogProps) {
  const t = useI18n();
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

  const tAny = t as unknown as Record<string, string>;
  const templates = TEMPLATE_KEYS.map((tmpl) => ({
    ...tmpl,
    label: tAny[tmpl.labelKey] || tmpl.labelKey,
    description: tAny[tmpl.descKey] || tmpl.descKey,
    blockLabels: tmpl.blocks.map((k) => tAny[k] || k),
  }));

  const selectedTemplate = templates.find((tmpl) => tmpl.value === template) || templates[0];

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
        <div className="dialog-title">{t.createNote}</div>

        {/* Title input */}
        <input
          type="text"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          onKeyDown={(e) => { if (e.key === "Enter") handleCreate(); }}
          placeholder={t.noteTitle + "..."}
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
          {templates.map((tmpl) => (
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
          <div className="template-preview-title">{t.preview}: {selectedTemplate.label}</div>
          <div className="template-preview-blocks">
            {selectedTemplate.blockLabels.map((block, i) => (
              <div key={i} className="template-preview-block">
                <span className="template-preview-index">{i + 1}</span>
                {block}
              </div>
            ))}
          </div>
        </div>

        {/* Actions */}
        <div style={{ display: "flex", justifyContent: "flex-end", gap: 8, marginTop: 16 }}>
          <button className="btn btn-ghost" onClick={onClose}>{t.cancel}</button>
          <button
            className="btn btn-primary"
            onClick={handleCreate}
            disabled={!title.trim() || creating}
          >
            {creating ? t.saving : t.createNote}
          </button>
        </div>
      </div>
    </div>
  );
}

export default CreateNoteDialog;
