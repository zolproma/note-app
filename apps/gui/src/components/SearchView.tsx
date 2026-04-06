import { useEffect, useState } from "react";
import {
  invoke,
  type SearchResultItem,
  type SearchFilterInput,
  type SavedSearchItem,
  type NotebookItem,
  type TagItem,
} from "../tauri";
import { useI18n } from "../i18n";

interface SearchViewProps {
  query: string;
  onOpenNote: (id: string) => void;
}

function SearchView({ query, onOpenNote }: SearchViewProps) {
  const t = useI18n();
  const [results, setResults] = useState<SearchResultItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [searched, setSearched] = useState(false);

  // Filters
  const [filterLifecycle, setFilterLifecycle] = useState("");
  const [filterNotebook, setFilterNotebook] = useState("");
  const [filterTag, setFilterTag] = useState("");
  const [showFilters, setShowFilters] = useState(false);

  // Data for filter dropdowns
  const [notebooks, setNotebooks] = useState<NotebookItem[]>([]);
  const [allTags, setAllTags] = useState<TagItem[]>([]);

  // Saved searches
  const [savedSearches, setSavedSearches] = useState<SavedSearchItem[]>([]);
  const [saveName, setSaveName] = useState("");
  const [showSaveInput, setShowSaveInput] = useState(false);

  useEffect(() => {
    invoke<NotebookItem[]>("list_notebooks").then(setNotebooks).catch(console.error);
    invoke<TagItem[]>("list_all_tags").then(setAllTags).catch(console.error);
    invoke<SavedSearchItem[]>("list_saved_searches").then(setSavedSearches).catch(console.error);
  }, []);

  useEffect(() => {
    if (query.trim()) {
      doSearch();
    }
  }, [query]);

  async function doSearch(overrideFilter?: SearchFilterInput) {
    setLoading(true);
    const filter: SearchFilterInput = overrideFilter || {
      query: query.trim() || undefined,
      lifecycle: filterLifecycle || undefined,
      notebook_id: filterNotebook || undefined,
      tags: filterTag ? [filterTag] : undefined,
    };
    try {
      const items = await invoke<SearchResultItem[]>("filtered_search", { filter });
      setResults(items);
      setSearched(true);
    } catch (e) {
      console.error("Search failed:", e);
    }
    setLoading(false);
  }

  async function handleSaveSearch() {
    if (!saveName.trim()) return;
    const filter: SearchFilterInput = {
      query: query.trim() || undefined,
      lifecycle: filterLifecycle || undefined,
      notebook_id: filterNotebook || undefined,
      tags: filterTag ? [filterTag] : undefined,
    };
    try {
      const ss = await invoke<SavedSearchItem>("save_search", { name: saveName, filter });
      setSavedSearches((prev) => [...prev, ss]);
      setSaveName("");
      setShowSaveInput(false);
    } catch (e) {
      console.error("Save search failed:", e);
    }
  }

  async function runSavedSearch(ss: SavedSearchItem) {
    const filter: SearchFilterInput = JSON.parse(ss.filter_json);
    await doSearch(filter);
  }

  async function deleteSavedSearch(id: string) {
    await invoke("delete_saved_search", { id });
    setSavedSearches((prev) => prev.filter((s) => s.id !== id));
  }

  if (loading) return <div className="empty-state"><div className="empty-state-desc">Loading...</div></div>;

  return (
    <div>
      {/* Saved searches */}
      {savedSearches.length > 0 && (
        <div className="saved-searches">
          <div className="saved-searches-label">{t.savedSearches}</div>
          <div className="saved-searches-list">
            {savedSearches.map((ss) => (
              <div key={ss.id} className="saved-search-chip">
                <span onClick={() => runSavedSearch(ss)}>{ss.name}</span>
                <button className="saved-search-remove" onClick={() => deleteSavedSearch(ss.id)}>&times;</button>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Filters */}
      <div className="search-toolbar">
        <button className="btn btn-ghost" onClick={() => setShowFilters(!showFilters)}>
          {t.filters} {showFilters ? "^" : "v"}
        </button>
        <button className="btn btn-ghost" onClick={() => doSearch()}>{t.search}</button>
        {searched && (
          <>
            <span style={{ fontSize: 12, color: "var(--muted)" }}>{results.length} {t.resultCount}</span>
            {showSaveInput ? (
              <span className="save-search-inline">
                <input
                  type="text"
                  value={saveName}
                  onChange={(e) => setSaveName(e.target.value)}
                  onKeyDown={(e) => { if (e.key === "Enter") handleSaveSearch(); if (e.key === "Escape") setShowSaveInput(false); }}
                  placeholder={t.searchName}
                  autoFocus
                />
                <button className="btn btn-primary" style={{ padding: "4px 10px", fontSize: 11 }} onClick={handleSaveSearch}>{t.save}</button>
              </span>
            ) : (
              <button className="btn btn-ghost" style={{ fontSize: 11 }} onClick={() => setShowSaveInput(true)}>{t.saveSearch}</button>
            )}
          </>
        )}
      </div>

      {showFilters && (
        <div className="search-filters">
          <select value={filterLifecycle} onChange={(e) => setFilterLifecycle(e.target.value)} className="search-filter-select">
            <option value="">{t.anyStatus}</option>
            <option value="inbox">{t.inbox}</option>
            <option value="active">{t.active}</option>
            <option value="archived">{t.archived}</option>
          </select>
          <select value={filterNotebook} onChange={(e) => setFilterNotebook(e.target.value)} className="search-filter-select">
            <option value="">{t.anyNotebook}</option>
            {notebooks.filter((nb) => !nb.is_inbox).map((nb) => (
              <option key={nb.id} value={nb.id}>{nb.name}</option>
            ))}
          </select>
          <select value={filterTag} onChange={(e) => setFilterTag(e.target.value)} className="search-filter-select">
            <option value="">{t.anyTag}</option>
            {allTags.map((tag) => (
              <option key={tag.id} value={tag.name}>{tag.name}</option>
            ))}
          </select>
          <button className="btn btn-ghost" style={{ fontSize: 11 }} onClick={() => { setFilterLifecycle(""); setFilterNotebook(""); setFilterTag(""); }}>{t.clear}</button>
        </div>
      )}

      {/* Results */}
      {!searched || !query.trim() ? (
        <div className="empty-state">
          <div className="empty-state-title">{t.searchEmpty}</div>
          <div className="empty-state-desc">{t.searchEmptyDesc}</div>
        </div>
      ) : results.length === 0 ? (
        <div className="empty-state">
          <div className="empty-state-title">{t.noResults}</div>
          <div className="empty-state-desc">{t.searchEmptyDesc}</div>
        </div>
      ) : (
        <div className="note-list">
          {results.map((r) => (
            <div key={r.note_id} className="note-item" onClick={() => onOpenNote(r.note_id)} style={{ cursor: "pointer" }}>
              <div style={{ flex: 1 }}>
                <div className="note-title">{r.title}</div>
                {r.snippet && (
                  <div className="search-snippet">{r.snippet}</div>
                )}
                <div className="note-meta">
                  {new Date(r.updated_at).toLocaleDateString("zh-CN", { month: "short", day: "numeric", hour: "2-digit", minute: "2-digit" })}
                </div>
              </div>
              <span className={`note-lifecycle ${r.lifecycle}`}>{r.lifecycle}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

export default SearchView;
