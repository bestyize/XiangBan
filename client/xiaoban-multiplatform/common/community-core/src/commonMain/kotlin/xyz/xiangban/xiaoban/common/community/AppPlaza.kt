package xyz.xiangban.xiaoban.common.community

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.contentOrNull

/** A device-local shortcut, never a cached download credential or executable. */
@Serializable
data class AppShortcut(
    val source: String,
    val id: String,
    val title: String,
    val coverUrl: String? = null,
    val slug: String? = null,
) {
    val key: String get() = "$source:$id"
    val valid: Boolean get() = title.isNotBlank() && title.length <= 200 && when (source) {
        "builtin" -> id in BuiltinMobileGames.ids
        "local" -> runCatching { MobileGameFormat.id(id) }.isSuccess
        "community" -> (id.toLongOrNull() ?: 0) > 0
        else -> false
    }
}

/** The last public featured catalog, kept briefly to cover a slow refresh. */
@Serializable
data class FeaturedToolsCache(val fetchedAtMs: Long, val shortcuts: List<AppShortcut>) {
    companion object {
        private const val MaxAgeMs = 24 * 60 * 60 * 1000L

        fun restore(raw: String?, nowMs: Long): List<AppShortcut> =
            raw?.let { runCatching { CommunityJson.decodeFromString<FeaturedToolsCache>(it) }.getOrNull() }
                ?.takeIf { nowMs - it.fetchedAtMs in 0..MaxAgeMs }?.shortcuts.orEmpty()
    }
}

object AppFavorites {
    private val json = Json { ignoreUnknownKeys = true }
    fun decode(raw: String?): List<AppShortcut> = runCatching {
        json.decodeFromString<List<AppShortcut>>(raw ?: "[]").filter { it.valid }.distinctBy { it.key }
    }.getOrDefault(emptyList())
    fun encode(items: List<AppShortcut>): String = json.encodeToString(items.filter { it.valid }.distinctBy { it.key })
    fun toggle(items: List<AppShortcut>, item: AppShortcut): List<AppShortcut> {
        require(item.valid)
        return if (items.any { it.key == item.key }) items.filterNot { it.key == item.key }
        else listOf(item) + items
    }
}

/** Both host histories are newest first, but have no timestamps to compare across hosts. */
fun recentAppShortcuts(localIds: List<String>, available: List<AppShortcut>, community: List<RecentCommunityGame>): List<AppShortcut> =
    (localIds.mapNotNull { id -> available.firstOrNull { it.id == id } } + community.map {
        AppShortcut("community", it.packId.toString(), it.title)
    }).filter { it.valid }.distinctBy { it.key }.take(12)

/** Unknown orientation stays unknown; never label an uninspected game as portrait. */
fun appOrientation(pack: Pack): String? =
    (pack.versions.firstOrNull { it.version == pack.version }?.manifest?.get("orientation") as? JsonPrimitive)
        ?.contentOrNull?.takeIf { it in listOf("portrait", "landscape") }

fun appRemixPrompt(title: String, summary: String, idea: String, kind: String = "game"): String {
    val work = if (kind == "app") "小程序（kind=app）" else "离线小游戏（kind=game）"
    return "请参考《${title.trim()}》制作一个我的版本。原作简介：${summary.trim()}。我的想法：${idea.trim()}。" +
        "先用 app_read_format 读取小伴应用规范，给出简短方案让我确认，再按规范制作支持手机触控和电脑键鼠的$work。"
}
