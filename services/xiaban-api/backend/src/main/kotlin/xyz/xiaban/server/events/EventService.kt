package xyz.xiaban.server.events

import java.sql.Types
import java.time.Clock
import java.time.Duration
import java.time.Instant
import java.time.LocalDateTime
import java.time.OffsetDateTime
import java.time.ZoneId
import java.time.ZoneOffset
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import xyz.xiaban.contracts.telemetry.EventIn
import xyz.xiaban.server.checkin.WorkDay
import xyz.xiaban.server.db.Db
import xyz.xiaban.server.db.execute
import xyz.xiaban.server.users.SessionService

/** Who an event belongs to. Guests are identified only by a hash of their anonymous device id. */
sealed interface Actor {
    val key: String
    val type: String

    data class Registered(val userId: Long) : Actor {
        override val key = "u:$userId"
        override val type = "registered"
    }

    data class Guest(val deviceHash: String) : Actor {
        override val key = "d:$deviceHash"
        override val type = "guest"
    }

    companion object {
        fun guest(deviceId: String): Actor? =
            deviceId.takeIf(SessionService::isValidDeviceId)?.let { Guest(SessionService.sha256(it)) }
    }
}

/** Usage events for the PRD 10 metrics and the admin dashboard. */
class EventService(private val db: Db, private val clock: Clock = Clock.systemUTC()) {

    /** Stores up to [MAX_BATCH] allowed events; returns how many were kept. */
    suspend fun record(actor: Actor, events: List<EventIn>, userAgent: String?): Int {
        val allowed = if (actor is Actor.Guest) EventNames.GUEST_ALLOWED else EventNames.ALLOWED
        val kept = events.asSequence().filter { it.name in allowed }.take(MAX_BATCH).toList()
        if (kept.isEmpty()) return 0
        val now = clock.instant()
        val workDay = metricsDay(now)
        db.read { c ->
            c.prepareStatement(
                "insert into events (user_id, actor, user_type, work_day, name, props, client_ts, user_agent) values (?, ?, ?, ?, ?, ?::jsonb, ?, ?)",
            ).use { ps ->
                for (event in kept) {
                    if (actor is Actor.Registered) ps.setLong(1, actor.userId) else ps.setNull(1, Types.BIGINT)
                    ps.setString(2, actor.key)
                    ps.setString(3, actor.type)
                    ps.setObject(4, workDay)
                    ps.setString(5, event.name)
                    ps.setString(6, propsJson(event.name, event.props))
                    ps.setObject(7, clientTime(event.ts, now))
                    ps.setString(8, userAgent?.take(300))
                    ps.addBatch()
                }
                ps.executeBatch()
            }
        }
        return kept.size
    }

    /** Server-side events for actions the backend witnesses itself. */
    suspend fun server(actor: Actor, name: String, props: Map<String, String> = emptyMap()) =
        record(actor, listOf(EventIn(name, clock.millis(), props)), "server")

    suspend fun server(userId: Long, name: String, props: Map<String, String> = emptyMap()) =
        server(Actor.Registered(userId), name, props)

    /** Guest events have no account to be deleted with, so they expire (隐私政策: usage events kept 180 days). */
    suspend fun purgeGuestEvents(): Int = db.read { c ->
        c.execute("delete from events where user_type = 'guest' and created_at < ?", clock.instant().minus(Duration.ofDays(RETENTION_DAYS)))
    }

    /** Only the parameters each event is defined with are kept, so a client cannot stuff arbitrary data in. */
    private fun propsJson(name: String, props: Map<String, String>): String {
        val keys = PROPS[name]
        val kept = props.entries.filter { keys == null || it.key in keys }.take(MAX_PROPS)
        return JsonObject(kept.associate { (k, v) -> k.take(40) to JsonPrimitive(v.take(100)) }).toString()
    }

    /** Client clocks can be wildly off; anything outside a day either way is dropped. */
    private fun clientTime(ts: Long?, now: Instant): OffsetDateTime? = ts
        ?.let(Instant::ofEpochMilli)
        ?.takeIf { Duration.between(it, now).abs() <= Duration.ofDays(1) }
        ?.let { OffsetDateTime.ofInstant(it, ZoneOffset.UTC) }

    companion object {
        private const val MAX_BATCH = 50
        private const val MAX_PROPS = 10
        private const val RETENTION_DAYS = 180L

        /** Statistics use one fixed day for everyone: Beijing time, cut at 05:00 like the 下班日. */
        val METRICS_ZONE: ZoneId = ZoneId.of("Asia/Shanghai")

        fun metricsDay(at: Instant) = WorkDay.of(LocalDateTime.ofInstant(at, METRICS_ZONE))

        /** Allowed parameter names per event; events not listed keep any (bounded) parameters. */
        private val PROPS: Map<String, Set<String>> = mapOf(
            "page_view" to setOf("path"),
            "checkin" to setOf("title", "id"),
            "card_generate" to setOf("style", "title"),
            "card_save" to setOf("via", "style"),
            "card_share" to setOf("via", "style"),
            "game_start" to setOf("game"),
            "game_finish" to setOf("game", "score"),
            "quiz_start" to setOf("quiz", "mode"),
            "quiz_complete" to setOf("quiz"),
        )
    }
}
