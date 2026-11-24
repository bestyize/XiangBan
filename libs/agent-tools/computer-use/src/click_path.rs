//! Run known canvas points in order, under the same approval as the existing click tool.
use agent_kernel::{Part, Tool, ToolCtx, ToolResource, ToolResult, ToolSpec};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

const MAX_POINTS: usize = 32;
type NativeTool = Arc<dyn Tool>;

#[derive(Clone, PartialEq)]
struct Window {
    id: u32,
    bounds: Value,
    title: String,
}

#[derive(Clone)]
struct Observation {
    app: String,
    window: Window,
    pixels: (u32, u32),
}

pub(super) fn install(tools: &mut [NativeTool], activity: Option<super::WindowActivity>) {
    let find = |name: &str| tools.iter().find(|tool| tool.spec().name == name).cloned();
    let (Some(query), Some(click), Some(state)) =
        (find("query"), find("click"), find("get_app_state"))
    else {
        return;
    };
    let methods = &click.spec().schema["properties"]["click_method"]["enum"];
    if !methods
        .as_array()
        .is_some_and(|v| v.iter().any(|m| m == "sky_click"))
        || query.spec().schema["properties"].get("window_id").is_none()
    {
        return;
    }
    let observed = Arc::new(Mutex::new(None));
    let key = find("press_key").filter(|tool| {
        tool.spec().schema["properties"]["key_method"]["enum"]
            .as_array()
            .is_some_and(|methods| methods.iter().any(|m| m == "sky_key"))
    });
    for inner in tools {
        let mut spec = inner.spec().clone();
        match spec.name.as_str() {
            "get_app_state" => spec
                .description
                .push_str(" 有截图时可返回 window_id，供同窗口 click(points) 使用。"),
            "click" => {
                let props = &mut spec.schema["properties"];
                if key.is_some() {
                    props["freeform_new_stroke"] = json!({"type":"boolean","description":"在 com.apple.freeform 新画独立钢笔路径时设 true，同时提供 prepare_buttons。先后台按 Escape 退出旧路径，画完再按 Escape 并回图。"});
                }
                props["window_id"] = json!({"type":"integer","minimum":1,"maximum":u32::MAX,
                    "description":"连续落点时，填写最近一次 get_app_state 截图返回的 window_id。"});
                props["prepare_buttons"] = json!({"type":"array","minItems":1,"maxItems":3,
                    "items":{"type":"string","minLength":1,"maxLength":120},
                    "description":"可选。在画布落点前按顺序选择已知且精确命名的绘图工具按钮，如 [\"Insert Shape\",\"Draw with Pen\"]。引擎只点目标窗口内唯一匹配的按钮。"});
                props["points"] = json!({"type":"array","minItems":1,"maxItems":MAX_POINTS,
                    "description":"同一画布的有序窗口截图坐标，一次执行一笔。终点双击可设 click_count:2。",
                    "items":{"type":"object","required":["x","y"],"additionalProperties":false,
                    "properties":{"x":{"type":"number","minimum":0},"y":{"type":"number","minimum":0},
                    "click_count":{"type":"integer","enum":[1,2]}}}});
                spec.description.push_str(" 已知同一画布的一串落点时，用 {app,window_id,points:[{x,y},{x,y,click_count:2}]} 一次派发，按序后台点击，最后回图。已知绘图工具按钮名时，可将 prepare_buttons 与 points 一起提交。");
            }
            _ => continue,
        }
        *inner = Arc::new(WindowTool {
            spec,
            inner: inner.clone(),
            query: query.clone(),
            key: key.clone(),
            state: state.clone(),
            observed: observed.clone(),
            activity: activity.clone(),
        });
    }
}

struct WindowTool {
    spec: ToolSpec,
    inner: NativeTool,
    query: NativeTool,
    key: Option<NativeTool>,
    state: NativeTool,
    observed: Arc<Mutex<Option<Observation>>>,
    activity: Option<super::WindowActivity>,
}

// Native click accepts an app, not a CGWindowID. Query its *current* window rather
// than querying the requested ID (which would also succeed for a different, covered window).
fn current_window(query: &dyn Tool, app: &str, ctx: &ToolCtx) -> Result<Window, String> {
    let result = query.execute_result(
        json!({"app":app,"role":"AXWindow","limit":1,"max_nodes":1}),
        ctx,
    );
    if result.is_error {
        return Err(result.output);
    }
    let data = result
        .structured_content
        .or_else(|| serde_json::from_str(&result.output).ok())
        .ok_or("引擎没有返回窗口标识")?;
    let id = data["window_id"]
        .as_u64()
        .and_then(|v| u32::try_from(v).ok())
        .filter(|v| *v > 0)
        .ok_or("无法确认当前窗口编号")?;
    let root = &data["matches"][0];
    if root["role"] != "AXWindow"
        || !["x", "y", "w", "h"]
            .iter()
            .all(|k| root["bounds"][k].is_number())
    {
        return Err("当前界面不是可定位的普通窗口".into());
    }
    Ok(Window {
        id,
        title: root["title"].as_str().unwrap_or_default().to_owned(),
        bounds: root["bounds"].clone(),
    })
}

fn png_size(result: &ToolResult) -> Option<(u32, u32)> {
    let url = result.parts.iter().find_map(|part| match part {
        Part::Image { url } => url.strip_prefix("data:image/png;base64,"),
        _ => None,
    })?;
    // The PNG signature and IHDR dimensions occupy the first 24 bytes.
    let mut header = [0; 24];
    STANDARD.decode_slice(url.get(..32)?, &mut header).ok()?;
    if &header[..8] != b"\x89PNG\r\n\x1a\n" || &header[12..16] != b"IHDR" {
        return None;
    }
    Some((
        u32::from_be_bytes(header[16..20].try_into().ok()?),
        u32::from_be_bytes(header[20..24].try_into().ok()?),
    ))
}

impl WindowTool {
    fn observe_window(&self, args: Value, ctx: &ToolCtx) -> ToolResult {
        // Clear the old binding even if the next observation fails or is tree-only.
        self.observed.lock().unwrap().take();
        let app = args["app"].as_str().unwrap_or_default().to_owned();
        if args["include_screenshot"] == false || app.is_empty() {
            return self.inner.execute_result(args, ctx);
        }
        let before = current_window(self.query.as_ref(), &app, ctx);
        let mut result = self.inner.execute_result(args, ctx);
        if !result.is_error {
            if let (Ok(window), Some(pixels)) = (before, png_size(&result)) {
                if current_window(self.query.as_ref(), &app, ctx).ok().as_ref() == Some(&window) {
                    result.output.push_str(&format!(
                        "\nwindow_id: {} ({}×{} px)",
                        window.id, pixels.0, pixels.1
                    ));
                    *self.observed.lock().unwrap() = Some(Observation {
                        app,
                        window,
                        pixels,
                    });
                }
            }
        }
        result
    }

    fn check_target(&self, binding: &Observation, ctx: &ToolCtx) -> Result<(), String> {
        if ctx.is_cancelled() {
            return Err("已取消".into());
        }
        if current_window(self.query.as_ref(), &binding.app, ctx)? != binding.window {
            return Err("窗口编号、标题或引擎报告的尺寸位置已改变".into());
        }
        Ok(())
    }

    fn path(&self, args: &Value, ctx: &ToolCtx) -> ToolResult {
        let binding = self.observed.lock().unwrap().clone();
        let Some(binding) = binding else {
            return ToolResult::error("先用 get_app_state 读取目标窗口截图和 window_id");
        };
        if args["app"].as_str() != Some(&binding.app)
            || args["window_id"].as_u64() != Some(u64::from(binding.window.id))
        {
            return ToolResult::error("应用或 window_id 与最近一次截图不符，请重新观察目标窗口");
        }
        let actions = match planned_clicks(args, binding.pixels) {
            Ok(actions) => actions,
            Err(e) => return ToolResult::error(e),
        };
        if ctx.is_cancelled() {
            return stopped(0, "已取消", false);
        }
        // The native engine binds coordinate clicks to its most recent screenshot.
        // An AX-only read here invalidates that reference for a path without buttons.
        let state = self
            .state
            .execute_result(json!({"app":binding.app,"include_screenshot":true}), ctx);
        if state.is_error {
            return stopped(0, &state.output, false);
        }
        if png_size(&state) != Some(binding.pixels)
            || !same_canvas(&state.output, &binding.window.title)
        {
            return stopped(0, "当前界面已变化，请重新确认窗口和画布", false);
        }
        let buttons = match args.get("prepare_buttons") {
            None => &[][..],
            Some(v) => match v.as_array() {
                Some(b)
                    if (1..=3).contains(&b.len())
                        && b.iter().all(|v| {
                            v.as_str().is_some_and(|s| {
                                !s.trim().is_empty()
                                    && s.chars().count() <= 120
                                    && !s.chars().any(char::is_control)
                            })
                        }) =>
                {
                    b.as_slice()
                }
                _ => return ToolResult::error("绘图工具选择只接受 1–3 个精确按钮名"),
            },
        };
        let new_stroke = args["freeform_new_stroke"] == true;
        if new_stroke
            && (binding.app != "com.apple.freeform" || buttons.is_empty() || self.key.is_none())
        {
            return ToolResult::error(
                "freeform_new_stroke 需要无边记的窗口截图、后台按键及钢笔工具按钮名",
            );
        }
        if let Err(reason) = self.check_target(&binding, ctx) {
            return stopped(0, &reason, false);
        }
        let _indicator = self.activity.as_ref().map(|show| show(binding.window.id));
        if new_stroke {
            if let Err(result) = self.exit_pen(&binding, ctx, 0) {
                return result;
            }
        }
        for (i, button) in buttons.iter().enumerate() {
            if let Err(reason) = self.check_target(&binding, ctx) {
                return stopped(
                    0,
                    &format!("已选择 {i} 个工具按钮；{reason}"),
                    new_stroke || i > 0,
                );
            }
            let result = self.query.execute_result(
                json!({"app":binding.app,"window_id":binding.window.id,
                "role":"AXButton","text":button,"exact":true,"limit":2}),
                ctx,
            );
            let index = exact_button(&result, binding.window.id);
            let Some(index) = index else {
                return stopped(
                    0,
                    &format!(
                        "已选择 {i} 个工具按钮；当前窗口没有唯一可用的 {button}，请回读工具面板"
                    ),
                    new_stroke || i > 0,
                );
            };
            if ctx.is_cancelled() {
                return stopped(0, "已取消", new_stroke || i > 0);
            }
            let result = self.inner.execute_result(json!({"app":binding.app,"element_index":index.to_string(),"click_method":"accessibility"}),ctx);
            if result.is_error {
                return stopped(
                    0,
                    &format!("选择 {button} 时发生错误：{}", result.output),
                    true,
                );
            }
            if !window_header(&result.output, &binding.window.title)
                || (i + 1 == buttons.len() && !same_canvas(&result.output, &binding.window.title))
            {
                return stopped(
                    0,
                    &format!("已选择 {} 个工具按钮；当前界面需要重新观察", i + 1),
                    true,
                );
            }
        }
        let mut last = ToolResult::ok("");
        for i in 0..=actions.len() {
            if let Err(reason) = self.check_target(&binding, ctx) {
                return stopped(i, &reason, !buttons.is_empty());
            }
            if i == actions.len() {
                break;
            }
            if ctx.is_cancelled() {
                return stopped(i, "已取消", !buttons.is_empty());
            }
            last = self.inner.execute_result(actions[i].clone(), ctx);
            if last.is_error {
                return stopped(i, &last.output, true);
            }
            // A sheet, menu or different window is a new decision point. Retain
            // only the last successful frame; discard intermediate trees and images.
            if png_size(&last) != Some(binding.pixels)
                || !same_canvas(&last.output, &binding.window.title)
            {
                return stopped(i, "动作回图的窗口、尺寸或交互界面已变化", true);
            }
        }
        if new_stroke {
            last = match self.exit_pen(&binding, ctx, actions.len()) {
                Ok(frame) => frame,
                Err(result) => return result,
            };
        }
        last.output.insert_str(
            0,
            &format!(
                "已顺序派发 {} 个落点（window_id {}）。请按最终截图验收。\n",
                actions.len(),
                binding.window.id
            ),
        );
        last
    }

    fn exit_pen(
        &self,
        binding: &Observation,
        ctx: &ToolCtx,
        completed: usize,
    ) -> Result<ToolResult, ToolResult> {
        let key = self
            .key
            .as_ref()
            .ok_or_else(|| stopped(completed, "后台按键不可用", false))?;
        let result = key.execute_result(
            json!({"app":binding.app,"key":"Escape","key_method":"sky_key"}),
            ctx,
        );
        if result.is_error {
            return Err(stopped(completed, &result.output, true));
        }
        if png_size(&result) != Some(binding.pixels)
            || !same_canvas(&result.output, &binding.window.title)
        {
            return Err(stopped(completed, "退出钢笔后的画布需要重新确认", true));
        }
        self.check_target(binding, ctx)
            .map_err(|e| stopped(completed, &e, true))?;
        Ok(result)
    }
}

fn exact_button(result: &ToolResult, id: u32) -> Option<u64> {
    if result.is_error {
        return None;
    }
    let data = result
        .structured_content
        .clone()
        .or_else(|| serde_json::from_str(&result.output).ok())?;
    let [button] = data["matches"].as_array()?.as_slice() else {
        return None;
    };
    if data["window_id"] != id
        || data["truncated"] != false
        || button["role"] != "AXButton"
        || button["enabled"] == false
    {
        return None;
    }
    button["index"].as_u64()
}

fn window_header(output: &str, title: &str) -> bool {
    let mut lines = output.lines();
    lines.next().is_some_and(|s| s.starts_with("App="))
        && lines
            .next()
            .is_some_and(|s| s.starts_with(&format!("Window: \"{title}\", App: ")))
}

fn same_canvas(output: &str, title: &str) -> bool {
    let mut lines = output.lines().skip(2);
    if !window_header(output, title)
        || !lines
            .next()
            .is_some_and(|line| line.starts_with("0 standard window "))
    {
        return false;
    }
    !lines.any(|line| {
        line.trim_start()
            .split_once(' ')
            .is_some_and(|(idx, rest)| {
                idx.parse::<u32>().is_ok()
                    && (matches!(
                        rest.split_whitespace().next(),
                        Some("sheet" | "dialog" | "popover")
                    ) || rest == "menu"
                        || rest.starts_with("menu Secondary Actions:"))
            })
    })
}

fn planned_clicks(args: &Value, (width, height): (u32, u32)) -> Result<Vec<Value>, String> {
    let obj = args.as_object().ok_or("缺少落点参数")?;
    if !obj.keys().all(|k| {
        matches!(
            k.as_str(),
            "app"
                | "window_id"
                | "points"
                | "click_method"
                | "prepare_buttons"
                | "freeform_new_stroke"
        )
    }) || obj.get("click_method").is_some_and(|m| m != "sky_click")
    {
        return Err("连续落点使用后台 sky_click，其余参数请按 points 模式的定义填写".into());
    }
    let points = args["points"]
        .as_array()
        .filter(|p| (1..=MAX_POINTS).contains(&p.len()))
        .ok_or("每次需提供 1–32 个落点")?;
    let app = &args["app"];
    let mut actions = Vec::with_capacity(points.len());
    for (i, point) in points.iter().enumerate() {
        let fields = point.as_object().is_some_and(|v| {
            v.keys()
                .all(|k| matches!(k.as_str(), "x" | "y" | "click_count"))
        });
        let count = point.get("click_count").map_or(Some(1), Value::as_u64);
        let within = |key: &str, limit: u32| {
            point[key]
                .as_f64()
                .is_some_and(|v| (0.0..f64::from(limit)).contains(&v))
        };
        if !fields || !matches!(count, Some(1 | 2)) || !within("x", width) || !within("y", height) {
            return Err(format!(
                "第 {} 个落点无效，请核对截图范围 {width}×{height} 及单击或双击次数",
                i + 1
            ));
        }
        actions.push(json!({"app":app, "x":point["x"], "y":point["y"], "click_count":count, "click_method":"sky_click"}));
    }
    Ok(actions)
}

fn stopped(completed: usize, reason: &str, attempted: bool) -> ToolResult {
    let text = format!(
        "已派发并收到回图的前序落点数 {completed}；{reason}。请重新读取窗口，核实已生效的落点。"
    );
    if attempted || completed > 0 {
        ToolResult::unknown(&text)
    } else {
        ToolResult::error(text)
    }
}

impl Tool for WindowTool {
    fn spec(&self) -> &ToolSpec {
        &self.spec
    }
    fn resource(&self, args: &Value) -> Result<Option<ToolResource>, String> {
        self.inner.resource(args)
    }
    fn execute(&self, args: Value, ctx: &ToolCtx) -> Result<String, String> {
        let r = self.execute_result(args, ctx);
        if r.is_error {
            Err(r.output)
        } else {
            Ok(r.output)
        }
    }
    fn execute_result(&self, args: Value, ctx: &ToolCtx) -> ToolResult {
        match self.spec.name.as_str() {
            "get_app_state" => self.observe_window(args, ctx),
            "click" if args.get("points").is_some() => {
                let result = self.path(&args, ctx);
                if result.is_error {
                    self.observed.lock().unwrap().take();
                }
                result
            }
            "click"
                if args.get("window_id").is_some()
                    || args.get("prepare_buttons").is_some()
                    || args.get("freeform_new_stroke").is_some() =>
            {
                ToolResult::error("单点指定 window_id 时也请放入 points 数组")
            }
            _ => self.inner.execute_result(args, ctx),
        }
    }
    fn on_run_end(&self, end: &agent_kernel::RunEnd) -> Result<(), String> {
        self.inner.on_run_end(end)
    }
}

#[cfg(test)]
mod tests;
