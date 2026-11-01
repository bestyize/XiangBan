//! Download original Baidu files into the ordinary library; no token or dlink enters the webview/index.
use super::*;
use std::{
    collections::HashMap,
    io::Write,
    sync::{Arc, OnceLock},
    time::{Duration, Instant},
};
use url::Url;
const BUDGET: Duration = Duration::from_secs(600);
static ACTIVE: OnceLock<Mutex<HashMap<String, Arc<AtomicBool>>>> = OnceLock::new();
fn active() -> &'static Mutex<HashMap<String, Arc<AtomicBool>>> {
    ACTIVE.get_or_init(Mutex::default)
}
struct Lease {
    id: String,
    cancelled: Arc<AtomicBool>,
}
impl Lease {
    fn new(id: &str) -> Result<Self, String> {
        let mut downloads = active().lock().unwrap_or_else(|e| e.into_inner());
        if downloads.contains_key(id) {
            return Err("这份文件正在下载，请稍候".into());
        }
        let cancelled = Arc::new(AtomicBool::new(false));
        downloads.insert(id.into(), cancelled.clone());
        Ok(Self {
            id: id.into(),
            cancelled,
        })
    }
}
impl Drop for Lease {
    fn drop(&mut self) {
        active()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&self.id);
    }
}
struct Temporary(PathBuf);
impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
fn check(cancelled: &AtomicBool) -> Result<(), String> {
    if cancelled.load(Ordering::SeqCst) {
        Err("下载已取消".into())
    } else {
        Ok(())
    }
}
fn download_url(raw: &str) -> Result<Url, String> {
    let url = Url::parse(raw).map_err(|_| "百度未返回有效的下载链接")?;
    let trusted = url.host_str().is_some_and(|h| {
        h == "pcs.baidu.com"
            || h.ends_with(".pcs.baidu.com")
            || h == "pcs-data.baidu.com"
            || h == "baidupcs.com"
            || h.ends_with(".baidupcs.com")
    });
    if !trusted
        || url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port_or_known_default() != Some(443)
    {
        return Err("百度未返回受支持的安全下载链接，请在源站查看".into());
    }
    Ok(url)
}
struct Meta {
    name: String,
    size: u64,
    url: Url,
}
fn metadata(value: &Value, id: &str) -> Result<Meta, String> {
    if service_code(value, "errno").unwrap_or(-1) != 0 {
        return Err(match service_code(value, "errno") {
            Some(-6 | 110 | 111) => "百度下载授权已失效，请在连接器中重新连接",
            Some(6 | 31066) => "百度下载权限不足，请检查账号授权或源文件访问权限",
            _ => "百度暂未提供下载链接，请稍后重试或在源站下载",
        }
        .into());
    }
    let row = value["list"]
        .as_array()
        .and_then(|rows| {
            rows.iter()
                .find(|row| identifier(row, &["fs_id", "fsid"]) == id)
        })
        .ok_or("没有找到所选文件，请确认文件仍存在")?;
    if row["isdir"] == 1 || row["isdir"] == true {
        return Err("请选择文件，暂不下载整个文件夹".into());
    }
    let name = text(row, &["filename", "server_filename"])
        .or_else(|| row["path"].as_str().and_then(|s| s.rsplit('/').next()))
        .ok_or("百度未返回文件名")?;
    let name = clean_name(name)?;
    let size = row["size"].as_u64().ok_or("百度未返回有效的文件大小")?;
    if size > MAX_IMPORT {
        return Err("文件超过资料库的 200MB 导入上限，请在源站下载".into());
    }
    Ok(Meta {
        name,
        size,
        url: download_url(
            row["dlink"]
                .as_str()
                .ok_or("百度未提供此文件的下载链接，请检查账号下载权限")?,
        )?,
    })
}
fn network_error(error: reqwest::Error, cancelled: &AtomicBool) -> String {
    if cancelled.load(Ordering::SeqCst) {
        "下载已取消".into()
    } else if error.is_timeout() {
        "百度文件下载超时，请重试".into()
    } else {
        "百度文件下载失败，请检查网络后重试".into()
    }
}
struct Sink<'a, W: Write> {
    writer: W,
    received: u64,
    expected: u64,
    limit: u64,
    cancelled: &'a AtomicBool,
}
impl<W: Write> Sink<'_, W> {
    fn push(&mut self, bytes: &[u8]) -> Result<(), String> {
        check(self.cancelled)?;
        let size = self.received + bytes.len() as u64;
        if size > self.limit || size > self.expected {
            return Err("下载文件大小与云端信息不符，未加入资料库，请重试".into());
        }
        self.writer
            .write_all(bytes)
            .map_err(|_| "本机空间不足或无法保存下载文件")?;
        self.received = size;
        Ok(())
    }
    fn finish(mut self) -> Result<(), String> {
        check(self.cancelled)?;
        if self.received != self.expected {
            return Err("文件没有下载完整，未加入资料库，请重试".into());
        }
        self.writer.flush().map_err(|_| "无法保存下载文件")?;
        Ok(())
    }
}
/// Poll cancellation even while DNS, headers or the next body chunk are pending.
async fn receive<T>(
    operation: impl std::future::Future<Output = Result<T, reqwest::Error>>,
    cancelled: &AtomicBool,
) -> Result<T, String> {
    let mut operation = Box::pin(operation);
    let mut tick = tokio::time::interval(Duration::from_millis(100));
    std::future::poll_fn(|cx| {
        if let Err(error) = check(cancelled) {
            return std::task::Poll::Ready(Err(error));
        }
        if let std::task::Poll::Ready(result) = operation.as_mut().poll(cx) {
            return std::task::Poll::Ready(result.map_err(|e| network_error(e, cancelled)));
        }
        while tick.poll_tick(cx).is_ready() {}
        std::task::Poll::Pending
    })
    .await
}
async fn fetch(
    app: &tauri::AppHandle,
    id: &str,
    reference: &Reference,
    cancelled: &AtomicBool,
) -> Result<(String, Temporary), String> {
    let fsid = reference
        .id
        .parse::<u64>()
        .map_err(|_| "百度文件编号无效，请重新添加")?;
    let token = crate::chat::connectors::baidu::download_access_token(app)?;
    check(cancelled)?;
    let client = crate::network::async_builder(reqwest::Client::builder())
        .map_err(|_| "网络配置不可用")?
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(15))
        .read_timeout(Duration::from_secs(30))
        .timeout(BUDGET)
        .user_agent("pan.baidu.com XiangBan/3.0")
        .build()
        .map_err(|_| "无法初始化下载连接")?;
    let deadline = Instant::now() + BUDGET;
    let mut metadata_url = Url::parse("https://pan.baidu.com/rest/2.0/xpan/multimedia")
        .map_err(|_| "下载地址不可用")?;
    metadata_url.query_pairs_mut().extend_pairs([
        ("method", "filemetas"),
        ("fsids", &format!("[{fsid}]")),
        ("dlink", "1"),
        ("access_token", token.as_str()),
    ]);
    let mut response = receive(
        client
            .get(metadata_url)
            .timeout(Duration::from_secs(30))
            .send(),
        cancelled,
    )
    .await?;
    if !response.status().is_success() {
        return Err("百度下载服务暂不可用，请稍后重试".into());
    }
    let mut raw = Vec::new();
    while let Some(chunk) = receive(response.chunk(), cancelled).await? {
        if raw.len() + chunk.len() > 256 * 1024 {
            return Err("云端文件信息过大，未开始下载".into());
        }
        raw.extend_from_slice(&chunk);
    }
    let value: Value =
        serde_json::from_slice(&raw).map_err(|_| "百度下载服务返回格式异常，请稍后重试")?;
    let meta = metadata(&value, &reference.id)?;
    let mut url = meta.url;
    let mut response = None;
    for _ in 0..6 {
        check(cancelled)?;
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err("百度文件下载超时，请重试".into());
        }
        if !url.query_pairs().any(|(k, _)| k == "access_token") {
            url.query_pairs_mut().append_pair("access_token", &token);
        }
        let next = receive(client.get(url.clone()).timeout(remaining).send(), cancelled).await?;
        if next.status().is_redirection() {
            let location = next
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|v| v.to_str().ok())
                .ok_or("百度未返回有效的下载跳转")?;
            let next_url = url.join(location).map_err(|_| "百度下载跳转无效")?;
            url = download_url(next_url.as_str())?;
            continue;
        }
        if !next.status().is_success() {
            return Err("百度拒绝下载此文件，请检查下载权限或稍后重试".into());
        }
        if next
            .content_length()
            .is_some_and(|n| n > MAX_IMPORT || n != meta.size)
        {
            return Err("云端下载大小不符，未加入资料库，请重试".into());
        }
        response = Some(next);
        break;
    }
    let mut response = response.ok_or("百度下载跳转过多，请稍后重试")?;
    let directory = root(app)?.join(".downloads");
    fs::create_dir_all(&directory).map_err(|_| "无法创建下载目录")?;
    let temporary = Temporary(directory.join(format!("{}.part", new_id(app))));
    let output = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary.0)
        .map_err(|_| "无法保存下载文件")?;
    let mut sink = Sink {
        writer: output,
        received: 0,
        expected: meta.size,
        limit: MAX_IMPORT,
        cancelled,
    };
    let mut last = Instant::now() - Duration::from_secs(1);
    while let Some(chunk) = receive(response.chunk(), cancelled).await? {
        sink.push(&chunk)?;
        if last.elapsed() >= Duration::from_millis(300) || sink.received == meta.size {
            last = Instant::now();
            let _ = app.emit(
                "library-cloud-download-progress",
                json!({"id":id,"received":sink.received,"total":meta.size}),
            );
        }
    }
    sink.finish()?;
    Ok((meta.name, temporary))
}
/// Large cloud originals remain intact; chat receives their bounded text representation.
pub(crate) fn needs_text_attachment(doc: &Doc) -> bool {
    doc.size > MAX_DOCUMENT
        && doc.size <= MAX_IMPORT
        && OFFICE.contains(&doc.ext.as_str())
        && valid_url("baidu", &doc.origin)
}
fn enable_text_reading(doc: &mut Doc) {
    if needs_text_attachment(doc) && doc.status == "tooLarge" {
        doc.status = "reading".into();
        doc.note = "原文件已保存，正在提取用于提问的文字".into();
    }
}
pub(crate) fn text_attachment(doc: &Doc, path: &Path) -> Result<Value, String> {
    if !needs_text_attachment(doc) {
        return Err("这份文件不支持提取文字附件".into());
    }
    if fs::metadata(path).map_err(|_| "原文件已不在资料库")?.len() > MAX_IMPORT {
        return Err("文件超过资料库解析上限".into());
    }
    let text = agent_file_tools::extract_document_text(
        path,
        &doc.ext,
        &ToolCtx::detached("library-cloud-original"),
    )
    .map_err(|_| "未能提取原文件文字，可能是扫描件、加密文件或内容过大")?;
    let snapshot = content::snapshot(
        text,
        true,
        "从资料库原文件提取的文字，不包含图片、图表或扫描页；请勿声称覆盖原文件所有内容。",
    );
    let link = Link {
        title: format!("{}（提取文字）", doc.name),
        url: doc.origin.clone(),
        ..Default::default()
    };
    Ok(content::question(&link, &snapshot, "ask")?["attachments"][0].clone())
}
pub(super) fn local_document(library: &Library, link: &Link) -> Option<Doc> {
    library
        .docs
        .iter()
        .find(|doc| doc.origin == link.url)
        .cloned()
}
#[tauri::command]
pub fn library_cloud_download_cancel(
    window: tauri::WebviewWindow,
    id: String,
) -> Result<(), String> {
    control_only(&window)?;
    if let Some(cancelled) = active().lock().unwrap_or_else(|e| e.into_inner()).get(&id) {
        cancelled.store(true, Ordering::SeqCst);
    }
    Ok(())
}
#[tauri::command]
pub async fn library_cloud_download(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    id: String,
) -> Result<Doc, String> {
    control_only(&window)?;
    let link = content::saved(&app, &id)?;
    let reference = link
        .remote
        .as_ref()
        .filter(|r| r.provider == "baidu")
        .ok_or("当前仅支持下载百度网盘原文件")?
        .clone();
    if let Some(mut doc) = local_document(&load(&root(&app)?), &link) {
        if needs_text_attachment(&doc) && doc.status == "tooLarge" {
            change(&app, |library, _| {
                let current = library
                    .docs
                    .iter_mut()
                    .find(|d| d.id == doc.id)
                    .ok_or("原文件已移除")?;
                enable_text_reading(current);
                doc = current.clone();
                Ok(())
            })?;
            read_pending(&app);
        }
        return Ok(doc);
    }
    let lease = Lease::new(&id)?;
    let app2 = app.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let _lease = lease;
        let cancelled = &_lease.cancelled;
        let (name, temporary) =
            tauri::async_runtime::block_on(fetch(&app2, &id, &reference, cancelled))?;
        check(cancelled)?;
        let (doc, _) = change(&app2, |library, root| {
            let current = library
                .links
                .iter()
                .find(|l| l.id == id && l.remote == link.remote)
                .cloned()
                .ok_or("云端引用已移除，下载未加入资料库")?;
            if let Some(doc) = local_document(library, &current) {
                return Ok(doc);
            }
            folder_exists(library, &current.folder)?;
            let doc_id = copy_in(
                &app2,
                library,
                root,
                &name,
                None,
                Some(&temporary.0),
                current.folder.clone(),
            )?;
            let doc = library
                .docs
                .iter_mut()
                .find(|d| d.id == doc_id)
                .ok_or("保存文档失败")?;
            doc.origin = current.url;
            enable_text_reading(doc);
            Ok(doc.clone())
        })?;
        Ok(doc)
    })
    .await
    .map_err(|_| "下载任务已中断".to_string())?;
    if result.is_ok() {
        read_pending(&app);
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn metadata_is_bound_to_file_identity_and_safe_provider_urls() {
        let value = json!({"errno":0,"list":[{"fs_id":9007199254740993u64,"filename":"原文.pdf","size":4,"isdir":0,"dlink":"https://d.pcs.baidu.com/file/test"}]});
        assert_eq!(
            metadata(&value, "9007199254740993").unwrap().name,
            "原文.pdf"
        );
        assert!(metadata(&value, "1").is_err());
        let mut invalid = value.clone();
        invalid["list"][0]["size"] = json!(MAX_IMPORT + 1);
        assert!(metadata(&invalid, "9007199254740993").is_err());
        invalid = value.clone();
        invalid["list"][0]["filename"] = json!("../outside.pdf");
        assert!(metadata(&invalid, "9007199254740993").is_err());
        invalid = value.clone();
        invalid["list"][0]["isdir"] = json!(1);
        assert!(metadata(&invalid, "9007199254740993").is_err());

        for url in [
            "http://d.pcs.baidu.com/file/x",
            "https://evil.example/x",
            "https://d.pcs.baidu.com.evil.example/x",
            "https://user:secret@d.pcs.baidu.com/x",
            "https://127.0.0.1/x",
        ] {
            assert!(download_url(url).is_err());
        }
        assert!(download_url("https://njall01.baidupcs.com/file/x").is_ok());
        assert!(metadata(&json!({"errno":111}), "1")
            .err()
            .unwrap()
            .contains("授权"));
    }
    #[test]
    fn stream_rejects_truncation_overflow_and_cancellation_without_publishing() {
        let cancelled = AtomicBool::new(false);
        let mut output = Vec::new();
        {
            let mut sink = Sink {
                writer: &mut output,
                received: 0,
                expected: 4,
                limit: 10,
                cancelled: &cancelled,
            };
            sink.push(b"bo").unwrap();
            sink.push(b"dy").unwrap();
            sink.finish().unwrap();
        }
        assert_eq!(output, b"body");
        let mut sink = Sink {
            writer: Vec::new(),
            received: 0,
            expected: 4,
            limit: 10,
            cancelled: &cancelled,
        };
        sink.push(b"bad").unwrap();
        assert!(sink.finish().is_err());
        let mut sink = Sink {
            writer: Vec::new(),
            received: 0,
            expected: 4,
            limit: 10,
            cancelled: &cancelled,
        };
        assert!(sink.push(b"too large").is_err());
        let mut sink = Sink {
            writer: Vec::new(),
            received: 0,
            expected: 4,
            limit: 3,
            cancelled: &cancelled,
        };
        assert!(sink.push(b"body").is_err());
        cancelled.store(true, Ordering::SeqCst);
        assert!(sink.push(b"x").err().unwrap().contains("取消"));
    }
    #[tokio::test]
    async fn stalled_request_can_be_cancelled_without_waiting_for_network_timeout() {
        let cancelled = Arc::new(AtomicBool::new(false));
        let flag = cancelled.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(30)).await;
            flag.store(true, Ordering::SeqCst);
        });
        let result = tokio::time::timeout(
            Duration::from_secs(1),
            receive(
                std::future::pending::<Result<(), reqwest::Error>>(),
                &cancelled,
            ),
        )
        .await
        .unwrap();
        assert!(result.err().unwrap().contains("取消"));
    }
    #[test]
    fn oversized_cloud_pdf_is_sent_as_real_text_with_source_and_coverage() {
        let path =
            std::env::temp_dir().join(format!("cloud-original-text-{}.pdf", std::process::id()));
        fs::write(&path, simple_pdf("Original download text")).unwrap();
        let _temporary = Temporary(path.clone());
        let mut doc = Doc {
            name: "book.pdf".into(),
            ext: "pdf".into(),
            size: MAX_DOCUMENT + 1,
            origin: "https://pan.baidu.com/disk/main#/index?fileid=1".into(),
            status: "tooLarge".into(),
            ..Default::default()
        };
        enable_text_reading(&mut doc);
        assert_eq!(doc.status, "reading");
        let attachment = text_attachment(&doc, &path).unwrap();
        use base64::Engine;
        let body = String::from_utf8(
            base64::engine::general_purpose::STANDARD
                .decode(
                    attachment["dataUrl"]
                        .as_str()
                        .unwrap()
                        .split_once(',')
                        .unwrap()
                        .1,
                )
                .unwrap(),
        )
        .unwrap();
        assert!(body.contains("Original download text"));
        assert!(body.contains(&doc.origin));
        assert!(body.contains("不是全文"));
        assert_eq!(attachment["mime"], "text/plain");
        doc.origin.clear();
        assert!(!needs_text_attachment(&doc));
    }
    #[test]
    fn an_existing_local_original_is_reused_without_another_download() {
        let link = Link {
            url: "https://pan.baidu.com/disk/main#/index?fileid=1".into(),
            ..Default::default()
        };
        let mut library = Library::default();
        library.docs.push(Doc {
            id: "local".into(),
            origin: link.url.clone(),
            ..Default::default()
        });
        assert_eq!(local_document(&library, &link).unwrap().id, "local");
        library.docs.clear();
        assert!(local_document(&library, &link).is_none());
    }
    #[test]
    fn temporary_files_and_active_slots_are_released_on_failure() {
        let path = std::env::temp_dir().join(format!("xiaoban-cloud-test-{}", std::process::id()));
        fs::write(&path, b"partial").unwrap();
        {
            let _guard = Temporary(path.clone());
        }
        assert!(!path.exists());
        {
            let _lease = Lease::new("test-slot").unwrap();
            assert!(Lease::new("test-slot").is_err());
        }
        assert!(Lease::new("test-slot").is_ok());
    }
    fn simple_pdf(text: &str) -> Vec<u8> {
        let stream = format!("BT /F1 12 Tf 72 720 Td ({text}) Tj ET");
        let objects=["<< /Type /Catalog /Pages 2 0 R >>".to_string(),"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".into(),"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>".into(),"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".into(),format!("<< /Length {} >>\nstream\n{stream}\nendstream",stream.len())];
        let mut pdf = String::from("%PDF-1.4\n");
        let mut offsets = Vec::new();
        for (i, obj) in objects.iter().enumerate() {
            offsets.push(pdf.len());
            pdf.push_str(&format!("{} 0 obj\n{obj}\nendobj\n", i + 1));
        }
        let xref = pdf.len();
        pdf.push_str("xref\n0 6\n0000000000 65535 f \n");
        for offset in offsets {
            pdf.push_str(&format!("{offset:010} 00000 n \n"));
        }
        pdf.push_str(&format!(
            "trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n"
        ));
        pdf.into_bytes()
    }
}
