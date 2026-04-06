export type Locale = "en" | "zh";

export interface Messages {
  // App
  appName: string;
  appTagline: string;

  // Sidebar nav
  quickCapture: string;
  workflow: string;
  inbox: string;
  allNotes: string;
  search: string;
  graph: string;
  notebooks: string;
  settings: string;

  // Topbar
  note: string;
  searchPlaceholder: string;

  // Views
  inboxEmpty: string;
  inboxEmptyDesc: string;
  notesEmpty: string;
  notesEmptyDesc: string;
  searchEmpty: string;
  searchEmptyDesc: string;
  noResults: string;

  // Note editor
  untitled: string;
  save: string;
  saved: string;
  saving: string;
  addBlock: string;
  attach: string;
  ai: string;
  links: string;
  move: string;
  archive: string;
  ctrlEnterSave: string;

  // Block types
  blockText: string;
  blockHeading: string;
  blockCode: string;
  blockQuote: string;
  blockCue: string;
  blockSummary: string;
  blockAtomicIdea: string;
  blockSource: string;
  blockExpected: string;
  blockActual: string;
  blockDeviation: string;
  blockCause: string;
  blockAction: string;
  blockDivider: string;

  // Capture dialog
  captureTitle: string;
  capturePlaceholder: string;
  capture: string;
  cancel: string;

  // Create note dialog
  createNote: string;
  noteTitle: string;
  selectNotebook: string;
  selectTemplate: string;
  create: string;
  templateBlank: string;
  templateBlankDesc: string;
  templateCornell: string;
  templateCornellDesc: string;
  templateZettelkasten: string;
  templateZettelkastenDesc: string;
  templateFeedback: string;
  templateFeedbackDesc: string;
  preview: string;

  // Search
  searchName: string;
  savedSearches: string;
  filterLifecycle: string;
  filterNotebook: string;
  filterPinned: string;
  all: string;
  active: string;
  archived: string;
  pinned: string;

  // Attachments
  attachFiles: string;
  dropFiles: string;
  noAttachments: string;
  deleteAttachment: string;

  // AI
  aiSettings: string;
  suggestTags: string;
  summarize: string;
  classify: string;
  suggestLinks: string;
  thinking: string;
  apply: string;
  applyAll: string;

  // Links panel
  outgoingLinks: string;
  backlinks: string;
  noLinks: string;
  addLink: string;

  // Graph
  graphNodes: string;
  graphEdges: string;

  // Settings
  language: string;
  languageName: string;

  // Actions
  promote: string;
  delete: string;
  deleteBlock: string;
  moveUp: string;
  moveDown: string;
  newNote: string;

  // Extra templates
  templateDaily: string;
  templateDailyDesc: string;
  templateRetrospective: string;
  templateRetrospectiveDesc: string;
  templateMeeting: string;
  templateMeetingDesc: string;

  // Editor panels
  aiAssistant: string;
  provider: string;
  model: string;
  mode: string;
  localOnly: string;
  privateApi: string;
  applyTags: string;
  moveToNotebook: string;
  current: string;

  // Graph
  graphClickOpen: string;
  graphDragMove: string;
  noNotesToGraph: string;
  noNotesToGraphDesc: string;

  // Attachment
  uploading: string;

  // Search filters
  anyStatus: string;
  anyNotebook: string;
  anyTag: string;
  clear: string;
  filters: string;
  saveSearch: string;

  // Note counts
  noteCount: string;
  resultCount: string;

  // Editor hints
  editorHint: string;

  // Links panel
  forwardLinks: string;
  detectedWikiLinks: string;
  relatedNotes: string;
  wikiLinkHint: string;
}

const en: Messages = {
  appName: "ono",
  appTagline: "Local-first",

  quickCapture: "+ Quick Capture",
  workflow: "Workflow",
  inbox: "Inbox",
  allNotes: "All Notes",
  search: "Search",
  graph: "Graph",
  notebooks: "Notebooks",
  settings: "Settings",

  note: "Note",
  searchPlaceholder: "Search notes... (Ctrl+/)",

  inboxEmpty: "Inbox is empty",
  inboxEmptyDesc: "Capture a quick thought with Ctrl+Shift+N",
  notesEmpty: "No notes yet",
  notesEmptyDesc: "Create your first note to get started",
  searchEmpty: "Search your notes",
  searchEmptyDesc: "Enter a query to find notes",
  noResults: "No results found",

  untitled: "Untitled",
  save: "Save",
  saved: "Saved",
  saving: "Saving...",
  addBlock: "+ Add Block",
  attach: "+ Attach",
  ai: "AI",
  links: "Links",
  move: "Move",
  archive: "Archive",
  ctrlEnterSave: "Ctrl+Enter to save",

  blockText: "Text",
  blockHeading: "Heading",
  blockCode: "Code",
  blockQuote: "Quote",
  blockCue: "Cue",
  blockSummary: "Summary",
  blockAtomicIdea: "Atomic Idea",
  blockSource: "Source",
  blockExpected: "Expected",
  blockActual: "Actual",
  blockDeviation: "Deviation",
  blockCause: "Cause",
  blockAction: "Action",
  blockDivider: "Divider",

  captureTitle: "Quick Capture",
  capturePlaceholder: "Capture your thought...",
  capture: "Capture",
  cancel: "Cancel",

  createNote: "Create Note",
  noteTitle: "Note title",
  selectNotebook: "Select notebook",
  selectTemplate: "Select template",
  create: "Create",
  templateBlank: "Blank",
  templateBlankDesc: "Empty note with a text block",
  templateCornell: "Cornell",
  templateCornellDesc: "Cue + Notes + Summary",
  templateZettelkasten: "Zettelkasten",
  templateZettelkastenDesc: "Atomic idea + Source + Context",
  templateFeedback: "Feedback Analysis",
  templateFeedbackDesc: "Expected/Actual/Deviation/Cause/Action",
  preview: "Preview",

  searchName: "Search name",
  savedSearches: "Saved Searches",
  filterLifecycle: "Lifecycle",
  filterNotebook: "Notebook",
  filterPinned: "Pinned",
  all: "All",
  active: "Active",
  archived: "Archived",
  pinned: "Pinned",

  attachFiles: "Attach Files",
  dropFiles: "Drop files here or click to attach",
  noAttachments: "No attachments",
  deleteAttachment: "Delete attachment",

  aiSettings: "AI Settings",
  suggestTags: "Suggest Tags",
  summarize: "Summarize",
  classify: "Classify",
  suggestLinks: "Suggest Links",
  thinking: "Thinking...",
  apply: "Apply",
  applyAll: "Apply All",

  outgoingLinks: "Outgoing Links",
  backlinks: "Backlinks",
  noLinks: "No links",
  addLink: "Add Link",

  graphNodes: "nodes",
  graphEdges: "edges",

  language: "Language",
  languageName: "English",

  promote: "Promote",
  delete: "Delete",
  deleteBlock: "Delete block",
  moveUp: "Move up",
  moveDown: "Move down",
  newNote: "New Note",

  templateDaily: "Daily Log",
  templateDailyDesc: "Simple daily record with heading and free text",
  templateRetrospective: "Retrospective",
  templateRetrospectiveDesc: "Project or sprint retrospective template",
  templateMeeting: "Meeting Minutes",
  templateMeetingDesc: "Structured meeting notes with attendees and decisions",

  aiAssistant: "AI Assistant",
  provider: "Provider",
  model: "Model",
  mode: "Mode",
  localOnly: "Local Only",
  privateApi: "Private API",
  applyTags: "Apply Tags",
  moveToNotebook: "Move to Notebook",
  current: "current",

  graphClickOpen: "Click to open. Drag to move.",
  graphDragMove: "Drag to move",
  noNotesToGraph: "No notes to graph",
  noNotesToGraphDesc: "Create notes and links to see the relationship graph.",

  uploading: "Uploading...",

  anyStatus: "Any status",
  anyNotebook: "Any notebook",
  anyTag: "Any tag",
  clear: "Clear",
  filters: "Filters",
  saveSearch: "Save Search",

  noteCount: "note(s)",
  resultCount: "result(s)",

  editorHint: "Ctrl+Enter: new block | Ctrl+S: save | Alt+Arrow: reorder",

  forwardLinks: "Forward Links",
  detectedWikiLinks: "Detected Wiki-links",
  relatedNotes: "Related Notes",
  wikiLinkHint: "Type [[note title]] in any block to create links",
};

const zh: Messages = {
  appName: "ono",
  appTagline: "本地优先",

  quickCapture: "+ 快速捕获",
  workflow: "工作流",
  inbox: "收件箱",
  allNotes: "所有笔记",
  search: "搜索",
  graph: "图谱",
  notebooks: "笔记本",
  settings: "设置",

  note: "笔记",
  searchPlaceholder: "搜索笔记... (Ctrl+/)",

  inboxEmpty: "收件箱为空",
  inboxEmptyDesc: "按 Ctrl+Shift+N 快速捕获想法",
  notesEmpty: "暂无笔记",
  notesEmptyDesc: "创建你的第一篇笔记",
  searchEmpty: "搜索你的笔记",
  searchEmptyDesc: "输入关键词查找笔记",
  noResults: "未找到结果",

  untitled: "无标题",
  save: "保存",
  saved: "已保存",
  saving: "保存中...",
  addBlock: "+ 添加块",
  attach: "+ 附件",
  ai: "AI",
  links: "链接",
  move: "移动",
  archive: "归档",
  ctrlEnterSave: "Ctrl+Enter 保存",

  blockText: "文本",
  blockHeading: "标题",
  blockCode: "代码",
  blockQuote: "引用",
  blockCue: "线索",
  blockSummary: "总结",
  blockAtomicIdea: "原子想法",
  blockSource: "来源",
  blockExpected: "预期",
  blockActual: "实际",
  blockDeviation: "偏差",
  blockCause: "原因",
  blockAction: "行动",
  blockDivider: "分隔线",

  captureTitle: "快速捕获",
  capturePlaceholder: "记录你的想法...",
  capture: "捕获",
  cancel: "取消",

  createNote: "创建笔记",
  noteTitle: "笔记标题",
  selectNotebook: "选择笔记本",
  selectTemplate: "选择模板",
  create: "创建",
  templateBlank: "空白",
  templateBlankDesc: "包含一个文本块的空白笔记",
  templateCornell: "康奈尔笔记",
  templateCornellDesc: "线索 + 笔记 + 总结",
  templateZettelkasten: "卡片盒笔记",
  templateZettelkastenDesc: "原子想法 + 来源 + 上下文",
  templateFeedback: "反馈分析",
  templateFeedbackDesc: "预期/实际/偏差/原因/行动",
  preview: "预览",

  searchName: "搜索名称",
  savedSearches: "已保存的搜索",
  filterLifecycle: "生命周期",
  filterNotebook: "笔记本",
  filterPinned: "已固定",
  all: "全部",
  active: "活跃",
  archived: "已归档",
  pinned: "已固定",

  attachFiles: "添加附件",
  dropFiles: "拖放文件到此处或点击添加",
  noAttachments: "暂无附件",
  deleteAttachment: "删除附件",

  aiSettings: "AI 设置",
  suggestTags: "推荐标签",
  summarize: "生成摘要",
  classify: "自动分类",
  suggestLinks: "推荐链接",
  thinking: "思考中...",
  apply: "应用",
  applyAll: "全部应用",

  outgoingLinks: "出站链接",
  backlinks: "反向链接",
  noLinks: "暂无链接",
  addLink: "添加链接",

  graphNodes: "个节点",
  graphEdges: "条边",

  language: "语言",
  languageName: "中文",

  promote: "提升",
  delete: "删除",
  deleteBlock: "删除块",
  moveUp: "上移",
  moveDown: "下移",
  newNote: "新建笔记",

  templateDaily: "日志",
  templateDailyDesc: "包含标题和自由文本的日常记录",
  templateRetrospective: "回顾",
  templateRetrospectiveDesc: "项目或迭代回顾模板",
  templateMeeting: "会议记录",
  templateMeetingDesc: "包含参与者和决定的结构化会议笔记",

  aiAssistant: "AI 助手",
  provider: "提供者",
  model: "模型",
  mode: "模式",
  localOnly: "仅本地",
  privateApi: "私有 API",
  applyTags: "应用标签",
  moveToNotebook: "移动到笔记本",
  current: "当前",

  graphClickOpen: "点击打开。拖拽移动。",
  graphDragMove: "拖拽移动",
  noNotesToGraph: "暂无笔记可显示",
  noNotesToGraphDesc: "创建笔记和链接以查看关系图谱。",

  uploading: "上传中...",

  anyStatus: "任何状态",
  anyNotebook: "任何笔记本",
  anyTag: "任何标签",
  clear: "清除",
  filters: "筛选",
  saveSearch: "保存搜索",

  noteCount: "篇笔记",
  resultCount: "个结果",

  editorHint: "Ctrl+Enter: 新建块 | Ctrl+S: 保存 | Alt+方向键: 排序",

  forwardLinks: "正向链接",
  detectedWikiLinks: "检测到的 Wiki 链接",
  relatedNotes: "相关笔记",
  wikiLinkHint: "在任意块中输入 [[笔记标题]] 来创建链接",
};

export const messages: Record<Locale, Messages> = { en, zh };
