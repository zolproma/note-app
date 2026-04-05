import { useEffect, useState } from "react";
import { invoke, type NoteItem } from "../tauri";
import CreateNoteDialog from "./CreateNoteDialog";

interface NotesViewProps {
  onOpenNote: (id: string) => void;
  onRefresh: () => void;
}

function NotesView({ onOpenNote, onRefresh }: NotesViewProps) {
  const [notes, setNotes] = useState<NoteItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreate, setShowCreate] = useState(false);

  useEffect(() => {
    invoke<NoteItem[]>("list_all_notes")
      .then(setNotes)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  async function handleDelete(id: string) {
    await invoke("delete_note", { id });
    setNotes((prev) => prev.filter((n) => n.id !== id));
    onRefresh();
  }

  async function handleCreated() {
    setShowCreate(false);
    const updated = await invoke<NoteItem[]>("list_all_notes");
    setNotes(updated);
    onRefresh();
  }

  if (loading) return <div className="empty-state"><div className="empty-state-desc">Loading...</div></div>;

  return (
    <div>
      <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 16 }}>
        <span style={{ fontSize: 13, color: "var(--muted)" }}>{notes.length} note(s)</span>
        <button className="btn btn-primary" onClick={() => setShowCreate(true)}>+ New Note</button>
      </div>

      {notes.length === 0 ? (
        <div className="empty-state">
          <div className="empty-state-title">No notes yet</div>
          <div className="empty-state-desc">Click "+ New Note" to get started.</div>
        </div>
      ) : (
        <div className="note-list">
          {notes.map((note) => (
            <div key={note.id} className="note-item">
              <div style={{ cursor: "pointer", flex: 1 }} onClick={() => onOpenNote(note.id)}>
                <div className="note-title">{note.title}</div>
                <div className="note-meta">
                  {new Date(note.updated_at).toLocaleDateString("zh-CN", { month: "short", day: "numeric", hour: "2-digit", minute: "2-digit" })}
                </div>
              </div>
              <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
                <span className={`note-lifecycle ${note.lifecycle}`}>{note.lifecycle}</span>
                <button className="btn btn-ghost" style={{ fontSize: 11, padding: "4px 8px", color: "var(--danger)" }} onClick={() => handleDelete(note.id)}>
                  Delete
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {showCreate && (
        <CreateNoteDialog
          onClose={() => setShowCreate(false)}
          onCreated={handleCreated}
        />
      )}
    </div>
  );
}

export default NotesView;
