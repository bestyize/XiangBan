//! Product-owned image generation. Credentials stay native; history holds file references only.
use super::desktop_tools::Desktop;
use agent_kernel::{Risk, Tool, ToolCtx, ToolResult, ToolSpec};
use base64::{engine::general_purpose::STANDARD, Engine};
use image::{ImageFormat, ImageReader, Limits};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{fs, future::Future, io::Cursor, path::{Path, PathBuf}, sync::Arc, time::Duration};
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

const MAX_IMAGE: usize = 16 * 1024 * 1024;
const MAX_REPLY: usize = 48 * 1024 * 1024;
const UNKNOWN: &str = "生图结果未知，上游可能已生成并计费，请勿自动重试";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Request {
    prompt: String,
    #[serde(default = "one")]
    n: u8,
    size: Option<String>,
}
fn one() -> u8 { 1 }
impl Request {
    fn checked(value: Value) -> Result<Self, String> {
        let mut request: Self = serde_json::from_value(value).map_err(|_| "生图参数无效".to_string())?;
        request.prompt = request.prompt.trim().to_string();
        if !(1..=4000).contains(&request.prompt.chars().count()) { return Err("图片描述需要 1 到 4000 个字".into()); }
        if !(1..=4).contains(&request.n) { return Err("每次只能生成 1 到 4 张图片".into()); }
        Ok(request)
    }
    fn payload(&self, capabilities: &Value) -> Result<Value, String> {
        let image = &capabilities["media"]["image"];
        if image["available"].as_bool() != Some(true) {
            if image["requires_paid"].as_bool() == Some(true) { return Err("图片生成需要先兑换付费额度".into()); }
            return Err("小伴 AI 图片生成暂未开放，请在 AI 服务后台「产品与档位 → 媒体能力」启用图片生成".into());
        }
        let sizes = image["sizes"].as_array().ok_or("图片尺寸配置不可用，请联系管理员")?;
        let size = self.size.as_deref().or_else(|| sizes.first().and_then(Value::as_str)).ok_or("尚未配置图片尺寸")?;
        if !sizes.iter().any(|s| s.as_str() == Some(size)) {
            return Err(format!("当前可用的图片尺寸：{}", sizes.iter().filter_map(Value::as_str).collect::<Vec<_>>().join("、")));
        }
        Ok(json!({ "prompt": self.prompt, "n": self.n, "size": size }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeneratedImage { id: String, name: String, mime: String, width: u32, height: u32 }

struct Generate { spec: ToolSpec, desktop: Arc<dyn Desktop> }
pub(super) fn tool(desktop: Arc<dyn Desktop>) -> Arc<dyn Tool> {
    Arc::new(Generate {
        desktop,
        spec: ToolSpec::new("image_generate", "根据用户描述生成图片、绘画、插画或照片，通过小伴 AI 生图服务计费，与当前聊天模型独立。仅文生图，不支持参考图编辑。需要登录且后台已开放图片能力。prompt 描述画面，n 默认 1（最多 4）；size 默认后台首选尺寸，用户未指定时不要猜。返回的图片自动显示在聊天中并保存本机，无需用 Markdown 重复贴图。失败或取消后的结果可能未知，禁止自动重新生成，以免重复计费。", json!({
            "type":"object", "properties":{
                "prompt":{"type":"string","minLength":1,"maxLength":4000},
                "n":{"type":"integer","minimum":1,"maximum":4},
                "size":{"type":"string","description":"仅在用户指定时填写；须在后台开放的尺寸内"}
            }, "required":["prompt"], "additionalProperties":false
        })).risk(Risk::Write).timeout(Duration::from_secs(300)),
    })
}
impl Tool for Generate {
    fn spec(&self) -> &ToolSpec { &self.spec }
    fn execute(&self, args: Value, ctx: &ToolCtx) -> Result<String, String> {
        Request::checked(args.clone())?;
        self.desktop.generate_images(args, ctx).map(|value| value.to_string())
    }
    fn execute_result(&self, args: Value, ctx: &ToolCtx) -> ToolResult {
        if let Err(error) = Request::checked(args.clone()) { return ToolResult::error(error); }
        match self.desktop.generate_images(args, ctx) {
            Ok(value) => {
                let partial = value["warnings"].as_array().is_some_and(|warnings| !warnings.is_empty());
                ToolResult::ok(if partial { "图片已保存在本机，部分生成、保存或资料库入库未完成。请检查结果，勿自动重复生成。" } else { "图片已生成并自动存入资料库，聊天中可预览和另存为。" })
                    .with_structured_content(value)
            }
            Err(error) if error.contains(UNKNOWN) => ToolResult::error(error)
                .with_structured_content(json!({"harness_status":"outcome_unknown"})),
            Err(error) => ToolResult::error(error),
        }
    }
}

async fn cancellable<T>(ctx: &ToolCtx, future: impl Future<Output = Result<T, String>>) -> Result<T, String> {
    let mut future = std::pin::pin!(future);
    loop {
        if ctx.is_cancelled() { return Err("图片生成已停止".into()); }
        if let Ok(result) = tokio::time::timeout(Duration::from_millis(100), &mut future).await { return result; }
    }
}

async fn bounded(mut response: reqwest::Response, limit: usize) -> Result<Vec<u8>, String> {
    if response.content_length().is_some_and(|n| n > limit as u64) { return Err("图片服务返回内容过大".into()); }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| "读取图片服务响应失败")? {
        if bytes.len().saturating_add(chunk.len()) > limit { return Err("图片服务返回内容过大".into()); }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

async fn gateway(client: &reqwest::Client, base: &str, token: &str, payload: Option<&Value>) -> Result<Value, String> {
    let request = if let Some(payload) = payload {
        client.post(format!("{base}/v1/images/generations")).header("Content-Type", "application/json")
            .body(serde_json::to_vec(payload).map_err(|_| "生图参数编码失败")?)
    } else { client.get(format!("{base}/api/ai/tiers")) };
    // Never retry a POST: a transport failure can arrive after the upstream has charged for a picture.
    let response = request.bearer_auth(token).send().await.map_err(|_| if payload.is_some() { UNKNOWN } else { "连接小伴 AI 失败，请检查网络" })?;
    let status = response.status();
    let bytes = bounded(response, if status.is_success() && payload.is_some() { MAX_REPLY } else { 1024 * 1024 }).await
        .map_err(|e| if payload.is_some() { format!("{e}。{UNKNOWN}") } else { e })?;
    if !status.is_success() {
        let value: Value = serde_json::from_slice(&bytes).unwrap_or_default();
        let code = value["code"].as_str().or_else(|| value["error"]["code"].as_str()).unwrap_or_default();
        return Err(match (status.as_u16(), code) {
            (401, _) => "请先登录小伴账号，再使用图片生成".into(),
            (_, "paid_required") => "图片生成需要先兑换付费额度".into(),
            (_, "insufficient_quota") => "小伴 AI 额度不足，请充值后再生成图片".into(),
            (_, "image_unavailable") => "小伴 AI 图片生成暂未开放，请联系管理员配置媒体能力".into(),
            (429, _) => "图片生成请求过于频繁，请稍后再试".into(),
            (500..=599, _) if payload.is_some() => UNKNOWN.into(),
            _ => format!("小伴 AI 图片服务暂不可用（HTTP {}）", status.as_u16()),
        });
    }
    serde_json::from_slice(&bytes).map_err(|_| if payload.is_some() { UNKNOWN.into() } else { "图片能力信息无法识别".into() })
}

fn image_url(raw: &str) -> Result<reqwest::Url, String> {
    let url = reqwest::Url::parse(raw).map_err(|_| "图片下载地址无效")?;
    if url.scheme() != "https" || !url.username().is_empty() || url.password().is_some() || url.host_str().is_none() {
        return Err("图片下载地址必须是不含凭据的 HTTPS 网址".into());
    }
    Ok(url)
}

async fn image_bytes(client: &reqwest::Client, item: &Value) -> Result<Vec<u8>, String> {
    if let Some(encoded) = item["b64_json"].as_str() {
        if encoded.len() > MAX_IMAGE.div_ceil(3) * 4 { return Err("生成的单张图片超过 16 MB".into()); }
        let bytes = STANDARD.decode(encoded).map_err(|_| "生成的图片编码不完整")?;
        if bytes.len() > MAX_IMAGE { return Err("生成的单张图片超过 16 MB".into()); }
        return Ok(bytes);
    }
    let url = image_url(item["url"].as_str().ok_or("图片服务未返回图片内容")?)?;
    // The gateway bearer token must never reach an image CDN; redirects are disabled on this client.
    let response = client.get(url).send().await.map_err(|_| "下载生成图片失败")?;
    if !response.status().is_success() { return Err("下载生成图片失败".into()); }
    bounded(response, MAX_IMAGE).await
}

fn store_image(root: &Path, bytes: &[u8]) -> Result<GeneratedImage, String> {
    if bytes.len() > MAX_IMAGE { return Err("生成的单张图片超过 16 MB".into()); }
    let format = image::guess_format(bytes).map_err(|_| "生图服务未返回有效图片")?;
    let (extension, mime) = match format {
        ImageFormat::Png => ("png", "image/png"), ImageFormat::Jpeg => ("jpg", "image/jpeg"),
        ImageFormat::WebP => ("webp", "image/webp"), _ => return Err("生图服务仅支持 PNG、JPEG 或 WebP 图片".into()),
    };
    let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
    let mut limits = Limits::default();
    limits.max_image_width = Some(8192); limits.max_image_height = Some(8192); limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    let image = reader.decode().map_err(|_| "生成的图片损坏或尺寸过大")?;
    let id = format!("{:x}.{extension}", Sha256::digest(bytes));
    let mut preview = Cursor::new(Vec::new());
    image.thumbnail(768.min(image.width()), 768.min(image.height())).write_to(&mut preview, ImageFormat::Png).map_err(|_| "图片预览生成失败")?;
    fs::create_dir_all(root).map_err(|_| "无法创建生图存储目录")?;
    fs::write(root.join(&id), bytes).map_err(|_| "生成图片保存失败")?;
    fs::write(root.join(format!("{id}.preview.png")), preview.into_inner()).map_err(|_| "图片预览保存失败")?;
    Ok(GeneratedImage { name: format!("小伴生图-{}.{}", &id[..12], extension), id, mime: mime.into(), width: image.width(), height: image.height() })
}

fn root(app: &tauri::AppHandle) -> Result<PathBuf, String> { Ok(super::Dirs::of(app)?.chat.join("generated-images")) }
fn checked_path(root: &Path, id: &str) -> Result<PathBuf, String> {
    let (hash, extension) = id.split_once('.').ok_or("图片编号无效")?;
    if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()) || !["png", "jpg", "webp"].contains(&extension) {
        return Err("图片编号无效".into());
    }
    let path = root.join(id);
    let metadata = path.symlink_metadata().map_err(|_| "图片文件已不存在")?;
    if !metadata.is_file() || metadata.len() > MAX_IMAGE as u64 { return Err("图片引用无效".into()); }
    Ok(path)
}

pub(super) fn generate(app: &tauri::AppHandle, args: Value, ctx: &ToolCtx) -> Result<Value, String> {
    let request = Request::checked(args)?;
    let directory = root(app)?;
    let mut result = tauri::async_runtime::block_on(async {
        let token = cancellable(ctx, async { crate::account::fresh_access_token(app).await.map_err(|_| "请先登录小伴账号，再使用图片生成".into()) }).await?;
        let client = crate::network::async_builder(reqwest::Client::builder().connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(180)).redirect(reqwest::redirect::Policy::none()))?.build().map_err(|_| "无法创建生图连接")?;
        let base = crate::ai_gateway::ai_base_url();
        generate_request(&client, &base, &token, &directory, &request, ctx).await
    })?;
    let ids = result["generated_images"].as_array().into_iter().flatten()
        .filter_map(|image| image["id"].as_str().map(str::to_owned)).collect::<Vec<_>>();
    // Preserve successful generation even when indexing fails. Opening the library retries
    // the local copy only; it must never send another paid generation request.
    let warnings = crate::data_library::generated_images::register(app, &ids)
        .unwrap_or_else(|error| vec![format!("图片已保存在聊天中，资料库入库未完成：{error}")]);
    result["warnings"].as_array_mut().ok_or("生图结果无法读取")?.extend(warnings.into_iter().map(Value::String));
    Ok(result)
}

async fn generate_request(client: &reqwest::Client, base: &str, token: &str, directory: &Path, request: &Request, ctx: &ToolCtx) -> Result<Value, String> {
        let capabilities = cancellable(ctx, gateway(client, base, token, None)).await?;
        let payload = request.payload(&capabilities)?;
        let result = cancellable(ctx, gateway(client, base, token, Some(&payload))).await
            .map_err(|e| if ctx.is_cancelled() { format!("{e}。{UNKNOWN}") } else { e })?;
        let items = result["data"].as_array().filter(|v| !v.is_empty() && v.len() <= request.n as usize).ok_or(UNKNOWN)?;
        let mut images = Vec::new();
        let mut errors = Vec::new();
        if items.len() < request.n as usize { errors.push(format!("请求 {} 张，服务只返回 {} 张", request.n, items.len())); }
        for item in items {
            let saved = match cancellable(ctx, image_bytes(client, item)).await {
                Ok(bytes) => store_image(directory, &bytes), Err(error) => Err(error),
            };
            match saved { Ok(image) => images.push(image), Err(error) => errors.push(error) }
        }
        if images.is_empty() { return Err(format!("{}。{UNKNOWN}", errors.join("；"))); }
        Ok(json!({"generated_images": images, "warnings": errors, "requested":request.n}))
}

#[tauri::command]
pub async fn chat_generated_image_read(window: tauri::WebviewWindow, id: String) -> Result<String, String> {
    super::chat_window(&window)?;
    let directory = root(window.app_handle())?;
    tauri::async_runtime::spawn_blocking(move || {
        checked_path(&directory, &id)?;
        let preview = directory.join(format!("{id}.preview.png"));
        let metadata = preview.symlink_metadata().map_err(|_| "图片预览读取失败")?;
        if !metadata.is_file() || metadata.len() > MAX_IMAGE as u64 { return Err("图片预览无效".into()); }
        let bytes = fs::read(preview).map_err(|_| "图片预览读取失败")?;
        if bytes.len() > MAX_IMAGE { return Err("图片预览过大".into()); }
        Ok(format!("data:image/png;base64,{}", STANDARD.encode(bytes)))
    }).await.map_err(|_| "图片预览读取失败")?
}

#[tauri::command]
pub async fn chat_generated_image_save(window: tauri::WebviewWindow, id: String) -> Result<bool, String> {
    super::chat_window(&window)?;
    let app = window.app_handle().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let path = checked_path(&root(&app)?, &id)?;
        let extension = path.extension().and_then(|v| v.to_str()).ok_or("图片格式无效")?;
        let name = format!("小伴生图-{}.{}", &id[..12], extension);
        let Some(file) = app.dialog().file().set_title("保存生成图片").add_filter("图片", &[extension]).set_file_name(&name).blocking_save_file() else { return Ok(false); };
        let destination = file.into_path().map_err(|_| "请选择本机文件")?;
        if destination != path { fs::copy(&path, destination).map_err(|_| "图片保存失败")?; }
        Ok(true)
    }).await.map_err(|_| "图片保存失败")?
}

#[tauri::command]
pub fn chat_generated_image_open(window: tauri::WebviewWindow, id: String) -> Result<(), String> {
    super::chat_window(&window)?;
    crate::storage::show_in_file_manager(&checked_path(&root(window.app_handle())?, &id)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, Read, Write};
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Temp(PathBuf);
    impl Temp {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            Self(std::env::temp_dir().join(format!("xiaoban-image-test-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed))))
        }
    }
    impl Drop for Temp { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }
    fn png() -> Vec<u8> {
        let mut output = Cursor::new(Vec::new());
        image::DynamicImage::new_rgba8(3, 2).write_to(&mut output, ImageFormat::Png).unwrap();
        output.into_inner()
    }
    fn capabilities() -> Value { json!({"tiers":[], "media":{"image":{"available":true,"requires_paid":false,"sizes":["1024x1024","1536x1024"]}}}) }

    #[test]
    fn validates_requests_and_uses_only_server_offered_sizes() {
        let request = Request::checked(json!({"prompt":"  一只猫  "})).unwrap();
        assert_eq!(request.payload(&capabilities()).unwrap(), json!({"prompt":"一只猫","n":1,"size":"1024x1024"}));
        let custom = Request::checked(json!({"prompt":"猫","size":"1536x1024","n":4})).unwrap();
        assert_eq!(custom.payload(&capabilities()).unwrap()["size"], "1536x1024");
        assert!(Request::checked(json!({"prompt":"猫","size":"8x8"})).unwrap().payload(&capabilities()).is_err());
        assert!(request.payload(&json!({"media":{}})).unwrap_err().contains("暂未开放"));
        assert!(request.payload(&json!({"media":{"image":{"available":false,"requires_paid":true}}})).unwrap_err().contains("付费额度"));
        for invalid in [json!({"prompt":" "}), json!({"prompt":"a".repeat(4001)}), json!({"prompt":"cat","n":0}), json!({"prompt":"cat","n":5}), json!({"prompt":"cat","model":"hidden"}), json!({"prompt":"cat","reference":"data:x"})] {
            assert!(Request::checked(invalid).is_err());
        }
    }

    #[test]
    fn stores_validated_originals_and_previews_without_inline_history_data() {
        let temp = Temp::new();
        let bytes = png();
        let image = store_image(&temp.0, &bytes).unwrap();
        assert_eq!((image.width, image.height), (3, 2));
        assert_eq!(fs::read(checked_path(&temp.0, &image.id).unwrap()).unwrap(), bytes);
        let preview = fs::read(temp.0.join(format!("{}.preview.png", image.id))).unwrap();
        assert_eq!(image::load_from_memory(&preview).unwrap().width(), 3);
        assert_eq!(store_image(&temp.0, &bytes).unwrap().id, image.id);
        let metadata = serde_json::to_string(&image).unwrap();
        assert!(!metadata.contains("base64") && !metadata.contains("data:"));
        for invalid in ["../a.png", "/tmp/a.png", "a.png", &format!("{}.svg", "a".repeat(64)), &format!("{}.png/../x", "a".repeat(64))] {
            assert!(checked_path(&temp.0, invalid).is_err());
        }
        assert!(store_image(&temp.0, b"<svg><script>alert(1)</script></svg>").is_err());
        assert!(store_image(&temp.0, &bytes[..16]).is_err());
    }

    #[test]
    fn download_urls_reject_credentials_and_non_https_schemes() {
        for invalid in ["file:///tmp/image.png", "http://example.com/a.png", "https://user:secret@example.com/a.png", "data:image/png;base64,abc"] { assert!(image_url(invalid).is_err()); }
        assert!(image_url("https://cdn.example.com/a.png?signature=abc").is_ok());
    }

    type Captured = Vec<(String, String, Value)>;
    fn mock(replies: Vec<(u16, Value)>) -> (String, std::thread::JoinHandle<Captured>) {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let thread = std::thread::spawn(move || {
            let mut requests = Vec::new();
            for (status, reply) in replies {
                let (mut stream, _) = listener.accept().unwrap();
                stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
                let mut reader = std::io::BufReader::new(stream.try_clone().unwrap());
                let mut first = String::new(); reader.read_line(&mut first).unwrap();
                let mut headers = String::new();
                let mut length = 0;
                loop {
                    let mut line = String::new(); assert!(reader.read_line(&mut line).unwrap() > 0);
                    if line == "\r\n" { break; }
                    if let Some(n) = line.to_ascii_lowercase().strip_prefix("content-length:") { length = n.trim().parse().unwrap(); }
                    headers.push_str(&line);
                }
                let mut body = vec![0; length]; reader.read_exact(&mut body).unwrap();
                requests.push((first, headers, serde_json::from_slice(&body).unwrap_or(Value::Null)));
                let reply = serde_json::to_vec(&reply).unwrap();
                write!(stream, "HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", reply.len()).unwrap();
                stream.write_all(&reply).unwrap();
            }
            requests
        });
        (base, thread)
    }
    fn client() -> reqwest::Client { reqwest::Client::builder().no_proxy().timeout(Duration::from_secs(5)).redirect(reqwest::redirect::Policy::none()).build().unwrap() }

    #[tokio::test]
    async fn gateway_flow_authenticates_sends_no_model_and_keeps_partial_images() {
        let temp = Temp::new();
        let (base, server) = mock(vec![(200, capabilities()), (200, json!({"data":[{"b64_json":STANDARD.encode(png())},{"b64_json":"bad"}]}))]);
        let request = Request::checked(json!({"prompt":"cat","n":2})).unwrap();
        let result = generate_request(&client(), &base, "test-only-token", &temp.0, &request, &ToolCtx::detached("s1")).await.unwrap();
        assert_eq!(result["generated_images"].as_array().unwrap().len(), 1);
        assert_eq!(result["warnings"].as_array().unwrap().len(), 1);
        assert!(!result.to_string().contains("test-only-token") && !result.to_string().contains("b64_json"));
        let requests = server.join().unwrap();
        assert!(requests[0].0.starts_with("GET /api/ai/tiers "));
        assert!(requests[1].0.starts_with("POST /v1/images/generations "));
        assert!(requests.iter().all(|r| r.1.to_lowercase().contains("authorization: bearer test-only-token")));
        assert_eq!(requests[1].2, json!({"prompt":"cat","n":2,"size":"1024x1024"}));
    }

    #[tokio::test]
    async fn unavailable_capability_stops_before_charging_and_post_errors_are_not_replayed() {
        let temp = Temp::new();
        let request = Request::checked(json!({"prompt":"cat"})).unwrap();
        let ctx = ToolCtx::detached("s1");
        let (base, server) = mock(vec![(200, json!({"media":{}}))]);
        assert!(generate_request(&client(), &base, "test", &temp.0, &request, &ctx).await.unwrap_err().contains("暂未开放"));
        assert_eq!(server.join().unwrap().len(), 1);
        let (base, server) = mock(vec![(200, capabilities()), (502, json!({"message":"upstream failed"}))]);
        assert!(generate_request(&client(), &base, "test", &temp.0, &request, &ctx).await.unwrap_err().contains(UNKNOWN));
        assert_eq!(server.join().unwrap().len(), 2);
        assert!(!temp.0.exists());
    }
}
