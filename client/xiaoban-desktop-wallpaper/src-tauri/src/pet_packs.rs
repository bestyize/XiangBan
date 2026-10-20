//! Declarative, code-free pet themes shared with the frontend renderer.
use serde::{Deserialize, Serialize};
use base64::{engine::general_purpose::STANDARD, Engine};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};
pub const MAX_THEME_BYTES: usize = 24 * 1024 * 1024;
const MAX_IMAGE_BYTES: usize = 4 * 1024 * 1024;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Shape {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,
    pub stroke: Option<String>,
    pub stroke_width: Option<f64>,
    pub opacity: Option<f64>,
    pub d: Option<String>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub rx: Option<f64>,
    pub ry: Option<f64>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Part {
    pub id: String,
    pub name: String,
    pub pivot: [f64; 2],
    pub motion: String,
    pub shapes: Vec<Shape>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Pet {
    pub id: String,
    pub name: String,
    pub description: String,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub builtin_plant: Option<String>,
    pub parts: Vec<Part>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Theme {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub description: String,
    pub color: String,
    pub pets: Vec<Pet>,
}
#[derive(Deserialize)]
struct Catalog {
    themes: Vec<Theme>,
}
pub fn builtin() -> Vec<Theme> {
    static THEMES: std::sync::OnceLock<Vec<Theme>> = std::sync::OnceLock::new();
    THEMES.get_or_init(|| {
    let mut catalog = serde_json::from_str::<Catalog>(include_str!("../../public/pets/catalog.json"))
        .expect("shipped pet themes")
        .themes;
    let assets: [(String, Vec<u8>); 5] = [
        ("assets/sunflower.webp".into(), include_bytes!("../../src/pets/assets/sunflower.webp").to_vec()),
        ("assets/tulip.webp".into(), include_bytes!("../../src/pets/assets/tulip.webp").to_vec()),
        ("assets/echeveria.webp".into(), include_bytes!("../../src/pets/assets/echeveria.webp").to_vec()),
        ("assets/jade.webp".into(), include_bytes!("../../src/pets/assets/jade.webp").to_vec()),
        ("assets/bear-paw.webp".into(), include_bytes!("../../src/pets/assets/bear-paw.webp").to_vec()),
    ];
    for theme in &mut catalog {
        for pet in &mut theme.pets { for part in &mut pet.parts { for shape in &mut part.shapes {
            if shape.kind == "image" {
                let (path, bytes) = assets.iter().find(|(path, _)| Some(path.as_str()) == shape.src.as_deref()).expect("shipped plant asset");
                shape.src = Some(image_data(path, bytes).expect("shipped raster image"));
            }
        } } }
    }
    catalog
    }).clone()
}

fn image_extension(path: &str) -> Result<&str, String> {
    let parts: Vec<_> = path.split('/').collect();
    if parts.len() < 2 || parts.len() > 4 || parts[0] != "assets" || !parts.iter().all(|s| !s.is_empty() && !s.starts_with('.') && s.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))) {
        return Err("植物素材必须使用 assets/ 下的普通文件路径".into());
    }
    match path.rsplit('.').next() { Some(ext @ ("png" | "webp" | "gif")) => Ok(ext), _ => Err("植物素材只支持 PNG、WebP 和 GIF".into()) }
}
fn check_image(ext: &str, bytes: &[u8]) -> Result<(), String> {
    let valid = match ext {
        "png" => bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        "gif" => bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a"),
        "webp" => bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP",
        _ => false,
    };
    if !valid || bytes.len() > MAX_IMAGE_BYTES { return Err("植物图片内容与格式不符或超过 4 MiB".into()); }
    Ok(())
}
fn image_data(path: &str, bytes: &[u8]) -> Result<String, String> {
    let ext = image_extension(path)?;
    check_image(ext, bytes)?;
    Ok(format!("data:image/{ext};base64,{}", STANDARD.encode(bytes)))
}
fn decode_image(src: &str) -> Result<(&str, Vec<u8>), String> {
    if src.len() > MAX_IMAGE_BYTES.div_ceil(3) * 4 + 32 { return Err("植物图片超过 4 MiB".into()); }
    let (header, encoded) = src.split_once(',').ok_or("植物图片必须是内嵌素材，不能含外链")?;
    let ext = match header { "data:image/png;base64" => "png", "data:image/webp;base64" => "webp", "data:image/gif;base64" => "gif", _ => return Err("植物图片不能含 SVG、脚本或外链".into()) };
    let bytes = STANDARD.decode(encoded).map_err(|_| "植物图片编码无效")?;
    check_image(ext, &bytes)?;
    Ok((ext, bytes))
}
/// Resolve archive-local files before persisting. Runtime JSON is self-contained and needs no file access.
pub fn import_assets(theme: &mut Theme, assets: &[(String, Vec<u8>)]) -> Result<(), String> {
    if assets.len() > 60 || assets.iter().map(|(_, b)| b.len()).sum::<usize>() > 16 * 1024 * 1024 { return Err("植物素材总量超过限制".into()); }
    for (path, bytes) in assets { image_data(path, bytes)?; }
    let mut used = std::collections::HashSet::new();
    for pet in &mut theme.pets { for part in &mut pet.parts { for shape in &mut part.shapes {
        if shape.kind == "image" {
            let src = shape.src.as_deref().ok_or("植物图片缺少 src")?;
            if src.starts_with("data:") { decode_image(src)?; continue; }
            image_extension(src)?;
            let (_, bytes) = assets.iter().find(|(path, _)| path == src).ok_or_else(||format!("缺少植物素材：{src}"))?;
            used.insert(src.to_string());
            shape.src = Some(image_data(src, bytes)?);
        }
    } } }
    if assets.iter().any(|(path, _)| !used.contains(path)) { return Err("植物包包含未引用的素材".into()); }
    validate(theme)
}
/// Export only authoring content, with content-addressed filenames so shared images deduplicate.
pub fn export_assets(theme: &Theme) -> Result<(Theme, Vec<(String, Vec<u8>)>), String> {
    validate(theme)?;
    let mut theme = theme.clone();
    let mut files = std::collections::BTreeMap::new();
    for pet in &mut theme.pets { for part in &mut pet.parts { for shape in &mut part.shapes {
        if shape.kind == "image" {
            let (ext, bytes) = decode_image(shape.src.as_deref().ok_or("植物图片缺少 src")?)?;
            let path = format!("assets/{:x}.{ext}", Sha256::digest(&bytes));
            files.insert(path.clone(), bytes);
            shape.src = Some(path);
        }
    } } }
    Ok((theme, files.into_iter().collect()))
}
/// The assistant edits metadata using stable references, without putting base64 image bytes in context.
pub fn compact_images(theme: &mut Theme) -> Result<(), String> {
    for pet in &mut theme.pets { for part in &mut pet.parts { for shape in &mut part.shapes {
        if shape.kind == "image" {
            let (_, bytes)=decode_image(shape.src.as_deref().ok_or("植物图片缺少 src")?)?;
            shape.src=Some(format!("asset:{:x}",Sha256::digest(&bytes)));
        }
    } } }
    Ok(())
}
pub fn resolve_image_refs(theme: &mut Theme, existing: &[Theme]) -> Result<(), String> {
    let mut sources=std::collections::HashMap::new();
    for t in existing { for pet in &t.pets { for part in &pet.parts { for shape in &part.shapes {
        if shape.kind == "image" {
            let src=shape.src.as_deref().ok_or("植物图片缺少 src")?;
            let (_, bytes)=decode_image(src)?;
            sources.insert(format!("asset:{:x}",Sha256::digest(&bytes)),src);
        }
    } } } }
    for pet in &mut theme.pets { for part in &mut pet.parts { for shape in &mut part.shapes {
        if let Some(src)=shape.src.as_deref().filter(|s|s.starts_with("asset:")) {
            shape.src=Some((*sources.get(src).ok_or("图片素材引用不存在，请先导入对应资源包")?).to_string());
        }
    } } }
    Ok(())
}
pub fn valid_id(id: &str) -> bool {
    crate::custom_widgets::valid_widget_id(id)
}
pub fn color(s: &str) -> bool {
    s == "none"
        || (s.len() == 7 && s.starts_with('#') && s[1..].bytes().all(|b| b.is_ascii_hexdigit()))
}
fn text(s: &str, max: usize) -> bool {
    !s.trim().is_empty() && s.encode_utf16().count() <= max
}
fn range(n: f64, min: f64, max: f64) -> bool {
    n.is_finite() && (min..=max).contains(&n)
}
fn shapes(items: &[Shape]) -> Result<(), String> {
    if items.is_empty() || items.len() > 100 {
        return Err("图形数量需为 1–100".into());
    }
    for s in items {
        if s.kind == "image" {
            decode_image(s.src.as_deref().ok_or("植物图片缺少 src")?)?;
            if !s.x.is_some_and(|n|range(n,0.,320.)) || !s.y.is_some_and(|n|range(n,-64.,320.)) || !s.width.is_some_and(|n|range(n,1.,320.)) || !s.height.is_some_and(|n|range(n,1.,320.)) || s.opacity.is_some_and(|n|!range(n,0.,1.)) { return Err("植物图片尺寸无效".into()); }
            continue;
        }
        if !s.fill.as_deref().is_some_and(color)
            || s.stroke.as_ref().is_some_and(|s| !color(s))
            || s.stroke_width.is_some_and(|n| !range(n, 0., 12.))
            || s.opacity.is_some_and(|n| !range(n, 0., 1.))
        {
            return Err("宠物图形样式无效".into());
        }
        match s.kind.as_str() {
            "path" => {
                let d = s.d.as_deref().ok_or("path 缺少 d")?;
                if d.trim().is_empty()
                    || d.len() > 16000
                    || !d.bytes().all(|b| {
                        b.is_ascii_digit()
                            || b.is_ascii_whitespace()
                            || b"MmZzLlHhVvCcSsQqTtAaEe.,+-".contains(&b)
                    })
                {
                    return Err("SVG 路径无效，不能包含标签、脚本或外部引用".into());
                }
            }
            "ellipse" | "rect" => {
                if !s.x.is_some_and(|n| range(n, 0., 320.))
                    || !s.y.is_some_and(|n| range(n, 0., 320.))
                {
                    return Err("宠物坐标范围为 0–320".into());
                }
                if s.kind == "ellipse" {
                    if !s.rx.is_some_and(|n| range(n, 0., 160.))
                        || !s.ry.is_some_and(|n| range(n, 0., 160.))
                    {
                        return Err("椭圆尺寸无效".into());
                    }
                } else if !s.width.is_some_and(|n| range(n, 1., 320.))
                    || !s.height.is_some_and(|n| range(n, 1., 320.))
                    || s.rx.is_some_and(|n| !range(n, 0., 160.))
                {
                    return Err("矩形尺寸无效".into());
                }
            }
            _ => return Err("只支持 path、ellipse、rect 图形".into()),
        }
    }
    Ok(())
}
pub fn validate(theme: &Theme) -> Result<(), String> {
    if ![1,2].contains(&theme.schema_version)
        || !valid_id(&theme.id)
        || !text(&theme.name, 24)
        || !text(&theme.description, 160)
        || !color(&theme.color)
        || theme.pets.is_empty()
        || theme.pets.len() > 40
    {
        return Err("宠物主题格式无效".into());
    }
    let limit = if theme.schema_version == 1 { 512 * 1024 } else { MAX_THEME_BYTES };
    if serde_json::to_vec(theme).map_err(|e| e.to_string())?.len() > limit {
        return Err("宠物主题文件过大".into());
    }
    let mut image_bytes = 0usize;
    let mut image_count = 0usize;
    let mut ids = std::collections::HashSet::new();
    for p in &theme.pets {
        if !valid_id(&p.id)
            || !ids.insert(&p.id)
            || !text(&p.name, 24)
            || !text(&p.description, 160)
            || p.kind != "plant"
        {
            return Err("宠物名称、类型或 id 无效".into());
        }
        if p.builtin_plant
            .as_ref()
            .is_some_and(|id| !["pot", "pot-white-flower"].contains(&id.as_str()))
            || p.builtin_plant.is_some() && !p.parts.is_empty()
            || p.builtin_plant.is_none() && p.parts.is_empty()
        {
            return Err("选择内置植物或定义植物部件，花盆与表情由宿主保留".into());
        }
        if p.parts.len() > 16 {
            return Err("一只宠物最多 16 个活动部件".into());
        }
        let mut parts = std::collections::HashSet::new();
        for part in &p.parts {
            if !valid_id(&part.id)
                || !parts.insert(&part.id)
                || !text(&part.name, 24)
                || !part.pivot.iter().all(|n| range(*n, 0., 320.))
                || !["none", "sway", "left", "right"].contains(&part.motion.as_str())
            {
                return Err("活动部件无效".into());
            }
            shapes(&part.shapes)?;
            for s in &part.shapes {
                if s.kind == "image" {
                    if theme.schema_version != 2 { return Err("图片植物需要 schemaVersion=2".into()); }
                    image_bytes += decode_image(s.src.as_deref().unwrap())?.1.len();
                    image_count += 1;
                    if image_bytes > 16 * 1024 * 1024 || image_count > 60 { return Err("植物素材总量超过限制".into()); }
                }
            }
        }
    }
    Ok(())
}
pub fn load(dir: &Path) -> Result<Vec<Theme>, String> {
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut themes = vec![];
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().is_none_or(|x| x != "json") {
            continue;
        }
        let meta = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if !meta.is_file() || meta.len() > MAX_THEME_BYTES as u64 {
            return Err("宠物主题文件无效或过大".into());
        }
        let theme: Theme = serde_json::from_slice(&fs::read(&path).map_err(|e| e.to_string())?)
            .map_err(|e| format!("{}：{e}", path.display()))?;
        validate(&theme)?;
        if builtin().iter().any(|t| t.id == theme.id) {
            return Err("自创主题与官方主题重名".into());
        }
        if path.file_stem().and_then(|s| s.to_str()) != Some(&theme.id) {
            return Err("主题文件名与 id 不一致".into());
        }
        themes.push(theme);
    }
    themes.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(themes)
}
pub fn save(dir: &Path, theme: &Theme, overwrite: bool) -> Result<(), String> {
    validate(theme)?;
    if builtin().iter().any(|t| t.id == theme.id) {
        return Err("不能覆盖官方主题，请使用新的主题 id".into());
    }
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{}.json", theme.id));
    if path.exists() && !overwrite {
        return Err("主题已存在，修改时需明确 overwrite=true".into());
    }
    let temp = dir.join(format!(".{}.tmp", theme.id));
    fs::write(
        &temp,
        serde_json::to_vec_pretty(theme).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::rename(temp, path).map_err(|e| e.to_string())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn official_themes_validate() {
        let themes = builtin();
        assert_eq!(themes.len(), 2);
        for t in themes {
            validate(&t).unwrap();
        }
    }
    #[test]
    fn executable_art_and_external_references_are_rejected() {
        let mut theme = builtin().remove(0);
        theme.pets[2].parts[0].shapes[0].kind = "path".into();
        theme.pets[2].parts[0].shapes[0].fill = Some("url(https://example.com/a.svg)".into());
        assert!(validate(&theme).is_err());
        theme = builtin().remove(0);
        theme.pets[2].parts[0].shapes[0].kind = "path".into();
        theme.pets[2].parts[0].shapes[0].fill = Some("#ffffff".into());
        theme.pets[2].parts[0].shapes[0].d = Some("<script>alert(1)</script>".into());
        assert!(validate(&theme).is_err());
    }
    #[test]
    fn raster_assets_round_trip_and_legacy_themes_still_validate() {
        let original = builtin().remove(0);
        let (mut portable, files) = export_assets(&original).unwrap();
        assert_eq!(files.len(), 2);
        assert!(files.iter().all(|(p, _)|p.starts_with("assets/") && p.ends_with(".webp")));
        import_assets(&mut portable, &files).unwrap();
        assert_eq!(serde_json::to_value(portable).unwrap(), serde_json::to_value(&original).unwrap());
        let mut old = original.clone(); old.schema_version=1; old.pets.truncate(2);
        validate(&old).unwrap();
        let mut wrong_version=original; wrong_version.schema_version=1;
        assert!(validate(&wrong_version).is_err());
    }
    #[test]
    fn assistant_image_references_preserve_bytes_without_echoing_base64() {
        let original=builtin().remove(0);
        let mut compact=original.clone();compact_images(&mut compact).unwrap();
        let text=serde_json::to_string(&compact).unwrap();
        assert!(!text.contains("base64"));assert!(text.len()<5000);
        resolve_image_refs(&mut compact,&[original.clone()]).unwrap();
        assert_eq!(serde_json::to_value(compact).unwrap(),serde_json::to_value(original).unwrap());
    }
    #[test]
    fn missing_external_mislabeled_and_unused_assets_fail_before_save() {
        let original=builtin().remove(0);
        let (theme, files)=export_assets(&original).unwrap();
        assert!(import_assets(&mut theme.clone(), &[]).is_err());
        let mut bad=files.clone(); bad[0].1=b"not a picture".to_vec();
        assert!(import_assets(&mut theme.clone(), &bad).is_err());
        let mut extra=files.clone();extra.push(("assets/unused.webp".into(),files[0].1.clone()));
        assert!(import_assets(&mut theme.clone(), &extra).is_err());
        for src in ["https://example.com/a.webp","assets/../a.webp","data:image/svg+xml;base64,AAAA"] {
            let mut bad=theme.clone();bad.pets[2].parts[0].shapes[0].src=Some(src.into());
            assert!(import_assets(&mut bad, &files).is_err());
        }
    }
}

