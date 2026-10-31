package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.http.HttpMethod
import io.ktor.http.Url
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.serializer

/** Matches the anonymous web share contract; never contains a session, prompt or download credential. */
@Serializable
data class CommunityShare(
    val kind: String,
    val title: String,
    val summary: String = "",
    @SerialName("cover_url") val coverUrl: String? = null,
    val url: String,
    @SerialName("app_link") val appLink: String? = null,
    @SerialName("pack_kind") val packKind: String? = null,
    @SerialName("one_click") val oneClick: Boolean = false,
)

data class CommunityShareTarget(val kind: String, val id: String, val requestsApply: Boolean = false) {
    fun webUrl(config: CommunityConfig): String = config.forumBase.trimEnd('/') + when (kind) {
        "pack" -> "/packs/$id"
        "topic" -> "/t/$id"
        "scheme" -> "/schemes/$id"
        "project" -> "/projects/$id"
        "weekly" -> "/weekly/$id"
        else -> error("Unsupported community target")
    }
}

private val ShareKinds = setOf("pack", "topic", "scheme", "project", "weekly")
private val ApplyKinds = setOf("effect", "reminder", "routine", "task", "goal", "widget", "game", "wallpaper", "skill", "expert", "workflow")
private val ShareSlug = Regex("[A-Za-z0-9][A-Za-z0-9_-]{0,99}")
private fun validShareId(kind: String, id: String): Boolean = if (kind in setOf("pack", "topic", "scheme"))
    id.toLongOrNull()?.let { it > 0 && it.toString() == id } == true else ShareSlug.matches(id)

/** Exact allowlist: no credentials, fragments, encoded separators, alternate ports or extra queries.
 * An apply link is only a request to preview public content. It never executes anything on mobile.
 */
fun communityShareTarget(raw: String, config: CommunityConfig): CommunityShareTarget? {
    if (raw.length !in 1..2048 || raw.any { it.isWhitespace() || it.code < 32 } || '%' in raw || '\\' in raw || '#' in raw) return null
    val link = runCatching { Url(raw) }.getOrNull() ?: return null
    if (!link.user.isNullOrEmpty() || !link.password.isNullOrEmpty()) return null
    val parts = link.encodedPath.removePrefix("/").split('/')
    val target = when (link.protocol.name) {
        "https" -> {
            val base = runCatching { Url(config.forumBase) }.getOrNull() ?: return null
            if (base.protocol.name != "https" || base.encodedPath !in listOf("", "/") || link.host != base.host || link.port != base.port || link.parameters.names().isNotEmpty()) return null
            if (parts.size != 2) return null
            val kind = when (parts[0]) { "packs" -> "pack"; "t" -> "topic"; "schemes" -> "scheme"; "projects" -> "project"; "weekly" -> "weekly"; else -> return null }
            CommunityShareTarget(kind, parts[1])
        }
        "xiaoban" -> {
            if (link.host != "community" || link.specifiedPort != 0 || parts.size != 3) return null
            when (parts[0]) {
                "open" -> {
                    if (link.parameters.names().isNotEmpty() || parts[1] !in ShareKinds) return null
                    CommunityShareTarget(parts[1], parts[2])
                }
                "apply" -> {
                    if (parts[1] !in ApplyKinds || link.parameters.names().any { it != "version" }) return null
                    val versions = link.parameters.getAll("version")
                    if (versions != null && (versions.size != 1 || versions.single().toIntOrNull()?.let { it > 0 } != true)) return null
                    CommunityShareTarget("pack", parts[2], requestsApply = true)
                }
                else -> return null
            }
        }
        else -> return null
    }
    return target.takeIf { validShareId(it.kind, it.id) }
}

class CommunitySharingApi(client: HttpClient, private val config: CommunityConfig) {
    private val http = JsonHttp(client, config.forumBase)
    /** Intentionally anonymous even when the app is signed in: private content must not leak to a poster. */
    suspend fun publicInfo(target: CommunityShareTarget): CommunityShare {
        if (target.kind !in ShareKinds || !validShareId(target.kind, target.id)) throw ApiException(400, "不支持分享此链接")
        val info: CommunityShare = http.send(HttpMethod.Get, "/api/share/${target.kind}/${target.id}", serializer())
        val resolved = communityShareTarget(info.url, config)
        if (info.kind != target.kind || resolved?.kind != target.kind || resolved.id != target.id || resolved.requestsApply || info.title.isBlank() || info.title.length > 500 || info.summary.length > 10000)
            throw ApiException(200, "分享信息暂不可用，请重试")
        // Keep only canonical public addresses; optional app and cover addresses never reach platform actions.
        return info.copy(url = target.webUrl(config), appLink = null, coverUrl = null)
    }
}
