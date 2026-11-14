//! Markdown 渲染：服务端渲染并清洗，前端直接展示 HTML。

use pulldown_cmark::{html, Options, Parser};
use std::collections::HashSet;

/// 渲染并清洗。新人可展示站内上传的图片，外链与外站图片需要成员等级。
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
        .add_tag_attributes("code", ["class"])
        .set_tag_attribute_value("img", "loading", "lazy");
    if !trusted {
        builder.rm_tags(["a"]).attribute_filter(|tag, attribute, value| {
            if tag == "img" && attribute == "src" && !site_image_url(value) {
                None
            } else {
                Some(std::borrow::Cow::Borrowed(value))
            }
        });
    }
    builder.clean(&raw).to_string()
}

fn site_image_url(value: &str) -> bool {
    value.strip_prefix("/api/images/")
        .is_some_and(|id| id.len() == 36 && uuid::Uuid::try_parse(id).is_ok())
}

/// 纯文本摘要，用于列表和 SEO description。
pub fn excerpt(md: &str, max_chars: usize) -> String {
    excerpt_impl(md, max_chars, false)
}

/// Untitled image notes use article text, not image alt text, as their feed/share label.
pub fn article_excerpt(md: &str, max_chars: usize) -> String {
    excerpt_impl(md, max_chars, true)
}

fn excerpt_impl(md: &str, max_chars: usize, skip_images: bool) -> String {
    let mut image_depth = 0usize;
    let parser = Parser::new(md);
    let mut text = String::new();
    for event in parser {
        use pulldown_cmark::Event::*;
        if skip_images {
            match &event {
                Start(pulldown_cmark::Tag::Image { .. }) => { image_depth += 1; continue; }
                End(pulldown_cmark::TagEnd::Image) => { image_depth = image_depth.saturating_sub(1); text.push(' '); continue; }
                _ if image_depth > 0 => continue,
                _ => {}
            }
        }
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

/// 正文里引用的站内图片地址（`/api/images/{id}`），按出现顺序去重，最多 `max` 张，用作列表卡片的配图。
pub fn site_images(md: &str, max: usize) -> Vec<String> {
    const PREFIX: &str = "/api/images/";
    let mut found: Vec<String> = Vec::new();
    let mut rest = md;
    while let Some(pos) = rest.find(PREFIX) {
        let after = &rest[pos + PREFIX.len()..];
        let id: String = after.chars().take(36).collect();
        if uuid::Uuid::parse_str(&id).is_ok() {
            let url = format!("{PREFIX}{id}");
            if !found.contains(&url) {
                found.push(url);
                if found.len() >= max {
                    break;
                }
            }
        }
        rest = after;
    }
    found
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
    fn newcomers_can_render_uploaded_images() {
        let local = format!("/api/images/{}", uuid::Uuid::new_v4());
        let html = render(&format!("![配图]({local}) ![外站](https://a.com/x.png) [官网](https://a.com)"), false);
        assert!(html.contains(&format!("src=\"{local}\"")), "{html}");
        assert!(!html.contains("https://") && !html.contains("<a "));
        assert!(html.contains("官网"));
    }

    #[test]
    fn members_can_render_external_images() {
        assert!(render("![a](https://a.com/x.png)", true).contains("<img"));
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

    #[test]
    fn article_excerpt_excludes_cover_alt_but_preserves_links_and_text() {
        let md = "![封面](/api/images/id)\n\n今天的 **生活** [记录](https://example.com)";
        assert_eq!(article_excerpt(md, 80), "今天的 生活 记录");
        assert!(excerpt(md, 80).starts_with("封面"));
        assert_eq!(article_excerpt(&"字".repeat(81), 80).chars().count(), 81);
    }

    #[test]
    fn site_images_are_listed_once_in_order() {
        let a = "6f1c2d3e-0000-4000-8000-000000000001";
        let b = "6f1c2d3e-0000-4000-8000-000000000002";
        let md = format!("![x](/api/images/{a}) text ![y](/api/images/{b}) ![z](/api/images/{a}) /api/images/not-a-uuid");
        assert_eq!(site_images(&md, 3), vec![format!("/api/images/{a}"), format!("/api/images/{b}")]);
        assert_eq!(site_images(&md, 1).len(), 1);
        assert!(site_images("no images", 3).is_empty());
    }

    #[test]
    fn site_images_keep_their_relative_url_at_all_levels() {
        let md = "![图](/api/images/6f1c2d3e-0000-4000-8000-000000000000)";
        let html = render(md, false);
        assert!(html.contains(r#"src="/api/images/6f1c2d3e-0000-4000-8000-000000000000""#), "{html}");
        assert!(html.contains(r#"loading="lazy""#), "{html}");
    }
}
