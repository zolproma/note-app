# Document Model

## Overview

The document model is the contract between storage, editing, CLI, AI, and export layers. It must be defined and frozen before implementing any of these layers.

## Core Entities

### Note
The top-level container.

| Field            | Type              | Description                          |
| ---------------- | ----------------- | ------------------------------------ |
| id               | UUID              | Primary key                          |
| notebook_id      | UUID              | Parent notebook                      |
| title            | String            | Display title                        |
| template_id      | UUID?             | Template used to create              |
| lifecycle        | Enum              | inbox / active / archived / trashed  |
| visibility       | Enum              | normal / sensitive / private         |
| ai_policy        | Enum              | allowed / no_ai / no_remote          |
| pinned           | Boolean           | Pinned to top                        |
| created_at       | DateTime          | Creation time                        |
| updated_at       | DateTime          | Last modification                    |

### Block
The structural unit within a note. Notes are composed of ordered blocks.

| Field            | Type              | Description                          |
| ---------------- | ----------------- | ------------------------------------ |
| id               | UUID              | Stable block ID (for refs)           |
| note_id          | UUID              | Parent note                          |
| block_type       | Enum              | See block types below                |
| content          | String            | Block content (text/markdown)        |
| sort_order       | Integer           | Display order                        |
| metadata         | JSON?             | Type-specific metadata               |
| created_at       | DateTime          | Creation time                        |
| updated_at       | DateTime          | Last modification                    |

### Block Types

| Type                  | Usage                                      |
| --------------------- | ------------------------------------------ |
| text                  | General paragraph                          |
| heading               | Section heading                            |
| code                  | Code block                                 |
| quote                 | Quotation                                  |
| list                  | List (ordered/unordered)                   |
| image                 | Inline image reference                     |
| attachment            | File attachment reference                  |
| embed                 | External embed (video, etc.)               |
| divider               | Visual separator                           |
| cornell_cue           | Cornell note: cue column                   |
| cornell_summary       | Cornell note: summary section              |
| zettel_atom           | Zettelkasten: atomic idea                  |
| zettel_source         | Zettelkasten: source reference             |
| feedback_expected     | Feedback analysis: expected result         |
| feedback_actual       | Feedback analysis: actual result           |
| feedback_deviation    | Feedback analysis: deviation               |
| feedback_cause        | Feedback analysis: cause analysis          |
| feedback_action       | Feedback analysis: corrective action       |

### Tag
| Field            | Type              | Description                          |
| ---------------- | ----------------- | ------------------------------------ |
| id               | UUID              | Primary key                          |
| workspace_id     | UUID              | Parent workspace                     |
| name             | String            | Tag name (unique per workspace)      |
| color            | String?           | Display color                        |

### Link
| Field            | Type              | Description                          |
| ---------------- | ----------------- | ------------------------------------ |
| id               | UUID              | Primary key                          |
| source_note_id   | UUID              | Where the link originates            |
| target_note_id   | UUID              | Where the link points                |
| source_block_id  | UUID?             | Optional block-level source          |
| target_block_id  | UUID?             | Optional block-level target          |
| link_type        | Enum              | wiki_link / block_ref / related      |

### Alias
| Field            | Type              | Description                          |
| ---------------- | ----------------- | ------------------------------------ |
| id               | UUID              | Primary key                          |
| note_id          | UUID              | The note this is an alias for        |
| alias_text       | String            | Alternative name                     |

## Serialization

| Layer       | Format                                                  |
| ----------- | ------------------------------------------------------- |
| Storage     | Structured JSON blocks in SQLite                        |
| Export      | Markdown                                                |
| Search      | FTS5 indexes title, content, tags, aliases separately   |
| AI diff     | Block-level structured diff                             |

## Template Contract

Templates produce a pre-defined set of blocks when a note is created. Templates do not add runtime behavior — they are structural scaffolding only.

| Template           | Blocks Generated                                           |
| ------------------ | ---------------------------------------------------------- |
| Blank              | [text]                                                     |
| Cornell            | [cornell_cue, text, cornell_summary]                       |
| Zettelkasten       | [zettel_atom, zettel_source, text]                         |
| Feedback Analysis  | [expected, actual, deviation, cause, action]               |
| Daily Log          | [heading("Today"), text]                                   |
| Retrospective      | [heading+text] x 3 (went well, improve, actions)          |
| Meeting Minutes    | [heading+text] x 4 (attendees, agenda, decisions, actions) |
| Quick Capture      | [text]                                                     |
