//! Checks for the event attributes interpreted by the DC template runtime.
use crate::Project;
use html5ever::tokenizer::{
    states::RawKind, BufferQueue, TagKind, Token, TokenSink, TokenSinkResult, Tokenizer,
};
use std::cell::{Cell, RefCell};

#[derive(Default)]
struct Bindings {
    in_template: Cell<bool>,
    issues: RefCell<Vec<String>>,
}

impl TokenSink for Bindings {
    type Handle = ();
    fn process_token(&self, token: Token, line: u64) -> TokenSinkResult<()> {
        let Token::TagToken(tag) = token else {
            return TokenSinkResult::Continue;
        };
        let name = tag.name.as_ref();
        if name == "x-dc" {
            self.in_template.set(tag.kind == TagKind::StartTag);
        }
        if tag.kind != TagKind::StartTag {
            return TokenSinkResult::Continue;
        }
        let next = match name {
            "script" => TokenSinkResult::RawData(RawKind::ScriptData),
            "style" | "iframe" | "noembed" | "noframes" | "xmp" => {
                TokenSinkResult::RawData(RawKind::Rawtext)
            }
            "textarea" | "title" => TokenSinkResult::RawData(RawKind::Rcdata),
            _ => TokenSinkResult::Continue,
        };
        // These runtime directives consume their own props rather than installing DOM listeners.
        if !self.in_template.get()
            || matches!(
                name,
                "x-dc"
                    | "x-import"
                    | "dc-import"
                    | "slot"
                    | "dc-slot"
                    | "sc-if"
                    | "sc-else-if"
                    | "sc-else"
                    | "sc-for"
                    | "helmet"
                    | "sc-helmet"
                    | "style"
                    | "script"
            )
        {
            return next;
        }
        let id = tag
            .attrs
            .iter()
            .find(|a| matches!(a.name.local.as_ref(), "id" | "data-ocd-node"))
            .map(|a| format!("#{}", a.value.chars().take(70).collect::<String>()))
            .unwrap_or_default();
        for attr in tag.attrs {
            let raw = attr.name.local.as_ref();
            let event = raw.strip_prefix("sc-camel-").unwrap_or(raw);
            if !event.starts_with("on") {
                continue;
            }
            let value = attr.value.trim();
            if !(value.starts_with("{{") && value.ends_with("}}") && value.len() > 4) {
                self.issues.borrow_mut().push(format!("第 {line} 行 <{name}{id}> 的 {event} 未绑定函数。使用 {{{{handler}}}}，跨画板跳转使用 <a href>。"));
            }
        }
        next
    }
}

pub fn binding_issues(project: &Project) -> Vec<String> {
    let mut issues = Vec::new();
    for (file, source) in &project.files {
        if !file.ends_with(".html") {
            continue;
        }
        let input = BufferQueue::default();
        input.push_back(source.as_str().into());
        let tokenizer = Tokenizer::new(Bindings::default(), Default::default());
        let _ = tokenizer.feed(&input);
        tokenizer.end();
        issues.extend(
            tokenizer
                .sink
                .issues
                .into_inner()
                .into_iter()
                .map(|issue| format!("{file} · {issue}")),
        );
    }
    issues
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceEdit {
    file: String,
    before: String,
    after: String,
}

pub fn edit_sources(project: &mut Project, edits: Vec<SourceEdit>) -> Result<(), String> {
    for edit in edits {
        if !edit.file.ends_with(".dc.html") || edit.before.is_empty() {
            return Err("局部修改需要 .dc.html 文件和非空的 before 原文".into());
        }
        let source = project
            .files
            .get_mut(&edit.file)
            .ok_or_else(|| format!("找不到画板 {}", edit.file))?;
        let Some(start) = source
            .find(&edit.before)
            .filter(|first| Some(*first) == source.rfind(&edit.before))
        else {
            return Err(format!(
                "{} 中的 before 必须恰好匹配一次，请读取最新画板后重试",
                edit.file
            ));
        };
        source.replace_range(start..start + edit.before.len(), &edit.after);
    }
    Ok(())
}
