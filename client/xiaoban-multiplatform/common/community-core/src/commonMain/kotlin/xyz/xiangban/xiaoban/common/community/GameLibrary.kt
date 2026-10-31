package xyz.xiangban.xiaoban.common.community

import kotlinx.serialization.json.*

/** Bundled games are read-only assets, never user workspace entries. */
object BuiltinMobileGames {
    val ids = listOf("dusk-glider", "ink-blade", "jade-gomoku", "pro-cube", "snake-arena", "block-puzzle", "merge-2048", "water-sort", "falling-blocks")
    fun genre(id: String): String = when (id) {
        "water-sort", "snake-arena" -> "解压"
        "dusk-glider", "ink-blade" -> "冒险"
        else -> "益智"
    }
    fun read(id: String, manifest: String): MobileGame {
        require(id in ids) { "不是内置游戏" }
        val json = Json.parseToJsonElement(manifest).jsonObject
        fun text(key: String) = (json[key] as? JsonPrimitive)?.takeIf { it.isString }?.content
        require(text("id") == id && text("entry") == "index.html") { "内置游戏清单不匹配" }
        require(json["runtimeProtocol"]?.jsonPrimitive?.intOrNull == 1) { "不支持的游戏协议" }
        val orientation = text("orientation") ?: "portrait"
        require(orientation in listOf("portrait", "landscape"))
        return MobileGame(id, requireNotNull(text("name")), requireNotNull(text("description")), json["save"]?.jsonPrimitive?.booleanOrNull ?: false, orientation)
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
