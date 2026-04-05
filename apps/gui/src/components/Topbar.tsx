import type { RefObject } from "react";
import type { View } from "../App";

interface TopbarProps {
  view: View;
  searchQuery: string;
  onSearchChange: (q: string) => void;
  onSearch: () => void;
  onBack?: () => void;
  searchInputRef?: RefObject<HTMLInputElement | null>;
}

const viewTitles: Record<View, string> = {
  inbox: "Inbox",
  notes: "All Notes",
  search: "Search",
  graph: "Graph",
  editor: "Note",
};

function Topbar({ view, searchQuery, onSearchChange, onSearch, onBack, searchInputRef }: TopbarProps) {
  return (
    <div className="topbar">
      <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
        {onBack && (
          <button className="btn btn-ghost" onClick={onBack} style={{ padding: "6px 10px" }}>
            <svg width="16" height="16" viewBox="0 0 20 20" fill="currentColor">
              <path fillRule="evenodd" d="M9.707 16.707a1 1 0 01-1.414 0l-6-6a1 1 0 010-1.414l6-6a1 1 0 011.414 1.414L5.414 9H17a1 1 0 110 2H5.414l4.293 4.293a1 1 0 010 1.414z" clipRule="evenodd" />
            </svg>
          </button>
        )}
        <span className="topbar-title">{viewTitles[view]}</span>
      </div>
      <div className="topbar-actions">
        <div className="search-input">
          <svg width="14" height="14" viewBox="0 0 20 20" fill="currentColor" style={{ opacity: 0.4 }}>
            <path fillRule="evenodd" d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z" clipRule="evenodd" />
          </svg>
          <input
            ref={searchInputRef}
            type="text"
            placeholder="Search notes... (Ctrl+/)"
            value={searchQuery}
            onChange={(e) => onSearchChange(e.target.value)}
            onKeyDown={(e) => { if (e.key === "Enter") onSearch(); }}
          />
        </div>
      </div>
    </div>
  );
}

export default Topbar;
