//! Bounded source archives. No extraction to arbitrary filesystem paths.
use super::*;
use std::io::{Cursor, Read, Write};
use regex::Regex;

pub(crate) fn resolve_rel(base: &str, href: &str) -> Option<String> {
    let clean = href.trim();
    if clean.is_empty() || clean.starts_with('/') || clean.contains(':') || clean.contains(['?', '#']) { return None; }
    let mut parts: Vec<&str> = base.split('/').collect();
    parts.pop();
    for seg in clean.split('/') {
        match seg {
            "" | "." => {}
            ".." => { parts.pop()?; }
            other => parts.push(other),
        }
    }
    Some(parts.join("/"))
}

pub fn collect_tokens(canvas: &Value) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    if let Some(systems) = canvas.get("designSystems").and_then(Value::as_array) {
        for sys in systems {
            if let Some(tokens) = sys.get("tokens").and_then(Value::as_object) {
                for (k, v) in tokens {
                    if let Some(s) = v.as_str() { out.insert(k.clone(), s.to_string()); }
                    else if v.is_number() || v.is_boolean() { out.insert(k.clone(), v.to_string()); }
                }
            }
        }
    }
    if let Some(tokens) = canvas.get("tokens").and_then(Value::as_object) {
        for (k, v) in tokens {
            if let Some(s) = v.as_str() { out.insert(k.clone(), s.to_string()); }
        }
    }
    out
}

/// Parse `<!-- @dsCard group="…" name="…" subtitle="…" width="…" height="…" -->` header comments
/// and compile `_ds_manifest.json` compatible with Claude Design System cards.
pub fn ds_manifest(project: &Project) -> Value {
    let header_re = Regex::new(r#"(?s)^\s*<!--\s*@dsCard\b(.*?)-->"#).unwrap();
    let attr_re = Regex::new(r#"([A-Za-z0-9_-]+)\s*=\s*["']([^"']*)["']"#).unwrap();
    let mut cards = Vec::new();
    for (path, source) in &project.files {
        if !(path.ends_with(".dc.html") || path.ends_with(".html")) { continue; }
        let Some(cap) = header_re.captures(source) else { continue };
        let mut attrs = BTreeMap::new();
        for m in attr_re.captures_iter(&cap[1]) {
            attrs.insert(m[1].to_ascii_lowercase(), m[2].to_string());
        }
        let stem = Path::new(path).file_stem().and_then(|s| s.to_str()).unwrap_or(path).trim_end_matches(".dc");
        let name = attrs.remove("name").filter(|s| !s.is_empty()).unwrap_or_else(|| stem.to_string());
        let group = attrs.remove("group").filter(|s| !s.is_empty()).unwrap_or_else(|| "Components".into());
        let subtitle = attrs.remove("subtitle").unwrap_or_default();
        let width = attrs.get("width").and_then(|w| w.parse::<u32>().ok()).unwrap_or(360);
        let height = attrs.get("height").and_then(|h| h.parse::<u32>().ok()).unwrap_or(220);
        cards.push(json!({
            "name": name,
            "path": path,
            "group": group,
            "subtitle": subtitle,
            "viewport": { "width": width, "height": height }
        }));
    }
    json!({
        "v": 1,
        "title": project.title,
        "tokens": collect_tokens(&project.canvas),
        "cards": cards
    })
}

pub fn preview(source: &str) -> String {
    let support = Regex::new(r#"(?is)<script\b[^>]*\bsrc\s*=\s*["'][^"']*support\.js["'][^>]*>\s*</script\s*>"#).unwrap();
    let source = support.replace_all(source, "");
    let runtime = format!("<script>{RUNTIME}</script>");
    let head = Regex::new("(?i)<head(?:\\s[^>]*)?>").unwrap();
    if let Some(m) = head.find(&source) { format!("{}{}{}", &source[..m.end()], runtime, &source[m.end()..]) }
    else { format!("{runtime}{source}") }
}

pub fn preview_in_project(project: &Project, file: &str) -> Result<String, String> {
    let raw = project.files.get(file).ok_or("设计文件不存在")?;
    let link_re = Regex::new(r#"(?is)<link\b[^>]*rel\s*=\s*["']stylesheet["'][^>]*href\s*=\s*["']([^"']+)["'][^>]*/?>|<link\b[^>]*href\s*=\s*["']([^"']+)["'][^>]*rel\s*=\s*["']stylesheet["'][^>]*/?>"#).unwrap();
    let inlined = link_re.replace_all(raw, |caps: &regex::Captures| {
        let href = caps.get(1).or_else(|| caps.get(2)).map(|m| m.as_str()).unwrap_or("");
        if let Some(target) = resolve_rel(file, href).or_else(|| Some(href.trim_start_matches("./").to_string())) {
            if let Some(css) = project.files.get(&target) {
                return format!("<style data-ocd-inline-css=\"{}\">{}</style>", target.replace('"', "&quot;"), css.replace("</style", "<\\/style"));
            }
        }
        caps[0].to_string()
    });
    let tokens = collect_tokens(&project.canvas);
    let token_css = if tokens.is_empty() {
        String::new()
    } else {
        let vars = tokens.iter().map(|(k, v)| {
            let safe_k: String = k.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_').collect();
            let safe_v = v.replace([';', '<', '>', '{', '}'], "");
            format!("--ds-{safe_k}:{safe_v};")
        }).collect::<String>();
        format!("<style data-ocd-tokens>:root{{{vars}}}</style>")
    };
    let imports: BTreeMap<&String, &String> = project.files.iter()
        .filter(|(k, _)| k.as_str() != file && !k.ends_with("support.js") && k.as_str() != "_ds_manifest.json")
        .collect();
    let bundle_json = json!({
        "base": file,
        "files": imports,
        "tokens": tokens,
    }).to_string().replace('<', "\\u003c");
    let bundle_tag = format!("{token_css}<script id=\"__ocd_bundle__\" type=\"application/json\">{bundle_json}</script>");
    let rendered = preview(&inlined);
    let head = Regex::new("(?i)<head(?:\\s[^>]*)?>").unwrap();
    if let Some(m) = head.find(&rendered) {
        Ok(format!("{}{}{}", &rendered[..m.end()], bundle_tag, &rendered[m.end()..]))
    } else {
        Ok(format!("{bundle_tag}{rendered}"))
    }
}

pub fn to_zip(project: &Project) -> Result<Vec<u8>, String> {
    validate(project)?;
    let mut files = project.files.clone();
    files.insert("canvas.json".into(), serde_json::to_string_pretty(&project.canvas).map_err(|e| e.to_string())?);
    let manifest = ds_manifest(project);
    if manifest["cards"].as_array().is_some_and(|c| !c.is_empty()) || manifest["tokens"].as_object().is_some_and(|t| !t.is_empty()) {
        files.entry("_ds_manifest.json".into()).or_insert_with(|| serde_json::to_string_pretty(&manifest).unwrap_or_default());
    }
    // Runtime is supplied only where absent; imported originals remain unchanged.
    for name in project.files.keys().filter(|n| n.ends_with(".dc.html")) {
        let dir = Path::new(name).parent().unwrap_or(Path::new(""));
        files.entry(dir.join("support.js").to_string_lossy().replace('\\', "/")).or_insert_with(|| RUNTIME.into());
    }
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, source) in files {
        writer.start_file(name, zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated)).map_err(|e| e.to_string())?;
        writer.write_all(source.as_bytes()).map_err(|e| e.to_string())?;
    }
    Ok(writer.finish().map_err(|e| e.to_string())?.into_inner())
}

pub fn from_zip(bytes: &[u8]) -> Result<(Value, BTreeMap<String, String>), String> {
    if bytes.len() > MAX_PROJECT { return Err("ZIP 超过 16MiB".into()); }
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| e.to_string())?;
    if archive.len() > 500 { return Err("ZIP 文件数量过多".into()); }
    let mut all = BTreeMap::new(); let mut total = 0;
    for i in 0..archive.len() {
        let mut f = archive.by_index(i).map_err(|e| e.to_string())?;
        if f.is_dir() { continue; }
        if f.unix_mode().is_some_and(|m| m & 0o170000 == 0o120000) { return Err("拒绝符号链接".into()); }
        let name = f.name().map_err(|e| e.to_string())?.into_owned();
        if !file_name(&name) { return Err(format!("暂不支持的归档资源：{name}。当前支持文本源包；二进制素材请先内嵌为 data URL。")); }
        if f.size() > MAX_FILE as u64 { return Err(format!("{name} 超过 2MiB")); }
        let mut buf = Vec::new(); f.by_ref().take(MAX_FILE as u64 + 1).read_to_end(&mut buf).map_err(|e| e.to_string())?;
        total += buf.len(); if buf.len() > MAX_FILE || total > MAX_PROJECT { return Err("ZIP 解压内容过大".into()); }
        let text = String::from_utf8(buf).map_err(|_| format!("{name} 不是 UTF-8 文本"))?;
        if all.insert(name, text).is_some() { return Err("ZIP 中有重复文件路径".into()); }
    }
    let indexes: Vec<_> = all.keys().filter(|n| n.as_str() == "canvas.json" || n.ends_with("/canvas.json")).cloned().collect();
    if indexes.len() != 1 { return Err("源包必须含唯一的 canvas.json".into()); }
    let index = &indexes[0]; let prefix = index.strip_suffix("canvas.json").unwrap();
    let canvas = serde_json::from_str(all.get(index).unwrap()).map_err(|e| format!("索引无效：{e}"))?;
    if all.keys().any(|name| !name.starts_with(prefix)) { return Err("源文件必须位于 canvas.json 所在目录内".into()); }
    let files: BTreeMap<String, String> = all.into_iter().filter_map(|(n, t)| n.strip_prefix(prefix).filter(|n| *n != "canvas.json").map(|n| (n.to_string(), t))).collect();
    for (name, _) in boards(&canvas)? { if !files.contains_key(&name) { return Err(format!("缺少画板源文件：{name}")); } }
    Ok((canvas, files))
}

pub fn standalone(project: &Project) -> Result<String, String> {
    validate(project)?;
    let sources: BTreeMap<_, _> = project.files.keys().filter(|name| name.ends_with(".dc.html")).map(|name| Ok((name.clone(), preview_in_project(project, name)?))).collect::<Result<_, String>>()?;
    let data = json!({"boards":boards(&project.canvas)?,"sources":sources}).to_string().replace('<', "\\u003c");
    let title = project.title.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
    Ok(format!(r#"<!doctype html><html><meta charset="utf-8"><title>{title}</title><style>body{{margin:0;font:13px system-ui;background:#f5f5f7;color:#1f2026}}header{{position:sticky;top:0;display:flex;align-items:center;justify-content:space-between;background:white;border-bottom:1px solid #e6e7ec;padding:12px 20px;z-index:2}}main{{display:flex;flex-wrap:wrap;gap:24px;padding:24px;align-items:start}}section{{flex:none}}iframe{{border:1px solid #ddd;border-radius:8px;background:white;box-shadow:0 8px 24px rgba(0,0,0,.05)}}h2{{font-size:12px;margin:0 0 8px;color:#555}}</style><header><b>{title} · 独立预览</b><span>共 {} 个画板</span></header><main></main><script>const p={data};const frames=[];p.boards.forEach(([file,b])=>{{const s=document.createElement('section'),h=document.createElement('h2'),f=document.createElement('iframe');h.textContent=(b.title||file)+' ('+b.w+'×'+b.h+')';f.sandbox='allow-scripts';f.width=b.w;f.height=b.h;f.srcdoc='<meta http-equiv="Content-Security-Policy" content="default-src &apos;none&apos;; script-src &apos;unsafe-inline&apos; &apos;unsafe-eval&apos;; style-src &apos;unsafe-inline&apos; https://fonts.googleapis.com; img-src data: blob:; font-src data: https://fonts.gstatic.com; connect-src &apos;none&apos;; base-uri &apos;none&apos;; form-action &apos;none&apos;;">'+p.sources[file];s.append(h,f);document.querySelector('main').append(s);frames.push({{file,f}});}});addEventListener('message',e=>{{const frame=frames.find(x=>x.f.contentWindow===e.source);if(!frame||e.data?.channel!=='open-claude-design')return;if(e.data.type==='ready')frame.f.contentWindow.postMessage({{channel:'open-claude-design',type:'mode',editing:false}},'*');if(e.data.type==='error'){{const error=document.createElement('p');error.textContent='预览错误：'+String(e.data.message);error.style.color='#b54454';frame.f.parentElement.append(error);}}if(e.data.type==='navigate'){{const parts=frame.file.split('/');parts.pop();for(const part of e.data.file.split('/')){{if(part==='..')parts.pop();else if(part!=='.')parts.push(part);}}const dest=frames.find(x=>x.file===parts.join('/'));dest?.f.scrollIntoView({{behavior:'smooth'}});}}}});</script></html>"#, sources.len()))
}

pub fn markdown(project: &Project) -> Result<String, String> {
    validate(project)?;
    let board_list = boards(&project.canvas)?;
    let manifest = ds_manifest(project);
    let mut out = format!("# {}\n\n- 设计稿编号：`{}`\n- 当前版本：`v{}`\n- 画板数量：{}\n\n## 画板清单\n\n| 画板 | 文件 | 尺寸 | 页面 |\n| --- | --- | --- | --- |\n",
        project.title, project.id, project.revision, board_list.len());
    for (file, b) in &board_list {
        let title = b.get("title").and_then(Value::as_str).unwrap_or(file);
        let w = b.get("w").and_then(Value::as_f64).unwrap_or(0.0) as u64;
        let h = b.get("h").and_then(Value::as_f64).unwrap_or(0.0) as u64;
        let page = b.get("page").and_then(Value::as_str).unwrap_or("默认");
        out.push_str(&format!("| {title} | `{file}` | {w}×{h} | {page} |\n"));
    }
    if let Some(tokens) = manifest.get("tokens").and_then(Value::as_object).filter(|t| !t.is_empty()) {
        out.push_str("\n## 设计系统令牌 (Design Tokens)\n\n| 令牌 | CSS 变量 | 值 |\n| --- | --- | --- |\n");
        for (k, v) in tokens {
            out.push_str(&format!("| `{k}` | `--ds-{k}` | `{}` |\n", v.as_str().unwrap_or_default()));
        }
    }
    if let Some(cards) = manifest.get("cards").and_then(Value::as_array).filter(|c| !c.is_empty()) {
        out.push_str("\n## 组件卡片 (@dsCard)\n\n| 分组 | 名称 | 路径 | 说明 |\n| --- | --- | --- | --- |\n");
        for c in cards {
            out.push_str(&format!("| {} | {} | `{}` | {} |\n",
                c["group"].as_str().unwrap_or(""),
                c["name"].as_str().unwrap_or(""),
                c["path"].as_str().unwrap_or(""),
                c["subtitle"].as_str().unwrap_or("")));
        }
    }
    if let Some(comments) = project.canvas.pointer("/xiaoban/comments").and_then(Value::as_array).filter(|c| !c.is_empty()) {
        out.push_str("\n## 批注记录\n\n");
        for c in comments {
            let status = c["status"].as_str().unwrap_or("saved");
            let file = c["file"].as_str().unwrap_or("");
            let node = c["node"].as_str().unwrap_or("");
            let path = c["path"].as_str().unwrap_or("");
            let text = c["text"].as_str().unwrap_or("");
            out.push_str(&format!("- **[{}]** `{}`{}{}: {}\n",
                status, file,
                if node.is_empty() { String::new() } else { format!(" (`{node}`)") },
                if path.is_empty() { String::new() } else { format!(" `{path}`") },
                text));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_roundtrip_keeps_indexes_and_logic() {
        let canvas = json!({"v":3,"boards":{"a/Main.dc.html":{"w":400,"h":800,"custom":123}},"order":["a/Main.dc.html"],"notes":{}});
        let p = Project{status:ProjectStatus::Ready,id:format!("design-{}",nonce().unwrap()),title:"demo".into(),session_id:"s".into(),session_ids:Vec::new(),revision:1,updated_ms:0,canvas:canvas.clone(),files:[("a/Main.dc.html".into(),"<x-dc>{{value}}</x-dc>".into())].into(),request_id:String::new()};
        let (c, f) = from_zip(&to_zip(&p).unwrap()).unwrap();
        assert_eq!(c, canvas);
        assert_eq!(f["a/Main.dc.html"], p.files["a/Main.dc.html"]);
        assert!(f.contains_key("a/support.js"));
        assert!(standalone(&p).unwrap().contains("sandbox"));
    }
    #[test]
    fn support_is_replaced_only_for_preview() {
        let s = "<html><head><script src='./support.js'></script></head><body><x-dc>text</x-dc></body></html>";
        let p = preview(s);
        assert!(!p.contains("src='./support.js'"));
        assert!(p.contains("DCLogic"));
    }
    fn zip_files(files: Vec<(&str, Vec<u8>)>) -> Vec<u8> {
        let mut w = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for (n, b) in files { w.start_file(n, zip::write::SimpleFileOptions::default()).unwrap(); w.write_all(&b).unwrap(); }
        w.finish().unwrap().into_inner()
    }
    #[test]
    fn rejects_unsafe_or_incomplete_archives_before_import() {
        assert!(from_zip(&zip_files(vec![("../escape.dc.html", b"x".to_vec())])).is_err());
        assert!(from_zip(&zip_files(vec![("asset.png", vec![0, 255])])).unwrap_err().contains("二进制"));
        let index = br#"{"v":3,"boards":{"Main.dc.html":{"w":100,"h":200}},"order":["Main.dc.html"]}"#.to_vec();
        assert!(from_zip(&zip_files(vec![("canvas.json", index.clone())])).unwrap_err().contains("缺少画板"));
        assert!(from_zip(&zip_files(vec![("canvas.json", index.clone()), ("other/canvas.json", index.clone())])).unwrap_err().contains("唯一"));
        assert!(from_zip(&zip_files(vec![("canvas.json", index.clone()), ("Main.dc.html", vec![b'x'; MAX_FILE + 1])])).unwrap_err().contains("2MiB"));
    }
    #[test]
    fn maximum_source_count_roundtrips_with_generated_runtime() {
        let mut files = BTreeMap::from([("Main.dc.html".into(), "<x-dc>ok</x-dc>".into())]);
        for n in 0..299 { files.insert(format!("asset{n}.css"), "body{}".into()); }
        let mut p = Project{status:ProjectStatus::Ready,id:format!("design-{}",nonce().unwrap()),title:"full".into(),session_id:"s".into(),session_ids:Vec::new(),revision:1,updated_ms:0,canvas:json!({"v":3,"boards":{"Main.dc.html":{"w":100,"h":200}},"order":["Main.dc.html"]}),files,request_id:String::new()};
        let (c, f) = from_zip(&to_zip(&p).unwrap()).unwrap();
        p.canvas = c; p.files = f;
        assert_eq!(p.files.len(), 301);
        validate(&p).unwrap();
    }
    #[test]
    fn compiles_ds_card_manifest_inlines_css_and_exports_markdown() {
        let canvas = json!({"v":3,"boards":{"Main.dc.html":{"title":"首页","w":390,"h":844}},"order":["Main.dc.html"],"designSystems":[{"name":"Brand","tokens":{"primary":"#d94e60"}}]});
        let files = BTreeMap::from([
            ("Main.dc.html".into(), "<!-- @dsCard group=\"Actions\" name=\"Primary Button\" subtitle=\"CTA\" width=\"320\" height=\"120\" -->\n<!doctype html><html><head><link rel=\"stylesheet\" href=\"./theme.css\"></head><body><x-dc><dc-import src=\"components/Btn.dc.html\"></dc-import></x-dc></body></html>".into()),
            ("theme.css".into(), ".btn{color:var(--ds-primary)}".into()),
            ("components/Btn.dc.html".into(), "<x-dc><button class=\"btn\">OK</button></x-dc>".into()),
        ]);
        let p = Project{status:ProjectStatus::Ready,id:format!("design-{}",nonce().unwrap()),title:"DS Demo".into(),session_id:"s".into(),session_ids:Vec::new(),revision:1,updated_ms:0,canvas,files,request_id:String::new()};
        let m = ds_manifest(&p);
        assert_eq!(m["cards"][0]["group"], "Actions");
        assert_eq!(m["cards"][0]["name"], "Primary Button");
        assert_eq!(m["tokens"]["primary"], "#d94e60");
        let prev = preview_in_project(&p, "Main.dc.html").unwrap();
        assert!(prev.contains("--ds-primary:#d94e60;"));
        assert!(prev.contains(".btn{color:var(--ds-primary)}"));
        assert!(prev.contains("components/Btn.dc.html"));
        let md = markdown(&p).unwrap();
        assert!(md.contains("Primary Button"));
        assert!(md.contains("--ds-primary"));
    }
}
