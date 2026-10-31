package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.http.HttpMethod
import io.ktor.http.encodeURLParameter
import kotlinx.serialization.builtins.serializer
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonArray
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.longOrNull
import kotlinx.serialization.json.put
import kotlinx.serialization.serializer

/** 论坛接口（只读浏览 + 登录态相关的几项）。需要登录的请求带上 [token]，公开内容没登录也能看。 */
class ForumApi(client: HttpClient, config: CommunityConfig, private val token: suspend () -> String?) {
    private val http = JsonHttp(client, config.forumBase)
    private val site = config.forumBase.trimEnd('/')

    private fun query(vararg pairs: Pair<String, Any?>): String =
        pairs.filter { it.second != null && it.second.toString().isNotEmpty() }
            .joinToString("&") { "${it.first}=${it.second.toString().encodeURLParameter()}" }
            .let { if (it.isEmpty()) "" else "?$it" }

    private suspend inline fun <reified T> get(path: String): T = http.send(HttpMethod.Get, path, serializer<T>(), token = token())

    // ── 内容 ──
    suspend fun packs(kind: String? = null, sort: String? = null, q: String? = null, page: Int = 1): Page<Pack> =
        get("/api/packs${query("kind" to kind, "sort" to sort, "q" to q, "page" to page)}")

    suspend fun pack(id: Long): Pack = get("/api/packs/$id")
    suspend fun weeklyIssues(page: Int = 1): Page<WeeklyIssue> = get("/api/weekly${query("page" to page)}")
    suspend fun weekly(week: String): WeeklyDetail = get("/api/weekly/${week.encodeURLParameter()}")
    suspend fun projects(): ProjectList = get("/api/projects")
    suspend fun project(slug: String): Project = get("/api/projects/${slug.encodeURLParameter()}")

    // ── 讨论 ──
    /** [category] 可以用逗号同时指定几个；[home] 为 false 时连不上首页的板块（如闲聊灌水）也包含在内。 */
    suspend fun topics(
        sort: String = "latest",
        category: String? = null,
        page: Int = 1,
        tag: String? = null,
        helpStatus: String? = null,
        home: Boolean = true,
    ): Page<Topic> =
        get("/api/topics${query("sort" to sort, "category" to category, "tag" to tag, "help_status" to helpStatus, "home" to (if (home) true else null), "page" to page)}")

    suspend fun votePoll(topicId: Long, option: Int): Poll =
        write(HttpMethod.Put, "/api/topics/$topicId/poll/vote", buildJsonObject { put("option", option) })
            .let { CommunityJson.decodeFromJsonElement(serializer<VoteResult>(), it).poll }

    // ── 发现页 ──
    suspend fun home(): HomeBundle = get("/api/discover/home")
    suspend fun dailyNews(page: Int = 1): Page<NewsItem> = get("/api/discover/daily${query("page" to page)}")
    suspend fun repoIssue(issue: String? = null): RepoIssue = get("/api/discover/repos${query("issue" to issue)}")
    suspend fun repoIssues(): RepoIssueList = get("/api/discover/repos/issues")
    suspend fun modelBoard(): ModelBoard = get("/api/discover/models")
    suspend fun tracker(): Tracker = get("/api/discover/tracker")
    suspend fun navLinks(): NavGroups = get("/api/discover/nav")
    suspend fun hotSearch(): HotWords = get("/api/discover/hot-search")
    suspend fun docs(): DocGroups = get("/api/discover/docs")
    suspend fun tools(): ToolList = get("/api/tools")
    suspend fun tool(key: String): ToolDetail = get("/api/tools/${key.encodeURLParameter()}")
    suspend fun helpStats(): HelpStats = get("/api/help/stats")

    suspend fun topic(id: Long): Topic = get("/api/topics/$id")
    suspend fun posts(topicId: Long, page: Int = 1): Page<Post> = get("/api/topics/$topicId/posts${query("page" to page)}")
    suspend fun searchTopics(q: String, page: Int = 1): Page<Topic> = get("/api/search${query("q" to q, "page" to page)}")

    suspend fun categories(): List<Category> = get("/api/categories")

    // ── 发帖与互动（需要登录） ──
    private suspend fun write(method: HttpMethod, path: String, body: JsonObject? = null): JsonObject =
        http.send(method, path, JsonObject.serializer(), body = (body ?: buildJsonObject { }).toString(), token = token())

    /** 发新帖，返回帖子编号。分类必须是没有子分类的那一级。 */
    suspend fun createTopic(
        categoryId: Int,
        title: String,
        body: String,
        tags: List<String> = emptyList(),
        /** 只有资源求助板块可以设悬赏（0～1000 分），[bountyDays] 取 7、14 或 30。 */
        bounty: Int? = null,
        bountyDays: Int = 7,
        /** 2～6 个选项就带一个投票，[pollDays] 为空表示一直开放。 */
        pollOptions: List<String> = emptyList(),
        pollDays: Int? = null,
    ): Long {
        val resp = write(HttpMethod.Post, "/api/topics", buildJsonObject {
            put("category_id", categoryId); put("title", title); put("body", body)
            if (tags.isNotEmpty()) put("tags", buildJsonArray { tags.forEach { add(JsonPrimitive(it)) } })
            if (bounty != null) put("help", buildJsonObject { put("bounty", bounty); put("days", bountyDays) })
            if (pollOptions.isNotEmpty()) put("poll", buildJsonObject {
                put("options", buildJsonArray { pollOptions.forEach { add(JsonPrimitive(it)) } })
                if (pollDays != null) put("days", pollDays)
            })
        })
        return resp["id"]?.jsonPrimitive?.longOrNull ?: throw ApiException(200, "服务返回了无法识别的内容")
    }

    /** 回复帖子；[replyTo] 是被回复的评论编号。 */
    suspend fun reply(topicId: Long, body: String, replyTo: Long? = null) {
        write(HttpMethod.Post, "/api/topics/$topicId/posts", buildJsonObject { put("body", body); if (replyTo != null) put("reply_to", replyTo) })
    }

    /** 求助者采纳一条回答，悬赏积分随之发给回答者。 */
    suspend fun acceptAnswer(topicId: Long, postId: Long) { write(HttpMethod.Post, "/api/topics/$topicId/help/accept", buildJsonObject { put("post_id", postId) }) }

    suspend fun setTopicLiked(id: Long, liked: Boolean) { write(if (liked) HttpMethod.Put else HttpMethod.Delete, "/api/topics/$id/like") }
    suspend fun setTopicBookmarked(id: Long, on: Boolean) { write(if (on) HttpMethod.Put else HttpMethod.Delete, "/api/topics/$id/bookmark") }

    // ── 登录态 ──
    suspend fun session(): SessionInfo = http.send(HttpMethod.Post, "/api/session", serializer(), body = "{}", token = token())
    suspend fun unreadCount(): Int = get<Unread>("/api/me/notifications/unread").unread

    // ── 分享 ──
    /** 手机上分享的是网页链接，对方在电脑上打开后再用小伴应用。 */
    fun packUrl(id: Long) = "$site/packs/$id"
    fun topicUrl(id: Long) = "$site/t/$id"
    fun weeklyUrl(week: String) = "$site/weekly/$week"
    fun projectUrl(slug: String) = "$site/projects/$slug"

    /** 站内图片地址（`/api/images/{id}`）补成完整链接；已经是完整链接的原样返回。 */
    fun imageUrl(path: String) = if (path.startsWith("http")) path else "$site$path"
}
