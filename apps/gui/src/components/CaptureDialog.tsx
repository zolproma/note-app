import { useState } from "react";
import { invoke } from "../tauri";
import { useI18n } from "../i18n";

interface CaptureDialogProps {
  onClose: () => void;
  onCaptured: () => void;
}

function CaptureDialog({ onClose, onCaptured }: CaptureDialogProps) {
  const t = useI18n();
  const [content, setContent] = useState("");
  const [submitting, setSubmitting] = useState(false);

  async function handleSubmit() {
    if (!content.trim()) return;
    setSubmitting(true);
    try {
      await invoke("capture_note", { content });
      onCaptured();
    } catch (e) {
      console.error("Capture failed:", e);
    }
    setSubmitting(false);
  }

  return (
    <div
      style={{
        position: "fixed", inset: 0, background: "rgba(0,0,0,0.3)",
        display: "flex", alignItems: "center", justifyContent: "center", zIndex: 100,
      }}
      onClick={onClose}
    >
      <div
        className="card"
        style={{ width: 480, padding: 24, boxShadow: "var(--shadow-lg)" }}
        onClick={(e) => e.stopPropagation()}
      >
        <div style={{ fontSize: 16, fontWeight: 600, marginBottom: 16 }}>{t.captureTitle}</div>
        <textarea
          value={content}
          onChange={(e) => setContent(e.target.value)}
          placeholder={t.capturePlaceholder}
          autoFocus
          rows={4}
          style={{
            width: "100%", border: "1px solid var(--line)", borderRadius: "var(--radius-sm)",
            padding: 12, fontSize: 14, fontFamily: "var(--font-sans)", resize: "vertical",
            outline: "none", marginBottom: 16, lineHeight: 1.6,
          }}
          onKeyDown={(e) => {
            if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) handleSubmit();
          }}
        />
        <div style={{ display: "flex", justifyContent: "flex-end", gap: 8 }}>
          <button className="btn btn-ghost" onClick={onClose}>{t.cancel}</button>
          <button className="btn btn-primary" onClick={handleSubmit} disabled={!content.trim() || submitting}>
            {submitting ? t.saving : t.capture}
          </button>
        </div>
        <div style={{ fontSize: 11, color: "var(--muted)", marginTop: 8, textAlign: "right" }}>
          {t.ctrlEnterSave}
        </div>
      </div>
    </div>
  );
}

export default CaptureDialog;
