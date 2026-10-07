package xyz.xiaban.server.copy

import java.security.MessageDigest
import kotlinx.serialization.Serializable
import xyz.xiaban.contracts.afterwork.checkin.TitleCode
import xyz.xiaban.contracts.companion.personality.Personality
import xyz.xiaban.server.http.AppJson
import xyz.xiaban.server.users.code

enum class CopyMode { Normal, Care, Return, Idle }

enum class Daypart { Day, Evening, Night }

data class CopyEntry(
    val id: String,
    val personality: Personality,
    val mode: CopyMode,
    val title: TitleCode? = null,
    val weather: String? = null,
    val weekdays: Set<Int>? = null,
    val holiday: String? = null,
    val streakMin: Int? = null,
    val daypart: Daypart? = null,
    val text: String,
)

/** The tagged line library (PRD 5.4), one JSON file per personality under resources/copy. */
class CopyLibrary(val entries: List<CopyEntry>) {
    private val byId = entries.associateBy { it.id }

    fun find(id: String): CopyEntry? = byId[id]

    companion object {
        fun load(): CopyLibrary = CopyLibrary(Personality.entries.flatMap(::loadFile))

        private fun loadFile(personality: Personality): List<CopyEntry> {
            val text = CopyLibrary::class.java.getResourceAsStream("/copy/${personality.code}.json")!!
                .bufferedReader().use { it.readText() }
            val file = AppJson.decodeFromString<CopyFile>(text)
            check(file.personality == personality.code) { "copy/${personality.code}.json declares ${file.personality}" }
            return file.groups.flatMap { g ->
                g.lines.map { line ->
                    CopyEntry(
                        id = idFor(personality, line),
                        personality = personality,
                        mode = CopyMode.valueOf(g.mode.replaceFirstChar(Char::uppercase)),
                        title = g.title?.let(TitleCode::of),
                        weather = g.weather,
                        weekdays = g.weekday?.toSet(),
                        holiday = g.holiday,
                        streakMin = g.streakMin,
                        daypart = g.daypart?.let { Daypart.valueOf(it.replaceFirstChar(Char::uppercase)) },
                        text = line,
                    )
                }
            }.distinctBy { it.id }
        }

        /** Content-derived ids stay stable across reorderings; editing a line makes it a new line. */
        fun idFor(personality: Personality, text: String): String {
            val digest = MessageDigest.getInstance("SHA-1").digest("${personality.code}|$text".toByteArray())
            return personality.code + "-" + digest.take(5).joinToString("") { "%02x".format(it) }
        }
    }

    @Serializable
    private data class CopyFile(val personality: String, val groups: List<CopyGroup>)

    @Serializable
    private data class CopyGroup(
        val mode: String,
        val title: String? = null,
        val weather: String? = null,
        val weekday: List<Int>? = null,
        val holiday: String? = null,
        val streakMin: Int? = null,
        val daypart: String? = null,
        val lines: List<String>,
    )
}
