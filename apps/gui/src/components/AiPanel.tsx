import { useState } from "react";
import { invoke, AiSuggestionItem, AiConfig } from "../tauri";

interface AiPanelProps {
  noteId: string;
  onApplyTags?: (tags: string[]) => void;
  onNavigate?: (noteId: string) => void;
}

const DEFAULT_CONFIG: AiConfig = {
  provider: "ollama",
  model: "llama3",
  mode: "local_only",
};

type AiJob = "suggest_tags" | "summarize" | "classify" | "suggest_links";

export default function AiPanel({ noteId, onApplyTags, onNavigate }: AiPanelProps) {
  const [config, setConfig] = useState<AiConfig>(DEFAULT_CONFIG);
  const [showSettings, setShowSettings] = useState(false);
  const [loading, setLoading] = useState<AiJob | null>(null);
  const [suggestion, setSuggestion] = useState<AiSuggestionItem | null>(null);
  const [error, setError] = useState<string | null>(null);

  const runAi = async (job: AiJob) => {
    setLoading(job);
    setError(null);
    setSuggestion(null);
    try {
      const cmd = `ai_${job}`;
      const result = await invoke<AiSuggestionItem>(cmd, {
        noteId,
        config,
      });
      setSuggestion(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(null);
    }
  };

  const handleApplyTags = () => {
    if (!suggestion || suggestion.job_type !== "suggest_tags" || !onApplyTags) return;
    try {
      const tags = JSON.parse(suggestion.content) as string[];
      onApplyTags(tags);
    } catch {
      setError("Failed to parse tag suggestions");
    }
  };

  return (
    <div className="ai-panel">
      <div className="ai-panel-header">
        <span className="ai-panel-title">AI Assistant</span>
        <button
          className="btn-icon"
          onClick={() => setShowSettings(!showSettings)}
          title="Settings"
        >
          {showSettings ? "\u2715" : "\u2699"}
        </button>
      </div>

      {showSettings && (
        <div className="ai-settings">
          <label className="ai-setting-row">
            <span>Provider</span>
            <select
              value={config.provider}
              onChange={(e) => setConfig({ ...config, provider: e.target.value })}
            >
              <option value="ollama">Ollama (local)</option>
              <option value="openai">OpenAI</option>
            </select>
          </label>
          <label className="ai-setting-row">
            <span>Model</span>
            <input
              type="text"
              value={config.model}
              onChange={(e) => setConfig({ ...config, model: e.target.value })}
              placeholder="llama3"
            />
          </label>
          <label className="ai-setting-row">
            <span>Mode</span>
            <select
              value={config.mode}
              onChange={(e) => setConfig({ ...config, mode: e.target.value })}
            >
              <option value="local_only">Local Only</option>
              <option value="private_api">Private API</option>
            </select>
          </label>
          {config.provider === "openai" && (
            <label className="ai-setting-row">
              <span>API Key</span>
              <input
                type="password"
                value={config.api_key || ""}
                onChange={(e) =>
                  setConfig({ ...config, api_key: e.target.value || undefined })
                }
                placeholder="sk-..."
              />
            </label>
          )}
        </div>
      )}

      <div className="ai-actions">
        <button
          className="ai-action-btn"
          onClick={() => runAi("suggest_tags")}
          disabled={loading !== null}
        >
          {loading === "suggest_tags" ? "Thinking..." : "Suggest Tags"}
        </button>
        <button
          className="ai-action-btn"
          onClick={() => runAi("summarize")}
          disabled={loading !== null}
        >
          {loading === "summarize" ? "Thinking..." : "Summarize"}
        </button>
        <button
          className="ai-action-btn"
          onClick={() => runAi("classify")}
          disabled={loading !== null}
        >
          {loading === "classify" ? "Thinking..." : "Classify"}
        </button>
        <button
          className="ai-action-btn"
          onClick={() => runAi("suggest_links")}
          disabled={loading !== null}
        >
          {loading === "suggest_links" ? "Thinking..." : "Suggest Links"}
        </button>
      </div>

      {error && <div className="ai-error">{error}</div>}

      {suggestion && (
        <div className="ai-suggestion">
          <div className="ai-suggestion-header">
            <span className="ai-suggestion-type">{suggestion.job_type.replace(/_/g, " ")}</span>
            <span className="ai-suggestion-model">{suggestion.model}</span>
          </div>
          <div className="ai-suggestion-content">
            {suggestion.job_type === "suggest_tags" ? (
              <div className="ai-tags-result">
                {(() => {
                  try {
                    const tags = JSON.parse(suggestion.content) as string[];
                    return (
                      <>
                        <div className="ai-tag-list">
                          {tags.map((tag) => (
                            <span key={tag} className="tag">{tag}</span>
                          ))}
                        </div>
                        {onApplyTags && (
                          <button className="btn-sm ai-apply-btn" onClick={handleApplyTags}>
                            Apply Tags
                          </button>
                        )}
                      </>
                    );
                  } catch {
                    return <pre>{suggestion.content}</pre>;
                  }
                })()}
              </div>
            ) : suggestion.job_type === "suggest_links" ? (
              <div className="ai-links-result">
                {(() => {
                  try {
                    const links = JSON.parse(suggestion.content) as { note_id: string; reason: string }[];
                    if (links.length === 0) return <p>No links suggested.</p>;
                    return links.map((link) => (
                      <div key={link.note_id} className="ai-link-item">
                        <button
                          className="btn-link"
                          onClick={() => onNavigate?.(link.note_id)}
                        >
                          {link.note_id.slice(0, 8)}
                        </button>
                        <span className="ai-link-reason">{link.reason}</span>
                      </div>
                    ));
                  } catch {
                    return <pre>{suggestion.content}</pre>;
                  }
                })()}
              </div>
            ) : (
              <p>{suggestion.content}</p>
            )}
          </div>
          <div className="ai-suggestion-status">
            Status: {suggestion.status}
          </div>
        </div>
      )}
    </div>
  );
}
