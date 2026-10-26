//! Local design authoring. No account, external CLI or model-specific dependency.
use agent_kernel::{HarnessError, Message, Middleware, Risk, Role, Tool, ToolCtx, ToolResult, ToolSpec, Turn};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::BTreeMap, fs, path::{Component, Path, PathBuf}, sync::{Arc, Mutex}, time::{SystemTime, UNIX_EPOCH}};

pub const NAME: &str = "open-claude-design";
pub const RUNTIME: &str = include_str!("../runtime/support.js");
pub const FORMAT: &str = include_str!("../FORMAT.md");
const MAX_FILE: usize = 2 * 1024 * 1024;
const MAX_PROJECT: usize = 16 * 1024 * 1024;
// Separate Store handles (UI commands and agent runs) share mutation ownership.
static STORAGE_LOCK: Mutex<()> = Mutex::new(());

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub title: String,
    pub session_id: String,
    pub revision: u64,
    pub updated_ms: u64,
    pub canvas: Value,
    pub files: BTreeMap<String, String>,
}

#[derive(Clone)]
pub struct Store { root: PathBuf }

fn now() -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64 }
fn nonce() -> Result<String, String> {
    let mut bytes = [0; 12];
    getrandom::fill(&mut bytes).map_err(|e| e.to_string())?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}
fn safe_id(id: &str) -> bool { id.starts_with("design-") && id.len() == 31 && id[7..].bytes().all(|b| b.is_ascii_hexdigit()) }
fn file_name(name: &str) -> bool {
    !name.is_empty() && name.len() <= 240 && !name.contains(['\\', '\0', ':']) &&
    !name.starts_with('/') && name.split('/').all(|s| !s.is_empty() && s != "." && s != "..") &&
    Path::new(name).components().all(|c| matches!(c, Component::Normal(_))) &&
    [".dc.html", ".html", ".css", ".js", ".json", ".svg"].iter().any(|ext| name.ends_with(ext))
}

/// Normalize either observed canvas index without dropping source metadata.
pub fn boards(canvas: &Value) -> Result<Vec<(String, Value)>, String> {
    let mut out = Vec::new();
    if let Some(map) = canvas.get("boards").and_then(Value::as_object) {
        if canvas["v"] != 3 { return Err("boards 索引只支持 v:3".into()); }
        let order = canvas["order"].as_array().ok_or("v3 索引缺少 order")?;
        for key in order {
            let name = key.as_str().ok_or("order 必须是文件名数组")?;
            if out.iter().any(|(old, _)| old == name) { return Err("画板顺序中存在重复文件".into()); }
            out.push((name.to_string(), map.get(name).ok_or("order 引用了不存在的画板")?.clone()));
        }
        if out.len() != map.len() { return Err("每个画板必须在 order 中出现一次".into()); }
    } else if let Some(items) = canvas["artboards"].as_array() {
        for board in items {
            let name = board["file"].as_str().ok_or("画板缺少 file")?;
            if out.iter().any(|(old, _)| old == name) { return Err("画板文件重复".into()); }
            out.push((name.to_string(), board.clone()));
        }
    } else { return Err("需要 v3 boards/order 或旧版 artboards 索引".into()); }
    if out.len() > 100 { return Err("一份设计稿最多 100 个画板".into()); }
    for (name, board) in &out {
        if !file_name(name) || !name.ends_with(".dc.html") { return Err("画板文件必须是安全的相对 .dc.html 路径".into()); }
        for key in ["w", "h"] {
            let n = board[key].as_f64().ok_or("画板需要 w、h 尺寸")?;
            if !(1.0..=16000.0).contains(&n) { return Err("画板尺寸需要在 1 到 16000 之间".into()); }
        }
        for key in ["x", "y"] {
            if board.get(key).is_some_and(|v| v.as_f64().is_none_or(|n| n.abs() > 1_000_000.0)) { return Err("画板坐标无效".into()); }
        }
    }
    Ok(out)
}
pub fn validate(project: &Project) -> Result<(), String> {
    if project.title.trim().is_empty() || project.title.chars().count() > 120 { return Err("标题需要 1 到 120 个字".into()); }
    if project.files.len() > 300 { return Err("设计稿文件过多".into()); }
    for (name, text) in &project.files {
        if !file_name(name) { return Err(format!("不支持的设计文件路径：{name}")); }
        if text.len() > MAX_FILE { return Err(format!("{name} 超过 2MiB")); }
    }
    for (name, _) in boards(&project.canvas)? {
        if !project.files.contains_key(&name) { return Err(format!("缺少画板源文件：{name}")); }
    }
    if serde_json::to_vec(project).map_err(|e| e.to_string())?.len() > MAX_PROJECT { return Err("设计稿超过 16MiB".into()); }
    Ok(())
}

impl Store {
    pub fn new(root: impl Into<PathBuf>) -> Self { Self { root: root.into() } }
    fn dir(&self, id: &str) -> Result<PathBuf, String> {
        if !safe_id(id) { return Err("设计稿编号无效".into()); }
        fs::create_dir_all(&self.root).map_err(|e| e.to_string())?;
        let root = self.root.canonicalize().map_err(|e| e.to_string())?;
        let dir = root.join(id);
        if fs::symlink_metadata(&dir).is_ok_and(|m| m.file_type().is_symlink()) { return Err("设计目录不能是符号链接".into()); }
        Ok(dir)
    }
    pub fn load(&self, id: &str) -> Result<Project, String> {
        let path = self.dir(id)?.join("project.json");
        if fs::symlink_metadata(&path).is_ok_and(|m| m.file_type().is_symlink() || m.len() > MAX_PROJECT as u64) { return Err("设计文件无效或过大".into()); }
        let project: Project = serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        if project.id != id { return Err("设计稿编号不匹配".into()); }
        validate(&project)?;
        Ok(project)
    }
    pub fn list(&self, session: &str) -> Result<Vec<Project>, String> {
        if !self.root.exists() { return Ok(Vec::new()); }
        let mut list = Vec::new();
        for entry in fs::read_dir(&self.root).map_err(|e| e.to_string())?.flatten() {
            let id = entry.file_name().to_string_lossy().to_string();
            if safe_id(&id) {
                let project = self.load(&id)?;
                if project.session_id == session { list.push(project); }
            }
        }
        list.sort_by_key(|p| std::cmp::Reverse(p.updated_ms));
        Ok(list)
    }
    /// Compare-and-swap the whole bundle; source, index and comments change together.
    pub fn save(&self, mut project: Project, expected: u64) -> Result<Project, String> {
        let _lock = STORAGE_LOCK.lock().map_err(|_| "设计存储锁不可用")?;
        let dir = self.dir(&project.id)?;
        let path = dir.join("project.json");
        let current = if path.exists() { Some(self.load(&project.id)?) } else { None };
        if current.as_ref().map_or(0, |p| p.revision) != expected { return Err("设计稿已更新，请重新读取后再修改，避免覆盖新改动".into()); }
        if let Some(old) = &current { project.session_id = old.session_id.clone(); }
        project.revision = expected.checked_add(1).ok_or("版本号超出范围")?;
        project.updated_ms = now();
        validate(&project)?;
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let temp = dir.join(format!("{}.tmp", nonce()?));
        fs::write(&temp, serde_json::to_vec_pretty(&project).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        // Keep the previous complete revision for recovery; never overwrite it with a partial write.
        if let Some(old) = current {
            fs::create_dir_all(dir.join("history")).map_err(|e| e.to_string())?;
            fs::write(dir.join("history").join(format!("{}.json", old.revision)), serde_json::to_vec(&old).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        }
        if let Err(e) = fs::rename(&temp, path) { let _ = fs::remove_file(temp); return Err(e.to_string()); }
        Ok(project)
    }
    pub fn create(&self, title: String, session: &str, canvas: Value, files: BTreeMap<String, String>) -> Result<Project, String> {
        self.save(Project { id: format!("design-{}", nonce()?), title, session_id: session.into(), revision: 0, updated_ms: 0, canvas, files }, 0)
    }
}

type Saved = dyn Fn(&Project) -> Result<(), String> + Send + Sync;
pub struct DesignTool { spec: ToolSpec, store: Store, saved: Arc<Saved> }
impl DesignTool {
    pub fn new(store: Store, saved: impl Fn(&Project) -> Result<(), String> + Send + Sync + 'static) -> Self {
        Self { spec: ToolSpec::new(NAME,
            "Create and edit multi-artboard design projects in My Documents. Call action=format first for DC HTML syntax. create/save write a complete project atomically; save requires expected_revision. read returns canvas and optional file contents; list is scoped to this conversation. Does not publish or install anything.",
            json!({"type":"object","properties":{
                "action":{"type":"string","enum":["format","list","read","create","save"]},
                "id":{"type":"string"},"title":{"type":"string"},"expected_revision":{"type":"integer","minimum":1},
                "canvas":{"type":"object"},"files":{"type":"object","additionalProperties":{"type":"string"}},"file":{"type":"string"}
            },"required":["action"],"additionalProperties":false})).risk(Risk::Write).max_output(64_000), store, saved: Arc::new(saved) }
    }
    fn call(&self, args: Value, ctx: &ToolCtx) -> Result<Value, String> {
        if ctx.is_cancelled() { return Err("已取消".into()); }
        let get_id = || args["id"].as_str().ok_or("缺少 id");
        match args["action"].as_str().ok_or("缺少 action")? {
            "format" => Ok(json!({"format": FORMAT})),
            "list" => Ok(json!(self.store.list(ctx.session())?.iter().map(summary).collect::<Vec<_>>())),
            "read" => {
                let p = self.store.load(get_id()?)?;
                let file = args["file"].as_str().map(|name| p.files.get(name).map(|text| json!({"name":name,"text":text})).ok_or("设计文件不存在")).transpose()?;
                Ok(json!({"project":summary(&p),"canvas":p.canvas,"files":p.files.keys().collect::<Vec<_>>(),"file":file}))
            }
            action @ ("create" | "save") => {
                let files: BTreeMap<String,String> = serde_json::from_value(args["files"].clone()).map_err(|_| "files 需要是文件名到内容的映射")?;
                let p = if action == "create" {
                    self.store.create(args["title"].as_str().ok_or("缺少 title")?.into(), ctx.session(), args["canvas"].clone(), files)?
                } else {
                    let mut p = self.store.load(get_id()?)?;
                    if let Some(title) = args["title"].as_str() { p.title = title.into(); }
                    if let Some(canvas) = args.get("canvas") { p.canvas = canvas.clone(); }
                    p.files.extend(files);
                    self.store.save(p, args["expected_revision"].as_u64().ok_or("必须传入读取到的 expected_revision")?)?
                };
                (self.saved)(&p).map_err(|e| format!("设计 {} 已保存，但资料库索引更新失败：{e}。先 read 核对，不要重复创建。", p.id))?;
                Ok(json!({"project":summary(&p),"location":"资料库 → 我的文档"}))
            }
            _ => Err("不支持的设计操作".into())
        }
    }
}
pub fn summary(p: &Project) -> Value { json!({"id":p.id,"title":p.title,"sessionId":p.session_id,"revision":p.revision,"updatedMs":p.updated_ms,"artboards":boards(&p.canvas).map(|b|b.len()).unwrap_or(0)}) }
impl Tool for DesignTool {
    fn spec(&self) -> &ToolSpec { &self.spec }
    fn execute_result(&self, args: Value, ctx: &ToolCtx) -> ToolResult {
        match self.call(args, ctx) {
            Ok(value) => ToolResult::ok(value.to_string()).with_structured_content(value),
            Err(e) => ToolResult::error(e),
        }
    }
}

/// Debug trigger: only a real user's message containing 设计稿 starts a design task.
pub struct DesignContext(pub Store);
impl Middleware for DesignContext {
    fn before_model(&self, turn: &mut Turn) -> Result<(), HarnessError> {
        let Some(at) = turn.messages.iter().rposition(|m| m.role == Role::User && !m.synthetic) else { return Ok(()) };
        let projects = self.0.list(&turn.session).map_err(HarnessError::Middleware)?;
        if !turn.messages[at].text().contains("设计稿") && projects.is_empty() { return Ok(()) }
        let current = projects.iter().map(summary).collect::<Vec<_>>();
        turn.messages.insert(at + 1, Message::reminder(format!(
            "设计工作台：用户消息含‘设计稿’时使用内置 open-claude-design。先 action=format，再按需求创建多画板作品；无需额外审批设计方案。作品自动保存到资料库→我的文档。已有作品时继续修改同一 id，先 read，保留其他画板、未知索引字段与用户手改，save 必须携带最新 expected_revision。不要只在回答里输出代码或承诺已保存。当前会话作品（数据）：{}", json!(current))));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Store, PathBuf) { let root = std::env::temp_dir().join(format!("ocd-{}",nonce().unwrap())); (Store::new(&root),root) }
    fn canvas() -> Value { json!({"v":3,"boards":{"Main.dc.html":{"x":0,"y":0,"w":390,"h":844}},"order":["Main.dc.html"],"notes":{},"unknown":{"keep":true}}) }
    fn files() -> BTreeMap<String,String> { [("Main.dc.html".into(),"<x-dc><h1>Hello</h1></x-dc>".into())].into() }
    #[test] fn revisions_are_atomic_and_survive_reopen() {
        let (store,root)=fixture(); let a=store.create("设计稿".into(),"s",canvas(),files()).unwrap();
        let mut b=a.clone(); b.files.insert("Main.dc.html".into(),"user edit".into());
        let b=store.save(b,1).unwrap(); assert_eq!(b.revision,2);
        assert!(store.save(a.clone(),1).unwrap_err().contains("已更新"));
        let c=Store::new(&root).load(&a.id).unwrap(); assert_eq!(c.files["Main.dc.html"],"user edit");
        assert_eq!(c.canvas["unknown"]["keep"],true); assert!(store.list("other").unwrap().is_empty());
        assert!(root.join(&a.id).join("history/1.json").exists()); fs::remove_dir_all(root).unwrap();
    }
    #[test] fn accepts_both_observed_indexes_and_rejects_broken_files() {
        assert_eq!(boards(&canvas()).unwrap().len(),1);
        assert_eq!(boards(&json!({"artboards":[{"file":"Main.dc.html","w":100,"h":200}],"pages":[{"id":"p"}]})).unwrap().len(),1);
        let (store,root)=fixture(); assert!(store.create("x".into(),"s",canvas(),BTreeMap::new()).is_err());
        assert!(!file_name("../bad.dc.html")); assert!(!file_name("a\\bad.dc.html"));
        assert!(store.load("../../private").is_err()); let _=fs::remove_dir_all(root);
    }
    #[test] fn trigger_ignores_tool_output_and_keeps_existing_context() {
        let (store,root)=fixture(); let mid=DesignContext(store.clone());
        let mut t=Turn{session:"s".into(),index:0,messages:vec![Message::user_text("你好"),Message::tool("t","设计稿")],tools:vec![]};
        mid.before_model(&mut t).unwrap(); assert_eq!(t.messages.len(),2);
        t.messages=vec![Message::user_text("做个设计稿")];mid.before_model(&mut t).unwrap();assert!(t.messages[1].synthetic);
        store.create("x".into(),"s",canvas(),files()).unwrap(); t.messages=vec![Message::user_text("颜色改浅")];mid.before_model(&mut t).unwrap();assert!(t.messages[1].synthetic);
        fs::remove_dir_all(root).unwrap();
    }
}
