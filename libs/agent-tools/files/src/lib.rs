//! Product-independent file capabilities. The kernel has no dependency on this crate.
mod documents;
/// Text of a PDF or Office document, for hosts that show or index documents outside a tool call.
pub use documents::extract as extract_document_text;
mod storage;
use agent_kernel::{
    builtin::fs::fs_tools, HarnessError, Part, Risk, Tool, ToolCtx, ToolResult, ToolSpec,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};
pub use storage::{
    Attachment, AttachmentContext, AttachmentHistory, AttachmentInput, AttachmentStore,
    MAX_ATTACHMENTS, MAX_FILE_BYTES, MAX_TOTAL_BYTES,
};

pub const CSV_SUMMARY_SCRIPT: &str = include_str!("../scripts/csv_summary.py");
pub const ANALYSIS_SKILL: &str = include_str!("../skills/file-analysis/SKILL.md");
type FileAuthorizer = dyn Fn(&Path) -> Result<bool, String> + Send + Sync;
/// Replaces only read_file; directory/search/write tools remain the host's own tools.
pub struct ReadFile {
    spec: ToolSpec,
    roots: Vec<PathBuf>,
    attachments: AttachmentStore,
    file_authorizer: Option<Arc<FileAuthorizer>>,
}
impl ReadFile {
    pub fn new(roots: Vec<PathBuf>, attachments: AttachmentStore) -> Result<Self, HarnessError> {
        // Validate host roots using the same implementation as the kernel's text reader.
        fs_tools(roots.clone())?;
        Ok(Self {
            spec: ToolSpec::new("read_file", "Read a file. Text, PDF, DOCX, PPTX and XLSX return numbered extracted lines; offset/limit page through them. PNG/JPEG/GIF/WebP return visual content. Extraction is not OCR or spreadsheet calculation; use file-analysis skill and shell for complex work.",
                json!({"type":"object","properties":{"path":{"type":"string"},"offset":{"type":"integer","minimum":1},"limit":{"type":"integer","minimum":1,"maximum":500}},"required":["path"]}))
                .risk(Risk::Read).untrusted_output().max_output(48_000),
            roots, attachments, file_authorizer: None,
        })
    }
    /// Authorize an individual canonical file outside the workspace, e.g. a hash-verified installed skill reference.
    /// This grants no directory access to other tools and is evaluated again on every read.
    pub fn with_file_authorizer(mut self, authorize: impl Fn(&Path) -> Result<bool, String> + Send + Sync + 'static) -> Self {
        self.file_authorizer = Some(Arc::new(authorize));
        self
    }
    fn read(&self, args: Value, ctx: &ToolCtx) -> Result<ToolResult, String> {
        if ctx.is_cancelled() {
            return Err("cancelled".into());
        }
        let requested = args["path"].as_str().ok_or("path is required")?;
        let mut roots = self.roots.clone();
        if let Some(root) = self.attachments.existing_session_dir(ctx.session()) {
            roots.push(root);
        }
        let joined = if Path::new(requested).is_absolute() {
            PathBuf::from(requested)
        } else {
            roots[0].join(requested)
        };
        let path = joined
            .canonicalize()
            .map_err(|e| format!("file unavailable: {e}"))?;
        if !roots
            .iter()
            .filter_map(|p| p.canonicalize().ok())
            .any(|r| path.starts_with(r))
        {
            if !self.file_authorizer.as_ref().map(|authorize| authorize(&path)).transpose()?.unwrap_or(false) {
                return Err("file is outside the authorized workspace and this session's attachments".into());
            }
            // This request reads exactly the authorized canonical file. Never retain its parent as a shared root.
            roots.push(path.parent().ok_or("file has no parent")?.to_path_buf());
        }
        let meta = fs::metadata(&path).map_err(|e| e.to_string())?;
        if !meta.is_file() {
            return Err("not a regular file".into());
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let mut header = Vec::new();
        fs::File::open(&path)
            .map_err(|e| e.to_string())?
            .take(12)
            .read_to_end(&mut header)
            .map_err(|e| e.to_string())?;
        if let Some(mime) = storage::image_mime(&header) {
            if meta.len() > MAX_FILE_BYTES as u64 {
                return Err("image exceeds 10 MiB; resize before viewing".into());
            }
            let bytes = fs::read(&path).map_err(|e| e.to_string())?;
            let original = format!("data:{mime};base64,{}", STANDARD.encode(bytes));
            let prepared = match agent_kernel::image_payload::prepare_data_url(&original) {
                Ok(Some(prepared)) => prepared,
                Ok(None) => original,
                Err(_) if meta.len() <= 1024 * 1024 => original,
                Err(_) => return Err("could not decode large image; please resize it before viewing".into()),
            };
            return Ok(ToolResult::ok(
                "Image attached for visual inspection; requires a vision-capable model.",
            )
            .with_part(Part::Image {
                url: prepared,
            }));
        }
        if matches!(ext.as_str(), "pdf" | "docx" | "pptx" | "xlsx") {
            if meta.len() > MAX_FILE_BYTES as u64 {
                return Err("file exceeds the 10 MiB analysis limit".into());
            }
            let text = documents::extract(&path, &ext, ctx)?;
            return page(&text, &args, &path, ctx);
        }
        // Reuse proven text reading and path semantics; do not maintain a second text reader.
        let reader = fs_tools(roots)
            .map_err(|e| e.to_string())?
            .into_iter()
            .find(|t| t.spec().name == "read_file")
            .ok_or("text reader unavailable")?;
        let mut args = args;
        args["path"] = json!(path);
        Ok(reader.execute_result(args, ctx))
    }
}
impl Tool for ReadFile {
    fn spec(&self) -> &ToolSpec {
        &self.spec
    }
    fn execute_result(&self, args: Value, ctx: &ToolCtx) -> ToolResult {
        self.read(args, ctx).unwrap_or_else(ToolResult::error)
    }
}
fn page(text: &str, args: &Value, path: &Path, ctx: &ToolCtx) -> Result<ToolResult, String> {
    let offset = args
        .get("offset")
        .map(|v| v.as_u64().ok_or("offset must be an integer"))
        .transpose()?
        .unwrap_or(1)
        .max(1) as usize;
    let limit = args
        .get("limit")
        .map(|v| v.as_u64().ok_or("limit must be an integer"))
        .transpose()?
        .unwrap_or(200)
        .clamp(1, 500) as usize;
    let lines: Vec<_> = text.lines().collect();
    let mut out = String::new();
    let mut next = offset;
    for (i, line) in lines.iter().enumerate().skip(offset - 1).take(limit) {
        if ctx.is_cancelled() {
            return Err("cancelled".into());
        }
        // Never silently lose long spreadsheet rows or paragraphs.
        if line.len() > 20_000 {
            return Err("an extracted line exceeds the output budget; use the analysis skill to read a smaller range".into());
        }
        if out.len() + line.len() + 32 > 36_000 {
            break;
        }
        out.push_str(&format!("{}\t{}\n", i + 1, line));
        next = i + 2;
    }
    let more = next <= lines.len();
    if more {
        out.push_str(&format!(
            "[partial extraction view; continue with offset={next}]"
        ));
    }
    Ok(ToolResult::ok(out).with_structured_content(json!({"source":path.display().to_string(),"extracted":true,"total_lines":lines.len(),"offset":offset,"next_offset":if more {Some(next)} else {None},"coverage":"text only; embedded images, layout, formulas and scanned pages may be omitted"})))
}
