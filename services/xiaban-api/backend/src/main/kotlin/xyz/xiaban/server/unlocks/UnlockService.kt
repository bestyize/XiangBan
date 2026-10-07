package xyz.xiaban.server.unlocks

import xyz.xiaban.contracts.afterwork.unlocks.UnlockView
import xyz.xiaban.contracts.afterwork.unlocks.UnlocksResponse
import xyz.xiaban.server.db.Db
import xyz.xiaban.server.db.instant
import xyz.xiaban.server.db.query
import xyz.xiaban.server.db.queryOne

class UnlockService(private val db: Db) {

    /** Grants what is not yet unlocked and returns only the new ones, for the unlock animation. */
    suspend fun grant(userId: Long, ids: Set<String>): List<UnlockView> {
        val defs = ids.mapNotNull(UnlockCatalog::find)
        if (defs.isEmpty()) return emptyList()
        return db.tx { c ->
            defs.mapNotNull { def ->
                c.queryOne(
                    """
                    insert into unlocks (user_id, expression_id, reason) values (?, ?, ?)
                    on conflict do nothing
                    returning unlocked_at
                    """,
                    userId, def.id, def.hint,
                ) { UnlockView(def.id, def.name, def.hint, it.instant("unlocked_at")!!.toEpochMilli()) }
            }
        }.sortedBy { UnlockCatalog.all.indexOfFirst { def -> def.id == it.id } }
    }

    suspend fun list(userId: Long): UnlocksResponse {
        val unlocked = db.read { c ->
            c.query("select expression_id, unlocked_at from unlocks where user_id = ?", userId) {
                it.getString(1) to it.instant("unlocked_at")!!.toEpochMilli()
            }.toMap()
        }
        val items = UnlockCatalog.all.map { UnlockView(it.id, it.name, it.hint, unlocked[it.id]) }
        return UnlocksResponse(
            appTotal = UnlockCatalog.APP_TOTAL,
            webTotal = items.size,
            unlockedCount = items.count { it.unlockedAt != null },
            items = items,
        )
    }
}
