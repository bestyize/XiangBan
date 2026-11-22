package xyz.xiangban.xiaoban.common.community

enum class ForumSection { Ideas, Discuss, Resources, Site }

data class ForumDiscussionTab(
    val key: String,
    val label: String,
    val category: Category? = null,
    val topicTag: String? = null,
)

private val discussionOrder = listOf(
    "tag:design", "casual-games", "tag:3d", "learn", "casual-deals", "casual-travel",
    "casual-screen", "casual-feelings", "casual-food", "casual-work",
)

private val hiddenDiscussionBoards = setOf(
    "casual-offwork", "casual-loss", "casual-dream", "casual-think", "casual-books", "casual-fun",
)

private val navigationLabels = mapOf(
    "learn" to "教程",
    "tech" to "教程",
    "casual-screen" to "影视",
    "casual-feelings" to "情感",
    "casual-indie" to "副业",
    "casual-gear" to "数码",
    "casual-chat" to "闲聊",
    "resource-help" to "求助",
    "share-help" to "求助",
)

/** 分类接口提供板块名称、归属和发帖限制，浏览和编辑器使用同一份规则。 */
class ForumCategories(categories: List<Category>) {
    private val byId = categories.associateBy { it.id }
    private val parentIds = categories.mapNotNull { it.parentId }.toSet()
    private val active = categories.filter { category ->
        category.status == "active" && (category.parentId == null || parent(category)?.status == "active")
    }
    val boards = active.filter { it.id !in parentIds }

    fun parent(category: Category): Category? = byId[category.parentId]

    fun find(slug: String?): Category? = active.firstOrNull { it.slug == slug }

    fun section(category: Category): ForumSection {
        val parent = parent(category)
        return when (category.section ?: parent?.section) {
            "ideas" -> ForumSection.Ideas
            "discuss" -> ForumSection.Discuss
            "resources" -> ForumSection.Resources
            "site" -> ForumSection.Site
            else -> when (parent?.slug ?: category.slug) {
                "create", "xiaoban" -> ForumSection.Ideas
                "resources", "share" -> ForumSection.Resources
                "site" -> ForumSection.Site
                else -> ForumSection.Discuss
            }
        }
    }

    fun activeIn(section: ForumSection): List<Category> = active.filter { section(it) == section }

    fun boardsIn(section: ForumSection): List<Category> = boards.filter { section(it) == section }

    val discussionFeedBoards = boardsIn(ForumSection.Discuss).filterNot { it.slug in hiddenDiscussionBoards }

    val resourceTabBoards = boardsIn(ForumSection.Resources).filterNot(Category::isUncategorized)

    val discussionTabs: List<ForumDiscussionTab> = run {
        val primary = active.filter { category ->
            val area = section(category)
            val parent = parent(category)
            val isDiscussion = area == ForumSection.Discuss || area == ForumSection.Site
            val isTopLevel = parent == null && (category.slug != "casual" || category in boards)
            isDiscussion && category.slug !in hiddenDiscussionBoards && (isTopLevel || parent?.slug == "casual")
        }
        val tabs = primary.associate { it.slug to ForumDiscussionTab(it.slug, it.navigationLabel, category = it) } + mapOf(
            "tag:design" to ForumDiscussionTab("tag:design", "设计", topicTag = "设计"),
            "tag:3d" to ForumDiscussionTab("tag:3d", "3D", topicTag = "3d"),
        )
        val defaults = primary.filter(Category::isUncategorized).map(Category::slug)
        (defaults + discussionOrder + tabs.keys).distinct().mapNotNull(tabs::get)
    }

    val discussionBoards = discussionTabs.mapNotNull(ForumDiscussionTab::category).filter { it in boards }

    fun discussionTab(key: String?): ForumDiscussionTab? {
        if (key == null) return null
        discussionTabs.firstOrNull { it.key == key }?.let { return it }
        val parent = find(key)?.let(::parent)?.slug ?: when {
            key == "tech" || key.startsWith("tech-") || key.startsWith("learn-") -> "learn"
            key.startsWith("site-") -> "site"
            else -> return null
        }
        return discussionTabs.firstOrNull { it.key == parent }
    }

    fun allowsShortTitle(board: Category): Boolean =
        board.allowShortTitle ?: (board.slug == "casual" || parent(board)?.slug == "casual")

    fun canPost(board: Category, trustLevel: Int): Boolean = postingEnabled(board) && trustLevel >= board.requiredPostLevel

    fun defaultBoard(options: List<Category>, trustLevel: Int): Category? =
        options.firstOrNull { it.isUncategorized && canPost(it, trustLevel) }

    fun postingEnabled(board: Category): Boolean =
        board in boards && board.postingEnabled && parent(board)?.postingEnabled != false
}

val Category.requiredPostLevel: Int
    get() = if (adminOnly) 3 else minPostLevel.coerceIn(0, 3)

val Category.isUncategorized: Boolean
    get() = kind == "uncategorized"

val Category.navigationLabel: String
    get() = navigationLabels[slug] ?: name

val Category.isHelpBoard: Boolean
    get() = slug == "resource-help" || slug == "share-help"
