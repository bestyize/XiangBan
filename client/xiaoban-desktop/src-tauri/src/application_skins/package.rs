//! Declarative application skins. Packages are data only, never executable UI.
use base64::{engine::general_purpose::STANDARD, Engine};
use image::{ImageFormat, ImageReader, Limits};
use serde_json::{json, Value};
use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{Cursor, Read},
    path::{Path, PathBuf},
};
use zip::ZipArchive;

const MAX_PACK: u64 = 32 * 1024 * 1024;
const MAX_FILE: u64 = 4 * 1024 * 1024;
pub(super) const MAX_SKINS: usize = 20;
pub(super) const BUILTIN_SKINS: &[&str] = &[
    "builtin:spring-river",
    "builtin:jiangnan",
    "builtin:seaside-train",
];
fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}
fn field<'a>(v: &'a Value, key: &str, max: usize) -> Result<&'a str, String> {
    v.get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty() && s.len() <= max && !s.chars().any(char::is_control))
        .ok_or_else(|| format!("皮肤字段 {key} 无效"))
}
fn safe_path(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 160
        && !name.starts_with('/')
        && !name.contains(['\\', ':', '\0'])
        && name
            .split('/')
            .all(|p| !p.is_empty() && p != "." && p != "..")
}
fn image_data(name: &str, bytes: &[u8]) -> Result<String, String> {
    let fmt = image::guess_format(bytes).map_err(|_| format!("图片 {name} 无法识别"))?;
    let mime = match (
        name.rsplit('.')
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str(),
        fmt,
    ) {
        ("png", ImageFormat::Png) => "image/png",
        ("jpg" | "jpeg", ImageFormat::Jpeg) => "image/jpeg",
        ("webp", ImageFormat::WebP) => "image/webp",
        _ => return Err(format!("图片 {name} 格式不支持或扩展名不符")),
    };
    let mut reader = ImageReader::with_format(Cursor::new(bytes), fmt);
    let mut limits = Limits::default();
    limits.max_image_width = Some(4096);
    limits.max_image_height = Some(4096);
    limits.max_alloc = Some(64 * 1024 * 1024);
    reader.limits(limits);
    let decoded = reader
        .decode()
        .map_err(|_| format!("图片 {name} 损坏或过大"))?;
    let decoded = if name == "preview.webp" {
        decoded.thumbnail(512, 512)
    } else {
        decoded
    };
    // Decode and re-encode: animated WebP/APNG cannot execute outside the bounded renderer.
    let mut safe = Cursor::new(Vec::new());
    decoded
        .write_to(&mut safe, ImageFormat::Png)
        .map_err(|_| "图片转换失败")?;
    let _ = mime;
    Ok(format!(
        "data:image/png;base64,{}",
        STANDARD.encode(safe.into_inner())
    ))
}
pub(super) fn parse(bytes: &[u8]) -> Result<Value, String> {
    if bytes.len() as u64 > MAX_PACK {
        return Err("皮肤包超过 32 MB".into());
    }
    let mut zip = ZipArchive::new(Cursor::new(bytes)).map_err(|_| "不是有效的 ZIP")?;
    if zip.len() > 60 {
        return Err("皮肤包文件过多".into());
    }
    let mut files = HashMap::new();
    let mut folded = HashSet::new();
    let mut total = 0u64;
    for i in 0..zip.len() {
        let mut f = zip.by_index(i).map_err(|_| "无法读取 ZIP")?;
        let name = f.name().map_err(|_| "文件名无效")?.to_string();
        if f.is_dir() {
            continue;
        }
        if f.is_symlink() || !safe_path(&name) || !folded.insert(name.to_ascii_lowercase()) {
            return Err("皮肤包包含不安全或重复的路径".into());
        }
        let mut data = Vec::new();
        (&mut f)
            .take(MAX_FILE + 1)
            .read_to_end(&mut data)
            .map_err(|_| "无法解压皮肤")?;
        total += data.len() as u64;
        if data.len() as u64 > MAX_FILE || total > MAX_PACK {
            return Err("皮肤素材过大".into());
        }
        files.insert(name, data);
    }
    let parse_json = |name: &str| -> Result<Value, String> {
        let data = files.get(name).ok_or_else(|| format!("缺少 {name}"))?;
        if data.len() > 128 * 1024 {
            return Err(format!("{name} 过大"));
        }
        serde_json::from_slice(data).map_err(|_| format!("{name} 格式无效"))
    };
    let m = parse_json("manifest.json")?;
    if m["kind"] != "application-skin" || m["schemaVersion"] != 1 {
        return Err("不支持的皮肤类型或版本".into());
    }
    if let Some(min) = m.get("minAppVersion") {
        let version = |s: &str| -> Option<(u64, u64, u64)> {
            let mut p = s.split('.');
            let value = (
                p.next()?.parse().ok()?,
                p.next()?.parse().ok()?,
                p.next()?.parse().ok()?,
            );
            if p.next().is_some() {
                None
            } else {
                Some(value)
            }
        };
        let required = version(min.as_str().ok_or("最低版本无效")?).ok_or("最低版本无效")?;
        let current = version(env!("CARGO_PKG_VERSION")).ok_or("应用版本无效")?;
        if required > current {
            return Err("皮肤需要更新的小伴桌面版本".into());
        }
    }
    let id = field(&m, "id", 64)?;
    if !valid_id(id) {
        return Err("皮肤 ID 无效".into());
    }
    let name = field(&m, "name", 120)?;
    let version = field(&m, "version", 40)?;
    let author = m["author"].as_str().unwrap_or("");
    if author.len() > 120 || author.chars().any(char::is_control) {
        return Err("作者名无效".into());
    }
    if m["scene"] != "scene.json" || m["theme"] != "theme.json" || m["preview"] != "preview.webp" {
        return Err("皮肤文件入口无效".into());
    }
    let theme = parse_json("theme.json")?;
    if !theme.is_object() {
        return Err("theme.json 无效".into());
    }
    let scene = parse_json("scene.json")?;
    let layers = scene["layers"].as_array().ok_or("缺少图层")?;
    if layers.is_empty() || layers.len() > 20 {
        return Err("图层数量无效".into());
    }
    let mut used = HashSet::from([
        "manifest.json".to_string(),
        "theme.json".to_string(),
        "scene.json".to_string(),
        "preview.webp".to_string(),
    ]);
    let mut ids = HashSet::new();
    let mut output = Vec::new();
    let mut image_budget = 0usize;
    for l in layers {
        let lid = field(l, "id", 64)?;
        if !valid_id(lid) || !ids.insert(lid.to_owned()) {
            return Err("图层 ID 无效或重复".into());
        }
        let path = field(l, "image", 160)?;
        if !safe_path(path) || !path.starts_with("assets/") {
            return Err("素材路径无效".into());
        }
        let depth = l["depth"]
            .as_i64()
            .filter(|n| (0..=10).contains(n))
            .ok_or("图层深度无效")?;
        let anchor = field(l, "anchor", 32)?;
        if ![
            "center",
            "bottom-left",
            "bottom-right",
            "top-left",
            "top-right",
        ]
        .contains(&anchor)
        {
            return Err("图层锚点无效".into());
        }
        let url = image_data(path, files.get(path).ok_or("缺少图层素材")?)?;
        image_budget += url.len();
        if image_budget > 16 * 1024 * 1024 {
            return Err("皮肤图层解码后过大".into());
        }
        used.insert(path.to_string());
        output.push(json!({"id":lid,"image":url,"depth":depth,"anchor":anchor,"interactive":l["interactive"].as_bool().unwrap_or(false)}));
    }
    let interactions = scene
        .get("interactions")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if interactions.len() > 20 {
        return Err("互动数量过多".into());
    }
    for it in &interactions {
        let target = field(it, "target", 64)?;
        if !ids.contains(target) {
            return Err("互动目标不存在".into());
        }
        if !["click", "hover", "idle"].contains(&field(it, "trigger", 16)?)
            || !["sway", "glow"].contains(&field(it, "action", 24)?)
        {
            return Err("不支持的互动效果".into());
        }
        field(it, "label", 100)?;
        if it.get("repeat").is_some() && !["restart", "ignore"].contains(&field(it, "repeat", 16)?)
        {
            return Err("互动重复策略无效".into());
        }
    }
    if files.keys().any(|name| !used.contains(name)) {
        return Err("皮肤包含未声明文件".into());
    }
    let preview = image_data(
        "preview.webp",
        files.get("preview.webp").ok_or("缺少预览图")?,
    )?;
    Ok(
        json!({"id":id,"name":name,"version":version,"author":author,"preview":preview,"scene":{"layers":output,"interactions":interactions}}),
    )
}
pub(super) fn dir(root: &Path) -> PathBuf {
    root.join("application-skins")
}
pub(super) fn file(root: &Path, id: &str) -> Result<PathBuf, String> {
    if !valid_id(id) {
        return Err("皮肤 ID 无效".into());
    }
    Ok(dir(root).join(format!("{id}.zip")))
}
pub(super) fn read(path: &Path) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(MAX_PACK + 1)
        .read_to_end(&mut out)
        .map_err(|e| e.to_string())?;
    if out.len() as u64 > MAX_PACK {
        return Err("皮肤包过大".into());
    }
    Ok(out)
}
pub(super) fn listing(root: &Path) -> Result<Value, String> {
    fs::create_dir_all(dir(root)).map_err(|e| e.to_string())?;
    let mut active = String::new();
    if let Ok(f) = fs::File::open(dir(root).join("active")) {
        let _ = f.take(128).read_to_string(&mut active);
    }
    let active = active.trim();
    let mut skins = Vec::new();
    for entry in fs::read_dir(dir(root))
        .map_err(|e| e.to_string())?
        .take(MAX_SKINS * 4)
    {
        let e = entry.map_err(|e| e.to_string())?;
        let p = e.path();
        if p.extension().is_some_and(|e| e == "zip")
            && e.file_type().map(|t| t.is_file()).unwrap_or(false)
        {
            if let Ok(mut s) = read(&p).and_then(|b| parse(&b)) {
                if p.file_stem().and_then(|s| s.to_str()) == s["id"].as_str() {
                    if s["id"].as_str() != Some(active) {
                        s["scene"] = json!({"layers":[],"interactions":[]});
                    }
                    skins.push(s);
                    if skins.len() == MAX_SKINS {
                        break;
                    }
                }
            }
        }
    }
    skins.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
    let selected = if BUILTIN_SKINS.contains(&active) || skins.iter().any(|s| s["id"] == active) {
        Some(active)
    } else {
        None
    };
    Ok(json!({"skins":skins,"activeId":selected}))
}
pub(super) fn save_active(root: &Path, id: Option<&str>) -> Result<(), String> {
    let temp = dir(root).join("active.tmp");
    let result = fs::write(&temp, id.unwrap_or(""))
        .and_then(|_| fs::rename(&temp, dir(root).join("active")));
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result.map_err(|e| e.to_string())
}

pub(super) fn apply(root: &Path, id: Option<&str>) -> Result<Value, String> {
    let state = listing(root)?;
    if let Some(id) = id {
        if !BUILTIN_SKINS.contains(&id)
            && !state["skins"]
                .as_array()
                .unwrap()
                .iter()
                .any(|s| s["id"] == id)
        {
            return Err("皮肤不存在或已损坏，请先用 skin_list 查看可用皮肤".into());
        }
    }
    save_active(root, id)?;
    listing(root)
}

/// Installs exactly the bytes accepted by the UI importer. Never applies a new skin.
pub(super) fn install(root: &Path, bytes: &[u8]) -> Result<Value, String> {
    let skin = parse(bytes)?;
    let state = listing(root)?;
    let path = file(root, skin["id"].as_str().unwrap())?;
    if path.exists() {
        if read(&path)? == bytes {
            return Ok(skin);
        }
        return Err("同名皮肤已存在，请更换 ID；不会覆盖已有皮肤".into());
    }
    if state["skins"].as_array().unwrap().len() >= MAX_SKINS {
        return Err("已安装皮肤数量达到上限".into());
    }
    write_new(&path, bytes)?;
    Ok(skin)
}

/// A complete temporary file is linked into place without replacing existing files.
pub(super) fn write_new(path: &Path, bytes: &[u8]) -> Result<bool, String> {
    use std::io::Write;
    if path.symlink_metadata().is_ok() {
        if path.symlink_metadata().is_ok_and(|m| m.is_file()) && read(path)? == bytes {
            return Ok(false);
        }
        return Err(format!("文件已存在，不会覆盖：{}", path.display()));
    }
    let temp = path.with_extension("zip.tmp");
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(|e| e.to_string())?;
    let result = output.write_all(bytes).and_then(|_| output.sync_all());
    drop(output);
    let result = result.and_then(|_| fs::hard_link(&temp, path));
    let _ = fs::remove_file(&temp);
    result.map_err(|e| e.to_string())?;
    Ok(true)
}

pub(super) fn catalog(root: &Path) -> Result<Value, String> {
    let state = listing(root)?;
    let mut skins = vec![
        json!({"id":"builtin:spring-river","name":"春江花月夜","builtin":true}),
        json!({"id":"builtin:jiangnan","name":"江南烟雨","builtin":true}),
        json!({"id":"builtin:seaside-train","name":"海边列车","builtin":true}),
    ];
    skins.extend(state["skins"].as_array().unwrap().iter().map(|s| json!({
        "id":s["id"],"name":s["name"],"version":s["version"],"author":s["author"],"builtin":false
    })));
    Ok(
        json!({"skins":skins,"activeId":state["activeId"],"defaultId":"default","page":{"page":"studio","tab":"skins"}}),
    )
}

pub(super) fn format_guide() -> Value {
    json!({
        "kind":"application-skin", "schemaVersion":1,
        "scope":"小伴主窗口的背景场景与装饰图层；不改变桌面壁纸、桌宠、控件布局。theme.json 配色暂不生效，不承诺改变按钮或文字配色。",
        "workflow":["先 skin_read_format，无需用户提供样例或到源码目录找规范", "已有图片可直接 skin_create(background_image)，自动生成配置和预览图", "需要多图层时在工作区子目录制作下面的完整文件，再 skin_create(source_dir)", "原生打包并用导入器校验，安装到装扮→皮肤，并在工作区交付 ZIP", "创建不改变当前皮肤；用户要求换肤时先 skin_list，再 skin_apply；导出已有自制皮肤用 skin_export"],
        "background_image":"generated:<image_generate 返回的图片 id>，或当前工作区内的相对 PNG/JPEG/WebP 文件路径。不要传外链、绝对路径或 base64。工具自动生成 WebP 背景与 preview.webp；图片生成失败时可使用工作区已有或自行绘制的栅格图片，不要伪造图片编号。",
        "create_example":{"id":"earth-pulse","name":"地球脉动","background_image":"earth.png"},
        "source_dir_example":{"source_dir":"earth-skin"},
        "files":{
            "manifest.json":{"schemaVersion":1,"kind":"application-skin","id":"earth-pulse","name":"地球脉动","version":"1.0.0","author":"小伴","preview":"preview.webp","theme":"theme.json","scene":"scene.json"},
            "theme.json":{},
            "scene.json":{"layers":[{"id":"background","image":"assets/earth.webp","depth":0,"anchor":"center"}],"interactions":[]},
            "preview.webp":"真实 WebP 预览图，不能把 PNG 改后缀",
            "assets/earth.webp":"实际背景图片，不要将 UI 截图烘焙进背景"
        },
        "limits":{"packageBytes":MAX_PACK,"fileBytes":MAX_FILE,"zipEntries":60,"jsonBytes":131072,"maxImageWidth":4096,"maxImageHeight":4096,"decodedLayerDataBytes":16777216,"installedSkins":MAX_SKINS,"layers":20,"interactions":20},
        "layers":{"id":"1–64 字节，仅小写字母、数字、连字符且唯一","image":"包内 assets/ 下的 PNG/JPEG/WebP 图片","depth":"0–10 整数","anchor":["center","top-left","top-right","bottom-left","bottom-right"],"interactive":"可选布尔值；点击物件必须为 true。普通图层按 cover 铺满场景；点击物件使用宿主固定尺寸与锚点，不支持自由坐标"},
        "interactions":{"target":"已有图层 id","trigger":["idle","hover","click"],"action":["sway","glow"],"label":"必填无障碍名称","repeat":["restart","ignore"],"note":"hover 仅用于声明了 click 的交互物件；互动仅在新建任务页启用"},
        "unsupported":["HTML/CSS/JavaScript/SVG","外链、字体、音视频","未声明的多余文件","帧动画、粒子、换图、自由坐标、皮肤配色"],
        "version":"manifest 必须包含 id/name/version；author 可省略；minAppVersion 可省略，存在时为三段数字版本；不支持的 schema 或图片会明确报错"
    })
}

fn zip_files(files: &[(String, Vec<u8>)]) -> Result<Vec<u8>, String> {
    use std::io::Write;
    use zip::{write::SimpleFileOptions, ZipWriter};
    let mut output = ZipWriter::new(Cursor::new(Vec::new()));
    for (name, bytes) in files {
        output
            .start_file(name, SimpleFileOptions::default())
            .map_err(|e| e.to_string())?;
        output.write_all(bytes).map_err(|e| e.to_string())?;
    }
    let bytes = output.finish().map_err(|e| e.to_string())?.into_inner();
    parse(&bytes)?;
    Ok(bytes)
}

/// Build a basic scene from a real raster source; no Python, ZIP tools or image service required.
pub(super) fn from_image(
    id: &str,
    name: &str,
    author: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, String> {
    if !valid_id(id) {
        return Err("皮肤 ID 只能由 1–64 位小写字母、数字、连字符组成".into());
    }
    let metadata = json!({"name":name,"author":author});
    field(&metadata, "name", 120)?;
    if author.len() > 120 || author.chars().any(char::is_control) {
        return Err("作者名无效".into());
    }
    if bytes.len() > 16 * 1024 * 1024 {
        return Err("背景图片超过 16 MB".into());
    }
    let fmt = image::guess_format(bytes).map_err(|_| "背景图片无法识别")?;
    if !matches!(
        fmt,
        ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::WebP
    ) {
        return Err("背景只支持 PNG、JPEG、WebP".into());
    }
    let mut reader = ImageReader::with_format(Cursor::new(bytes), fmt);
    let mut limits = Limits::default();
    limits.max_image_width = Some(8192);
    limits.max_image_height = Some(8192);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    let image = reader.decode().map_err(|_| "背景图片损坏或过大")?;
    let mut background = image.thumbnail(1920, 1200);
    let encoded = loop {
        let mut data = Cursor::new(Vec::new());
        background
            .write_to(&mut data, ImageFormat::WebP)
            .map_err(|e| e.to_string())?;
        if data.get_ref().len() as u64 <= MAX_FILE {
            break data.into_inner();
        }
        background = background.thumbnail(
            (background.width() * 3 / 4).max(1),
            (background.height() * 3 / 4).max(1),
        );
    };
    let mut preview = Cursor::new(Vec::new());
    background
        .thumbnail(512, 320)
        .write_to(&mut preview, ImageFormat::WebP)
        .map_err(|e| e.to_string())?;
    let manifest = json!({"schemaVersion":1,"kind":"application-skin","id":id,"name":name,"version":"1.0.0","author":author,"preview":"preview.webp","theme":"theme.json","scene":"scene.json"});
    let scene = json!({"layers":[{"id":"background","image":"assets/background.webp","depth":0,"anchor":"center"}],"interactions":[]});
    zip_files(&[
        ("manifest.json".into(), manifest.to_string().into_bytes()),
        ("scene.json".into(), scene.to_string().into_bytes()),
        ("theme.json".into(), b"{}".to_vec()),
        ("preview.webp".into(), preview.into_inner()),
        ("assets/background.webp".into(), encoded),
    ])
}

pub(super) fn workspace_path(workspace: &Path, relative: &str) -> Result<PathBuf, String> {
    if !safe_path(relative) {
        return Err("素材或草稿必须使用当前工作区内的相对路径".into());
    }
    let workspace = workspace.canonicalize().map_err(|e| e.to_string())?;
    let path = workspace
        .join(relative)
        .canonicalize()
        .map_err(|e| e.to_string())?;
    if !path.starts_with(&workspace) || path == workspace {
        return Err("素材或草稿必须位于当前工作区内".into());
    }
    Ok(path)
}

pub(super) fn read_image(path: &Path) -> Result<Vec<u8>, String> {
    let metadata = path.symlink_metadata().map_err(|e| e.to_string())?;
    if !metadata.is_file() || metadata.len() > 16 * 1024 * 1024 {
        return Err("素材必须是最多 16 MB 的普通图片文件".into());
    }
    let mut data = Vec::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(16 * 1024 * 1024 + 1)
        .read_to_end(&mut data)
        .map_err(|e| e.to_string())?;
    if data.len() > 16 * 1024 * 1024 {
        return Err("素材图片超过 16 MB".into());
    }
    Ok(data)
}

pub(super) fn from_directory(workspace: &Path, relative: &str) -> Result<Vec<u8>, String> {
    fn visit(
        dir: &Path,
        prefix: &str,
        files: &mut Vec<(String, Vec<u8>)>,
        count: &mut usize,
        total: &mut u64,
    ) -> Result<(), String> {
        for child in fs::read_dir(dir).map_err(|e| e.to_string())? {
            *count += 1;
            if *count > 60 {
                return Err("皮肤草稿最多包含 60 个文件和目录".into());
            }
            let child = child.map_err(|e| e.to_string())?;
            let name = format!(
                "{prefix}{}",
                child.file_name().to_str().ok_or("草稿文件名无效")?
            );
            if !safe_path(&name) || name.split('/').any(|p| p.starts_with('.')) {
                return Err("草稿文件路径无效".into());
            }
            let kind = child.file_type().map_err(|e| e.to_string())?;
            if kind.is_dir() {
                if name.split('/').count() >= 4 {
                    return Err("皮肤素材路径最多 4 层".into());
                }
                visit(&child.path(), &format!("{name}/"), files, count, total)?;
            } else if kind.is_file() {
                let mut bytes = Vec::new();
                fs::File::open(child.path())
                    .map_err(|e| e.to_string())?
                    .take(MAX_FILE + 1)
                    .read_to_end(&mut bytes)
                    .map_err(|e| e.to_string())?;
                *total += bytes.len() as u64;
                if bytes.len() as u64 > MAX_FILE || *total > MAX_PACK {
                    return Err("皮肤草稿素材过大".into());
                }
                files.push((name, bytes));
            } else {
                return Err("皮肤草稿不能包含符号链接或特殊文件".into());
            }
        }
        Ok(())
    }
    let path = workspace_path(workspace, relative)?;
    let mut files = Vec::new();
    visit(&path, "", &mut files, &mut 0, &mut 0)?;
    files.sort_by(|a, b| a.0.cmp(&b.0));
    zip_files(&files)
}

pub(super) fn deliver(root: &Path, workspace: &Path, bytes: &[u8]) -> Result<Value, String> {
    let skin = parse(bytes)?;
    let id = skin["id"].as_str().ok_or("皮肤 ID 无效")?;
    let output = workspace
        .canonicalize()
        .map_err(|e| e.to_string())?
        .join(format!("skin-{id}.zip"));
    let created = write_new(&output, bytes)?;
    if let Err(error) = install(root, bytes) {
        if created {
            let _ = fs::remove_file(&output);
        }
        return Err(error);
    }
    Ok(
        json!({"id":id,"name":skin["name"],"installed":true,"applied":false,"packagePath":output.display().to_string(),"bytes":bytes.len(),"page":{"page":"studio","tab":"skins"}}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::{write::SimpleFileOptions, ZipWriter};
    fn pack(extra: Option<(&str, &[u8])>, scene: &str) -> Vec<u8> {
        let mut image = Cursor::new(Vec::new());
        image::DynamicImage::new_rgba8(2, 2)
            .write_to(&mut image, ImageFormat::WebP)
            .unwrap();
        let image = image.into_inner();
        let mut z = ZipWriter::new(Cursor::new(Vec::new()));
        for (name,data) in [
            ("manifest.json",br#"{"schemaVersion":1,"kind":"application-skin","id":"demo","name":"Demo","version":"1","preview":"preview.webp","scene":"scene.json","theme":"theme.json"}"#.as_slice()),
            ("scene.json",scene.as_bytes()), ("theme.json",b"{}".as_slice()), ("preview.webp",image.as_slice()), ("assets/a.webp",image.as_slice())
        ] { z.start_file(name,SimpleFileOptions::default()).unwrap(); z.write_all(data).unwrap(); }
        if let Some((name, data)) = extra {
            z.start_file(name, SimpleFileOptions::default()).unwrap();
            z.write_all(data).unwrap();
        }
        z.finish().unwrap().into_inner()
    }
    const SCENE: &str = r#"{"layers":[{"id":"base","image":"assets/a.webp","depth":0,"anchor":"center"}],"interactions":[]}"#;
    struct Temp(PathBuf);
    impl Temp {
        fn new() -> Self {
            static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let root =
                std::env::temp_dir().join(format!("xiaoban-skin-agent-{}-{n}", std::process::id()));
            let _ = fs::remove_dir_all(&root);
            fs::create_dir_all(root.join("work")).unwrap();
            Self(root)
        }
        fn work(&self) -> PathBuf {
            self.0.join("work")
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn creation_delivers_an_importable_zip_without_changing_the_current_skin() {
        let temp = Temp::new();
        apply(&temp.0, Some("builtin:jiangnan")).unwrap();
        let mut image = Cursor::new(Vec::new());
        image::DynamicImage::new_rgba8(4, 3)
            .write_to(&mut image, ImageFormat::Png)
            .unwrap();
        let bytes = from_image("earth-pulse", "地球脉动", "小伴", image.get_ref()).unwrap();
        let result = deliver(&temp.0, &temp.work(), &bytes).unwrap();
        let output = PathBuf::from(result["packagePath"].as_str().unwrap());
        assert_eq!(
            read(&output).unwrap(),
            read(&file(&temp.0, "earth-pulse").unwrap()).unwrap()
        );
        assert_eq!(parse(&read(&output).unwrap()).unwrap()["name"], "地球脉动");
        assert_eq!(catalog(&temp.0).unwrap()["activeId"], "builtin:jiangnan");
        assert_eq!(result["applied"], false);
        assert!(!result.to_string().contains("base64"));
        assert!(!catalog(&temp.0).unwrap().to_string().contains("base64"));
        // Retrying a completed creation is safe and returns the same deliverable.
        assert_eq!(deliver(&temp.0, &temp.work(), &bytes).unwrap(), result);
        apply(&temp.0, Some("earth-pulse")).unwrap();
        assert_eq!(listing(&temp.0).unwrap()["activeId"], "earth-pulse");
        assert!(apply(&temp.0, Some("missing")).is_err());
        assert_eq!(listing(&temp.0).unwrap()["activeId"], "earth-pulse");
        assert_eq!(apply(&temp.0, None).unwrap()["activeId"], Value::Null);
    }
    #[test]
    fn failed_installation_preserves_the_old_package_selection_and_user_files() {
        let temp = Temp::new();
        let original = pack(None, SCENE);
        install(&temp.0, &original).unwrap();
        apply(&temp.0, Some("demo")).unwrap();
        let changed = pack(None, &SCENE.replace("\"depth\":0", "\"depth\":1"));
        assert!(deliver(&temp.0, &temp.work(), &changed)
            .unwrap_err()
            .contains("同名"));
        assert!(!temp.work().join("skin-demo.zip").exists());
        assert_eq!(read(&file(&temp.0, "demo").unwrap()).unwrap(), original);
        assert_eq!(listing(&temp.0).unwrap()["activeId"], "demo");
        fs::write(temp.work().join("skin-demo.zip"), b"user's file").unwrap();
        assert!(deliver(&temp.0, &temp.work(), &original).is_err());
        assert_eq!(
            fs::read(temp.work().join("skin-demo.zip")).unwrap(),
            b"user's file"
        );
    }
    #[test]
    fn a_layered_draft_uses_the_same_import_checks_and_cannot_escape_the_workspace() {
        let temp = Temp::new();
        let draft = temp.work().join("draft");
        fs::create_dir_all(draft.join("assets")).unwrap();
        let bytes = pack(None, SCENE);
        let mut zip = ZipArchive::new(Cursor::new(bytes)).unwrap();
        for i in 0..zip.len() {
            let mut f = zip.by_index(i).unwrap();
            let target = draft.join(f.name().unwrap().as_ref());
            let mut data = Vec::new();
            f.read_to_end(&mut data).unwrap();
            fs::write(target, data).unwrap();
        }
        assert_eq!(
            parse(&from_directory(&temp.work(), "draft").unwrap()).unwrap()["id"],
            "demo"
        );
        fs::write(draft.join("extra.js"), b"bad").unwrap();
        assert!(from_directory(&temp.work(), "draft")
            .unwrap_err()
            .contains("未声明"));
        assert!(from_directory(&temp.work(), "../").is_err());
        assert!(workspace_path(&temp.work(), "/tmp").is_err());
        assert!(from_image("../bad", "bad", "", b"bad").is_err());
        assert!(from_image("valid", "valid", "", b"not an image").is_err());
    }
    #[cfg(unix)]
    #[test]
    fn draft_and_export_refuse_symlinks_to_external_files() {
        let temp = Temp::new();
        fs::write(temp.0.join("secret.png"), b"secret").unwrap();
        std::os::unix::fs::symlink(temp.0.join("secret.png"), temp.work().join("escape.png"))
            .unwrap();
        assert!(workspace_path(&temp.work(), "escape.png").is_err());
        fs::create_dir(temp.work().join("draft")).unwrap();
        std::os::unix::fs::symlink(
            temp.0.join("secret.png"),
            temp.work().join("draft/linked.png"),
        )
        .unwrap();
        assert!(from_directory(&temp.work(), "draft").is_err());
        std::os::unix::fs::symlink(temp.0.join("secret.png"), temp.work().join("skin-demo.zip"))
            .unwrap();
        assert!(write_new(&temp.work().join("skin-demo.zip"), b"replacement").is_err());
        assert_eq!(fs::read(temp.0.join("secret.png")).unwrap(), b"secret");
    }
    #[test]
    fn validated_package_exposes_only_safe_decoded_data() {
        let s = parse(&pack(None, SCENE)).unwrap();
        assert_eq!(s["id"], "demo");
        assert!(s["scene"]["layers"][0]["image"]
            .as_str()
            .unwrap()
            .starts_with("data:image/png;base64,"));
    }
    #[test]
    fn rejects_path_escape_and_undeclared_code() {
        assert!(parse(&pack(Some(("../evil.js", b"x")), SCENE)).is_err());
        assert!(parse(&pack(Some(("evil.js", b"x")), SCENE)).is_err());
    }
    #[test]
    fn listing_only_exposes_active_scene_and_replaces_selection() {
        let root = std::env::temp_dir().join(format!("xiaoban-skin-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(dir(&root)).unwrap();
        fs::write(file(&root, "demo").unwrap(), pack(None, SCENE)).unwrap();
        assert!(listing(&root).unwrap()["skins"][0]["scene"]["layers"]
            .as_array()
            .unwrap()
            .is_empty());
        save_active(&root, Some("demo")).unwrap();
        assert_eq!(
            listing(&root).unwrap()["skins"][0]["scene"]["layers"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        for id in BUILTIN_SKINS {
            save_active(&root, Some(id)).unwrap();
            assert_eq!(listing(&root).unwrap()["activeId"], *id);
        }
        save_active(&root, None).unwrap();
        assert!(listing(&root).unwrap()["activeId"].is_null());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn rejects_unknown_action_and_external_asset() {
        let bad = SCENE.replace("assets/a.webp", "https://evil/a.webp");
        assert!(parse(&pack(None, &bad)).is_err());
        let bad = r#"{"layers":[{"id":"base","image":"assets/a.webp","depth":0,"anchor":"center"}],"interactions":[{"target":"base","trigger":"click","action":"shell","label":"bad"}]}"#;
        assert!(parse(&pack(None, bad)).is_err());
    }
}
