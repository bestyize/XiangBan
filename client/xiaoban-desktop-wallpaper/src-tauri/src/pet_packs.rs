//! Declarative, code-free pet themes shared with the frontend renderer.
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
pub const MAX_THEME_BYTES: usize = 512 * 1024;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Shape {
    #[serde(rename = "type")]
    pub kind: String,
    pub fill: String,
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
    serde_json::from_str::<Catalog>(include_str!("../../public/pets/catalog.json"))
        .expect("shipped pet themes")
        .themes
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
        if !color(&s.fill)
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
    if theme.schema_version != 1
        || !valid_id(&theme.id)
        || !text(&theme.name, 24)
        || !text(&theme.description, 160)
        || !color(&theme.color)
        || theme.pets.is_empty()
        || theme.pets.len() > 40
    {
        return Err("宠物主题格式无效".into());
    }
    if serde_json::to_vec(theme).map_err(|e| e.to_string())?.len() > MAX_THEME_BYTES {
        return Err("宠物主题不能超过 512 KiB".into());
    }
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
        theme.pets[2].parts[0].shapes[0].fill = "url(https://example.com/a.svg)".into();
        assert!(validate(&theme).is_err());
        theme = builtin().remove(0);
        theme.pets[2].parts[0].shapes[0].d = Some("<script>alert(1)</script>".into());
        assert!(validate(&theme).is_err());
    }
}
