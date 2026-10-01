package xyz.xiaban.server.events

import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import xyz.xiaban.server.db.Db
import xyz.xiaban.shared.EventIn
import xyz.xiaban.shared.EventNames
import java.time.Instant

/** Usage events for the PRD 10 metrics (share rate, K factor, retention). */
class EventService(private val db: Db) {

    /** Stores up to [MAX_BATCH] allowed events; returns how many were kept. */
    suspend fun record(userId: Long?, events: List<EventIn>, userAgent: String?): Int {
        val kept = events.asSequence()
            .filter { it.name in EventNames.ALLOWED }
            .take(MAX_BATCH)
            .toList()
        if (kept.isEmpty()) return 0
        db.read { c ->
            c.prepareStatement("insert into events (user_id, name, props, client_ts, user_agent) values (?, ?, ?::jsonb, ?, ?)").use { ps ->
                for (event in kept) {
                    if (userId == null) ps.setNull(1, java.sql.Types.BIGINT) else ps.setLong(1, userId)
                    ps.setString(2, event.name)
                    ps.setString(3, propsJson(event.props))
                    ps.setObject(4, event.ts?.let { java.time.OffsetDateTime.ofInstant(Instant.ofEpochMilli(it), java.time.ZoneOffset.UTC) })
                    ps.setString(5, userAgent?.take(300))
                    ps.addBatch()
                }
                ps.executeBatch()
            }
        }
        return kept.size
    }

    /** Server-side events for actions the backend witnesses itself. */
    suspend fun server(userId: Long, name: String, props: Map<String, String> = emptyMap()) =
        record(userId, listOf(EventIn(name, Instant.now().toEpochMilli(), props)), "server")

    private fun propsJson(props: Map<String, String>): String =
        JsonObject(props.entries.take(MAX_PROPS).associate { (k, v) -> k.take(40) to JsonPrimitive(v.take(100)) }).toString()

    private companion object {
        const val MAX_BATCH = 50
        const val MAX_PROPS = 10
    }
}
