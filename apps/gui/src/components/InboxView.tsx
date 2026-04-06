import { useEffect, useState } from "react";
import { invoke, type NoteItem } from "../tauri";
import { useI18n } from "../i18n";

interface InboxViewProps {
  onOpenNote: (id: string) => void;
  onRefresh: () => void;
}

function InboxView({ onOpenNote, onRefresh }: InboxViewProps) {
  const t = useI18n();
  const [notes, setNotes] = useState<NoteItem[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    setLoading(true);
    invoke<NoteItem[]>("list_inbox").then(setNotes).catch(console.error).finally(() => setLoading(false));
  }, []);

  async function handlePromote(id: string) {
    await invoke("promote_inbox", { id });
    setNotes((prev) => prev.filter((n) => n.id !== id));
    onRefresh();
  }

  if (loading) return <div className="empty-state"><div className="empty-state-desc">Loading...</div></div>;

  if (notes.length === 0) {
    return (
      <div className="empty-state">
        <div className="empty-state-title">{t.inboxEmpty}</div>
        <div className="empty-state-desc">{t.inboxEmptyDesc}</div>
      </div>
    );
  }

  return (
    <div className="note-list">
      {notes.map((note) => (
        <div key={note.id} className="note-item">
          <div style={{ cursor: "pointer", flex: 1 }} onClick={() => onOpenNote(note.id)}>
            <div className="note-title">{note.title}</div>
            <div className="note-meta">
              {new Date(note.created_at).toLocaleDateString("zh-CN", { month: "short", day: "numeric", hour: "2-digit", minute: "2-digit" })}
            </div>
          </div>
          <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
            <button className="btn btn-ghost" style={{ fontSize: 12 }} onClick={() => handlePromote(note.id)}>
              {t.promote}
            </button>
            <span className="note-lifecycle inbox">{t.inbox.toLowerCase()}</span>
          </div>
        </div>
      ))}
    </div>
  );
}

export default InboxView;
