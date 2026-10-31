package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.http.HttpMethod
import io.ktor.http.encodeURLParameter
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.update
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonArray
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import kotlinx.serialization.serializer

@Serializable
data class NotificationActor(val uid: String, val nickname: String? = null, @SerialName("avatar_url") val avatarUrl: String? = null)
@Serializable
data class CommunityNotification(
    val id: Long, val kind: String, val title: String, val excerpt: String = "", val actor: NotificationActor? = null,
    @SerialName("topic_id") val topicId: Long? = null, @SerialName("post_id") val postId: Long? = null,
    @SerialName("pack_id") val packId: Long? = null, val read: Boolean = false,
    @SerialName("created_at") val createdAt: String = "",
)
@Serializable
data class CommunityProfile(
    val uid: String, val nickname: String,
    @SerialName("avatar_url") val avatarUrl: String? = null,
    @SerialName("trust_level") val trustLevel: Int = 0,
    @SerialName("topic_count") val topicCount: Long = 0,
    @SerialName("post_count") val postCount: Long = 0,
    @SerialName("following_count") val followingCount: Long = 0,
    @SerialName("follower_count") val followerCount: Long = 0,
    val banned: Boolean = false, val following: Boolean = false, val blocked: Boolean = false,
)
@Serializable
data class CommunityFollowState(val following: Boolean, @SerialName("following_count") val followingCount: Long, @SerialName("follower_count") val followerCount: Long)
@Serializable
data class CommunityBlockState(val blocked: Boolean)
@Serializable
data class CommunityBlocks(val items: List<CommunityBlockedUser> = emptyList())
@Serializable
data class CommunityBlockedUser(val uid: String, val nickname: String, @SerialName("blocked_at") val blockedAt: String = "")
@Serializable
private data class MarkedNotifications(val marked: Long)

val CommunityReportReasons = linkedMapOf("spam" to "垃圾广告或私发推广", "account_trade" to "分享或交易 API Key、账号",
    "abuse" to "人身攻击或开盒", "politics" to "引战或时政", "copyright" to "侵犯版权", "illegal" to "违法违规", "other" to "其他")

fun notificationHeadline(n: CommunityNotification): String {
    val who = n.actor?.nickname ?: "有人"
    return when (n.kind) {
        "reply" -> "$who 回复了你的帖子「${n.title}」"
        "reply_to" -> "$who 回复了你在「${n.title}」里的回复"
        "mention" -> "$who 在「${n.title}」里提到了你"
        "accepted" -> "你在「${n.title}」里的回答被采纳了"
        "pack_approved" -> "内容包「${n.title}」通过了审核"
        "pack_rejected" -> "内容包「${n.title}」未通过审核"
        else -> n.title.ifBlank { "社区消息" }
    }
}

/** 只使用既有社区接口；收藏和举报对象范围与服务器完全一致。 */
class SocialApi(client: HttpClient, config: CommunityConfig, private val token: suspend () -> String?) {
    private val http = JsonHttp(client, config.forumBase)
    private val changes = MutableStateFlow(0L)
    val notificationRevision: StateFlow<Long> = changes
    private suspend fun access() = token() ?: throw ApiException(401, "请先登录小伴账号")
    private suspend inline fun <reified T> privateGet(path: String): T = http.send(HttpMethod.Get, path, serializer(), token = access())
    private suspend inline fun <reified T> publicGet(path: String): T = http.send(HttpMethod.Get, path, serializer(), token = token())

    suspend fun notifications(page: Int = 1, unread: Boolean = false): Page<CommunityNotification> = privateGet("/api/me/notifications?page=$page&unread=$unread")
    suspend fun markRead(ids: List<Long>? = null): Long {
        if (ids != null && (ids.size > 300 || ids.any { it <= 0 })) throw ApiException(400, "通知编号无效")
        val body = buildJsonObject { ids?.let { put("ids", buildJsonArray { it.distinct().forEach { id -> add(JsonPrimitive(id)) } }) } }
        val marked: MarkedNotifications = http.send(HttpMethod.Post, "/api/me/notifications/read", serializer(), body = body.toString(), token = access())
        changes.update { it + 1 }
        return marked.marked
    }
    suspend fun bookmarks(page: Int = 1): Page<Topic> = privateGet("/api/me/bookmarks?page=$page")
    suspend fun profile(uid: String): CommunityProfile = publicGet("/api/users/${uid.encodeURLParameter()}")
    suspend fun userTopics(uid: String, page: Int = 1): Page<Topic> = publicGet("/api/users/${uid.encodeURLParameter()}/topics?page=$page")
    suspend fun connections(uid: String, followers: Boolean, page: Int = 1): Page<PublicUser> =
        publicGet("/api/users/${uid.encodeURLParameter()}/${if (followers) "followers" else "following"}?page=$page")
    suspend fun follow(uid: String, on: Boolean): CommunityFollowState = http.send(if (on) HttpMethod.Put else HttpMethod.Delete,
        "/api/users/${uid.encodeURLParameter()}/follow", serializer(), body = "{}", token = access())
    suspend fun block(uid: String, on: Boolean): CommunityBlockState = http.send(if (on) HttpMethod.Put else HttpMethod.Delete,
        "/api/users/${uid.encodeURLParameter()}/block", serializer(), body = "{}", token = access())
    suspend fun blocks(): CommunityBlocks = privateGet("/api/me/blocks")
    suspend fun report(targetType: String, id: Long, reason: String, detail: String) {
        if (targetType !in listOf("topic", "post") || id <= 0 || reason !in CommunityReportReasons || detail.length > 500)
            throw ApiException(400, "请检查举报对象、原因和说明（最多 500 字）")
        val body = buildJsonObject { put("target_type", targetType); put("target_id", id); put("reason", reason); put("detail", detail.trim()) }
        http.send(HttpMethod.Post, "/api/reports", serializer<JsonObject>(), body = body.toString(), token = access())
    }
}
