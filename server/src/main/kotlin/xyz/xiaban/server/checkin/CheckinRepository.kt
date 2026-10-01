package xyz.xiaban.server.checkin

import xyz.xiaban.server.db.Db
import xyz.xiaban.server.db.execute
import xyz.xiaban.server.db.instant
import xyz.xiaban.server.db.localDate
import xyz.xiaban.server.db.query
import xyz.xiaban.server.db.queryOne
import xyz.xiaban.shared.ReplySource
import xyz.xiaban.shared.TitleCode
import java.sql.ResultSet
import java.time.Instant
import java.time.LocalDate

data class CheckinRow(
    val id: Long,
    val userId: Long,
    val workDay: LocalDate,
    val checkedAt: Instant,
    val title: TitleCode,
    val copyId: String,
    val libraryReply: String,
    val finalReply: String?,
    val replySource: ReplySource,
    val revokedAt: Instant?,
) {
    val reply: String get() = finalReply ?: libraryReply
}

data class NewCheckin(
    val userId: Long,
    val workDay: LocalDate,
    val checkedAt: Instant,
    val title: TitleCode,
    val copyId: String,
    val libraryReply: String,
)

class CheckinRepository(private val db: Db) {

    suspend fun active(userId: Long, workDay: LocalDate): CheckinRow? = db.read { c ->
        c.queryOne("$SELECT where user_id = ? and work_day = ? and revoked_at is null", userId, workDay, map = ::map)
    }

    suspend fun undoUsed(userId: Long, workDay: LocalDate): Boolean = db.read { c ->
        c.queryOne(
            "select exists (select 1 from checkins where user_id = ? and work_day = ? and revoked_at is not null)",
            userId, workDay,
        ) { it.getBoolean(1) } ?: false
    }

    /** Returns null when another request already checked in for that day. */
    suspend fun insert(new: NewCheckin): CheckinRow? = db.read { c ->
        c.queryOne(
            """
            insert into checkins (user_id, work_day, checked_at, title, copy_id, library_reply)
            values (?, ?, ?, ?, ?, ?)
            on conflict do nothing
            returning $COLUMNS
            """,
            new.userId, new.workDay, new.checkedAt, new.title.code, new.copyId, new.libraryReply,
            map = ::map,
        )
    }

    suspend fun revoke(id: Long, at: Instant): Boolean = db.read { c ->
        c.execute("update checkins set revoked_at = ? where id = ? and revoked_at is null", at, id) == 1
    }

    /** Active check-ins from [since] on, newest first. */
    suspend fun since(userId: Long, since: LocalDate): List<CheckinRow> = db.read { c ->
        c.query("$SELECT where user_id = ? and work_day >= ? and revoked_at is null order by work_day desc", userId, since, map = ::map)
    }

    private fun map(rs: ResultSet) = CheckinRow(
        id = rs.getLong("id"),
        userId = rs.getLong("user_id"),
        workDay = rs.localDate("work_day"),
        checkedAt = rs.instant("checked_at")!!,
        title = TitleCode.of(rs.getString("title")),
        copyId = rs.getString("copy_id"),
        libraryReply = rs.getString("library_reply"),
        finalReply = rs.getString("final_reply"),
        replySource = if (rs.getString("reply_source") == "llm") ReplySource.Llm else ReplySource.Library,
        revokedAt = rs.instant("revoked_at"),
    )

    private companion object {
        const val COLUMNS = "id, user_id, work_day, checked_at, title, copy_id, library_reply, final_reply, reply_source, revoked_at"
        const val SELECT = "select $COLUMNS from checkins"
    }
}
