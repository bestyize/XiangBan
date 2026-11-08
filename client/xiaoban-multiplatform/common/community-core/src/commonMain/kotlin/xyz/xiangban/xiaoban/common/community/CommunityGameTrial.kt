package xyz.xiangban.xiaoban.common.community

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.*

/** Resource credentials are short lived; never persisted in history or passed to game JavaScript. */
@Serializable
data class CommunityGameDownload(
    @SerialName("pack_id") val packId: Long,
    val kind: String,
    val version: Int,
    val sha256: String,
    val size: Long,
    val url: String,
) {
    fun validate(pack: Pack, requestedVersion: Int): Int {
        require(pack.id == packId && pack.kind == kind && kind in setOf("game", "app") && pack.status == "published" && version == requestedVersion && version > 0) { "作品或版本已不可用" }
        val published = pack.versions.firstOrNull { it.version == version } ?: error("游戏版本不存在")
        val protocol = (published.manifest["runtimeProtocol"] as? JsonPrimitive)?.takeUnless { it.isString }?.intOrNull
        require(protocol == 2) { "这个作品使用旧版运行协议，手机已不再支持，请等作者更新" }
        require(Regex("[a-fA-F0-9]{64}").matches(sha256) && size in 1..MAX_RUNTIME_V2_BYTES.toLong()) { "作品包校验信息不完整或超过试玩大小限制" }
        require(published.sha256.equals(sha256, true) && published.size == size) { "游戏版本已更新，请重试" }
        return protocol
    }
    companion object {
        const val MAX_RUNTIME_V2_BYTES = 100 * 1024 * 1024
    }
}

@Serializable
data class RecentCommunityGame(val packId: Long, val version: Int, val title: String)

/** Device-local history, independent of collection and author workspace; only successful ready events count. */
object CommunityGameHistory {
    fun decode(raw: String?): List<RecentCommunityGame> = runCatching {
        Json.decodeFromString<List<RecentCommunityGame>>(raw ?: "[]").filter { it.packId > 0 && it.version > 0 && it.title.isNotBlank() && it.title.length <= 200 }.distinctBy { it.packId }.take(12)
    }.getOrDefault(emptyList())
    fun opened(previous: List<RecentCommunityGame>, game: RecentCommunityGame) =
        (listOf(game) + previous).distinctBy { it.packId }.take(12)
    fun encode(games: List<RecentCommunityGame>) = Json.encodeToString(games)
}

/** Only public, exact same-site pack links become playable cards; ordinary links remain text. */
fun linkedCommunityPacks(html: String, config: CommunityConfig): List<Long> =
    Regex("""<a\b[^>]*\bhref\s*=\s*["']([^"']+)["']""", RegexOption.IGNORE_CASE).findAll(html).mapNotNull { match ->
        val href = match.groupValues[1]
        val url = if (href.startsWith("/packs/")) config.forumBase.trimEnd('/') + href else href
        communityShareTarget(url, config)?.takeIf { it.kind == "pack" }?.id?.toLongOrNull()
    }.distinct().take(3).toList()
