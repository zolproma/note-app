interface TauriWindow {
  __TAURI_INTERNALS__?: unknown;
}

const isTauri = typeof window !== "undefined" && !!(window as TauriWindow).__TAURI_INTERNALS__;

export interface NoteItem {
  id: string;
  title: string;
  lifecycle: string;
  notebook_id: string;
  pinned: boolean;
  created_at: string;
  updated_at: string;
}

export interface NoteDetail {
  id: string;
  title: string;
  lifecycle: string;
  notebook_id: string;
  blocks: BlockItem[];
  tags: TagItem[];
  created_at: string;
  updated_at: string;
}

export interface BlockItem {
  id: string;
  block_type: string;
  content: string;
  sort_order: number;
}

export interface TagItem {
  id: string;
  name: string;
}

export interface NotebookItem {
  id: string;
  name: string;
  is_inbox: boolean;
}

export interface LinkItem {
  id: string;
  source_note_id: string;
  target_note_id: string;
  target_title: string;
  link_type: string;
}

export interface SearchResultItem {
  note_id: string;
  title: string;
  lifecycle: string;
  notebook_id: string;
  snippet: string;
  pinned: boolean;
  updated_at: string;
}

export interface SearchFilterInput {
  query?: string;
  tags?: string[];
  notebook_id?: string;
  lifecycle?: string;
  pinned?: boolean;
}

export interface SavedSearchItem {
  id: string;
  name: string;
  filter_json: string;
}

export interface GraphData {
  nodes: GraphNode[];
  edges: GraphEdge[];
}

export interface GraphNode {
  id: string;
  title: string;
  lifecycle: string;
  link_count: number;
}

export interface GraphEdge {
  source: string;
  target: string;
}

export interface AttachmentItem {
  id: string;
  note_id: string;
  filename: string;
  media_type: string;
  storage_path: string;
  size_bytes: number;
  created_at: string;
}

export interface AiSuggestionItem {
  id: string;
  job_type: string;
  note_id: string;
  content: string;
  status: string;
  model: string;
  created_at: string;
}

export interface AiConfig {
  provider: string;
  model: string;
  api_key?: string;
  mode: string;
}

// Mock data for browser dev
const mockNotebooks: NotebookItem[] = [
  { id: "nb-1", name: "Inbox", is_inbox: true },
  { id: "nb-2", name: "Engineering", is_inbox: false },
  { id: "nb-3", name: "Reading", is_inbox: false },
];

const mockNotes: NoteItem[] = [
  { id: "m-001", title: "Welcome to Notes", lifecycle: "active", notebook_id: "nb-2", pinned: false, created_at: new Date().toISOString(), updated_at: new Date().toISOString() },
  { id: "m-002", title: "Quick capture example", lifecycle: "inbox", notebook_id: "nb-1", pinned: false, created_at: new Date().toISOString(), updated_at: new Date().toISOString() },
  { id: "m-003", title: "Raft Consensus", lifecycle: "active", notebook_id: "nb-2", pinned: false, created_at: new Date().toISOString(), updated_at: new Date().toISOString() },
];

const mockDetail: NoteDetail = {
  id: "m-001", title: "Welcome to Notes", lifecycle: "active", notebook_id: "nb-2",
  blocks: [
    { id: "b-1", block_type: "heading", content: "Getting Started", sort_order: 0 },
    { id: "b-2", block_type: "text", content: "This is your local-first note-taking app.", sort_order: 1 },
  ],
  tags: [{ id: "t-1", name: "welcome" }],
  created_at: new Date().toISOString(), updated_at: new Date().toISOString(),
};

export async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (isTauri) {
    const { invoke: tauriInvoke } = await import("@tauri-apps/api/core");
    return tauriInvoke<T>(cmd, args);
  }

  // Mock responses for browser dev
  switch (cmd) {
    case "list_inbox":
      return mockNotes.filter((n) => n.lifecycle === "inbox") as T;
    case "list_all_notes":
      return mockNotes as T;
    case "list_notebooks":
      return mockNotebooks as T;
    case "search_notes":
      return mockNotes.filter((n) =>
        n.title.toLowerCase().includes(((args?.query as string) || "").toLowerCase())
      ) as T;
    case "get_note":
      return { ...mockDetail, id: args?.id as string } as T;
    case "get_backlinks":
      return [] as T;
    case "create_note":
      return { id: "new-" + Date.now(), title: args?.title as string, lifecycle: "active", notebook_id: "nb-2", pinned: false, created_at: new Date().toISOString(), updated_at: new Date().toISOString() } as T;
    case "capture_note":
      return { id: "cap-" + Date.now(), title: (args?.content as string).slice(0, 60), lifecycle: "inbox", notebook_id: "nb-1", pinned: false, created_at: new Date().toISOString(), updated_at: new Date().toISOString() } as T;
    case "promote_inbox":
      return { ...(mockNotes.find((n) => n.id === args?.id) || mockNotes[0]), lifecycle: "active" } as T;
    case "delete_note":
      return undefined as T;
    case "update_note_title":
      return { ...(mockNotes.find((n) => n.id === args?.id) || mockNotes[0]), title: args?.title as string } as T;
    case "update_note_blocks":
      return undefined as T;
    case "move_note_to_notebook":
      return { ...(mockNotes.find((n) => n.id === args?.id) || mockNotes[0]), notebook_id: args?.notebook_id as string } as T;
    case "archive_note":
      return { ...(mockNotes.find((n) => n.id === args?.id) || mockNotes[0]), lifecycle: "archived" } as T;
    case "set_lifecycle":
      return { ...(mockNotes.find((n) => n.id === args?.id) || mockNotes[0]), lifecycle: args?.lifecycle as string } as T;
    case "list_all_tags":
      return [{ id: "t-1", name: "welcome" }, { id: "t-2", name: "important" }] as T;
    case "add_tag":
      return { id: "t-" + Date.now(), name: args?.tag_name as string } as T;
    case "remove_tag":
      return undefined as T;
    case "list_links_from":
      return [] as T;
    case "resolve_wiki_link":
      return null as T;
    case "create_wiki_link":
      return { id: "lnk-" + Date.now(), source_note_id: args?.source_id, target_note_id: args?.target_id, target_title: "Linked Note", link_type: "WikiLink" } as T;
    case "delete_link":
      return undefined as T;
    case "filtered_search":
      return mockNotes.map((n) => ({ note_id: n.id, title: n.title, lifecycle: n.lifecycle, notebook_id: n.notebook_id, snippet: "Preview text...", pinned: n.pinned, updated_at: n.updated_at })) as T;
    case "list_saved_searches":
      return [] as T;
    case "save_search":
      return { id: "ss-" + Date.now(), name: args?.name, filter_json: "{}" } as T;
    case "delete_saved_search":
      return undefined as T;
    case "get_graph_data":
      return { nodes: mockNotes.map((n) => ({ id: n.id, title: n.title, lifecycle: n.lifecycle, link_count: 0 })), edges: [] } as T;
    case "get_related_notes":
      return [] as T;
    case "list_note_attachments":
      return [] as T;
    case "upload_attachment":
      return { id: "att-" + Date.now(), note_id: args?.note_id, filename: "mock.png", media_type: "image", storage_path: "mock.png", size_bytes: 1024, created_at: new Date().toISOString() } as T;
    case "delete_attachment":
      return undefined as T;
    case "get_attachment_path":
      return "/tmp/mock.png" as T;
    case "ai_suggest_tags":
      return { id: "ai-" + Date.now(), job_type: "suggest_tags", note_id: args?.note_id, content: '["rust", "programming", "notes"]', status: "pending", model: "mock", created_at: new Date().toISOString() } as T;
    case "ai_summarize":
      return { id: "ai-" + Date.now(), job_type: "summarize", note_id: args?.note_id, content: "This note covers the main concepts and ideas discussed.", status: "pending", model: "mock", created_at: new Date().toISOString() } as T;
    case "ai_classify":
      return { id: "ai-" + Date.now(), job_type: "classify", note_id: args?.note_id, content: '{"notebook_id": "nb-2", "reason": "Engineering content"}', status: "pending", model: "mock", created_at: new Date().toISOString() } as T;
    case "ai_suggest_links":
      return { id: "ai-" + Date.now(), job_type: "suggest_links", note_id: args?.note_id, content: '[]', status: "pending", model: "mock", created_at: new Date().toISOString() } as T;
    default:
      console.warn(`Unknown command: ${cmd}`);
      return [] as T;
  }
}
