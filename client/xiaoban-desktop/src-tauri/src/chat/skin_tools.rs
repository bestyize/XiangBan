//! Discoverable skin actions; image bytes and ZIP processing stay in the native host.
use super::desktop_tools::Desktop;
use agent_kernel::{Risk, Tool, ToolCtx, ToolSpec};
use serde_json::{json, Value};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

struct SkinTool {
    spec: ToolSpec,
    desktop: Arc<dyn Desktop>,
    workspace: PathBuf,
}
impl Tool for SkinTool {
    fn spec(&self) -> &ToolSpec {
        &self.spec
    }
    fn execute(&self, args: Value, ctx: &ToolCtx) -> Result<String, String> {
        if ctx.is_cancelled() {
            return Err("皮肤操作已取消".into());
        }
        let text = |key: &str| {
            args.get(key)
                .and_then(Value::as_str)
                .filter(|v| !v.trim().is_empty())
                .ok_or_else(|| format!("缺少或无效的 {key}"))
        };
        let value = match self.spec.name.as_str() {
            "skin_read_format" => crate::application_skins::read_format(),
            "skin_list" => self.desktop.skin_list()?,
            "skin_apply" => {
                let id = match args.get("id") {
                    Some(Value::Null) => None,
                    Some(Value::String(id)) if id == "default" => None,
                    Some(Value::String(id)) if !id.trim().is_empty() => Some(id.as_str()),
                    _ => return Err("id 必须来自 skin_list，恢复默认请传 default 或 null".into()),
                };
                self.desktop.skin_apply(id)?
            }
            "skin_create" => self.desktop.skin_create(&self.workspace, &args, ctx)?,
            "skin_export" => self.desktop.skin_export(&self.workspace, text("id")?)?,
            _ => return Err("未知皮肤操作".into()),
        };
        Ok(value.to_string())
    }
}

pub(super) fn tools(desktop: Arc<dyn Desktop>, workspace: &Path) -> Vec<Arc<dyn Tool>> {
    let empty = || json!({"type":"object","properties":{},"additionalProperties":false});
    let specs = vec![
        ToolSpec::new("skin_list", "列出小伴界面皮肤（内置与已安装）及当前 activeId。换肤前先查名称和精确 id；这是应用背景场景，与桌面壁纸、桌宠外壳、主题色分别管理。", empty()).risk(Risk::Read).parallel(),
        ToolSpec::new("skin_read_format", "读取程序内置的完整应用皮肤格式、可用示例、限制与创作流程。创作皮肤 ZIP 前必读；无需用户提供模板，不要搜索源码文档。theme.json 配色暂不生效，不能承诺改按钮或文字颜色。", empty()).risk(Risk::Read).parallel(),
        ToolSpec::new("skin_apply", "切换小伴界面皮肤，id 使用 skin_list 的精确编号；default 或 null 恢复默认外观。持久保存并立即通知主界面更新。不改变桌面壁纸、桌宠、明暗模式或主题色。", json!({"type":"object","properties":{"id":{"type":["string","null"]}},"required":["id"],"additionalProperties":false})).risk(Risk::Write).self_managed().serial(),
        ToolSpec::new("skin_create", "创作可导入的应用皮肤 ZIP，先 skin_read_format。简单场景传 id/name/background_image，图片用 generated:<生图返回的 id> 或工作区相对栅格图片路径，原生自动生成配置、WebP 背景和预览；多图层改传 source_dir（工作区皮肤草稿子目录，含完整配置和素材）。两种方式二选一。打包后按真实导入器校验，安装到装扮→皮肤，并在当前工作区生成 ZIP 供交付；不会自动应用，不覆盖已有不同内容或泄露大段图片数据。", json!({
            "type":"object","properties":{
                "source_dir":{"type":"string","description":"工作区皮肤草稿子目录，与图片创建参数互斥"},
                "id":{"type":"string","pattern":"^[a-z0-9-]{1,64}$"},"name":{"type":"string"},"author":{"type":"string"},
                "background_image":{"type":"string","description":"generated:<图片编号> 或当前工作区相对 PNG/JPEG/WebP 路径；不传外链、绝对路径或 base64"}
            },"oneOf":[{"required":["source_dir"],"not":{"anyOf":[{"required":["id"]},{"required":["name"]},{"required":["author"]},{"required":["background_image"]}]}},{"required":["id","name","background_image"],"not":{"required":["source_dir"]}}],"additionalProperties":false
        })).risk(Risk::Write).serial().timeout(std::time::Duration::from_secs(120)),
        ToolSpec::new("skin_export", "将 skin_list 中 builtin=false 的已安装自制皮肤导出为当前工作区内的可导入 ZIP，返回 packagePath 供交付；不改变当前皮肤，不覆盖不同内容。内置皮肤不能导出。", json!({"type":"object","properties":{"id":{"type":"string"}},"required":["id"],"additionalProperties":false})).risk(Risk::Write).serial(),
    ];
    specs
        .into_iter()
        .map(|spec| {
            Arc::new(SkinTool {
                spec,
                desktop: desktop.clone(),
                workspace: workspace.to_path_buf(),
            }) as Arc<dyn Tool>
        })
        .collect()
}
