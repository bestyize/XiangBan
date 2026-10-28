package xyz.xiangban.xiaoban.common.community

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.longOrNull

@Serializable
data class Page<T>(val items: List<T> = emptyList(), val page: Int = 1, @SerialName("page_size") val pageSize: Int = 20, val total: Int = 0) {
    val hasMore: Boolean get() = page * pageSize < total
}

// ───────────── 内容包 ─────────────

@Serializable
data class PackAuthor(val uid: String, val nickname: String? = null)

@Serializable
data class PackVersion(
    val version: Int,
    val label: String = "",
    val sha256: String = "",
    val size: Long = 0,
    val changelog: String = "",
    @SerialName("created_at") val createdAt: String = "",
)

/** 内容包。列表里没有 [versions]，详情里有。 */
@Serializable
data class Pack(
    val id: Long,
    val kind: String,
    val slug: String = "",
    val title: String,
    val summary: String = "",
    val tags: List<String> = emptyList(),
    val author: PackAuthor? = null,
    @SerialName("trust_level") val trustLevel: Int = 2,
    val status: String = "published",
    val version: Int = 1,
    @SerialName("cover_url") val coverUrl: String? = null,
    @SerialName("apply_count") val applyCount: Long = 0,
    @SerialName("topic_id") val topicId: Long? = null,
    @SerialName("one_click") val oneClick: Boolean = false,
    @SerialName("pure_data") val pureData: Boolean = false,
    @SerialName("updated_at") val updatedAt: String = "",
    val versions: List<PackVersion> = emptyList(),
) {
    /** 手机上不能应用内容包，只能看、复制链接或下载，应用要回到电脑上的小伴。 */
    val trustLabel: String get() = when (trustLevel) { 0 -> "官方"; 1 -> "已审核"; else -> "未审核" }
}

// ───────────── 周刊与项目 ─────────────

@Serializable
data class WeeklyIssue(val week: String, val title: String, val summary: String = "", @SerialName("published_at") val publishedAt: String? = null)

@Serializable
data class WeeklyDetail(
    val week: String,
    val title: String,
    val summary: String = "",
    val items: List<JsonObject> = emptyList(),
) {
    fun entries(): List<WeeklyEntry> = items.mapNotNull(::parseWeeklyEntry)
}

sealed interface WeeklyEntry {
    data class PackEntry(val packId: Long, val kind: String, val title: String, val summary: String, val coverUrl: String?, val note: String) : WeeklyEntry
    data class TopicEntry(val topicId: Long, val title: String, val summary: String, val note: String) : WeeklyEntry
    data class TextEntry(val title: String?, val note: String) : WeeklyEntry
}

private fun JsonObject.str(key: String): String? = (this[key] as? JsonElement)?.takeIf { it !is kotlinx.serialization.json.JsonNull }?.jsonPrimitive?.contentOrNull
private fun JsonObject.long(key: String): Long? = (this[key] as? JsonElement)?.takeIf { it !is kotlinx.serialization.json.JsonNull }?.jsonPrimitive?.longOrNull

/** 周刊条目按 `type` 区分；不认识的类型直接跳过，服务端以后加新类型不会让旧版本崩溃。 */
fun parseWeeklyEntry(item: JsonObject): WeeklyEntry? = when (item.str("type")) {
    "pack" -> item.long("id")?.let { WeeklyEntry.PackEntry(it, item.str("kind").orEmpty(), item.str("title").orEmpty(), item.str("summary").orEmpty(), item.str("cover_url"), item.str("note").orEmpty()) }
    "topic" -> item.long("id")?.let { WeeklyEntry.TopicEntry(it, item.str("title").orEmpty(), item.str("summary").orEmpty(), item.str("note").orEmpty()) }
    "text" -> WeeklyEntry.TextEntry(item.str("title"), item.str("note").orEmpty())
    else -> null
}

@Serializable
data class ProjectLink(val title: String, val url: String)

@Serializable
data class Project(
    val slug: String,
    val name: String,
    val summary: String = "",
    val links: List<ProjectLink> = emptyList(),
    @SerialName("body_html") val bodyHtml: String = "",
)

@Serializable
data class ProjectList(val items: List<Project> = emptyList())

// ───────────── 帖子 ─────────────

@Serializable
data class CategoryRef(val id: Int, val slug: String, val name: String)

@Serializable
data class PublicUser(val uid: String, val nickname: String, @SerialName("avatar_url") val avatarUrl: String? = null, @SerialName("trust_level") val trustLevel: Int = 0)

@Serializable
data class Topic(
    val id: Long,
    val title: String,
    val excerpt: String = "",
    val category: CategoryRef,
    val author: PublicUser,
    val tags: List<String> = emptyList(),
    val pinned: Boolean = false,
    val featured: Boolean = false,
    @SerialName("reply_count") val replyCount: Int = 0,
    @SerialName("like_count") val likeCount: Int = 0,
    @SerialName("created_at") val createdAt: String = "",
    @SerialName("last_reply_at") val lastReplyAt: String = "",
    /** 详情里才有 */
    @SerialName("body_html") val bodyHtml: String = "",
)

@Serializable
data class Post(
    val id: Long,
    val floor: Int,
    @SerialName("reply_to_floor") val replyToFloor: Int? = null,
    val author: PublicUser,
    @SerialName("body_html") val bodyHtml: String = "",
    val status: String = "normal",
    @SerialName("like_count") val likeCount: Int = 0,
    @SerialName("created_at") val createdAt: String = "",
    /** 你屏蔽了这个作者：不返回正文 */
    val blocked: Boolean = false,
)

// ───────────── 账号 ─────────────

@Serializable
data class AccountIdentity(val kind: String, val masked: String)

@Serializable
data class AccountUser(val uid: String, val username: String, @SerialName("avatar_url") val avatarUrl: String? = null, val identities: List<AccountIdentity> = emptyList())

@Serializable
data class TokenPair(
    @SerialName("access_token") val accessToken: String,
    @SerialName("refresh_token") val refreshToken: String,
    @SerialName("expires_in") val expiresIn: Long,
)

@Serializable
data class LoginResponse(
    @SerialName("access_token") val accessToken: String,
    @SerialName("refresh_token") val refreshToken: String,
    @SerialName("expires_in") val expiresIn: Long,
    @SerialName("is_new_user") val isNewUser: Boolean = false,
    val user: AccountUser,
)

/** 验证码登录的结果：要么登录成功，要么验证码通过但还没有账号，需要去设置用户名和密码。 */
sealed interface CodeLogin {
    data class Done(val login: LoginResponse) : CodeLogin
    data class NeedsRegistration(val registrationToken: String) : CodeLogin
}

@Serializable
data class ForumUser(
    val uid: String,
    val nickname: String,
    @SerialName("avatar_url") val avatarUrl: String? = null,
    @SerialName("trust_level") val trustLevel: Int = 0,
    val points: Int = 0,
    val status: String = "active",
)

@Serializable
data class SessionInfo(val user: ForumUser)

@Serializable
data class Unread(val unread: Int = 0)
