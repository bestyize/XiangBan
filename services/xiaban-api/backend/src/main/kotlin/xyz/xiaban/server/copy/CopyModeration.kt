package xyz.xiaban.server.copy

import java.time.LocalDate
import kotlinx.coroutines.future.await
import xyz.xiaban.contracts.admin.reports.ReportView
import xyz.xiaban.server.checkin.CheckinRow
import xyz.xiaban.server.db.Db
import xyz.xiaban.server.db.execute
import xyz.xiaban.server.db.instant
import xyz.xiaban.server.db.query
import xyz.xiaban.server.http.tooMany
import xyz.xiaban.server.redis.Redis

/** PRD 11: a reported line is taken down immediately; admins can restore it. */
class CopyModeration(private val db: Db, private val redis: Redis) {
    @Volatile private var cache: Set<String> = emptySet()
    @Volatile private var loadedAtMs = 0L

    suspend fun disabled(): Set<String> {
        if (System.currentTimeMillis() - loadedAtMs > CACHE_MS) {
            cache = db.read { c -> c.query("select copy_id from disabled_copies") { it.getString(1) }.toSet() }
            loadedAtMs = System.currentTimeMillis()
        }
        return cache
    }

    suspend fun report(userId: Long, checkin: CheckinRow, reason: String?, today: LocalDate) {
        val key = "xb:reports:$today:$userId"
        val count = redis.cmd.incr(key).await()
        if (count == 1L) redis.cmd.expire(key, 2 * 24 * 3600L).await()
        if (count > DAILY_REPORTS) tooMany("今天举报得有点多了，明天再来吧", "report_limit")

        db.tx { c ->
            c.execute(
                "insert into copy_reports (user_id, checkin_id, copy_id, text, reason) values (?, ?, ?, ?, ?)",
                userId, checkin.id, checkin.copyId, checkin.reply, reason?.take(200),
            )
            c.execute("insert into disabled_copies (copy_id, reason) values (?, 'reported') on conflict do nothing", checkin.copyId)
        }
        cache = cache + checkin.copyId
    }

    suspend fun restore(copyId: String) {
        db.read { c -> c.execute("delete from disabled_copies where copy_id = ?", copyId) }
        cache = cache - copyId
    }

    suspend fun reports(limit: Int): List<ReportView> = db.read { c ->
        c.query("select id, copy_id, text, reason, created_at from copy_reports order by id desc limit ?", limit) {
            ReportView(it.getLong("id"), it.getString("copy_id"), it.getString("text"), it.getString("reason"), it.instant("created_at")!!.toEpochMilli())
        }
    }

    private companion object {
        const val CACHE_MS = 30_000L
        const val DAILY_REPORTS = 5
    }
}
