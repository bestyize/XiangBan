package xyz.xiaban.server.memory

import xyz.xiaban.server.db.Db
import xyz.xiaban.server.db.execute
import xyz.xiaban.server.db.query

enum class MemoryKind(val code: String) { Event("event"), Preference("preference"), Emotion("emotion") }

/** Short notes about the user, shared with the 相伴 App and fed to the model as context. */
class MemoryService(private val db: Db) {

    suspend fun record(userId: Long, kind: MemoryKind, content: String, source: String, sourceRef: String? = null) = db.read { c ->
        c.execute(
            "insert into memories (user_id, kind, content, source, source_ref) values (?, ?, ?, ?, ?)",
            userId, kind.code, content, source, sourceRef,
        )
    }

    suspend fun forget(userId: Long, sourceRef: String) = db.read { c ->
        c.execute("delete from memories where user_id = ? and source_ref = ?", userId, sourceRef)
    }

    suspend fun recent(userId: Long, limit: Int = 3): List<String> = db.read { c ->
        c.query("select content from memories where user_id = ? order by created_at desc, id desc limit ?", userId, limit) {
            it.getString(1)
        }
    }
}
