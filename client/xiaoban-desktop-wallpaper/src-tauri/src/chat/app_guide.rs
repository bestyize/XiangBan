//! What 小伴桌面 can do and where it lives, written for the model: `app_guide` answers "can it / where is it",
//! `open_page` takes the user there. Add a row here whenever a tool or console page is added; the tests in this
//! file and in `desktop_tools` fail when a tool, a page or a settings category is missing.

pub(super) struct Feature {
    pub id: &'static str,
    pub name: &'static str,
    /// Where the user finds it in the console, in the words of the sidebar.
    pub path: &'static str,
    /// The `open_page` arguments that lead there, when it has a page.
    pub page: Option<(&'static str, Option<&'static str>)>,
    pub does: &'static str,
    /// What the user cannot do by talking, and must do on the page. Empty when everything is possible in chat.
    pub by_hand: &'static str,
    pub tools: &'static [&'static str],
}

pub(super) const PAGES: &[&str] = &["new-task", "ai-store", "library", "studio", "games", "care", "settings"];
pub(super) const STUDIO_TABS: &[&str] = &["wallpapers", "effects", "widgets", "pets"];
pub(super) const SETTINGS_TABS: &[&str] = &["general", "appearance", "ai", "assistant", "voice", "network", "safety", "about"];

const NONE: &[&str] = &[];

pub(super) const FEATURES: &[Feature] = &[
    Feature { id: "chat", name: "对话与任务", path: "侧栏「新建任务」", page: Some(("new-task", None)),
        does: "向小伴提问、写文档做表格、整理文件、截图识别、深度研究、学习辅导；可选工作区、带附件、设置权限模式。",
        by_hand: "切换模型、工作区、权限模式在对话输入框旁操作。", tools: NONE },
    Feature { id: "ai-store-experts", name: "AI 插件·专家", path: "侧栏「AI 插件」→ 顶部「专家」标签", page: Some(("ai-store", None)),
        does: "浏览专家角色，可按领域筛选、搜索，用「已安装」筛选只看自己装过的。专家只改变做事的专业方法与交付标准，不会增加工具、权限或记忆。",
        by_hand: "挑选和启用专家在页面里操作；打开页面后点顶部「专家」标签。", tools: NONE },
    Feature { id: "ai-store-skills", name: "AI 插件·技能", path: "侧栏「AI 插件」→ 顶部「技能」标签", page: Some(("ai-store", None)),
        does: "技能（Skill）来自 SkillHub，可搜索、按领域/来源/排序筛选。对话里能检索已安装技能、读取并按流程执行、在线搜索候选并审查（说明、文件清单、脚本预览、SHA256）。安装不会执行脚本，脚本运行需单独确认。已安装的技能点「使用技能」会带着它新建任务，也可在这里卸载。",
        by_hand: "安装和卸载要用户在页面里勾选「已查看」并确认；对话里的 skill_install 也需用户授权。打开页面后点「技能」标签。",
        tools: &["skill_find", "load_skill", "skill_search", "skill_inspect", "skill_install", "skill_read", "skill_run", "skill_manage"] },
    Feature { id: "ai-store-connectors", name: "AI 插件·连接器", path: "侧栏「AI 插件」→ 顶部「连接器」标签", page: Some(("ai-store", None)),
        does: "连接器（MCP）把外部服务接给小伴，「已配置」筛选只看已连接的。里面还有「本机电脑控制」（Computer Use）：启用后小伴可通过截图、鼠标键盘操作其他应用；首次使用要在系统里授予屏幕录制和辅助功能权限。",
        by_hand: "连接器的账号授权、启用/停用电脑控制都要用户在页面里完成；系统权限可让小伴先 computer_use_diagnose 检查。打开页面后点「连接器」标签。",
        tools: &["computer_use_diagnose", "request_accessibility_permission", "computer_use_repair_permissions"] },
    Feature { id: "library-docs", name: "资料库·我的文档", path: "侧栏「资料库」→「我的文档」标签", page: Some(("library", None)),
        does: "导入文档/文件夹（保存的是副本，单个最多 50MB，总量 200MB），用文件夹树整理（最多 6 层）、打标签。导入后小伴会在后台读出文字，状态为可参考/读取中/读不出/暂不支持/文件太大。详情里「用这份资料提问」会把文件作为附件带进新建任务。",
        by_hand: "导入、整理、删除要用户在页面里操作（对话里不能导入或删除资料）。打开页面后点「我的文档」。", tools: NONE },
    Feature { id: "library-cloud", name: "资料库·云文档", path: "侧栏「资料库」→「云文档」标签", page: Some(("library", None)),
        does: "只保存云文档链接（飞书/Lark、腾讯文档、百度网盘、石墨、语雀、WPS，其余按网页），自动识别平台和类型，百度网盘提取码一并保存。目前只保存链接，小伴读不了里面的内容（各平台「连接账号」即将支持）。",
        by_hand: "添加和移除链接要用户在页面里操作。打开页面后点「云文档」。", tools: NONE },
    Feature { id: "library-favorites", name: "资料库·收藏", path: "侧栏「资料库」→「收藏」标签", page: Some(("library", None)),
        does: "保存的小伴回答。来源是回答下的 ☆「存到资料库」；每条可「回到对话」「接着问」、复制、取消收藏。",
        by_hand: "收藏、取消收藏要用户点击，小伴不能代为收藏。打开页面后点「收藏」。", tools: NONE },
    Feature { id: "library-memory", name: "资料库·记忆", path: "侧栏「资料库」→「记忆」标签", page: Some(("library", None)),
        does: "小伴留下的记忆：等你确认的候选经验、小伴的笔记（每次对话都带上）、做事经验、长期目标。对话里小伴能读写笔记、检索经验。右侧「让小伴记住我」开关可整体关闭记忆（已有记忆保留），也能清空全部记忆。记忆只存在本机。",
        by_hand: "确认/修改/忘掉某条、开关记忆、清空全部记忆要用户在页面里操作。打开页面后点「记忆」。", tools: &["memory_read", "memory_write", "experience_recall"] },
    Feature { id: "wallpaper", name: "壁纸", path: "侧栏「装扮」→「壁纸」", page: Some(("studio", Some("wallpapers"))),
        does: "查看、更换、创作、删除壁纸。", by_hand: "", tools: &["wallpaper_list", "wallpaper_apply", "wallpaper_create", "wallpaper_delete"] },
    Feature { id: "effect", name: "桌面特效", path: "侧栏「装扮」→「特效」", page: Some(("studio", Some("effects"))),
        does: "选用或创作粒子类桌面特效。", by_hand: "", tools: &["effect_list", "effect_set", "effect_create", "effect_delete", "effect_read_format"] },
    Feature { id: "widget", name: "桌面组件", path: "侧栏「装扮」→「组件」", page: Some(("studio", Some("widgets"))),
        does: "放置、移动、缩放、隐藏、移除时钟/天气/待办/日程/股票/木鱼/月历/纪念日组件；待办条目（增加、标记完成）、纪念日（标题、日期、倒数或已过去、每年重复）、今日日程、股票自选的内容也能直接设置（widget_add / widget_update 的 content）；还能创作自定义组件；可设层级：置顶（始终浮在其他窗口上方）或贴桌面，层级会保存，重启后仍保持，不是临时的。",
        by_hand: "", tools: &["widget_list", "widget_add", "widget_update", "widget_remove", "widget_read_format", "widget_create", "widget_delete_custom"] },
    Feature { id: "pet", name: "桌面伙伴（宠物）", path: "侧栏「装扮」→「伙伴」", page: Some(("studio", Some("pets"))),
        does: "添加多只桌面宠物，换形象、改表情、位置、大小、层级（置顶/贴桌面，会保存、重启后仍保持），创作新的宠物主题；盆栽小伴的显示、表情和外壳。",
        by_hand: "", tools: &["pet_read_format", "pet_catalog", "pet_theme_save", "pet_theme_delete", "pet_add", "pet_update", "pet_remove", "companion_set"] },
    Feature { id: "games", name: "游玩（小游戏）", path: "侧栏「游玩」，分官方游戏、我的游戏", page: Some(("games", None)),
        does: "开始或关闭小游戏，创作和删除自己的游戏；桌面层游戏直接浮在桌面上。", by_hand: "",
        tools: &["game_list", "game_open", "game_close", "game_read_format", "game_create", "game_delete"] },
    Feature { id: "care", name: "提醒与定时任务", path: "侧栏「提醒」", page: Some(("care", None)),
        does: "到点提醒：用户说“X点提醒我”“到时候叫我”“定个闹钟”“每天…提醒我”都用它。能管理关怀提醒（下班、休息、自定义；一次性、每天、每周几）和定时任务（到点由小伴在后台做事），小伴运行时到点会在桌面弹出提醒气泡。", by_hand: "天气来源等提醒设置在页面里改。",
        tools: &["care_list", "care_set", "care_delete"] },
    Feature { id: "windows", name: "小伴窗口位置与层级", path: "对话里直接说，没有单独页面", page: None,
        does: "查看和移动小伴自己的窗口（盆栽、超级小伴悬浮条、组件、宠物、控制台），设置置顶或贴桌面，收起悬浮条、显示或隐藏。组件和宠物的层级会保存；盆栽、超级小伴、控制台的层级只在本次运行有效。不能操作其他应用的窗口。",
        by_hand: "", tools: &["desktop_windows", "desktop_window_set", "desktop_status"] },
    Feature { id: "settings-general", name: "设置·通用", path: "头像菜单 →「设置」→「通用」", page: Some(("settings", Some("general"))),
        does: "开机启动、动画与帧率、暂停动画、数据目录。", by_hand: "暂停/恢复桌面动画可以在对话里说；开机启动、帧率、数据目录对话里不能改，打开页面让用户自己改。", tools: &["desktop_settings_set"] },
    Feature { id: "settings-appearance", name: "设置·外观", path: "设置 →「外观」", page: Some(("settings", Some("appearance"))),
        does: "深色/浅色/跟随系统、主题色。", by_hand: "", tools: &["desktop_settings_set"] },
    Feature { id: "settings-ai", name: "设置·AI 模型", path: "设置 →「AI 模型」", page: Some(("settings", Some("ai"))),
        does: "配置模型供应商、API 密钥、网关。", by_hand: "密钥必须用户自己填，不要在对话里索要。", tools: NONE },
    Feature { id: "settings-assistant", name: "设置·超级小伴", path: "设置 →「超级小伴」", page: Some(("settings", Some("assistant"))),
        does: "悬浮球/悬浮条、呼出快捷键、截图等。", by_hand: "对话里不能改（收起悬浮条可用 desktop_window_set）。", tools: NONE },
    Feature { id: "settings-voice", name: "设置·声音", path: "设置 →「声音」", page: Some(("settings", Some("voice"))),
        does: "语音朗读、语音识别、麦克风、音色。", by_hand: "对话里不能改。", tools: NONE },
    Feature { id: "settings-network", name: "设置·连接与网络", path: "设置 →「连接与网络」", page: Some(("settings", Some("network"))),
        does: "代理（HTTP/HTTPS/SOCKS5）、连接测试。", by_hand: "对话里不能改。", tools: NONE },
    Feature { id: "settings-safety", name: "设置·安全与权限", path: "设置 →「安全与权限」", page: Some(("settings", Some("safety"))),
        does: "文件与命令的权限、沙箱、确认策略。", by_hand: "权限策略必须用户自己改。",
        tools: &["request_accessibility_permission", "computer_use_diagnose", "computer_use_repair_permissions"] },
    Feature { id: "settings-about", name: "设置·关于与更新", path: "设置 →「关于与更新」，或侧栏铃铛", page: Some(("settings", Some("about"))),
        does: "版本、检查更新、公告、官网。", by_hand: "检查更新由用户点击。", tools: NONE },
    Feature { id: "community", name: "官网与论坛", path: "系统浏览器", page: None,
        does: "导出自创的壁纸/特效/组件/宠物/游戏为资源包，并打开论坛对应板块；打开官网或论坛页面。", by_hand: "发帖需要用户自己登录论坛。",
        tools: &["community_share", "community_open"] },
    Feature { id: "guide", name: "功能导览", path: "对话里", page: None,
        does: "app_guide 查功能清单和位置，open_page 打开控制台页面。", by_hand: "", tools: &["app_guide", "open_page"] },
];

/// Every tool the guide mentions, for the test that keeps it complete.
#[cfg(test)]
pub(super) fn covered_tools() -> std::collections::BTreeSet<&'static str> {
    FEATURES.iter().flat_map(|f| f.tools.iter().copied()).collect()
}

pub(super) fn validate_page(page: &str, tab: Option<&str>) -> Result<(), String> {
    if !PAGES.contains(&page) {
        return Err(format!("page 只能是 {}", PAGES.join("、")));
    }
    let allowed: &[&str] = match page { "studio" => STUDIO_TABS, "settings" => SETTINGS_TABS, _ => NONE };
    match tab {
        None => Ok(()),
        Some(tab) if allowed.contains(&tab) => Ok(()),
        Some(_) if allowed.is_empty() => Err(format!("{page} 没有 tab，不要传 tab")),
        Some(_) => Err(format!("{page} 的 tab 只能是 {}", allowed.join("、"))),
    }
}

/// The console's features in one paragraph for the system prompt, built from the table so it cannot drift from it.
/// Names with a "·" are tabs of one page ("资料库·收藏"), so they are listed under the page.
pub(super) fn overview() -> String {
    let mut groups: Vec<(&str, Vec<&str>)> = Vec::new();
    for f in FEATURES.iter().filter(|f| f.id != "guide") {
        let (group, tab) = match f.name.split_once('·') { Some((g, t)) => (g, Some(t)), None => (f.name, None) };
        match groups.iter_mut().find(|(g, _)| *g == group) {
            Some((_, tabs)) => tabs.extend(tab),
            None => groups.push((group, tab.into_iter().collect())),
        }
    }
    groups.iter().map(|(group, tabs)| if tabs.is_empty() { group.to_string() } else { format!("{group}（{}）", tabs.join("、")) }).collect::<Vec<_>>().join("；")
}

fn line(f: &Feature, detail: bool) -> String {
    let open = f.page.map(|(page, tab)| match tab {
        Some(tab) => format!("；打开：open_page(page={page}, tab={tab})"),
        None => format!("；打开：open_page(page={page})"),
    }).unwrap_or_default();
    let tools = if f.tools.is_empty() { String::new() } else { format!("；工具：{}", f.tools.join("、")) };
    if !detail { return format!("- {} {}｜{}{}", f.id, f.name, f.path, open); }
    let by_hand = if f.by_hand.is_empty() { String::new() } else { format!("\n  用户自己做：{}", f.by_hand) };
    format!("- {} {}｜位置：{}\n  能做：{}{}{}{}", f.id, f.name, f.path, f.does, tools, open, by_hand)
}

/// `topic` empty → the whole index; otherwise the features that mention it, in full.
pub(super) fn describe(topic: Option<&str>) -> String {
    let topic = topic.map(|t| t.trim().to_lowercase()).filter(|t| !t.is_empty());
    let Some(topic) = topic else {
        let index: Vec<String> = FEATURES.iter().map(|f| line(f, false)).collect();
        return format!("小伴桌面的功能（用 app_guide(topic) 看某项详情）：\n{}", index.join("\n"));
    };
    let hits: Vec<String> = FEATURES.iter().filter(|f| {
        [f.id, f.name, f.path, f.does, f.by_hand].iter().any(|text| text.to_lowercase().contains(&topic))
            || f.tools.iter().any(|tool| tool.contains(&topic))
    }).map(|f| line(f, true)).collect();
    if hits.is_empty() {
        let index: Vec<String> = FEATURES.iter().map(|f| line(f, false)).collect();
        return format!("没有功能提到“{topic}”。这不代表不支持：先看下面的完整清单，再决定是否需要告诉用户确实做不到。\n{}", index.join("\n"));
    }
    hits.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The quoted ids of `export type <name> = "a" | "b" ...;` in the console's types.
    fn ids(name: &str) -> Vec<String> {
        let source = include_str!("../../../src/ui/control-types.ts");
        let start = source.find(&format!("export type {name} =")).unwrap_or_else(|| panic!("{name} not found"));
        let declaration = &source[start..start + source[start..].find(';').unwrap()];
        declaration.split('"').skip(1).step_by(2).map(str::to_string).collect()
    }

    #[test]
    fn every_console_page_and_settings_category_is_in_the_guide() {
        // "chat" is a conversation, reached through new-task or the session list; "interactive" has no entry in the sidebar.
        for page in ids("MainPage").iter().filter(|p| !["chat", "interactive"].contains(&p.as_str())) {
            assert!(PAGES.contains(&page.as_str()), "page `{page}` is not openable by open_page");
            assert!(FEATURES.iter().any(|f| f.page.is_some_and(|(p, _)| p == page)), "page `{page}` has no entry in app_guide");
        }
        for (name, tabs, page) in [("StudioTab", STUDIO_TABS, "studio"), ("SettingsTab", SETTINGS_TABS, "settings")] {
            let mut declared = ids(name);
            let mut known: Vec<String> = tabs.iter().map(|t| t.to_string()).collect();
            declared.sort();
            known.sort();
            assert_eq!(declared, known, "{name} changed: update app_guide");
            for tab in tabs {
                assert!(FEATURES.iter().any(|f| f.page == Some((page, Some(*tab)))), "{page}/{tab} has no entry in app_guide");
            }
        }
    }

    #[test]
    fn feature_ids_are_unique_and_every_page_target_is_valid() {
        let mut seen = std::collections::HashSet::new();
        for f in FEATURES {
            assert!(seen.insert(f.id), "duplicate feature id {}", f.id);
            if let Some((page, tab)) = f.page { validate_page(page, tab).unwrap_or_else(|e| panic!("{}: {e}", f.id)); }
        }
    }

    #[test]
    fn open_page_validation_rejects_unknown_pages_and_tabs() {
        assert!(validate_page("studio", Some("widgets")).is_ok());
        assert!(validate_page("settings", None).is_ok());
        assert!(validate_page("chat", None).is_err());
        assert!(validate_page("studio", Some("games")).is_err());
        assert!(validate_page("games", Some("x")).is_err());
    }

    #[test]
    fn the_overview_names_every_page_and_tab_the_user_can_ask_about() {
        let map = overview();
        for expected in ["资料库（我的文档、云文档、收藏、记忆）", "AI 插件（专家、技能、连接器）", "设置（通用、外观、AI 模型、超级小伴、声音、连接与网络、安全与权限、关于与更新）", "桌面组件", "提醒与定时任务"] {
            assert!(map.contains(expected), "{expected} missing from {map}");
        }
        assert!(!map.contains("功能导览") && map.chars().count() < 600, "short enough to sit in every prompt: {}", map.chars().count());
    }

    #[test]
    fn describe_finds_by_word_and_never_dead_ends() {
        assert!(describe(Some("置顶")).contains("widget"));
        assert!(describe(Some("代理")).contains("settings-network"));
        assert!(describe(Some("widget_add")).contains("widget"));
        assert!(describe(Some("不存在的东西")).contains("完整清单"));
        assert!(describe(None).contains("open_page(page=studio, tab=pets)"));
    }
}
