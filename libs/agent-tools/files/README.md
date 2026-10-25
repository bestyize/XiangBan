# agent-file-tools

Optional file capabilities for agent-kernel hosts. The dependency goes from this package to the kernel; no kernel source or default features are changed.

The model receives one extended `read_file` tool rather than a tool per file type:
- Text/code/CSV/logs: delegate to the kernel's existing bounded text reader.
- PNG/JPEG/GIF/WebP: return `Part::Image` for a vision-capable model.
- PDF: extract the text layer; scanned pages and illustrations may be missing. No OCR.
- DOCX/PPTX: extract body/slide text with source markers; no visual/layout fidelity.
- XLSX: extract cached cell values with sheet/row/column locations. No formula recalculation or macro execution.

`AttachmentStore` imports files into per-session copies, validates format/size, generates random names and rolls back failed batches. `ReadFile` permits the host's declared roots and only the current ToolCtx session's attachment directory. The existing shell remains a powerful host capability, not an attachment sandbox.

Hosts can use `with_file_authorizer` to authorize individual canonical files, such as verified installed skill references. The callback runs on each out-of-workspace read; errors remain tool errors. This does not grant persistent directory access or change write/search permissions.

`AttachmentHistory<S>` records attachment cards using existing Message.structured_content. `AttachmentContext` injects a request-only manifest before ContextTrim, including the current newly appended message. No new kernel message type, Provider-specific file API, or sentinel parsing is introduced.

The bundled file-analysis Skill explains partial extraction, calculations, dependency checks and source citations. Its CSV helper uses only Python's standard library. Python is optional for complex scripted operations; basic document reading uses the native Rust libraries above.

## Host integration

1. Create an AttachmentStore beneath host-managed user data.
2. Import selected bytes and retain Attachment metadata for the pending turn.
3. Wrap the SessionStore in AttachmentHistory and register AttachmentContext before ContextTrim.
4. Replace the existing read_file tool with this crate's ReadFile; keep directory, grep and shell tools.
5. Install ANALYSIS_SKILL and CSV_SUMMARY_SCRIPT in a read-only Skills root.
6. Render attachment metadata in chat history. Keep inline images compatible with older histories.
7. Reject deleting an active session, remove its attachments with its conversation, and roll back unused imports.

Files default to 10 MiB each, images to 3 MiB, 8 attachments / 30 MiB per message. Office archives are limited to 64 MiB declared expanded content and 10,000 members; extracted text is capped at 8 MiB. Reader outputs are paginated and bounded.

## Verification

```sh
cargo test --manifest-path libs/agent-tools/files/Cargo.toml --locked
python3 -m unittest discover -s libs/agent-tools/files/tests -p 'test_*.py'
```

## Limits

Extraction currently reprocesses a document for each read; there is no persistent parsing cache. Native parser calls are in-process and cancellation is checked around the library call, not inside it; the kernel's cooperative timeout does not hard-kill a busy parser. A separately supervised worker is needed for stronger resource isolation.

This package does not install Python or Python dependencies, render PDF pages, perform OCR, or publish generated files as download cards. Pure-text models cannot understand returned image parts. Hosts must select an appropriate model; automatic model fallback is not implemented.

