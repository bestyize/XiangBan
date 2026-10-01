package xyz.xiaban.server.account

import xyz.xiaban.server.db.Db
import xyz.xiaban.server.db.execute

/**
 * Moves an anonymous account's data into an existing one (PRD 9.4 "数据完整迁移"): check-ins for days the
 * target has not covered, the union of unlocks, memories, devices and referrals. The target keeps its own
 * profile, companion and settings; the source account is then deleted.
 */
class AccountMerge(private val db: Db) {
    suspend fun merge(source: Long, target: Long) = db.tx { c ->
        c.execute(
            """
            update checkins set user_id = ? where user_id = ? and revoked_at is null
              and work_day not in (select work_day from checkins where user_id = ? and revoked_at is null)
            """,
            target, source, target,
        )
        c.execute(
            """
            insert into unlocks (user_id, expression_id, unlocked_at, reason)
            select ?, expression_id, unlocked_at, reason from unlocks where user_id = ?
            on conflict do nothing
            """,
            target, source,
        )
        c.execute("update memories set user_id = ? where user_id = ?", target, source)
        c.execute("update devices set user_id = ? where user_id = ?", target, source)
        c.execute("update copy_reports set user_id = ? where user_id = ?", target, source)
        c.execute("update users set referred_by = ? where referred_by = ?", target, source)
        c.execute("delete from users where id = ?", source)
    }
}
