package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.client.request.header
import io.ktor.client.request.put
import io.ktor.client.request.setBody
import io.ktor.client.statement.bodyAsText
import io.ktor.http.ContentType
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpMethod
import io.ktor.http.Url
import io.ktor.http.contentType
import io.ktor.http.encodeURLParameter
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.withContext
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonArray
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.put
import kotlinx.serialization.serializer

val CapabilityPackKinds = setOf("routine", "reminder", "task", "goal", "skill", "expert", "workflow")
val ContentPackLimits: Map<String, Long> = mapOf(
    "effect" to 1L, "reminder" to 1L, "routine" to 1L, "task" to 1L, "goal" to 1L, "expert" to 1L,
    "widget" to 8L, "game" to 8L, "skill" to 8L, "workflow" to 8L, "wallpaper" to 32L,
).mapValues { (_, mb) -> mb * 1024 * 1024 }

@Serializable
data class Scheme(
    val id: Long,
    val author: PackAuthor,
    val title: String,
    val summary: String = "",
    val items: List<SchemePackRef>,
    @SerialName("apply_count") val applyCount: Long = 0,
    @SerialName("created_at") val createdAt: String = "",
)
@Serializable
data class SchemePackRef(
    @SerialName("pack_id") val packId: Long,
    val kind: String = "",
    val slug: String = "",
    val version: Int,
    val available: Boolean = true,
    val title: String? = null,
    val status: String? = null,
)
@Serializable
data class MyContentPacks(val items: List<Pack> = emptyList())
@Serializable
data class ContentUploadTicket(
    val ticket: String,
    @SerialName("upload_url") val uploadUrl: String,
    @SerialName("file_key") val fileKey: String,
    @SerialName("expires_at") val expiresAt: Long,
)
@Serializable
data class ContentUploadStatus(@SerialName("file_key") val fileKey: String, val verified: Boolean, val used: Boolean)
@Serializable
data class ContentPublishResult(val id: Long, val slug: String = "", val version: Int? = null, @SerialName("trust_level") val trustLevel: Int = 2)

data class ContentPublishDraft(val kind: String, val title: String, val summary: String, val tags: List<String>, val rightsAck: Boolean, val changelog: String = "")

fun contentDraftError(draft: ContentPublishDraft, newVersion: Boolean = false): String? = when {
    draft.kind !in ContentPackLimits -> "请选择支持的内容包类型"
    !draft.rightsAck -> "请先确认拥有这份内容的发布权利"
    !newVersion && draft.title.trim().isEmpty() -> "请填写标题"
    draft.title.trim().length > 60 -> "标题最多 60 个字"
    draft.summary.trim().length > 500 -> "简介最多 500 个字"
    draft.tags.distinct().size > 8 || draft.tags.any { it.length > 20 } -> "最多 8 个标签，每个不超过 20 个字"
    draft.changelog.length > 500 -> "更新说明最多 500 个字"
    else -> null
}
fun contentWorkState(pack: Pack): String = when {
    pack.status == "removed" -> "已下架"
    pack.status == "rejected" -> "未通过审核"
    pack.inReview -> "审核中 · 未审核"
    pack.trustLevel <= 1 -> pack.trustLabel
    else -> "已发布 · 未审核"
}

/** 只读取实际存在的清单字段；完整原文另供界面展开查看，不猜测权限和运行行为。 */
fun manifestValue(manifest: JsonObject, vararg keys: String): String? = keys.firstNotNullOfOrNull { key ->
    manifest[key]?.takeUnless { it is JsonNull }?.let { element ->
        when (element) { is JsonPrimitive -> element.contentOrNull?.takeIf { it.isNotBlank() }; else -> element.toString() }
    }
}

/** 作者操作、资源直传和公开方案保持各自接口，资源机绝不接收论坛 Bearer。 */
class ContentPublishingApi(private val client: HttpClient, private val config: CommunityConfig, private val token: suspend () -> String?) {
    private val http = JsonHttp(client, config.forumBase)
    private suspend fun access() = token() ?: throw ApiException(401, "请登录后发布作品")
    suspend fun schemes(page: Int = 1): Page<Scheme> = http.send(HttpMethod.Get, "/api/schemes?page=$page", serializer(), token = token())
    suspend fun scheme(id: Long): Scheme = http.send(HttpMethod.Get, "/api/schemes/$id", serializer(), token = token())
    suspend fun myPacks(): MyContentPacks = http.send(HttpMethod.Get, "/api/me/packs", serializer(), token = access())
    suspend fun removeOwn(id: Long) { http.send(HttpMethod.Post, "/api/packs/$id/remove", serializer<JsonObject>(), body = "{}", token = access()) }
    suspend fun uploadStatus(fileKey: String): ContentUploadStatus = http.send(HttpMethod.Get,
        "/api/packs/upload-status?file_key=${fileKey.encodeURLParameter()}", serializer(), token = access())

    suspend fun upload(kind: String, file: ForumUploadFile): ContentUploadTicket {
        val limit = ContentPackLimits[kind] ?: throw ApiException(400, "未知的内容类型")
        if (!file.name.endsWith(".zip", ignoreCase = true) || file.bytes.size < 4 || file.bytes.size > limit
            || file.bytes[0] != 'P'.code.toByte() || file.bytes[1] != 'K'.code.toByte())
            throw ApiException(400, "请选择有效 .zip 内容包，不超过 ${limit / 1024 / 1024} MB")
        val sha = withContext(Dispatchers.Default) { Sha256().apply { update(file.bytes) }.hex() }
        val ticket: ContentUploadTicket = http.send(HttpMethod.Post, "/api/packs/upload-ticket", serializer(), token = access(),
            body = buildJsonObject { put("kind", kind); put("size", file.bytes.size); put("sha256", sha) }.toString())
        val destination = runCatching { Url(ticket.uploadUrl) }.getOrElse { throw ApiException(200, "上传地址无效") }
        val localDevelopment = Url(config.forumBase).protocol.name == "http" && destination.host in listOf("localhost", "127.0.0.1", "::1")
        if ((destination.protocol.name != "https" && !localDevelopment) || destination.user != null || destination.password != null || destination.fragment.isNotEmpty())
            throw ApiException(200, "上传地址无效")
        // config() 共用连接引擎但不拥有它；单独客户端禁重定向，避免自定义凭证随 307 流向其他站点。
        val direct = client.config { followRedirects = false }
        try {
            val response = direct.put(ticket.uploadUrl) {
                headers.remove(HttpHeaders.Authorization)
                header("X-Upload-Ticket", ticket.ticket)
                contentType(ContentType.Application.Zip)
                setBody(file.bytes)
            }
            if (response.status.value !in 200..299) {
                if (response.status.value in 300..399) throw ApiException(400, "资源上传地址发生跳转，请联系管理员")
                throw parseApiError(response.status.value, response.bodyAsText())
            }
        } catch (e: CancellationException) { throw e }
        catch (e: ApiException) { throw e }
        catch (e: Exception) { throw ApiException(0, "资源上传失败，请重试") }
        finally { direct.close() }
        return ticket
    }

    /** 最多五次只读校验状态；仍未完成时保留凭证和表单，交由用户稍后继续。 */
    suspend fun awaitVerified(fileKey: String, attempts: Int = 5, pauseMillis: Long = 1000): Boolean {
        require(attempts in 1..5 && pauseMillis in 0..2000)
        repeat(attempts) { index ->
            val status = uploadStatus(fileKey)
            if (status.fileKey != fileKey) throw ApiException(200, "上传校验结果不一致")
            if (status.used) throw ApiException(409, "这份上传已用于发布，请先查看我的作品")
            if (status.verified) return true
            if (index + 1 < attempts) delay(pauseMillis)
        }
        return false
    }

    suspend fun publish(draft: ContentPublishDraft, fileKey: String, packId: Long? = null): ContentPublishResult {
        contentDraftError(draft, packId != null)?.let { throw ApiException(400, it) }
        val body = buildJsonObject {
            put("file_key", fileKey)
            if (packId != null) put("changelog", draft.changelog.trim()) else {
                put("kind", draft.kind); put("title", draft.title.trim()); put("summary", draft.summary.trim())
                put("rights_ack", draft.rightsAck)
                put("tags", buildJsonArray { draft.tags.distinct().forEach { add(JsonPrimitive(it)) } })
            }
        }
        return http.send(HttpMethod.Post, if (packId == null) "/api/packs" else "/api/packs/$packId/versions", serializer(), body = body.toString(), token = access())
    }
}
