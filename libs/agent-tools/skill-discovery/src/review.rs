//! Model reviews are complete JSON summaries, with hash-pinned, pageable detail reads.
use crate::{download::Download, package::{is_script, Plan}, content_with};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

pub(crate) fn prefix(text: &str, max: usize) -> &str {
    let mut end = text.len().min(max); while !text.is_char_boundary(end) { end -= 1; }
    &text[..end]
}
pub(crate) fn summary(slug: &str, plan: &Plan, markdown: &str) -> Value {
    let manifest = plan.summary();
    let metadata = agent_kernel::skills::parse_skill_metadata_named(markdown, Some(slug)).ok();
    json!({"provider":"skillhub","slug":slug,"namespace":plan.namespace,"version":plan.version,"sha256":plan.sha256,
        "declaredName":metadata.as_ref().map(|m| prefix(&m.name, 200)),
        "description":metadata.as_ref().map(|m| prefix(&m.description, 600)).or_else(||plan.metadata.as_ref()?.get("description")?.as_str().map(|s|prefix(s,600))),
        "fileCount":plan.files.len(),"totalBytes":plan.files.iter().map(|f|f.size).sum::<u64>(),
        "files":manifest["files"].as_array().map(|files|files.iter().take(8).collect::<Vec<_>>()),
        "manifestHasMore":plan.files.len()>8,"nextManifestOffset":plan.files.len().min(8),
        "scriptCount":plan.files.iter().filter(|f|is_script(&f.path)).count(),
        "scriptPaths":plan.files.iter().filter(|f|is_script(&f.path)).take(8).map(|f|&f.path).collect::<Vec<_>>(),
        "contentPreview":prefix(markdown,1500),"contentTruncated":markdown.len()>1500,
        "reviewStatus":"summary_only","requiresInstallationApproval":true,"requiresExecutionApproval":true,
        "next_step":"Use skill_review_read with these exact coordinates and sha256: section=manifest pages list all files; section=file reads SKILL.md or a listed script in pages. Do not claim complete review until the relevant pages are read. This summary does not audit code or prove runtime, dependency or network behavior. Installing never executes scripts."})
}
pub(crate) fn check_digest(plan: &Plan, args: &Value) -> Result<(), String> {
    let expected = args["sha256"].as_str().ok_or("expected manifest SHA256 required")?;
    if expected.len()!=64 || !expected.bytes().all(|b|b.is_ascii_hexdigit()) || plan.sha256 != expected.to_ascii_lowercase() {
        return Err("SkillHub file manifest differs from the reviewed SHA256".into());
    }
    Ok(())
}
pub(crate) fn page(text: &str, offset: usize, limit: usize) -> Result<Value, String> {
    if offset>text.len() || !text.is_char_boundary(offset) { return Err("offset must be a UTF-8 boundary returned by nextOffset".into()); }
    let content=prefix(&text[offset..], limit.clamp(4,2048)); let next=offset+content.len();
    Ok(json!({"content":content,"offset":offset,"nextOffset":next,"totalBytes":text.len(),"hasMore":next<text.len()}))
}
pub(crate) fn read(slug: &str, plan: &Plan, args: &Value, download: &Download<'_>) -> Result<Value,String> {
    check_digest(plan,args)?;
    let offset:usize=args["offset"].as_u64().unwrap_or(0).try_into().map_err(|_|"invalid offset")?;
    let section=args["section"].as_str().unwrap_or("file");
    if section=="manifest" {
        if offset>plan.files.len() { return Err("manifest offset exceeds file count".into()); }
        let summary=plan.summary(); let files=summary["files"].as_array().unwrap(); let next=(offset+8).min(files.len());
        return Ok(json!({"slug":slug,"namespace":plan.namespace,"version":plan.version,"sha256":plan.sha256,"files":&files[offset..next],"offset":offset,"nextOffset":next,"fileCount":files.len(),"hasMore":next<files.len()}));
    }
    if section!="file" { return Err("section must be manifest or file".into()); }
    let path=args["path"].as_str().unwrap_or("SKILL.md");
    let item=plan.files.iter().find(|f|f.path==path).ok_or("file is not in the reviewed manifest")?;
    let bytes=content_with(slug,&plan.version,plan.namespace.as_deref(),path,item.size,download)?;
    if bytes.len() as u64!=item.size || format!("{:x}",Sha256::digest(&bytes))!=item.sha256 { return Err("reviewed file changed; inspect the package again".into()); }
    let text=std::str::from_utf8(&bytes).map_err(|_|"review reader only supports UTF-8 text files")?;
    let mut result=page(text,offset,args["limit"].as_u64().unwrap_or(2048).min(2048) as usize)?;
    result["path"]=json!(path); result["fileSha256"]=json!(item.sha256); result["sha256"]=json!(plan.sha256);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn summary_of_a_large_multifile_skill_stays_valid_json_below_spill_threshold() {
        let files=(0..512).map(|i|json!({"path":format!("scripts/{}-{i}.py","file".repeat(18)),"size":10,"sha256":"a".repeat(64)})).collect::<Vec<_>>();
        let mut files=files; files[0]["path"]=json!("SKILL.md");
        let mut plan=crate::package::plan("1.0",&files).unwrap(); plan.sha256="b".repeat(64);
        let raw=format!("---\nname: original-name\ndescription: 科幻小说\n---\n{}","正文".repeat(30000));
        let result=summary("market-slug",&plan,&raw); let text=result.to_string();
        assert!(text.len()<16000); assert_eq!(result["fileCount"],512); assert_eq!(result["scriptCount"],511);
        assert_eq!(result["reviewStatus"],"summary_only"); assert_eq!(result["sha256"],"b".repeat(64));
        assert_eq!(result["declaredName"],"original-name"); assert_eq!(result["contentTruncated"],true);
    }
    #[test]
    fn unicode_pages_are_contiguous_and_never_silently_drop_content() {
        let text="中文🪴".repeat(2000); let mut offset=0; let mut joined=String::new();
        loop { let result=page(&text,offset,2048).unwrap(); joined.push_str(result["content"].as_str().unwrap());
            offset=result["nextOffset"].as_u64().unwrap() as usize; if result["hasMore"]==false {break;}
        }
        assert_eq!(joined,text); assert!(page(&text,1,2048).is_err());
    }
}
