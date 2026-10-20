use agent_kernel::ToolCtx;
use calamine::{open_workbook_auto, Reader};
use quick_xml::{events::Event, Reader as XmlReader};
use std::{fs::File, io::Read, path::Path};
const MAX_EXPANDED: u64 = 64 * 1024 * 1024;
const MAX_TEXT: usize = 8 * 1024 * 1024;

pub fn extract(path: &Path, ext: &str, ctx: &ToolCtx) -> Result<String, String> {
    if ext == "pdf" {
        let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
        if !bytes.starts_with(b"%PDF-") {
            return Err("invalid PDF header".into());
        }
        let text = pdf_extract::extract_text_from_mem(&bytes)
            .map_err(|e| format!("PDF extraction failed (encrypted or malformed file): {e}"))?;
        if ctx.is_cancelled() {
            return Err("cancelled".into());
        }
        if text.len() > MAX_TEXT {
            return Err("PDF text exceeds the extraction budget; split the document first".into());
        }
        if text.trim().is_empty() {
            return Err("PDF has no extractable text. It may be scanned; OCR or rendered-page vision is required and is not provided by this reader.".into());
        }
        return Ok(format!("[PDF text only; scanned pages and illustrations can be missing. Do not claim visual or complete-page coverage.]\n{text}"));
    }
    let mut zip = zip::ZipArchive::new(File::open(path).map_err(|e| e.to_string())?)
        .map_err(|e| format!("invalid Office archive: {e}"))?;
    let mut expanded = 0u64;
    if zip.len() > 10_000 {
        return Err("too many archive entries".into());
    }
    for i in 0..zip.len() {
        let entry = zip.by_index(i).map_err(|e| e.to_string())?;
        expanded = expanded
            .checked_add(entry.size())
            .ok_or("archive size overflow")?;
        if expanded > MAX_EXPANDED {
            return Err("Office archive exceeds expanded size budget".into());
        }
    }
    if ext == "xlsx" {
        drop(zip);
        let mut book =
            open_workbook_auto(path).map_err(|e| format!("spreadsheet extraction failed: {e}"))?;
        let mut out=String::from("[Cached cell values only; formulas are not recalculated. Use scripts for full-table calculations.]\n");
        for name in book.sheet_names().to_vec() {
            if ctx.is_cancelled() {
                return Err("cancelled".into());
            }
            out.push_str(&format!(
                "[Sheet {}]\n",
                serde_json::to_string(&name).map_err(|e| e.to_string())?
            ));
            let range = book.worksheet_range(&name).map_err(|e| e.to_string())?;
            let start = range.start().unwrap_or((0, 0));
            for (n, row) in range.rows().enumerate() {
                if ctx.is_cancelled() {
                    return Err("cancelled".into());
                }
                out.push_str(&format!("[row {}] ", start.0 as usize + n + 1));
                for (c, cell) in row.iter().enumerate() {
                    out.push_str(&format!(
                        "C{}={}\t",
                        start.1 as usize + c + 1,
                        serde_json::to_string(&cell.to_string()).map_err(|e| e.to_string())?
                    ));
                    if out.len() > MAX_TEXT {
                        return Err("spreadsheet extraction exceeds budget; use scripts to inspect or aggregate ranges".into());
                    }
                }
                out.push('\n');
            }
        }
        return Ok(out);
    }
    let mut entries: Vec<String> = zip
        .file_names()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?
        .into_iter()
        .filter(|n| {
            if ext == "docx" {
                n.as_ref() == "word/document.xml"
            } else {
                n.starts_with("ppt/slides/slide") && n.ends_with(".xml") && !n.contains("/_rels/")
            }
        })
        .map(|n| n.into_owned())
        .collect();
    entries.sort_by_key(|n| {
        n.rsplit('/')
            .next()
            .unwrap_or("")
            .trim_start_matches("slide")
            .trim_end_matches(".xml")
            .parse::<u32>()
            .unwrap_or(0)
    });
    if entries.is_empty() {
        return Err("Office document has no readable body".into());
    }
    let mut out = String::new();
    for name in entries {
        if ctx.is_cancelled() {
            return Err("cancelled".into());
        }
        out.push_str(&format!("[{name}]\n"));
        let mut xml = String::new();
        zip.by_name(&name)
            .map_err(|e| e.to_string())?
            .take(MAX_EXPANDED + 1)
            .read_to_string(&mut xml)
            .map_err(|e| e.to_string())?;
        let mut reader = XmlReader::from_str(&xml);
        let mut in_text = false;
        loop {
            match reader
                .read_event()
                .map_err(|e| format!("invalid document XML: {e}"))?
            {
                Event::Start(e) => {
                    if e.local_name().as_ref() == b"t" {
                        in_text = true;
                    }
                }
                Event::End(e) => {
                    let name = e.local_name();
                    if name.as_ref() == b"t" {
                        in_text = false;
                    }
                    if matches!(name.as_ref(), b"p" | b"tr") {
                        out.push('\n');
                    }
                }
                Event::Empty(e) => {
                    if matches!(e.local_name().as_ref(), b"tab" | b"br") {
                        out.push(' ');
                    }
                }
                Event::Text(e) if in_text => {
                    let decoded = e.decode().map_err(|e| e.to_string())?;
                    out.push_str(
                        &quick_xml::escape::unescape(&decoded).map_err(|e| e.to_string())?,
                    );
                }
                Event::GeneralRef(e) if in_text => {
                    let value = e.decode().map_err(|e| e.to_string())?;
                    out.push_str(
                        &quick_xml::escape::unescape(&format!("&{value};"))
                            .map_err(|e| e.to_string())?,
                    );
                }
                Event::Eof => break,
                _ => {}
            }
            if out.len() > MAX_TEXT {
                return Err("document text exceeds extraction budget".into());
            }
            if ctx.is_cancelled() {
                return Err("cancelled".into());
            }
        }
        out.push('\n');
    }
    Ok(out)
}
