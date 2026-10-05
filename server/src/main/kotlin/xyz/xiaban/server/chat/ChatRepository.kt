package xyz.xiaban.server.chat

import xyz.xiaban.server.db.Db
import xyz.xiaban.server.db.execute
import xyz.xiaban.server.db.query
import xyz.xiaban.server.db.queryOne
import xyz.xiaban.shared.ChatMessageView
import java.time.Instant

class ChatRepository(private val db: Db) {

    suspend fun append(userId: Long, role: String, content: String): ChatMessageView = db.read { c ->
        c.queryOne(
            "insert into chat_messages (user_id, role, content) values (?, ?, ?) returning id, role, content, (extract(epoch from created_at) * 1000)::bigint",
            userId, role, content,
        ) {
            ChatMessageView(it.getLong(1), it.getString(2), it.getString(3), it.getLong(4))
        } ?: error("Failed to insert chat message")
    }

    suspend fun list(userId: Long, limit: Int = 30): List<ChatMessageView> = db.read { c ->
        val rows = c.query(
            "select id, role, content, (extract(epoch from created_at) * 1000)::bigint from chat_messages where user_id = ? order by created_at desc, id desc limit ?",
            userId, limit,
        ) {
            ChatMessageView(it.getLong(1), it.getString(2), it.getString(3), it.getLong(4))
        }
        rows.reversed()
    }

    suspend fun clear(userId: Long): Int = db.read { c ->
        c.execute("delete from chat_messages where user_id = ?", userId)
    }
}
