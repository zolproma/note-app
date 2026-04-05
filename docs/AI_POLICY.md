# AI Policy

This document defines the hard constraints for AI behavior in the Notes application. These are system invariants, not user preferences.

## Default Behavior

| Setting                    | Default       |
| -------------------------- | ------------- |
| AI enabled                 | OFF           |
| AI write access            | READ-ONLY     |
| Full-library index         | DISABLED      |
| Attachment upload           | DISABLED      |
| Remote model calls         | BLOCKED       |
| Silent fallback to remote  | FORBIDDEN     |

## Operating Modes

### Local Only
- Only local model inference
- Only local embeddings
- No remote calls of any kind
- No telemetry, crash reporting, or remote URL fetching
- If a capability is missing, fail — do not fall back

### Private API
- Only whitelisted commercial providers
- Must display: provider, model, region, retention policy, training policy
- Must declare zero-retention status
- Must declare abuse logging / subprocessor presence
- Requires explicit user confirmation before activation

### Blocked Remote
- All remote AI disabled
- All remote embedding disabled
- All external model URL requests blocked

## Access Scope

AI can only access content explicitly granted by the user:

| Scope Type        | Example                           |
| ----------------- | --------------------------------- |
| Single note       | User selects one note             |
| Selected blocks   | User highlights specific blocks   |
| Notebook          | User grants a notebook            |
| Tag set           | User grants notes with tag X      |
| Search results    | User checks items from results    |

### Forbidden

- Scanning the entire library without authorization
- Reading notes tagged `sensitive`, `no_ai`, `no_remote`
- Reading private attachments
- Transferring one agent's context to another without approval
- Expanding task scope without confirmation

## Write-Back Rules

1. AI can only produce suggestions (diffs)
2. User must preview the diff before any write
3. User must confirm before data is committed
4. All write-backs enter the audit log
5. Multi-agent write-backs must preserve source agent and approval chain

## Embedding Rules

- Embeddings are sensitive derived data
- No cross-workspace reuse
- Real-time ACL check at retrieval time
- Deleting a note triggers embedding deletion
- Changing permissions triggers rebuild or invalidation
- Embeddings cannot bypass source note permissions

## Attachment Security

- Attachments stored locally by default
- EXIF metadata stripped by default
- OCR, transcription, PDF text extraction require separate authorization
- Thumbnails and transcoders run in restricted context
- External embeds do not auto-load
- Remote preview only in permitted modes

## Audit Log

Every AI operation records:

| Field              | Description                                  |
| ------------------ | -------------------------------------------- |
| job_id             | Unique job identifier                        |
| mode               | local_only / private_api / blocked_remote    |
| scope_snapshot     | What content was authorized                  |
| policy_snapshot    | Active policy at time of execution           |
| notes_accessed     | List of note IDs actually read               |
| network_targets    | External endpoints contacted                 |
| diff_summary       | What changes were proposed                   |
| approval_state     | pending / approved / rejected                |
| created_at         | Timestamp                                    |

Audit log properties:
- Append-only or tamper-evident
- Survives content deletion (governance retention)
- User can inspect exactly what AI read and did
