import { useEffect, useState } from "react";
import type { View } from "../App";
import { invoke, type NotebookItem } from "../tauri";
import { type Locale, useI18n } from "../i18n";

interface SidebarProps {
  currentView: View;
  onNavigate: (view: View) => void;
  onCapture: () => void;
  locale: Locale;
  onLocaleChange: (locale: Locale) => void;
}

function Sidebar({ currentView, onNavigate, onCapture, locale, onLocaleChange }: SidebarProps) {
  const t = useI18n();
  const [notebooks, setNotebooks] = useState<NotebookItem[]>([]);
  const [showSettings, setShowSettings] = useState(false);

  useEffect(() => {
    invoke<NotebookItem[]>("list_notebooks").then(setNotebooks).catch(console.error);
  }, []);

  return (
    <aside className="sidebar">
      <div className="sidebar-brand">
        <div className="brand-icon">O</div>
        <div>
          <div className="brand-name">{t.appName}</div>
          <div className="brand-sub">{t.appTagline}</div>
        </div>
      </div>

      <div style={{ padding: "0 16px 12px" }}>
        <button className="btn btn-primary" style={{ width: "100%" }} onClick={onCapture}>
          {t.quickCapture}
        </button>
      </div>

      <div className="nav-group-label">{t.workflow}</div>
      <NavButton icon={inboxIcon} label={t.inbox} active={currentView === "inbox"} onClick={() => onNavigate("inbox")} />
      <NavButton icon={notesIcon} label={t.allNotes} active={currentView === "notes"} onClick={() => onNavigate("notes")} />
      <NavButton icon={searchIcon} label={t.search} active={currentView === "search"} onClick={() => onNavigate("search")} />
      <NavButton icon={graphIcon} label={t.graph} active={currentView === "graph"} onClick={() => onNavigate("graph")} />

      {notebooks.length > 0 && (
        <>
          <div className="nav-group-label">{t.notebooks}</div>
          {notebooks.map((nb) => (
            <button key={nb.id} className="nav-item" style={{ fontSize: 12, paddingLeft: 28 }}>
              <span style={{ opacity: 0.5 }}>{nb.is_inbox ? "📥" : "📓"}</span>
              {nb.name}
            </button>
          ))}
        </>
      )}

      <div style={{ flex: 1 }} />

      {/* Settings panel */}
      {showSettings && (
        <div className="settings-panel">
          <div className="settings-row">
            <span>{t.language}</span>
            <select
              value={locale}
              onChange={(e) => onLocaleChange(e.target.value as Locale)}
            >
              <option value="zh">中文</option>
              <option value="en">English</option>
            </select>
          </div>
        </div>
      )}

      {/* Settings button */}
      <NavButton
        icon={settingsIcon}
        label={t.settings}
        active={showSettings}
        onClick={() => setShowSettings(!showSettings)}
      />
    </aside>
  );
}

function NavButton({ icon, label, active, onClick }: { icon: string; label: string; active: boolean; onClick: () => void }) {
  return (
    <button className={`nav-item ${active ? "active" : ""}`} onClick={onClick}>
      <svg viewBox="0 0 20 20" fill="currentColor" dangerouslySetInnerHTML={{ __html: icon }} />
      {label}
    </button>
  );
}

const inboxIcon = '<path d="M4 3a2 2 0 00-2 2v10a2 2 0 002 2h12a2 2 0 002-2V5a2 2 0 00-2-2H4zm2 3a1 1 0 011-1h6a1 1 0 110 2H7a1 1 0 01-1-1zm1 4a1 1 0 100 2h6a1 1 0 100-2H7z" />';
const notesIcon = '<path fill-rule="evenodd" d="M4 4a2 2 0 012-2h4.586A2 2 0 0112 2.586L15.414 6A2 2 0 0116 7.414V16a2 2 0 01-2 2H6a2 2 0 01-2-2V4z" clip-rule="evenodd" />';
const searchIcon = '<path fill-rule="evenodd" d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z" clip-rule="evenodd" />';
const graphIcon = '<path d="M10 3.5a1.5 1.5 0 100 3 1.5 1.5 0 000-3zM5.5 8a1.5 1.5 0 100 3 1.5 1.5 0 000-3zm9 0a1.5 1.5 0 100 3 1.5 1.5 0 000-3zM10 13.5a1.5 1.5 0 100 3 1.5 1.5 0 000-3z" /><path d="M9.5 6.5L6.5 8.5M10.5 6.5L13.5 8.5M6 11l3.5 3M14 11l-3.5 3" stroke="currentColor" stroke-width="1" fill="none" />';
const settingsIcon = '<path fill-rule="evenodd" d="M11.49 3.17c-.38-1.56-2.6-1.56-2.98 0a1.532 1.532 0 01-2.286.948c-1.372-.836-2.942.734-2.106 2.106.54.886.061 2.042-.947 2.287-1.561.379-1.561 2.6 0 2.978a1.532 1.532 0 01.947 2.287c-.836 1.372.734 2.942 2.106 2.106a1.532 1.532 0 012.287.947c.379 1.561 2.6 1.561 2.978 0a1.533 1.533 0 012.287-.947c1.372.836 2.942-.734 2.106-2.106a1.533 1.533 0 01.947-2.287c1.561-.379 1.561-2.6 0-2.978a1.532 1.532 0 01-.947-2.287c.836-1.372-.734-2.942-2.106-2.106a1.532 1.532 0 01-2.287-.947zM10 13a3 3 0 100-6 3 3 0 000 6z" clip-rule="evenodd" />';

export default Sidebar;
