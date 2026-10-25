# Agent Learning (stage II)

Independent Rust library for local SQLite + FTS5 learning metadata. Does not execute models, tools, or Tauri commands.

- Persistent schema version 1 with migrations (SQLite bundled through rusqlite)
- Conservatively indexed task outcomes, candidate/approved/rejected experiences, FTS5 retrieval
- Skill proposals and deterministic evaluation case/result records
- Raw conversation and RunEvent remain in existing Markdown / JSONL files
- `LearningDb::open` uses WAL and a busy timeout; bundled SQLite requires no external server
- Local SQLite is **not encrypted**. Never auto-send its contents to an API.

Desktop file: user data `chat/agent-learning.sqlite`.
