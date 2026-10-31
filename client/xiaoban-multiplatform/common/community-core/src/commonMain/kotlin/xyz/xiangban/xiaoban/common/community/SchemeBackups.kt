package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.http.HttpMethod
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.serializer

/** Owner-only selected references. This DTO intentionally has no settings, file, credential or execution fields. */
@Serializable
data class SchemeBackup(
    val id: String,
    @SerialName("schema_version") val schemaVersion: Int,
    val title: String,
    val summary: String = "",
    val items: List<SchemePackRef>,
    val revision: Long,
    val status: String,
    @SerialName("updated_at") val updatedAt: String,
)

/** Read-only API with an explicit opt-in guard, checked before even asking the session for a token. */
class SchemeBackupsApi(client: HttpClient, config: CommunityConfig, private val token: suspend () -> String?, private val accessEnabled: () -> Boolean = { false }) {
    private val http = JsonHttp(client, config.forumBase)
    private suspend fun access(): String {
        if (!accessEnabled()) throw ApiException(403, "云端访问已关闭，请主动开启查看")
        val current = token() ?: throw ApiException(401, "请登录后查看自己的方案备份")
        if (!accessEnabled()) throw ApiException(403, "云端访问已关闭")
        return current
    }
    suspend fun list(page: Int = 1, removed: Boolean = false): Page<SchemeBackup> {
        val result: Page<SchemeBackup> = http.send(HttpMethod.Get,
            "/api/me/scheme-backups?page=${page.coerceAtLeast(1)}&status=${if (removed) "removed" else "active"}", serializer(), token = access())
        result.items.forEach { validate(it) }
        return result
    }
    suspend fun detail(id: String): SchemeBackup {
        if (!ForumAssetId.matches(id)) throw ApiException(400, "方案备份编号无效")
        val result: SchemeBackup = http.send(HttpMethod.Get, "/api/me/scheme-backups/$id", serializer(), token = access())
        validate(result)
        if (!result.id.equals(id, ignoreCase = true)) throw ApiException(200, "方案备份暂不可用，请刷新")
        return result
    }
    private fun validate(value: SchemeBackup) {
        if (!ForumAssetId.matches(value.id) || value.schemaVersion != 1 || value.revision <= 0 || value.status !in listOf("active", "removed") ||
            value.title.isBlank() || value.title.length > 120 || value.summary.length > 1000 || value.items.size !in 1..20 ||
            value.items.any { it.packId <= 0 || it.version <= 0 } || value.items.map { it.packId }.distinct().size != value.items.size)
            throw ApiException(200, "方案备份格式暂不可用，请刷新")
    }
}
