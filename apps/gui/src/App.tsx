import { useState, useCallback, useEffect, useRef } from "react";
import Sidebar from "./components/Sidebar";
import Topbar from "./components/Topbar";
import InboxView from "./components/InboxView";
import NotesView from "./components/NotesView";
import SearchView from "./components/SearchView";
import NoteEditorView from "./components/NoteEditorView";
import GraphView from "./components/GraphView";
import CaptureDialog from "./components/CaptureDialog";

export type View = "inbox" | "notes" | "search" | "graph" | "editor";

function App() {
  const [view, setView] = useState<View>("inbox");
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedNoteId, setSelectedNoteId] = useState<string | null>(null);
  const [showCapture, setShowCapture] = useState(false);
  const [refreshKey, setRefreshKey] = useState(0);
  const searchInputRef = useRef<HTMLInputElement>(null);

  const refresh = useCallback(() => setRefreshKey((k) => k + 1), []);

  // Global keyboard shortcuts
  useEffect(() => {
    function handleGlobalKeyDown(e: KeyboardEvent) {
      // Ctrl+Shift+N: Quick Capture
      if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key === "N") {
        e.preventDefault();
        setShowCapture(true);
        return;
      }
      // Ctrl+/: Focus search
      if ((e.metaKey || e.ctrlKey) && e.key === "/") {
        e.preventDefault();
        setView("search");
        setTimeout(() => searchInputRef.current?.focus(), 50);
        return;
      }
    }
    window.addEventListener("keydown", handleGlobalKeyDown);
    return () => window.removeEventListener("keydown", handleGlobalKeyDown);
  }, []);

  const openNote = useCallback((id: string) => {
    setSelectedNoteId(id);
    setView("editor");
  }, []);

  const goBack = useCallback(() => {
    setSelectedNoteId(null);
    setView("notes");
    refresh();
  }, [refresh]);

  return (
    <div className="app-layout">
      <Sidebar
        currentView={view}
        onNavigate={(v) => {
          setView(v);
          setSelectedNoteId(null);
        }}
        onCapture={() => setShowCapture(true)}
      />
      <div className="main-content">
        <Topbar
          view={view}
          searchQuery={searchQuery}
          onSearchChange={setSearchQuery}
          onSearch={() => {
            if (searchQuery.trim()) setView("search");
          }}
          onBack={view === "editor" ? goBack : undefined}
          searchInputRef={searchInputRef}
        />
        <div className="content-area">
          {view === "inbox" && <InboxView key={refreshKey} onOpenNote={openNote} onRefresh={refresh} />}
          {view === "notes" && <NotesView key={refreshKey} onOpenNote={openNote} onRefresh={refresh} />}
          {view === "search" && <SearchView query={searchQuery} onOpenNote={openNote} />}
          {view === "graph" && <GraphView onOpenNote={openNote} />}
          {view === "editor" && selectedNoteId && (
            <NoteEditorView noteId={selectedNoteId} onBack={goBack} onOpenNote={openNote} />
          )}
        </div>
      </div>
      {showCapture && (
        <CaptureDialog
          onClose={() => setShowCapture(false)}
          onCaptured={() => {
            setShowCapture(false);
            setView("inbox");
            refresh();
          }}
        />
      )}
    </div>
  );
}

export default App;
