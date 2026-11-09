package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.client.request.header
import io.ktor.client.request.prepareGet
import io.ktor.client.statement.bodyAsChannel
import io.ktor.client.statement.bodyAsText
import io.ktor.http.HttpHeaders
import io.ktor.utils.io.readRemaining
import kotlinx.io.readByteArray
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

private val ProjectReadmePath = Regex("projects/[a-f0-9]{64}[.]json")

/** 论坛接口（只读浏览 + 登录态相关的几项）。需要登录的请求带上 [token]，公开内容没登录也能看。 */
class ForumApi(private val client: HttpClient, config: CommunityConfig, private val token: suspend () -> String?) {
    private val http = JsonHttp(client, config.forumBase)
    private val packHttp = if (config.runtimeV2) JsonHttp(client, config.forumBase, mapOf("X-Xiaoban-Runtime" to "1,2")) else http
    private val site = config.forumBase.trimEnd('/')
    private val uploads = ForumUploads(client, config, token)

    private fun query(vararg pairs: Pair<String, Any?>): String =
        pairs.filter { it.second != null && it.second.toString().isNotEmpty() }
            .joinToString("&") { "${it.first}=${it.second.toString().encodeURLParameter()}" }
            .let { if (it.isEmpty()) "" else "?$it" }

    private suspend inline fun <reified T> get(path: String): T = http.send(HttpMethod.Get, path, serializer<T>(), token = token())
    private suspend inline fun <reified T> getPack(path: String): T = packHttp.send(HttpMethod.Get, path, serializer<T>(), token = token())

    // ── 内容 ──
    suspend fun packs(kind: String? = null, sort: String? = null, q: String? = null, page: Int = 1, tag: String? = null, following: Boolean = false, featured: Boolean = false): Page<Pack> =
        getPack("/api/packs${query("kind" to kind, "sort" to sort, "q" to q, "page" to page, "tag" to tag, "following" to following.takeIf { it }, "featured" to featured.takeIf { it })}")

    /** The public tool catalog can load while the account session is being restored. */
    suspend fun featuredTools(): List<Pack> =
        packHttp.send(HttpMethod.Get, "/api/packs?kind=app&page=1&featured=true", serializer<Page<Pack>>()).items

    suspend fun pack(id: Long, recordView: Boolean = false): Pack =
        getPack("/api/packs/$id${query("view" to recordView.takeIf { it })}")
    suspend fun gameDownload(id: Long, version: Int): CommunityGameDownload = getPack("/api/packs/$id/download?version=$version")
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
        featured: Boolean = false,
        activeOnly: Boolean = false,
    ): Page<Topic> =
        get("/api/topics${query("sort" to sort, "category" to category, "tag" to tag, "help_status" to helpStatus, "home" to (if (home) true else null), "featured" to (if (featured) true else null), "page" to page, "active_only" to activeOnly.takeIf { it })}")

    suspend fun votePoll(topicId: Long, option: Int): Poll =
        write(HttpMethod.Put, "/api/topics/$topicId/poll/vote", buildJsonObject { put("option", option) })
            .let { CommunityJson.decodeFromJsonElement(serializer<VoteResult>(), it).poll }

    // ── 发现页 ──
    suspend fun home(): HomeBundle = get("/api/discover/home/v2")
    suspend fun dailyNews(page: Int = 1): Page<NewsItem> = get("/api/discover/daily${query("page" to page)}")
    suspend fun frontier(page: Int = 1): Page<NewsItem> = get("/api/discover/frontier${query("page" to page)}")
    private fun articlePath(key: String, frontier: Boolean) = "/api/discover/${if (frontier) "frontier" else "daily"}/${key.encodeURLParameter()}"
    suspend fun news(key: String, frontier: Boolean = false): NewsItem = get(articlePath(key, frontier))
    suspend fun newsComments(key: String, page: Int = 1, frontier: Boolean = false): Page<NewsComment> = get("${articlePath(key, frontier)}/comments${query("page" to page)}")
    suspend fun commentNews(key: String, body: String, replyTo: Long? = null, frontier: Boolean = false) {
        write(HttpMethod.Post, "${articlePath(key, frontier)}/comments", buildJsonObject {
            put("body", body); if (replyTo != null) put("reply_to", replyTo)
        })
    }
    suspend fun likeNewsComment(key: String, id: Long, liked: Boolean, frontier: Boolean = false) {
        write(if (liked) HttpMethod.Put else HttpMethod.Delete, "${articlePath(key, frontier)}/comments/$id/like")
    }
    suspend fun deleteNewsComment(key: String, id: Long, frontier: Boolean = false) {
        write(HttpMethod.Delete, "${articlePath(key, frontier)}/comments/$id")
    }
    private fun repoPath(owner: String, repo: String) = "/api/discover/repo/${owner.encodeURLParameter()}/${repo.encodeURLParameter()}"
    suspend fun repoDetail(owner: String, repo: String): RepoDetail = get(repoPath(owner, repo))
    suspend fun softwareSummaries(): Map<String, SoftwareSummary> =
        http.send(HttpMethod.Get, "/api/open-source/software", serializer())

    suspend fun softwareSnapshot(project: String): SoftwareSnapshot? =
        http.send(HttpMethod.Get, "/api/open-source/software/${project.split('/').joinToString("/") { it.encodeURLParameter() }}", serializer())

    suspend fun openSourceCatalog(): OpenSourceCatalog = get("/api/open-source")
    suspend fun openSourceProject(owner: String, repo: String): OpenSourceCatalog = http.send(
        HttpMethod.Get,
        "/api/open-source/projects/${owner.encodeURLParameter()}/${repo.encodeURLParameter()}",
        OpenSourceCatalog.serializer(),
    )
    suspend fun projectReadme(fullName: String): ProjectReadme? {
        val root = "/open-source-readmes"
        val index = try {
            http.send(HttpMethod.Get, "$root/index.json", ProjectReadmeIndex.serializer())
        } catch (e: ApiException) {
            if (e.status == 404) return null
            throw e
        }
        val path = index.projects[fullName.lowercase()] ?: return null
        if (!ProjectReadmePath.matches(path)) throw ApiException(200, "README 索引无效")
        val readme = http.send(HttpMethod.Get, "$root/$path", ProjectReadme.serializer())
        if (readme.schema != 1 || !readme.fullName.equals(fullName, ignoreCase = true)) {
            throw ApiException(200, "README 内容校验失败")
        }
        return readme
    }
    suspend fun repoComments(owner: String, repo: String, page: Int = 1): Page<NewsComment> = get("${repoPath(owner, repo)}/comments${query("page" to page)}")
    suspend fun commentRepo(owner: String, repo: String, body: String, replyTo: Long? = null) {
        write(HttpMethod.Post, "${repoPath(owner, repo)}/comments", buildJsonObject {
            put("body", body); if (replyTo != null) put("reply_to", replyTo)
        })
    }
    suspend fun likeRepoComment(owner: String, repo: String, id: Long, liked: Boolean) {
        write(if (liked) HttpMethod.Put else HttpMethod.Delete, "${repoPath(owner, repo)}/comments/$id/like")
    }
    suspend fun deleteRepoComment(owner: String, repo: String, id: Long) {
        write(HttpMethod.Delete, "${repoPath(owner, repo)}/comments/$id")
    }
    suspend fun repoIssue(issue: String? = null): RepoIssue = get("/api/discover/repos/v2${query("issue" to issue)}")
    suspend fun repoIssues(): RepoIssueList = get("/api/discover/repos/issues/v2")
    suspend fun modelBoard(): ModelBoard = get("/api/discover/models")
    suspend fun liveModelBoard(): LiveModelBoard = get("/api/discover/models/v2")
    suspend fun modelUsage(date: String? = null): ModelUsageBoard = get("/api/discover/model-usage${query("date" to date)}")
    suspend fun modelUsageDates(): ModelUsageDates = get("/api/discover/model-usage/dates")
    suspend fun tracker(): Tracker = get("/api/discover/tracker/v2")
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
    suspend fun followingTopics(page: Int = 1, category: String? = null, tag: String? = null, home: Boolean = false, activeOnly: Boolean = false): Page<Topic> =
        get("/api/me/following/topics${query("page" to page, "category" to category, "tag" to tag, "home" to home.takeIf { it }, "active_only" to activeOnly.takeIf { it })}")
    suspend fun topicTags(): List<DiscussionTag> = get("/api/tags")

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
        attachments: List<ForumAttachment> = emptyList(),
    ): Long {
        validateTopicAttachments(attachments)
        val resp = write(HttpMethod.Post, "/api/topics", buildJsonObject {
            put("category_id", categoryId); put("title", title); put("body", body)
            if (tags.isNotEmpty()) put("tags", buildJsonArray { tags.forEach { add(JsonPrimitive(it)) } })
            if (bounty != null) put("help", buildJsonObject { put("bounty", bounty); put("days", bountyDays) })
            if (attachments.isNotEmpty()) {
                put("attachment_ids", buildJsonArray { attachments.forEach { add(JsonPrimitive(it.id)) } })
                put("attachment_prices", buildJsonArray { attachments.forEach { attachment -> add(buildJsonObject {
                    put("id", attachment.id); put("price", attachment.price)
                }) } })
            }
            if (pollOptions.isNotEmpty()) put("poll", buildJsonObject {
                put("options", buildJsonArray { pollOptions.forEach { add(JsonPrimitive(it)) } })
                if (pollDays != null) put("days", pollDays)
            })
        })
        return resp["id"]?.jsonPrimitive?.longOrNull ?: throw ApiException(200, "服务返回了无法识别的内容")
    }

    /** 回复帖子；[replyTo] 是被回复的评论编号。 */
    suspend fun reply(topicId: Long, body: String, replyTo: Long? = null, workRefs: List<WorkReference>? = null, attachments: List<ForumAttachment>? = null) {
        if (topicId <= 0 || body.trim().length < 2) throw ApiException(400, "请输入至少 2 个字的回复")
        write(HttpMethod.Post, "/api/topics/$topicId/posts", buildJsonObject {
            put("body", body); if (replyTo != null) put("reply_to", replyTo)
            replyAssociations(workRefs, attachments)
        })
    }
    suspend fun replySource(postId: Long): ReplySource {
        if (postId <= 0) throw ApiException(400, "回复编号无效")
        return get("/api/posts/$postId/source")
    }
    suspend fun editReply(postId: Long, body: String, workRefs: List<WorkReference>? = null, attachments: List<ForumAttachment>? = null) {
        if (postId <= 0 || body.trim().length < 2) throw ApiException(400, "请输入至少 2 个字的回复")
        write(HttpMethod.Patch, "/api/posts/$postId", buildJsonObject { put("body", body); replyAssociations(workRefs, attachments) })
    }

    /** 求助者采纳一条回答，悬赏积分随之发给回答者。 */
    suspend fun closeHelp(topicId: Long) {
        write(HttpMethod.Post, "/api/topics/$topicId/help/close")
    }

    suspend fun acceptAnswer(topicId: Long, postId: Long) { write(HttpMethod.Post, "/api/topics/$topicId/help/accept", buildJsonObject { put("post_id", postId) }) }

    suspend fun setPostLiked(id: Long, liked: Boolean): LikeState {
        if (id <= 0) throw ApiException(400, "请选择有效评论")
        return http.send(if (liked) HttpMethod.Put else HttpMethod.Delete, "/api/posts/$id/like", serializer(), body = "{}", token = token() ?: throw ApiException(401, "请先登录"))
    }
    suspend fun setTopicLiked(id: Long, liked: Boolean): LikeState =
        CommunityJson.decodeFromJsonElement(LikeState.serializer(), write(if (liked) HttpMethod.Put else HttpMethod.Delete, "/api/topics/$id/like"))
    suspend fun setTopicBookmarked(id: Long, on: Boolean) { write(if (on) HttpMethod.Put else HttpMethod.Delete, "/api/topics/$id/bookmark") }

    // ── 登录态 ──
    suspend fun uploadSession(): ForumUploadSession = http.send(HttpMethod.Post, "/api/session", serializer(), body = "{}", token = token())
    suspend fun uploadImage(file: ForumUploadFile, maxBytes: Long): ForumImage = uploads.image(file, maxBytes)
    suspend fun purchaseAttachment(attachment: ForumAttachment): AttachmentPurchase =
        write(HttpMethod.Post, "/api/attachments/${attachment.id.encodeURLParameter()}/purchase",
            buildJsonObject { put("price", attachment.price) })
            .let { CommunityJson.decodeFromJsonElement(serializer<AttachmentPurchase>(), it) }

    suspend fun downloadAttachment(attachment: ForumAttachment): ByteArray {
        val size = attachment.size
        if (size !in 0..(64L * 1024 * 1024)) throw ApiException(0, "附件大小超出 App 下载范围")
        val access = token()
        return client.prepareGet("$site/api/attachments/${attachment.id.encodeURLParameter()}") {
            if (access != null) header(HttpHeaders.Authorization, "Bearer $access")
        }.execute { response ->
            if (response.status.value !in 200..299) {
                throw parseApiError(response.status.value, response.bodyAsText())
            }
            if (response.status.value != 200) throw ApiException(0, "附件下载响应无效")
            val bytes = response.bodyAsChannel().readRemaining(size + 1).readByteArray()
            if (bytes.size.toLong() != size) throw ApiException(0, "附件下载不完整，请重试")
            bytes
        }
    }

    suspend fun uploadAttachment(file: ForumUploadFile, maxBytes: Long): ForumAttachment = uploads.attachment(file, maxBytes)
    suspend fun deletePendingAttachment(id: String) {
        if (!ForumAssetId.matches(id)) throw ApiException(400, "附件编号无效")
        write(HttpMethod.Delete, "/api/attachments/$id")
    }
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
