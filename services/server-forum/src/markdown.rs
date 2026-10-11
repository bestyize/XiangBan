//! Markdown 渲染：服务端渲染并清洗，前端直接展示 HTML。

use pulldown_cmark::{html, Options, Parser};
use std::collections::HashSet;

/// 渲染并清洗。外链加 nofollow 并在新窗口打开；trusted 为 false（新人）时去掉图片和链接，只留文字。
pub fn render(md: &str, trusted: bool) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(md, options);
    let mut raw = String::with_capacity(md.len() * 2);
    html::push_html(&mut raw, parser);

    let mut builder = ammonia::Builder::default();
    builder
        .link_rel(Some("nofollow noopener noreferrer ugc"))
        .set_tag_attribute_value("a", "target", "_blank")
        .url_schemes(HashSet::from(["http", "https", "mailto"]))
        .add_tags(["input"])
        .add_tag_attributes("input", ["type", "checked", "disabled"])
        .add_tag_attributes("code", ["class"]);
    if trusted {
        builder.set_tag_attribute_value("img", "loading", "lazy");
    } else {
        builder.rm_tags(["img", "a"]);
    }
    builder.clean(&raw).to_string()
}

/// 纯文本摘要，用于列表和 SEO description。
pub fn excerpt(md: &str, max_chars: usize) -> String {
    let parser = Parser::new(md);
    let mut text = String::new();
    for event in parser {
        use pulldown_cmark::Event::*;
        match event {
            Text(t) | Code(t) => text.push_str(&t),
            SoftBreak | HardBreak | End(_) => text.push(' '),
            _ => {}
        }
        if text.chars().count() > max_chars * 2 {
            break;
        }
    }
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() > max_chars {
        let mut s: String = collapsed.chars().take(max_chars).collect();
        s.push('…');
        s
    } else {
        collapsed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_scripts_and_marks_links() {
        let html = render("hi <script>alert(1)</script> [x](https://a.com) [y](javascript:alert(1))", true);
        assert!(!html.contains("<script"));
        assert!(!html.contains("javascript:"));
        assert!(html.contains("rel=\"nofollow noopener noreferrer ugc\""));
        assert!(html.contains("target=\"_blank\""));
    }

    #[test]
    fn untrusted_loses_images_and_links_but_keeps_text() {
        assert!(render("![a](https://a.com/x.png)", true).contains("<img"));
        let html = render("![a](https://a.com/x.png) [官网](https://a.com)", false);
        assert!(!html.contains("<img") && !html.contains("<a"));
        assert!(html.contains("官网"));
    }

    #[test]
    fn code_blocks_keep_language_class() {
        let html = render("```bash\ncurl https://api\n```", false);
        assert!(html.contains("<pre><code class=\"language-bash\">"));
    }

    #[test]
    fn excerpt_is_plain_and_truncated() {
        assert_eq!(excerpt("# 标题\n\n**粗体** `code`", 50), "标题 粗体 code");
        assert_eq!(excerpt(&"字".repeat(20), 5), "字字字字字…");
    }
}
