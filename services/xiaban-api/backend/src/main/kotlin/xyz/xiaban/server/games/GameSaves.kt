package xyz.xiaban.server.games

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonObject
import xyz.xiaban.server.db.*
import xyz.xiaban.server.http.AppJson
import xyz.xiaban.server.http.badRequest

@Serializable data class GameSaveRequest(val data: JsonObject, val updatedAt: Long)
@Serializable data class GameSaveView(val data: JsonObject? = null, val updatedAt: Long? = null)

/**
 * Stores each game pack's own progress document for a registered account. The server only checks the game,
 * the size and the clock; the document's contents are the pack's business. A stale write (older client clock)
 * never replaces a newer one, so two open tabs cannot roll progress back.
 */
class GameSaves(private val db: Db) {
    suspend fun get(userId: Long, gameId: String): GameSaveView {
        requireSaveGame(gameId)
        return db.read { c -> c.queryOne("select data::text, client_updated_at from game_saves where user_id=? and game_id=?", userId, gameId, map = ::view) } ?: GameSaveView()
    }

    suspend fun put(userId: Long, gameId: String, request: GameSaveRequest): GameSaveView {
        requireSaveGame(gameId)
        val json = AppJson.encodeToString(JsonObject.serializer(), request.data)
        if (json.toByteArray().size > MAX_BYTES) badRequest("游戏存档太大")
        if (request.updatedAt !in 1..(System.currentTimeMillis() + 300_000L)) badRequest("存档时间不正确")
        return db.tx { c ->
            c.execute("""insert into game_saves(user_id,game_id,data,client_updated_at) values (?,?,?::jsonb,?)
                on conflict (user_id,game_id) do update set data=excluded.data,client_updated_at=excluded.client_updated_at,updated_at=now()
                where game_saves.client_updated_at < excluded.client_updated_at""",
                userId, gameId, json, request.updatedAt)
            c.queryOne("select data::text, client_updated_at from game_saves where user_id=? and game_id=?", userId, gameId, map = ::view)!!
        }
    }

    private fun view(rs: java.sql.ResultSet) = GameSaveView(AppJson.decodeFromString(JsonObject.serializer(), rs.getString(1)), rs.getLong(2))

    companion object {
        const val MAX_BYTES = 32 * 1024
        /** Packs whose manifest declares `"save": true`. */
        val SAVE_GAMES = setOf("ink-blade")
        fun requireSaveGame(gameId: String) { if (gameId !in SAVE_GAMES) badRequest("这款游戏没有存档") }
    }
}
