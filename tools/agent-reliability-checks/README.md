# Agent reliability checks

Runs the desktop's real Broker, compaction, run history, checkpoint, Git snapshot and task outcome modules, including their unit tests, without Tauri or a platform WebView. No copied implementation or production model credentials are used. Git tests use temporary directories and need `git` on PATH.

From the repository root:

```sh
cargo test --manifest-path tools/agent-reliability-checks/Cargo.toml
```

This checks the platform-independent modules; it does not replace Windows/macOS desktop builds or native permission tests. No workflow is triggered automatically.
