package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.http.HttpMethod
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import kotlinx.serialization.serializer

/** Minimal public pack/scheme card. Private manifests, files and moderation reasons are never needed. */
@Serializable
data class CommunityContentCard(
    val id: Long, val title: String, val summary: String = "", val kind: String? = null,
    val version: Int? = null, val author: PackAuthor? = null, val status: String? = null,
    val items: List<SchemePackRef> = emptyList(),
)
@Serializable data class ContentBookmarkState(val bookmarked: Boolean)
@Serializable data class ContentBookmark(val kind: String, val id: Long, @SerialName("created_at") val createdAt: String, val content: CommunityContentCard)

internal fun requireContentTarget(kind: String, id: Long) {
    if (kind !in listOf("pack", "scheme") || id <= 0) throw ApiException(400, "请选择有效的作品或方案")
}

class ContentSocialApi(client: HttpClient, config: CommunityConfig, private val token: suspend () -> String?) {
    private val http = JsonHttp(client, config.forumBase)
    private suspend fun access() = token() ?: throw ApiException(401, "请登录后收藏或举报")
    suspend fun state(kind: String, id: Long): ContentBookmarkState {
        requireContentTarget(kind, id)
        return http.send(HttpMethod.Get, "/api/content-social/$kind/$id", serializer(), token = access())
    }
    suspend fun bookmark(kind: String, id: Long, on: Boolean): ContentBookmarkState {
        requireContentTarget(kind, id)
        return http.send(if (on) HttpMethod.Put else HttpMethod.Delete, "/api/content-social/$kind/$id/bookmark", serializer(), body = "{}", token = access())
    }
    suspend fun bookmarks(kind: String, page: Int = 1): Page<ContentBookmark> {
        requireContentTarget(kind, 1)
        val result: Page<ContentBookmark> = http.send(HttpMethod.Get, "/api/me/content-bookmarks?kind=$kind&page=${page.coerceAtLeast(1)}", serializer(), token = access())
        if (result.items.any { it.kind != kind || it.id != it.content.id || it.id <= 0 }) throw ApiException(200, "收藏内容暂不可用，请刷新")
        return result
    }
    suspend fun report(kind: String, id: Long, reason: String, detail: String) {
        requireContentTarget(kind, id)
        if (reason !in CommunityReportReasons || detail.length > 500) throw ApiException(400, "请选择举报原因，说明最多 500 字")
        http.send(HttpMethod.Post, "/api/content-social/$kind/$id/report", serializer<JsonObject>(), body = buildJsonObject {
            put("reason", reason); put("detail", detail.trim())
        }.toString(), token = access())
    }
}
