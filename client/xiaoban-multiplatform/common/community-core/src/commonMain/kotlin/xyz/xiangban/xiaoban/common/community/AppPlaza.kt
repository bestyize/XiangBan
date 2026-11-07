package xyz.xiangban.xiaoban.common.community

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.contentOrNull

/** A device-local shortcut, never a cached download credential or executable. */
@Serializable
data class AppShortcut(val source: String, val id: String, val title: String, val coverUrl: String? = null) {
    val key: String get() = "$source:$id"
    val valid: Boolean get() = title.isNotBlank() && title.length <= 200 && when (source) {
        "builtin" -> id in BuiltinMobileGames.ids
        "local" -> runCatching { MobileGameFormat.id(id) }.isSuccess
        "community" -> (id.toLongOrNull() ?: 0) > 0
        else -> false
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

/** Greedily fill the shorter column; retain source order within each column. */
fun <T> appWaterfallColumns(items: List<T>, height: (T) -> Float): List<List<T>> {
    val columns = listOf(mutableListOf<T>(), mutableListOf<T>())
    val heights = floatArrayOf(0f, 0f)
    items.forEach { item ->
        val column = if (heights[0] <= heights[1]) 0 else 1
        columns[column] += item
        heights[column] += height(item).takeIf { it.isFinite() && it > 0 } ?: 1f
    }
    return columns
}

/** Unknown orientation stays unknown; never label an uninspected game as portrait. */
fun appOrientation(pack: Pack): String? =
    (pack.versions.firstOrNull { it.version == pack.version }?.manifest?.get("orientation") as? JsonPrimitive)
        ?.contentOrNull?.takeIf { it in listOf("portrait", "landscape") }

fun appRemixPrompt(title: String, summary: String, idea: String): String =
    "请参考《${title.trim()}》制作一个我的版本。原作简介：${summary.trim()}。我的想法：${idea.trim()}。" +
        "先给出简短玩法方案让我确认，再按小伴游戏格式制作支持手机触控和电脑键鼠的离线小游戏。"
