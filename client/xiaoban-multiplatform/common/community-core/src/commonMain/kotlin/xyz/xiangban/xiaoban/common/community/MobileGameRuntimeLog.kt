package xyz.xiangban.xiaoban.common.community

import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.longOrNull
import kotlinx.serialization.json.put

/** The log is reset per open. Evidence from an older installed version cannot validate this one. */
fun mobileGameRuntimeReport(work: MobileGame, log: String): JsonObject {
    val lines = log.lineSequence().mapNotNull { line ->
        runCatching { CommunityJson.parseToJsonElement(line) as? JsonObject }.getOrNull()
    }.toList()
    fun JsonObject.field(key: String): String? = (get(key) as? JsonPrimitive)?.content
    val open = lines.indexOfLast { it.field("type") == "host.open" }
    val current = lines.getOrNull(open)?.takeIf { it.field("gameId") == work.id && it.field("version") == work.version }
    val events = if (current == null) emptyList() else lines.drop(open)
    val ready = events.indexOfFirst { it.field("type") == "game.ready" }
    val round = if (ready >= 0) events.drop(ready + 1).dropWhile { it.field("type") != "game.start" } else emptyList()
    val result = round.drop(1).takeWhile { it.field("type") != "game.start" }
        .firstOrNull { it.field("type") == "game.result" }
    val score = (result?.get("score") as? JsonPrimitive)?.takeUnless { it.isString }?.longOrNull?.takeIf { it >= 0 }
    val errors = setOf("runtime.error", "console.error", "game.error")
    return buildJsonObject {
        put("id", work.id)
        put("version", work.version)
        put("startup", when {
            current == null -> "not_opened"
            ready < 0 -> "not_ready"
            else -> "ready"
        })
        if (work.kind == "game") {
            put("first_round", when {
                round.isEmpty() -> "no_start_recorded"
                score == null -> "no_result_recorded"
                else -> "result_recorded"
            })
            score?.let { put("first_round_score", it) }
        }
        put("errors", events.count { it.field("type") in errors })
        put("events", JsonArray(events))
    }
}
