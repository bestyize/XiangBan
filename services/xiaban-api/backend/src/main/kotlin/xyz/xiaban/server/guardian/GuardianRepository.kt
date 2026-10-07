package xyz.xiaban.server.guardian

import xyz.xiaban.server.db.Db
import xyz.xiaban.server.db.execute
import xyz.xiaban.server.db.instant
import xyz.xiaban.server.db.query
import xyz.xiaban.server.db.queryOne
import java.sql.ResultSet
import java.time.Instant

data class GuardianRow(
    val userId: Long,
    val contactName: String,
    val contactEmail: String,
    val selfName: String,
    val thresholdDays: Int,
    val paused: Boolean,
    val baseAt: Instant,
    val nextRemindAt: Instant?,
    val nextNotifyAt: Instant?,
)

class GuardianRepository(private val db: Db) {

    suspend fun find(userId: Long): GuardianRow? = db.read { c -> c.queryOne("$SELECT where user_id = ?", userId, map = ::map) }

    suspend fun upsert(userId: Long, contactName: String, contactEmail: String, selfName: String, thresholdDays: Int, now: Instant) = db.read { c ->
        c.execute(
            """
            insert into guardians (user_id, contact_name, contact_email, self_name, threshold_days, base_at, consented_at, informed_at)
            values (?, ?, ?, ?, ?, ?, ?, ?)
            on conflict (user_id) do update set contact_name = excluded.contact_name, contact_email = excluded.contact_email,
              self_name = excluded.self_name, threshold_days = excluded.threshold_days, paused = false,
              base_at = excluded.base_at, consented_at = excluded.consented_at, informed_at = excluded.informed_at, updated_at = now()
            """,
            userId, contactName, contactEmail, selfName, thresholdDays, now, now, now,
        )
    }

    /** Resuming restarts the clock from now, so a vacation never triggers an alert the moment it ends. */
    suspend fun setPaused(userId: Long, paused: Boolean, now: Instant) = db.read { c ->
        c.execute(
            "update guardians set paused = ?, base_at = case when ? then base_at else ? end, updated_at = now() where user_id = ?",
            paused, paused, now, userId,
        )
    }

    suspend fun setSchedule(userId: Long, remindAt: Instant?, notifyAt: Instant?) = db.read { c ->
        c.execute("update guardians set next_remind_at = ?, next_notify_at = ? where user_id = ?", remindAt, notifyAt, userId)
    }

    suspend fun delete(userId: Long) = db.read { c -> c.execute("delete from guardians where user_id = ?", userId) }

    suspend fun scheduled(): List<GuardianRow> = db.read { c ->
        c.query("$SELECT where not paused and next_notify_at is not null", map = ::map)
    }

    /** Returns the alert id only the first time for this due time; that is what makes sending idempotent. */
    suspend fun claimAlert(userId: Long, kind: String, dueAt: Instant): Long? = db.read { c ->
        c.queryOne(
            "insert into guardian_alerts (user_id, kind, due_at) values (?, ?, ?) on conflict do nothing returning id",
            userId, kind, dueAt,
        ) { it.getLong(1) }
    }

    suspend fun markAlert(alertId: Long, status: String) = db.read { c ->
        c.execute("update guardian_alerts set status = ? where id = ?", status, alertId)
    }

    private fun map(rs: ResultSet) = GuardianRow(
        userId = rs.getLong("user_id"),
        contactName = rs.getString("contact_name"),
        contactEmail = rs.getString("contact_email"),
        selfName = rs.getString("self_name"),
        thresholdDays = rs.getInt("threshold_days"),
        paused = rs.getBoolean("paused"),
        baseAt = rs.instant("base_at")!!,
        nextRemindAt = rs.instant("next_remind_at"),
        nextNotifyAt = rs.instant("next_notify_at"),
    )

    private companion object {
        const val SELECT = """
            select user_id, contact_name, contact_email, self_name, threshold_days, paused, base_at, next_remind_at, next_notify_at
            from guardians
        """
    }
}
