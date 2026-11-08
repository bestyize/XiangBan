package xyz.xiangban.xiaoban.common.community

import kotlinx.serialization.json.*

/** Bundled games are read-only assets, never user workspace entries. */
object BuiltinMobileGames {
    val ids = listOf("dusk-glider", "ink-blade", "jade-gomoku", "pro-cube", "snake-arena", "block-puzzle", "merge-2048", "water-sort", "falling-blocks", "mbti")
    /** Builtins shown under 精选工具 instead of 精选游戏. */
    val tools = setOf("mbti")
    fun isTool(id: String): Boolean = id in tools
    /** Tools draw a one-character serif mark instead of a cropped cover. */
    fun glyph(id: String): String? = when (id) {
        "mbti" -> "格"
        else -> null
    }
    fun genre(id: String): String = when (id) {
        "mbti" -> "测评"
        "water-sort", "snake-arena" -> "解压"
        "dusk-glider", "ink-blade" -> "冒险"
        else -> "益智"
    }
}

/** Most recent first, bounded and deduplicated; called only after the host receives game.ready. */
object MobileGameHistory {
    fun decode(raw: String?): List<String> = runCatching {
        Json.parseToJsonElement(raw ?: "[]").jsonArray.mapNotNull { value ->
            (value as? JsonPrimitive)?.takeIf { it.isString }?.content?.takeIf { Regex("[a-z0-9][a-z0-9-]{1,39}").matches(it) }
        }.distinct().take(12)
    }.getOrDefault(emptyList())
    fun opened(previous: List<String>, id: String): List<String> {
        require(Regex("[a-z0-9][a-z0-9-]{1,39}").matches(id))
        return (listOf(id) + previous).distinct().take(12)
    }
    fun encode(ids: List<String>): String = JsonArray(ids.map(::JsonPrimitive)).toString()
    fun available(ids: List<String>, games: List<MobileGame>): List<MobileGame> = ids.mapNotNull { id -> games.find { it.id == id } }
}
