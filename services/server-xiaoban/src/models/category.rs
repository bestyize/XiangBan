use serde::Serialize;

/// A browsable wallpaper category. `keyword` is the query sent to every provider,
/// so categories behave the same regardless of each platform's own taxonomy.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Category {
    pub id: &'static str,
    pub name: &'static str,
    pub keyword: &'static str,
}

pub const CATEGORIES: &[Category] = &[
    Category { id: "nature", name: "自然", keyword: "nature landscape" },
    Category { id: "space", name: "星空", keyword: "galaxy stars space" },
    Category { id: "city", name: "城市", keyword: "city night skyline" },
    Category { id: "cyberpunk", name: "赛博朋克", keyword: "cyberpunk neon" },
    Category { id: "minimal", name: "极简", keyword: "minimalist" },
    Category { id: "abstract", name: "抽象", keyword: "abstract art" },
    Category { id: "dark", name: "暗色", keyword: "dark moody" },
    Category { id: "ocean", name: "海洋", keyword: "ocean sea waves" },
    Category { id: "mountain", name: "山峦", keyword: "mountains" },
    Category { id: "animals", name: "动物", keyword: "wildlife animals" },
    Category { id: "architecture", name: "建筑", keyword: "architecture" },
    Category { id: "flowers", name: "花卉", keyword: "flowers" },
];

pub fn find(id: &str) -> Option<&'static Category> {
    CATEGORIES.iter().find(|c| c.id == id)
}
