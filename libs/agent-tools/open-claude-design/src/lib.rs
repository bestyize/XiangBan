//! Local design authoring. No account, external CLI or model-specific dependency.
use agent_kernel::{HarnessError, Message, Middleware, Risk, Role, Tool, ToolCtx, ToolResult, ToolSpec, Turn, Stop, StopDecision, Nudge, FinishReason};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::BTreeMap, fs, path::{Component, Path, PathBuf}, sync::{Arc, Mutex}, time::{SystemTime, UNIX_EPOCH}};

pub const NAME: &str = "open-claude-design";
pub const RUNTIME: &str = concat!(include_str!("../runtime/support.js"),"\n",include_str!("../runtime/editor.js"));
pub mod archive;
pub const FORMAT: &str = include_str!("../FORMAT.md");
/// Quality guidance returned with the format: tokens first, real content, complete states, a check before complete.
pub const CRAFT: &str = include_str!("../CRAFT.md");
const MAX_FILE: usize = 2 * 1024 * 1024;
const MAX_PROJECT: usize = 16 * 1024 * 1024;
// Separate Store handles (UI commands and agent runs) share mutation ownership.
static STORAGE_LOCK: Mutex<()> = Mutex::new(());

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    #[serde(default)]
    pub status: ProjectStatus,
    pub id: String,
    pub title: String,
    pub session_id: String,
    #[serde(default)]
    pub session_ids: Vec<String>,
    pub revision: u64,
    pub updated_ms: u64,
    pub canvas: Value,
    pub files: BTreeMap<String, String>,
    #[serde(default)]
    pub request_id: String,
}

/// Older complete projects deserialize as ready; partial projects require an explicit state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus { Draft, Generating, #[default] Ready, Failed }

/// Static delivery checks. Runtime errors are additionally surfaced by each sandbox preview.
pub fn diagnostics(p: &Project) -> Vec<String> {
    let mut errors = Vec::new();
    let Ok(items) = boards(&p.canvas) else { return vec!["画板索引无效".into()] };
    if items.is_empty() { errors.push("至少需要一个画板".into()); }
    for (name, _) in items {
        match p.files.get(&name) {
            None => errors.push(format!("缺少画板源文件：{name}")),
            Some(source) if source.trim().is_empty() => errors.push(format!("画板源文件为空：{name}")),
            Some(_) => (),
        }
    }
    let refs = regex::Regex::new(r#"(?i)(?:href|src)\s*=\s*["']([^"']+)["']"#).unwrap();
    for (name, source) in &p.files {
        if !name.ends_with(".html") { continue; }
        for cap in refs.captures_iter(source) {
            let href = cap[1].split(['?', '#']).next().unwrap_or("");
            if href.is_empty() || href.contains(':') || href.starts_with('/') || href.contains("{{") { continue; }
            let Some(target) = archive::resolve_rel(name, href) else { continue };
            if target.ends_with("support.js") { continue; }
            if !p.files.contains_key(&target) { errors.push(format!("{name} 引用的文件不存在：{target}")); }
        }
    }
    errors.sort(); errors.dedup(); errors
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
    if project.session_ids.len()>64 || project.session_ids.iter().any(|id|id.is_empty()||id.len()>200||id.chars().any(char::is_control)){return Err("关联会话无效或超过 64 个".into());}
    let source_count=project.files.iter().filter(|(name,text)|!(((name.as_str()=="support.js"||name.ends_with("/support.js"))&&text.as_str()==RUNTIME)||name.as_str()=="_ds_manifest.json"||name.ends_with("/_ds_manifest.json"))).count();
    if source_count > 300 || project.files.len()>405 { return Err("设计稿文件过多".into()); }
    for (name, text) in &project.files {
        if !file_name(name) { return Err(format!("不支持的设计文件路径：{name}")); }
        if text.len() > MAX_FILE { return Err(format!("{name} 超过 2MiB")); }
    }
    boards(&project.canvas)?;
    if project.status == ProjectStatus::Ready {
        let issues = diagnostics(project);
        if !issues.is_empty() { return Err(format!("设计稿尚未完成：{}。使用 status=generating 分批保存，补齐后 complete。", issues.join("；"))); }
    }
    if serde_json::to_vec(project).map_err(|e| e.to_string())?.len() > MAX_PROJECT { return Err("设计稿超过 16MiB".into()); }
    Ok(())
}

/// Agent canvas updates are patches: omitted boards and unknown metadata survive.
fn merge_canvas(target:&mut Value,patch:&Value){
    fn merge(target:&mut Value,patch:&Value){
        if let (Some(old),Some(new))=(target.as_object_mut(),patch.as_object()){
            for(key,value)in new{if let Some(slot)=old.get_mut(key){merge(slot,value)}else{old.insert(key.clone(),value.clone());}}
        }else{*target=patch.clone();}
    }
    let previous=boards(target).unwrap_or_default();merge(target,patch);
    if target["boards"].is_object(){
        for(file,board)in previous{
            if target["boards"].get(&file).is_none(){target["boards"][&file]=board;}
            if let Some(order)=target["order"].as_array_mut(){if !order.iter().any(|v|v==&file){order.push(json!(file));}}
        }
    }else if let Some(items)=target["artboards"].as_array_mut(){
        for(file,mut board)in previous{
            if let Some(current)=items.iter_mut().find(|b|b["file"].as_str()==Some(file.as_str())){merge(&mut board,current);*current=board;}
            else{items.push(board);}
        }
    }
}

impl Store {
    pub fn new(root: impl Into<PathBuf>) -> Self { Self { root: root.into() } }
    pub fn dir(&self, id: &str) -> Result<PathBuf, String> {
        if !safe_id(id) { return Err("设计稿编号格式错误，请使用创建成功后返回的 id，不能使用 request_id".into()); }
        fs::create_dir_all(&self.root).map_err(|e| e.to_string())?;
        let root = self.root.canonicalize().map_err(|e| e.to_string())?;
        let dir = root.join(id);
        if fs::symlink_metadata(&dir).is_ok_and(|m| m.file_type().is_symlink()) { return Err("设计目录不能是符号链接".into()); }
        Ok(dir)
    }
    pub fn load(&self, id: &str) -> Result<Project, String> {
        let path = self.dir(id)?.join("project.json");
        if fs::symlink_metadata(&path).is_ok_and(|m| m.file_type().is_symlink() || m.len() > MAX_PROJECT as u64) { return Err("设计文件无效或过大".into()); }
        let project: Project = serde_json::from_slice(&fs::read(path).map_err(|e| if e.kind()==std::io::ErrorKind::NotFound { "设计稿不存在，可能已被删除".into() } else { format!("设计稿无法读取：{e}") })?).map_err(|e| format!("设计稿内容损坏：{e}"))?;
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
                let Ok(project) = self.load(&id) else { continue };
                if session == "*" || (project.session_id == session || project.session_ids.iter().any(|id|id==session)) { list.push(project); }
            }
        }
        list.sort_by_key(|p| std::cmp::Reverse(p.updated_ms));
        Ok(list)
    }
    pub fn search(&self, session: &str, query: &str, since: Option<u64>, before: Option<u64>) -> Result<Vec<Project>, String> {
        let query = query.trim().to_lowercase();
        Ok(self.list(session)?.into_iter().filter(|p| {
            (query.is_empty() || p.title.to_lowercase().contains(&query) || p.id==query || p.request_id.to_lowercase()==query)
            && since.is_none_or(|ms|p.updated_ms>=ms) && before.is_none_or(|ms|p.updated_ms<ms)
        }).collect())
    }
    pub fn link(&self, id: &str, session: &str) -> Result<Project,String> {
        if session.is_empty() || session=="*" { return Err("请选择有效会话".into()); }
        let mut p=self.load(id)?;
        if p.session_id!=session&&!p.session_ids.iter().any(|id|id==session) {
            p.session_ids.push(session.into()); let rev=p.revision; p=self.save(p,rev)?;
        }
        self.set_active(session,id)?;Ok(p)
    }
    pub fn copy(&self, id: &str, session: &str, title: Option<&str>) -> Result<Project,String> {
        let mut p=self.load(id)?; p.id=format!("design-{}",nonce()?); p.revision=0; p.updated_ms=0;
        p.session_id=session.into(); p.session_ids.clear(); p.request_id.clear();
        p.title=title.map(str::to_string).unwrap_or_else(||format!("{} 副本",p.title.chars().take(116).collect::<String>()));
        self.save(p,0)
    }
    /// Small UI metadata is separate from design revisions and cannot follow symlinks.
    pub fn read_view(&self,id:&str,name:&str)->Result<Option<Value>,String>{
        if !["view.json","window.json"].contains(&name){return Err("无效的视图记录".into())}
        self.load(id)?;let path=self.dir(id)?.join(name);
        if fs::symlink_metadata(&path).is_ok_and(|m|m.file_type().is_symlink()||m.len()>4096){return Err("视图记录无效".into())}
        Ok(fs::read(path).ok().and_then(|b|serde_json::from_slice(&b).ok()))
    }
    pub fn save_view(&self,id:&str,name:&str,value:&Value)->Result<(),String>{
        let _lock=STORAGE_LOCK.lock().map_err(|_|"设计存储锁不可用")?;
        if !["view.json","window.json"].contains(&name){return Err("无效的视图记录".into())}
        self.load(id)?;let bytes=serde_json::to_vec(value).map_err(|e|e.to_string())?;
        if bytes.len()>4096||!value.is_object(){return Err("视图记录无效".into())}
        let dir=self.dir(id)?;let temp=dir.join(format!("{}.tmp",nonce()?));
        fs::write(&temp,bytes).map_err(|e|e.to_string())?;fs::rename(&temp,dir.join(name)).map_err(|e|e.to_string())
    }
    pub fn active(&self,session:&str)->Option<String>{
        let path=self.root.join("active.json");
        if fs::symlink_metadata(&path).is_ok_and(|m|m.file_type().is_symlink()||m.len()>65536){return None}
        let map:BTreeMap<String,String>=serde_json::from_slice(&fs::read(path).ok()?).ok()?;
        let id=map.get(session)?;self.load(id).ok()?;Some(id.clone())
    }
    fn set_active(&self,session:&str,id:&str)->Result<(),String>{
        let _lock=STORAGE_LOCK.lock().map_err(|_|"设计存储锁不可用")?;
        if session.is_empty()||session.len()>200{return Err("会话无效".into())}self.load(id)?;
        let path=self.root.join("active.json");
        let mut map:BTreeMap<String,String>=if fs::symlink_metadata(&path).is_ok_and(|m|!m.file_type().is_symlink()&&m.len()<=65536){fs::read(&path).ok().and_then(|b|serde_json::from_slice(&b).ok()).unwrap_or_default()}else{BTreeMap::new()};
        while map.len()>=256{if let Some(key)=map.keys().find(|key|key.as_str()!=session).cloned(){map.remove(&key);}else{break}}
        map.insert(session.into(),id.into());let temp=self.root.join(format!("{}.tmp",nonce()?));
        fs::write(&temp,serde_json::to_vec(&map).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;fs::rename(temp,path).map_err(|e|e.to_string())
    }
    pub fn remove_checked(&self,id:&str,expected:u64)->Result<(),String>{
        let _lock=STORAGE_LOCK.lock().map_err(|_|"设计存储锁不可用")?;let p=self.load(id)?;
        if p.revision!=expected{return Err("设计稿已更新，重新读取后再删除".into())}fs::remove_dir_all(self.dir(id)?).map_err(|e|e.to_string())
    }
    /// Compare-and-swap the whole bundle; source, index and comments change together.
    pub fn save(&self, mut project: Project, expected: u64) -> Result<Project, String> {
        let _lock = STORAGE_LOCK.lock().map_err(|_| "设计存储锁不可用")?;
        let dir = self.dir(&project.id)?;
        let path = dir.join("project.json");
        let current = if path.exists() { Some(self.load(&project.id)?) } else { None };
        if current.as_ref().map_or(0, |p| p.revision) != expected { return Err("设计稿已更新，请重新读取后再修改，避免覆盖新改动".into()); }
        if let Some(old) = &current { project.session_id = old.session_id.clone(); for id in &old.session_ids { if !project.session_ids.contains(id){project.session_ids.push(id.clone());} } project.request_id = old.request_id.clone(); }
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
        // Bound history to 30 complete revisions. Never touch the current snapshot.
        let history = dir.join("history");
        if let Ok(entries) = fs::read_dir(&history) {
            let mut revisions: Vec<_> = entries.flatten().filter_map(|e| e.path().file_stem()?.to_str()?.parse::<u64>().ok().map(|n| (n,e.path()))).collect();
            revisions.sort_by_key(|(n,_)| *n);
            let remove = revisions.len().saturating_sub(30);
            for (_, path) in revisions.into_iter().take(remove) { let _ = fs::remove_file(path); }
        }
        Ok(project)
    }
    /// Failed UI edits remain recoverable without changing the canonical revision.
    pub fn save_draft(&self, project: &Project) -> Result<(), String> {
        let _lock=STORAGE_LOCK.lock().map_err(|_|"设计存储锁不可用")?;
        self.load(&project.id)?;let mut check=project.clone();check.status=ProjectStatus::Draft;validate(&check)?;
        let dir=self.dir(&project.id)?;let temp=dir.join(format!("{}.tmp",nonce()?));
        fs::write(&temp,serde_json::to_vec(project).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
        fs::rename(&temp,dir.join("draft.json")).map_err(|e|e.to_string())
    }
    pub fn draft(&self, id: &str) -> Result<Option<Project>, String> {
        self.load(id)?;let path=self.dir(id)?.join("draft.json");
        if !path.exists(){return Ok(None)}
        let meta=fs::symlink_metadata(&path).map_err(|e|e.to_string())?;
        if meta.file_type().is_symlink()||meta.len()>MAX_PROJECT as u64{return Err("恢复草稿无效或过大".into())}
        let p:Project=serde_json::from_slice(&fs::read(path).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
        if p.id!=id{return Err("草稿编号不匹配".into())}let mut check=p.clone();check.status=ProjectStatus::Draft;validate(&check)?;Ok(Some(p))
    }
    pub fn clear_draft(&self, expected: &Project) -> Result<(), String> {
        let _lock=STORAGE_LOCK.lock().map_err(|_|"设计存储锁不可用")?;
        if self.draft(&expected.id)?.as_ref()==Some(expected){fs::remove_file(self.dir(&expected.id)?.join("draft.json")).map_err(|e|e.to_string())?;}Ok(())
    }
    pub fn remove(&self, id: &str) -> Result<(), String> {
        let _lock = STORAGE_LOCK.lock().map_err(|_| "设计存储锁不可用")?;
        fs::remove_dir_all(self.dir(id)?).or_else(|e| if e.kind()==std::io::ErrorKind::NotFound { Ok(()) } else { Err(e) }).map_err(|e|e.to_string())
    }
    pub fn history(&self, id: &str) -> Result<Vec<Project>, String> {
        self.load(id)?;
        let mut out=Vec::new();
        if let Ok(entries)=fs::read_dir(self.dir(id)?.join("history")) {
            for e in entries.flatten() {
                if e.file_type().map_err(|e|e.to_string())?.is_file() && e.metadata().map_err(|e|e.to_string())?.len()<=MAX_PROJECT as u64 {
                    if let Ok(p)=serde_json::from_slice::<Project>(&fs::read(e.path()).map_err(|e|e.to_string())?) { if p.id==id { out.push(p); } }
                }
            }
        }
        out.sort_by_key(|p|std::cmp::Reverse(p.revision));Ok(out)
    }
    pub fn create(&self, title: String, session: &str, canvas: Value, files: BTreeMap<String, String>) -> Result<Project, String> {
        self.save(Project { status: ProjectStatus::Ready, id: format!("design-{}", nonce()?), title, session_id: session.into(), session_ids: Vec::new(), revision: 0, updated_ms: 0, canvas, files, request_id: String::new() }, 0)
    }
}

type Saved = dyn Fn(&Project) -> Result<(), String> + Send + Sync;
pub struct DesignTool { spec: ToolSpec, store: Store, saved: Arc<Saved>, removed:Arc<dyn Fn(&str)->Result<(),String>+Send+Sync> }
static CREATE_LOCK: Mutex<()> = Mutex::new(());
impl DesignTool {
    pub fn new(store: Store, saved: impl Fn(&Project) -> Result<(), String> + Send + Sync + 'static) -> Self {
        Self { spec: ToolSpec::new(NAME,
            "Create and edit multi-artboard design projects in My Documents. Call action=format first for DC HTML syntax and the quality guide (craft). create allocates a durable id even with missing boards (draft). save merges changed files and canvas fields, preserving omitted boards and metadata (remove_boards explicitly removes boards), with status=generating; complete validates all sources and references before marking ready. Never deliver an incomplete project. save/complete require expected_revision. read returns canvas, design system manifest and optional file contents; manifest returns compiled @dsCard cards and tokens; list defaults to all projects; scope=session restricts it. query searches title, exact id or request_id; since_ms/before_ms filter updated time (epoch milliseconds). open links the same project; copy creates an independent project. Does not publish or install anything.",
            json!({"type":"object","properties":{
                "action":{"type":"string","enum":["format","list","read","manifest","create","save","complete","open","copy","delete"]},
                "scope":{"type":"string","enum":["all","session"]},"query":{"type":"string"},"since_ms":{"type":"integer","minimum":0},"before_ms":{"type":"integer","minimum":0},"status":{"type":"string","enum":["draft","generating","ready","failed"]},"id":{"type":"string"},"title":{"type":"string"},"expected_revision":{"type":"integer","minimum":1},
                "remove_boards":{"type":"array","items":{"type":"string"}},"canvas":{"type":"object"},"files":{"type":"object","additionalProperties":{"type":"string"}},"file":{"type":"string"},"request_id":{"type":"string"}
            },"required":["action"],"additionalProperties":false})).risk(Risk::Write).max_output(64_000), store, saved: Arc::new(saved),removed:Arc::new(|_|Ok(())) }
    }
    pub fn on_remove(mut self,removed:impl Fn(&str)->Result<(),String>+Send+Sync+'static)->Self{self.removed=Arc::new(removed);self}
    fn call(&self, args: Value, ctx: &ToolCtx) -> Result<Value, String> {
        if ctx.is_cancelled() { return Err("已取消".into()); }
        let get_id = || args["id"].as_str().ok_or("缺少 id");
        match args["action"].as_str().ok_or("缺少 action")? {
            "format" => Ok(json!({"format": FORMAT, "craft": CRAFT})),
            "list" => Ok(json!(self.store.search(if args["scope"]=="session"{ctx.session()}else{"*"},args["query"].as_str().unwrap_or(""),args["since_ms"].as_u64(),args["before_ms"].as_u64())?.iter().take(100).map(summary).collect::<Vec<_>>())),
            "copy" => {
                let key=args["request_id"].as_str().filter(|k|!k.is_empty()&&k.len()<=120).ok_or("copy 需要稳定 request_id，重试时沿用")?;
                let _lock=CREATE_LOCK.lock().map_err(|_|"创建锁不可用")?;
                let p=if let Some(p)=self.store.list(ctx.session())?.into_iter().find(|p|p.request_id==key){p}else{
                    let mut p=self.store.load(get_id()?)?;p.id=format!("design-{}",nonce()?);p.revision=0;p.updated_ms=0;
                    p.title=args["title"].as_str().map(str::to_string).unwrap_or_else(||format!("{} 副本",p.title.chars().take(116).collect::<String>()));
                    p.session_id=ctx.session().into();p.session_ids.clear();p.request_id=key.into();self.store.save(p,0)?
                };
                (self.saved)(&p)?;Ok(json!({"project":summary(&p)}))
            }
            "delete" => {
                let id=get_id()?;self.store.remove_checked(id,args["expected_revision"].as_u64().ok_or("删除必须传入最新 expected_revision")?)?;
                (self.removed)(id)?;Ok(json!({"deleted":id}))
            }
            "manifest" => {
                let p = self.store.load(get_id()?)?;
                Ok(archive::ds_manifest(&p))
            }
            "read" | "open" => {
                let p = self.store.link(get_id()?,ctx.session())?;
                // Opening a library project in a new conversation links the same source, never a copy.
                (self.saved)(&p)?;
                let file = args["file"].as_str().map(|name| p.files.get(name).map(|text| json!({"name":name,"text":text})).ok_or("设计文件不存在")).transpose()?;
                Ok(json!({"project":summary(&p),"canvas":p.canvas,"files":p.files.keys().collect::<Vec<_>>(),"file":file,"manifest":archive::ds_manifest(&p)}))
            }
            action @ ("create" | "save" | "complete") => {
                let files: BTreeMap<String,String> = serde_json::from_value(args.get("files").cloned().unwrap_or(json!({}))).map_err(|_| "files 需要是文件名到内容的映射")?;
                let p = if action == "create" {
                    let key=args["request_id"].as_str().filter(|k| !k.is_empty() && k.len()<=120).ok_or("create 需要稳定的 request_id，重试时沿用")?;
                    let _lock=CREATE_LOCK.lock().map_err(|_|"创建锁不可用")?;
                    if let Some(existing)=self.store.list(ctx.session())?.into_iter().find(|p|p.request_id==key) { existing } else {
                        let p=Project{status:ProjectStatus::Draft,id:format!("design-{}",nonce()?),title:args["title"].as_str().ok_or("缺少 title")?.into(),session_id:ctx.session().into(),session_ids:Vec::new(),revision:0,updated_ms:0,canvas:args["canvas"].clone(),files,request_id:key.into()};
                        self.store.save(p,0)?
                    }
                } else {
                    let mut p = self.store.load(get_id()?)?;
                    if let Some(title) = args["title"].as_str() { p.title = title.into(); }
                    if let Some(canvas) = args.get("canvas") { merge_canvas(&mut p.canvas,canvas); }
                    if let Some(remove)=args["remove_boards"].as_array(){for name in remove{let name=name.as_str().ok_or("remove_boards 必须是文件名数组")?;if let Some(map)=p.canvas["boards"].as_object_mut(){map.remove(name);}if let Some(order)=p.canvas["order"].as_array_mut(){order.retain(|v|v!=name);}if let Some(items)=p.canvas["artboards"].as_array_mut(){items.retain(|b|b["file"]!=name);}}}
                    p.files.extend(files);
                    p.status = if action == "complete" { ProjectStatus::Ready }
                        else if let Some(status) = args.get("status") { serde_json::from_value(status.clone()).map_err(|_|"无效的设计稿状态")? }
                        else if p.status == ProjectStatus::Ready { ProjectStatus::Ready } else { ProjectStatus::Generating };
                    self.store.save(p, args["expected_revision"].as_u64().ok_or("必须传入读取到的 expected_revision")?)?
                };
                self.store.set_active(ctx.session(),&p.id)?;
                (self.saved)(&p).map_err(|e| format!("设计 {} 已保存，但资料库索引更新失败：{e}。先 read 核对，不要重复创建。", p.id))?;
                Ok(json!({"project":summary(&p),"issues":diagnostics(&p),"next":if p.status==ProjectStatus::Ready {"已完成"}else{"使用返回的 id 和 revision 分批 save 缺失或错误的源文件；最后 complete，不要再次 create"},"location":"资料库 → 我的文档"}))
            }
            _ => Err("不支持的设计操作".into())
        }
    }
}
pub fn summary(p: &Project) -> Value { json!({"id":p.id,"title":p.title,"sessionId":p.session_id,"sessionIds":p.session_ids,"revision":p.revision,"updatedMs":p.updated_ms,"status":p.status,"issues":diagnostics(p),"artboards":boards(&p.canvas).map(|b|b.len()).unwrap_or(0)}) }
impl Tool for DesignTool {
    fn spec(&self) -> &ToolSpec { &self.spec }
    fn resource(&self, args: &Value) -> Result<Option<agent_kernel::ToolResource>, String> {
        // Store already enforces revision checks; its writes must not reserve unrelated user files.
        Ok(if args["action"] == "format" { None } else { Some(agent_kernel::ToolResource::Named("design-store".into())) })
    }
    fn execute_result(&self, args: Value, ctx: &ToolCtx) -> ToolResult {
        match self.call(args, ctx) {
            Ok(value) => ToolResult::ok(value.to_string()).with_structured_content(value),
            Err(e) => ToolResult::error(e.clone()).with_structured_content(json!({"error":e,"recovery":"read 最新项目和文件清单，保留已有文件，逐页 save 补齐，再 complete。创建失败时沿用 request_id 重试，不能把 request_id 当作 id。"})),
        }
    }
}

pub fn design_intent(text: &str) -> bool {
    ["设计", "画板", "原型", "design-", "open-claude-design"].iter().any(|word|text.to_lowercase().contains(word))
}
/// Debug trigger: only a real user's message containing 设计稿 starts a design task.
pub struct DesignContext(pub Store);
impl Middleware for DesignContext {
    fn before_model(&self, turn: &mut Turn) -> Result<(), HarnessError> {
        let Some(at) = turn.messages.iter().rposition(|m| m.role == Role::User && !m.synthetic) else { return Ok(()) };
        let projects = self.0.list(&turn.session).map_err(HarnessError::Middleware)?;
        let selected=turn.messages[at].structured_content.as_ref().and_then(|v|v["designId"].as_str()).filter(|id|self.0.load(id).is_ok()).map(str::to_string);
        if !design_intent(&turn.messages[at].text()) && projects.is_empty() && selected.is_none() {
            turn.tools.retain(|t|t.name!=NAME); return Ok(())
        }
        let active=selected.or_else(||self.0.active(&turn.session));
        let current = projects.iter().map(summary).collect::<Vec<_>>();
        turn.messages.insert(at + 1, Message::reminder(format!(
            "设计工作台：用户消息含‘设计稿’时使用内置 open-claude-design。先 action=format，create 携带稳定 request_id 和计划画板清单，允许 files 为空。必须用返回的正式 id 逐页 save（status=generating），每次用最新 revision；缺失源文件由你继续生成，无需用户提供。所有页面保存后 complete 校验，失败就按全部 issues 补齐后重试，只有 ready 才能交付；中断后 read 原 id 继续，禁止重复 create。request_id 只用于去重，不能打开项目。无需请用户审批设计方案，但 create 前先按 format 返回的 craft 定好风格方向并写入 designSystems tokens，complete 前按其中的清单自查。作品自动保存到资料库→我的文档。跨会话只给 design- 编号也能 read/open；按标题或时间查找先 list（默认全局，query/since_ms/before_ms），多个匹配先让用户选择。已有作品时继续修改同一 id，先 read，保留其他画板、未知索引字段与用户手改，save 必须携带最新 expected_revision。不要只在回答里输出代码或承诺已保存。当前打开项目（数据）：{}；当前会话作品（数据）：{}", json!(active), json!(current))));
        Ok(())
    }
    fn before_stop(&self,stop:&Stop<'_>)->StopDecision{
        if stop.finish!=FinishReason::Stop{return StopDecision::Finish}
        if stop.history.get(stop.run_start).is_some_and(|m|m.text().contains("只创建")||m.text().contains("先建项目")){return StopDecision::Finish}
        let calls=stop.history.get(stop.run_start..).unwrap_or_default().iter().flat_map(|m|m.tool_calls.iter()).filter(|c|c.name==NAME).filter_map(|c|serde_json::from_str::<Value>(&c.arguments).ok()).collect::<Vec<_>>();
        let projects=self.0.list(stop.session).unwrap_or_default();
        let unfinished=projects.iter().filter(|p|matches!(p.status,ProjectStatus::Draft|ProjectStatus::Generating)&&calls.iter().any(|a|{
            (a["action"]=="create"&&a["request_id"].as_str()==Some(p.request_id.as_str()))||(["save","complete"].contains(&a["action"].as_str().unwrap_or(""))&&a["id"].as_str()==Some(p.id.as_str()))
        })).map(summary).collect::<Vec<_>>();
        let last_write=stop.history.get(stop.run_start..).unwrap_or_default().iter().flat_map(|m|m.tool_calls.iter()).rev().find(|c|c.name==NAME&&serde_json::from_str::<Value>(&c.arguments).is_ok_and(|a|["create","save","complete"].contains(&a["action"].as_str().unwrap_or(""))));
        let write_failed=last_write.is_some_and(|c|stop.history.iter().any(|m|m.tool_call_id.as_deref()==Some(c.id.as_str())&&m.is_error));
        if unfinished.is_empty()&&!write_failed{return StopDecision::Finish}
        if stop.continuations>=4||stop.continuations_without_progress>=2{return StopDecision::Pause("design_incomplete_saved".into())}
        StopDecision::Continue(Nudge{reason:"design_incomplete".into(),message:format!("本轮写入的设计稿还未完成，草稿已保存：{}。先 read 最新版本，逐页 save 补齐全部 issues，再 complete；只有 ready 才能交付。若用户只要求草稿、或确实受阻，save status=failed 保留现场并如实说明。",json!(unfinished))})
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Store, PathBuf) { let root = std::env::temp_dir().join(format!("ocd-{}",nonce().unwrap())); (Store::new(&root),root) }
    fn canvas() -> Value { json!({"v":3,"boards":{"Main.dc.html":{"x":0,"y":0,"w":390,"h":844}},"order":["Main.dc.html"],"notes":{},"unknown":{"keep":true}}) }
    fn files() -> BTreeMap<String,String> { [("Main.dc.html".into(),"<x-dc><h1>Hello</h1></x-dc>".into())].into() }
    #[test] fn format_comes_with_the_quality_guide() {
        let (store,_root)=fixture();let tool=DesignTool::new(store,|_|Ok(()));
        let out=tool.call(json!({"action":"format"}),&ToolCtx::detached("s")).unwrap();
        assert_eq!(out["format"],FORMAT);assert!(out["craft"].as_str().unwrap().contains("complete 之前自查"));
    }
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
    #[test] fn retries_reuse_project_and_save_preserves_manual_source() {
        let (store,root)=fixture();let tool=DesignTool::new(store.clone(), |_|Ok(()));let ctx=ToolCtx::detached("s");
        let args=json!({"action":"create","title":"demo","request_id":"stable-1","canvas":canvas(),"files":files()});
        let a=tool.call(args.clone(),&ctx).unwrap();let b=tool.call(args,&ctx).unwrap();assert_eq!(a["project"]["id"],b["project"]["id"]);
        let id=a["project"]["id"].as_str().unwrap();let mut manual=store.load(id).unwrap();manual.files.insert("extra.css".into(),"/* user */".into());let manual=store.save(manual,1).unwrap();
        assert!(tool.call(json!({"action":"save","id":id,"expected_revision":1,"files":{}}),&ctx).is_err());
        tool.call(json!({"action":"save","id":id,"expected_revision":manual.revision,"files":{"Main.dc.html":"AI edit"}}),&ctx).unwrap();
        assert_eq!(store.load(id).unwrap().files["extra.css"],"/* user */");assert_eq!(store.list("s").unwrap().len(),1);fs::remove_dir_all(root).unwrap();
    }
    #[test] fn history_is_bounded_and_deleted_projects_cannot_be_resurrected() {
        let (store,root)=fixture();let mut p=store.create("demo".into(),"s",canvas(),files()).unwrap();
        for _ in 0..34 {let rev=p.revision;p=store.save(p,rev).unwrap();}
        let h=store.history(&p.id).unwrap();assert_eq!(h.len(),30);assert_eq!(h[0].revision,34);assert_eq!(h[29].revision,5);
        store.remove(&p.id).unwrap();assert!(store.save(p,35).is_err());fs::remove_dir_all(root).unwrap();
    }
    #[test] fn one_corrupt_project_does_not_hide_other_documents() {
        let (store,root)=fixture();let p=store.create("demo".into(),"s",canvas(),files()).unwrap();
        let bad=root.join("design-000000000000000000000000");fs::create_dir_all(&bad).unwrap();fs::write(bad.join("project.json"),"broken").unwrap();
        assert_eq!(store.list("*").unwrap().len(),1);assert_eq!(store.list("s").unwrap()[0].id,p.id);fs::remove_dir_all(root).unwrap();
    }

    #[test] fn draft_recovery_does_not_change_saved_revision_or_clear_newer_edits() {
        let (store,root)=fixture();let p=store.create("demo".into(),"s",canvas(),files()).unwrap();
        let mut first=p.clone();first.title="unsaved".into();store.save_draft(&first).unwrap();assert_eq!(Store::new(&root).draft(&p.id).unwrap(),Some(first.clone()));assert_eq!(store.load(&p.id).unwrap().title,"demo");
        let mut newer=first.clone();newer.title="newer draft".into();store.save_draft(&newer).unwrap();store.clear_draft(&first).unwrap();assert_eq!(store.draft(&p.id).unwrap(),Some(newer.clone()));
        store.clear_draft(&newer).unwrap();assert!(store.draft(&p.id).unwrap().is_none());fs::remove_dir_all(root).unwrap();
    }

    #[test] fn library_project_continues_in_a_new_conversation_without_duplication() {
        let (store,root)=fixture();let p=store.create("demo".into(),"deleted-session",canvas(),files()).unwrap();let tool=DesignTool::new(store.clone(),|_|Ok(()));
        let read=tool.call(json!({"action":"read","id":p.id}),&ToolCtx::detached("new-session")).unwrap();assert_eq!(read["project"]["revision"],2);
        assert_eq!(store.list("new-session").unwrap()[0].id,p.id);assert_eq!(store.list("*").unwrap().len(),1);
        let latest=store.load(&p.id).unwrap();assert_eq!(latest.files,p.files);assert_eq!(latest.canvas,p.canvas);
        tool.call(json!({"action":"read","id":p.id}),&ToolCtx::detached("new-session")).unwrap();assert_eq!(store.load(&p.id).unwrap().revision,2);
        let mut stale=p;stale.revision=2;let saved=store.save(stale,2).unwrap();assert!(saved.session_ids.contains(&"new-session".into()));fs::remove_dir_all(root).unwrap();
    }

    #[test] fn progressive_creation_survives_restart_and_only_completes_when_all_sources_exist() {
        let (store,root)=fixture(); let tool=DesignTool::new(store.clone(),|_|Ok(())); let ctx=ToolCtx::detached("s");
        let mut c=canvas(); c["boards"]["More.dc.html"]=json!({"w":390,"h":844}); c["order"].as_array_mut().unwrap().push(json!("More.dc.html"));
        let args=json!({"action":"create","request_id":"five-pages","title":"demo","canvas":c,"files":{"base.css":"body{}"}});
        let created=tool.call(args.clone(),&ctx).unwrap(); let id=created["project"]["id"].as_str().unwrap();
        assert_eq!(created["project"]["status"],"draft"); assert_eq!(created["issues"].as_array().unwrap().len(),2);
        assert_eq!(tool.call(args,&ctx).unwrap()["project"]["id"],id);
        assert!(tool.call(json!({"action":"complete","id":id,"expected_revision":1}),&ctx).unwrap_err().contains("More.dc.html"));
        tool.call(json!({"action":"save","id":id,"expected_revision":1,"files":files()}),&ctx).unwrap();
        let reopened=Store::new(&root).load(id).unwrap(); assert_eq!(reopened.status,ProjectStatus::Generating); assert_eq!(reopened.files["base.css"],"body{}");
        tool.call(json!({"action":"save","id":id,"expected_revision":2,"files":{"More.dc.html":"<x-dc>more</x-dc>"}}),&ctx).unwrap();
        let ready=tool.call(json!({"action":"complete","id":id,"expected_revision":3}),&ctx).unwrap(); assert_eq!(ready["project"]["status"],"ready");
        assert_eq!(store.history(id).unwrap().len(),3); fs::remove_dir_all(root).unwrap();
    }
    #[test] fn legacy_status_and_missing_link_diagnostics() {
        let (store,root)=fixture(); let p=store.create("demo".into(),"s",canvas(),files()).unwrap();
        let mut value=serde_json::to_value(&p).unwrap(); value.as_object_mut().unwrap().remove("status");
        assert_eq!(serde_json::from_value::<Project>(value).unwrap().status,ProjectStatus::Ready);
        let mut broken=p; broken.files.insert("Main.dc.html".into(),r#"<x-dc><link href="base.css"><a href="More.dc.html">next</a></x-dc>"#.into());
        assert_eq!(diagnostics(&broken).len(),2); assert!(validate(&broken).is_err());
        broken.status=ProjectStatus::Failed; assert!(validate(&broken).is_ok()); fs::remove_dir_all(root).unwrap();
    }

    #[test] fn global_search_finds_titles_request_ids_and_times_and_exact_id_triggers() {
        let (store,root)=fixture();let p=store.create("B站手机设计".into(),"old",canvas(),files()).unwrap();
        assert!(design_intent(&format!("看看 {}",p.id)));assert!(!design_intent("你好"));
        assert_eq!(store.search("*","B站",Some(p.updated_ms),None).unwrap().len(),1);
        assert!(store.search("new","B站",None,None).unwrap().is_empty());
        assert!(store.search("*","",None,Some(p.updated_ms)).unwrap().is_empty());
        let tool=DesignTool::new(store.clone(),|_|Ok(()));let ctx=ToolCtx::detached("new");
        assert_eq!(tool.call(json!({"action":"list","query":p.id}),&ctx).unwrap()[0]["id"],p.id);
        tool.call(json!({"action":"open","id":p.id}),&ctx).unwrap();assert_eq!(store.list("new").unwrap().len(),1);
        let copied=tool.call(json!({"action":"copy","id":p.id,"request_id":"copy-once"}),&ctx).unwrap();
        assert_ne!(copied["project"]["id"],p.id);
        assert_eq!(tool.call(json!({"action":"copy","id":p.id,"request_id":"copy-once"}),&ctx).unwrap(),copied);
        let copy=store.load(copied["project"]["id"].as_str().unwrap()).unwrap();assert_eq!(copy.files,p.files);assert!(copy.session_ids.is_empty());
        fs::remove_dir_all(root).unwrap();
    }

    #[test] fn incomplete_delivery_continues_only_a_project_written_in_this_run_and_is_bounded(){
        let(store,root)=fixture();let tool=DesignTool::new(store.clone(),|_|Ok(()));let ctx=ToolCtx::detached("s");
        let args=json!({"action":"create","title":"draft","request_id":"unfinished","canvas":canvas()});
        let result=tool.call(args.clone(),&ctx).unwrap();let id=result["project"]["id"].as_str().unwrap();
        let history=vec![Message::user_text("设计一个应用"),Message::assistant("",vec![agent_kernel::ToolCall{id:"call".into(),name:NAME.into(),arguments:args.to_string()}]),Message::assistant("完成了",vec![])];
        let mid=DesignContext(store.clone());let mut stop=Stop{session:"s",history:&history,run_start:0,finish:FinishReason::Stop,continuations:0,continuations_without_progress:0};
        assert!(matches!(mid.before_stop(&stop),StopDecision::Continue(_)));stop.continuations=4;assert!(matches!(mid.before_stop(&stop),StopDecision::Pause(_)));
        let inspect=vec![Message::user_text("看看设计稿")];stop.history=&inspect;stop.continuations=0;assert_eq!(mid.before_stop(&stop),StopDecision::Finish);
        tool.call(json!({"action":"save","id":id,"expected_revision":1,"status":"failed"}),&ctx).unwrap();stop.history=&history;assert_eq!(mid.before_stop(&stop),StopDecision::Finish);fs::remove_dir_all(root).unwrap();
    }
    #[test] fn view_metadata_and_active_projects_survive_without_changing_revisions(){
        let(store,root)=fixture();let first=store.create("first".into(),"old",canvas(),files()).unwrap();let second=store.copy(&first.id,"old",Some("second")).unwrap();
        store.link(&first.id,"new").unwrap();store.link(&second.id,"new").unwrap();assert_eq!(Store::new(&root).active("new").as_deref(),Some(second.id.as_str()));
        let rev=store.load(&first.id).unwrap().revision;store.save_view(&first.id,"view.json",&json!({"scale":1.5,"left":321})).unwrap();assert_eq!(store.load(&first.id).unwrap().revision,rev);assert_eq!(store.read_view(&first.id,"view.json").unwrap().unwrap()["left"],321);assert!(store.save_view(&first.id,"../private",&json!({})).is_err());
        let mut message=Message::user_text("首页更紧凑");message.structured_content=Some(json!({"designId":first.id}));let mut turn=Turn{session:"new".into(),index:0,messages:vec![message],tools:vec![]};DesignContext(store.clone()).before_model(&mut turn).unwrap();
        assert!(turn.messages[1].text().contains(&format!("当前打开项目（数据）：\"{}\"",first.id)));
        assert!(store.remove_checked(&first.id,rev-1).is_err());store.remove_checked(&first.id,rev).unwrap();assert_eq!(store.list("*").unwrap().len(),1);assert_eq!(store.load(&second.id).unwrap().files,second.files);fs::remove_dir_all(root).unwrap();
    }

    #[test] fn canvas_patch_keeps_other_boards_and_unknown_fields(){
        let mut c=canvas();c["boards"]["More.dc.html"]=json!({"w":200,"h":300,"unknown":"keep"});c["order"].as_array_mut().unwrap().push(json!("More.dc.html"));
        merge_canvas(&mut c,&json!({"boards":{"Main.dc.html":{"title":"changed"}},"order":["Main.dc.html"]}));
        assert_eq!(boards(&c).unwrap().len(),2);assert_eq!(c["boards"]["Main.dc.html"]["w"],390);assert_eq!(c["unknown"]["keep"],true);assert_eq!(c["boards"]["More.dc.html"]["unknown"],"keep");
    }

    #[test] fn an_invalid_edit_remains_recoverable_without_weakening_canonical_validation(){
        let(store,root)=fixture();let saved=store.create("demo".into(),"s",canvas(),files()).unwrap();let mut draft=saved.clone();draft.files.clear();
        assert!(validate(&draft).is_err());store.save_draft(&draft).unwrap();assert_eq!(store.draft(&saved.id).unwrap(),Some(draft));assert_eq!(store.load(&saved.id).unwrap().files,saved.files);fs::remove_dir_all(root).unwrap();
    }

}
