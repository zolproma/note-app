import { useEffect, useState, useCallback } from "react";
import { invoke, type AttachmentItem } from "../tauri";
import { useI18n } from "../i18n";

interface AttachmentPanelProps {
  noteId: string;
}

const mediaTypeIcons: Record<string, string> = {
  image: "IMG",
  audio: "AUD",
  video: "VID",
  pdf: "PDF",
  document: "DOC",
  other: "FILE",
};

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function AttachmentPanel({ noteId }: AttachmentPanelProps) {
  const t = useI18n();
  const [attachments, setAttachments] = useState<AttachmentItem[]>([]);
  const [uploading, setUploading] = useState(false);
  const [dragOver, setDragOver] = useState(false);

  useEffect(() => {
    invoke<AttachmentItem[]>("list_note_attachments", { noteId })
      .then(setAttachments)
      .catch(console.error);
  }, [noteId]);

  const handleUpload = useCallback(async () => {
    try {
      // Use Tauri dialog to pick files
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        multiple: true,
        title: t.attachFiles,
      });
      if (!selected) return;

      setUploading(true);
      const files = Array.isArray(selected) ? selected : [selected];
      for (const file of files) {
        const path = typeof file === "string" ? file : (file as { path?: string })?.path;
        if (!path) continue;
        try {
          const att = await invoke<AttachmentItem>("upload_attachment", {
            noteId,
            filePath: path,
          });
          setAttachments((prev) => [...prev, att]);
        } catch (e) {
          console.error("Upload failed:", e);
        }
      }
      setUploading(false);
    } catch (e) {
      console.error("Dialog failed:", e);
      setUploading(false);
    }
  }, [noteId, t]);

  async function handleDelete(id: string) {
    try {
      await invoke("delete_attachment", { id });
      setAttachments((prev) => prev.filter((a) => a.id !== id));
    } catch (e) {
      console.error("Delete failed:", e);
    }
  }

  async function handleOpen(id: string) {
    try {
      const path = await invoke<string>("get_attachment_path", { id });
      const { openPath } = await import("@tauri-apps/plugin-opener");
      await openPath(path);
    } catch (e) {
      console.error("Open failed:", e);
    }
  }

  // Drag & drop handling
  function handleDragOver(e: React.DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    setDragOver(true);
  }

  function handleDragLeave(e: React.DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    setDragOver(false);
  }

  async function handleDrop(e: React.DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    setDragOver(false);

    // Tauri drag-and-drop gives file paths
    const files = e.dataTransfer.files;
    if (files.length === 0) return;

    setUploading(true);
    for (let i = 0; i < files.length; i++) {
      const file = files[i];
      // In Tauri, we can get the path from the file
      const path = (file as unknown as { path?: string }).path;
      if (path) {
        try {
          const att = await invoke<AttachmentItem>("upload_attachment", {
            noteId,
            filePath: path,
          });
          setAttachments((prev) => [...prev, att]);
        } catch (err) {
          console.error("Drop upload failed:", err);
        }
      }
    }
    setUploading(false);
  }

  return (
    <div
      className={`attachment-panel ${dragOver ? "drag-over" : ""}`}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
    >
      <div className="attachment-panel-header">
        <span className="attachment-panel-title">
          {t.attachFiles} ({attachments.length})
        </span>
        <button
          className="btn btn-ghost"
          style={{ fontSize: 11, padding: "4px 8px" }}
          onClick={handleUpload}
          disabled={uploading}
        >
          {uploading ? t.uploading : t.attach}
        </button>
      </div>

      {attachments.length === 0 ? (
        <div className="attachment-empty">
          {t.dropFiles}
        </div>
      ) : (
        <div className="attachment-list">
          {attachments.map((att) => (
            <div key={att.id} className="attachment-item">
              <div
                className={`attachment-icon attachment-icon-${att.media_type}`}
              >
                {mediaTypeIcons[att.media_type] || "FILE"}
              </div>
              <div className="attachment-info" onClick={() => handleOpen(att.id)}>
                <div className="attachment-filename">{att.filename}</div>
                <div className="attachment-meta">
                  {formatSize(att.size_bytes)}
                </div>
              </div>
              <button
                className="block-action-btn block-action-delete"
                onClick={() => handleDelete(att.id)}
                title={t.deleteAttachment}
              >
                &#x2715;
              </button>
            </div>
          ))}
        </div>
      )}

      {/* Image previews */}
      {attachments.filter((a) => a.media_type === "image").length > 0 && (
        <div className="attachment-previews">
          {attachments
            .filter((a) => a.media_type === "image")
            .map((att) => (
              <div
                key={att.id}
                className="attachment-preview-thumb"
                onClick={() => handleOpen(att.id)}
                title={att.filename}
              >
                <div className="preview-placeholder">
                  {att.filename}
                </div>
              </div>
            ))}
        </div>
      )}
    </div>
  );
}

export default AttachmentPanel;
