package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.http.HttpMethod
import io.ktor.http.encodeURLParameter
import kotlinx.serialization.builtins.serializer
import kotlinx.serialization.json.JsonObject
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
    suspend fun topics(sort: String = "latest", category: String? = null, page: Int = 1): Page<Topic> =
        get("/api/topics${query("sort" to sort, "category" to category, "home" to true, "page" to page)}")

    suspend fun topic(id: Long): Topic = get("/api/topics/$id")
    suspend fun posts(topicId: Long, page: Int = 1): Page<Post> = get("/api/topics/$topicId/posts${query("page" to page)}")
    suspend fun searchTopics(q: String, page: Int = 1): Page<Topic> = get("/api/search${query("q" to q, "page" to page)}")

    suspend fun categories(): List<Category> = get("/api/categories")

    // ── 发帖与互动（需要登录） ──
    private suspend fun write(method: HttpMethod, path: String, body: JsonObject? = null): JsonObject =
        http.send(method, path, JsonObject.serializer(), body = (body ?: buildJsonObject { }).toString(), token = token())

    /** 发新帖，返回帖子编号。分类必须是没有子分类的那一级。 */
    suspend fun createTopic(categoryId: Int, title: String, body: String): Long {
        val resp = write(HttpMethod.Post, "/api/topics", buildJsonObject { put("category_id", categoryId); put("title", title); put("body", body) })
        return resp["id"]?.jsonPrimitive?.longOrNull ?: throw ApiException(200, "服务返回了无法识别的内容")
    }

    /** 回复帖子；[replyTo] 是被回复的评论编号。 */
    suspend fun reply(topicId: Long, body: String, replyTo: Long? = null) {
        write(HttpMethod.Post, "/api/topics/$topicId/posts", buildJsonObject { put("body", body); if (replyTo != null) put("reply_to", replyTo) })
    }

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
}
