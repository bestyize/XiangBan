use agent_file_tools::{
    AttachmentContext, AttachmentHistory, AttachmentInput, AttachmentStore, ReadFile,
};
use agent_kernel::{MemoryStore, Message, Middleware, Part, SessionStore, Tool, ToolCtx, Turn};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::json;
use std::{
    fs,
    io::{Cursor, Write},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "agent-files-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&p).unwrap();
        Self(p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn input(name: &str, bytes: &[u8]) -> AttachmentInput {
    AttachmentInput {
        name: name.into(),
        data_url: format!(
            "data:application/octet-stream;base64,{}",
            STANDARD.encode(bytes)
        ),
    }
}
fn office(entries: &[(&str, &str)]) -> Vec<u8> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, xml) in entries {
        zip.start_file(*name, zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(xml.as_bytes()).unwrap();
    }
    zip.finish().unwrap().into_inner()
}
#[test]
fn imports_session_copies_and_preserves_display_names() {
    let temp = Temp::new();
    let store = AttachmentStore::new(temp.0.join("attachments"));
    let items = store
        .import("s1", &[input("../../预算.txt", "金额: 20".as_bytes())])
        .unwrap();
    assert_eq!(items[0].name, "../../预算.txt");
    assert_eq!(fs::read_to_string(&items[0].path).unwrap(), "金额: 20");
    assert!(PathBuf::from(&items[0].path).starts_with(store.existing_session_dir("s1").unwrap()));
    store.remove_session("s1").unwrap();
    assert!(!PathBuf::from(&items[0].path).exists());
}
#[test]
fn invalid_batch_never_leaves_partial_files() {
    let temp = Temp::new();
    let store = AttachmentStore::new(&temp.0);
    assert!(store
        .import("s", &[input("valid.txt", b"ok"), input("bad.exe", b"bad")])
        .is_err());
    assert!(store.existing_session_dir("s").is_none());
    assert!(store.import("s", &[input("fake.png", b"not png")]).is_err());
    assert!(store
        .import("s", &[input("binary.txt", &[0, 1, 2])])
        .is_err());
}
#[test]
fn prevents_cross_session_reads_but_allows_the_selected_attachment() {
    let temp = Temp::new();
    let workspace = temp.0.join("work");
    fs::create_dir(&workspace).unwrap();
    let store = AttachmentStore::new(temp.0.join("attachments"));
    let items = store
        .import("one", &[input("report.txt", b"one\ntwo\nthree")])
        .unwrap();
    let reader = ReadFile::new(vec![workspace], store).unwrap();
    let args = json!({"path":items[0].path,"offset":2,"limit":1});
    let ok = reader.execute_result(args.clone(), &ToolCtx::detached("one"));
    assert!(!ok.is_error);
    assert!(ok.output.contains("two"));
    assert!(!ok.output.contains("three"));
    assert!(
        reader
            .execute_result(args, &ToolCtx::detached("other"))
            .is_error
    );
}
#[cfg(unix)]
#[test]
fn rejects_symlink_escape() {
    use std::os::unix::fs::symlink;
    let temp = Temp::new();
    let workspace = temp.0.join("work");
    fs::create_dir(&workspace).unwrap();
    fs::write(temp.0.join("secret.txt"), "secret").unwrap();
    symlink(temp.0.join("secret.txt"), workspace.join("escape.txt")).unwrap();
    let reader = ReadFile::new(
        vec![workspace.clone()],
        AttachmentStore::new(temp.0.join("attachments")),
    )
    .unwrap();
    assert!(
        reader
            .execute_result(
                json!({"path":workspace.join("escape.txt")}),
                &ToolCtx::detached("s")
            )
            .is_error
    );
}
#[test]
fn images_are_multimodal_results() {
    let temp = Temp::new();
    let store = AttachmentStore::new(temp.0.join("files"));
    let png=STANDARD.decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+aZ1cAAAAASUVORK5CYII=").unwrap();
    let item = store
        .import("s", &[input("image.png", &png)])
        .unwrap()
        .remove(0);
    let reader = ReadFile::new(vec![temp.0.join("files")], store).unwrap();
    let result = reader.execute_result(json!({"path":item.path}), &ToolCtx::detached("s"));
    assert!(!result.is_error);
    assert!(matches!(result.parts.first(), Some(Part::Image { .. })));
}
#[test]
fn extracts_docx_text_entities_and_continuation() {
    let temp = Temp::new();
    let store = AttachmentStore::new(temp.0.join("files"));
    let bytes = office(&[(
        "word/document.xml",
        r#"<w:document xmlns:w="w"><w:body><w:p><w:r><w:t>甲 &amp; 乙</w:t></w:r></w:p><w:p><w:r><w:t>next paragraph</w:t></w:r></w:p></w:body></w:document>"#,
    )]);
    let item = store
        .import("s", &[input("report.docx", &bytes)])
        .unwrap()
        .remove(0);
    let reader = ReadFile::new(vec![temp.0.join("files")], store).unwrap();
    let first = reader.execute_result(json!({"path":item.path,"limit":2}), &ToolCtx::detached("s"));
    assert!(!first.is_error, "{}", first.output);
    assert!(first.output.contains("甲 & 乙"));
    assert!(first.structured_content.as_ref().unwrap()["next_offset"].is_number());
    let next = reader.execute_result(
        json!({"path":item.path,"offset":3}),
        &ToolCtx::detached("s"),
    );
    assert!(next.output.contains("next paragraph"));
}
#[test]
fn pptx_slides_use_numeric_order() {
    let temp = Temp::new();
    let store = AttachmentStore::new(temp.0.join("files"));
    let xml = r#"<a:p xmlns:a="a"><a:r><a:t>slide text</a:t></a:r></a:p>"#;
    let bytes = office(&[
        ("ppt/slides/slide10.xml", xml),
        ("ppt/slides/slide2.xml", xml),
    ]);
    let item = store
        .import("s", &[input("slides.pptx", &bytes)])
        .unwrap()
        .remove(0);
    let reader = ReadFile::new(vec![temp.0.join("files")], store).unwrap();
    let r = reader.execute_result(json!({"path":item.path}), &ToolCtx::detached("s"));
    assert!(!r.is_error, "{}", r.output);
    assert!(r.output.find("slide2.xml").unwrap() < r.output.find("slide10.xml").unwrap());
}
#[test]
fn corrupt_pdf_is_an_error_instead_of_an_empty_success() {
    let temp = Temp::new();
    let store = AttachmentStore::new(temp.0.join("files"));
    let item = store
        .import("s", &[input("broken.pdf", b"%PDF-invalid")])
        .unwrap()
        .remove(0);
    let reader = ReadFile::new(vec![temp.0.join("files")], store).unwrap();
    assert!(
        reader
            .execute_result(json!({"path":item.path}), &ToolCtx::detached("s"))
            .is_error
    );
}
#[test]
fn metadata_survives_history_and_only_request_copies_get_manifests() {
    let temp = Temp::new();
    let store = AttachmentStore::new(&temp.0);
    let items = store
        .import("s", &[input("budget.csv", b"amount\n10")])
        .unwrap();
    let history = AttachmentHistory::new(MemoryStore::new(), items.clone());
    history
        .append("s", &[Message::user_text("analyze")])
        .unwrap();
    history
        .append("s", &[Message::user_text("followup")])
        .unwrap();
    let saved = history.load("s").unwrap();
    assert_eq!(
        saved[0].structured_content.as_ref().unwrap()["attachments"][0]["name"],
        "budget.csv"
    );
    assert!(saved[1].structured_content.is_none());
    let mut turn = Turn {
        session: "s".into(),
        index: 0,
        messages: saved.clone(),
        tools: vec![],
    };
    AttachmentContext::new(vec![])
        .before_model(&mut turn)
        .unwrap();
    assert!(turn.messages[0].text().contains(&items[0].path));
    assert_eq!(saved[0].text(), "analyze");
}

#[test]
fn extracts_xlsx_cells_with_sheet_and_row_sources() {
    let temp = Temp::new();
    let store = AttachmentStore::new(temp.0.join("files"));
    let bytes = office(&[
        (
            "[Content_Types].xml",
            r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/><Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/></Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/></Relationships>"#,
        ),
        (
            "xl/workbook.xml",
            r#"<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets><sheet name="Budget" sheetId="1" r:id="rId1"/></sheets></workbook>"#,
        ),
        (
            "xl/_rels/workbook.xml.rels",
            r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/></Relationships>"#,
        ),
        (
            "xl/worksheets/sheet1.xml",
            r#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData><row r="1"><c r="A1" t="inlineStr"><is><t>amount</t></is></c></row><row r="2"><c r="A2"><v>12.5</v></c></row></sheetData></worksheet>"#,
        ),
    ]);
    let item = store
        .import("s", &[input("budget.xlsx", &bytes)])
        .unwrap()
        .remove(0);
    let reader = ReadFile::new(vec![temp.0.join("files")], store).unwrap();
    let r = reader.execute_result(json!({"path":item.path}), &ToolCtx::detached("s"));
    assert!(!r.is_error, "{}", r.output);
    assert!(r.output.contains("Budget"));
    assert!(r.output.contains("[row 2] C1=\"12.5\""));
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
#[test]
fn extracts_real_pdf_text_and_warns_about_visual_coverage() {
    let temp = Temp::new();
    let store = AttachmentStore::new(temp.0.join("files"));
    let item = store
        .import("s", &[input("report.pdf", &simple_pdf("Revenue 125"))])
        .unwrap()
        .remove(0);
    let reader = ReadFile::new(vec![temp.0.join("files")], store).unwrap();
    let r = reader.execute_result(json!({"path":item.path}), &ToolCtx::detached("s"));
    assert!(!r.is_error, "{}", r.output);
    assert!(r.output.contains("Revenue 125"));
    assert!(r.output.contains("text only"));
}
#[test]
fn attachment_reaches_model_and_tool_result_without_changing_kernel_messages() {
    use agent_kernel::{
        provider::scripted::{ScriptedProvider, Step},
        Agent, Role,
    };
    use std::sync::{atomic::AtomicBool, Arc};
    let temp = Temp::new();
    let store = AttachmentStore::new(temp.0.join("files"));
    let attachments = store
        .import("s", &[input("data.csv", b"amount\n25")])
        .unwrap();
    let provider = Arc::new(ScriptedProvider::new(vec![
        Step::tool_call(
            "read",
            "read_file",
            &json!({"path":attachments[0].path}).to_string(),
        ),
        Step::text("25"),
    ]));
    let history = Arc::new(AttachmentHistory::new(
        MemoryStore::new(),
        attachments.clone(),
    ));
    let agent = Agent::builder(provider.clone(), history.clone())
        .tool(ReadFile::new(vec![temp.0.join("files")], store).unwrap())
        .middleware(AttachmentContext::new(attachments.clone()))
        .build()
        .unwrap();
    agent
        .run(
            "s",
            "analyze",
            &mut |_| {},
            &Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    let requests = provider.requests();
    assert!(requests[0]
        .messages
        .iter()
        .any(|m| m.text().contains(&attachments[0].path)));
    assert!(requests[1]
        .messages
        .iter()
        .any(|m| m.role == Role::Tool && m.text().contains("25")));
    assert_eq!(history.load("s").unwrap()[0].text(), "analyze");
}
