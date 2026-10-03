package xyz.xiaban.server.games

import kotlinx.serialization.Serializable
import xyz.xiaban.server.db.*
import xyz.xiaban.server.http.badRequest
import xyz.xiaban.server.http.notFound
import java.util.UUID

@Serializable data class GameStart(val gameId: String)
@Serializable data class GameProgress(val score: Long = 0, val lines: Int = 0, val maxTile: Int = 0, val finished: Boolean = false)
@Serializable data class GameSessionView(val id: String, val gameId: String, val startedAt: Long, val score: Long, val lines: Int, val maxTile: Int, val finishedAt: Long?)
@Serializable data class GameSummary(val gameId: String, val sessions: Long, val bestScore: Long, val bestTile: Int)

class GameRecords(private val db: Db) {
    suspend fun start(userId: Long, gameId: String): GameSessionView {
        if (gameId !in setOf("merge-2048", "falling-blocks")) badRequest("没有找到这款游戏")
        val id = UUID.randomUUID()
        return db.read { c -> c.queryOne("insert into game_sessions(id,user_id,game_id) values (?,?,?) returning *", id, userId, gameId, map = ::view)!! }
    }
    suspend fun progress(userId: Long, rawId: String, update: GameProgress): GameSessionView {
        val id = runCatching { UUID.fromString(rawId) }.getOrElse { badRequest("游戏记录编号不正确") }
        if (update.score !in 0..1000000000L || update.lines !in 0..1000000 || update.maxTile !in 0..1073741824) badRequest("游戏成绩不正确")
        return db.read { c -> c.queryOne("""update game_sessions set score=greatest(score,?),lines=greatest(lines,?),max_tile=greatest(max_tile,?),
            finished_at=case when ? then coalesce(finished_at,now()) else finished_at end where id=? and user_id=? returning *""",
            update.score, update.lines, update.maxTile, update.finished, id, userId, map = ::view) ?: notFound("没有找到你的游戏记录") }
    }
    suspend fun summaries(userId: Long): List<GameSummary> = db.read { c -> c.query(
        "select game_id,count(*) as sessions,max(score) as best_score,max(max_tile) as best_tile from game_sessions where user_id=? group by game_id", userId) {
        GameSummary(it.getString("game_id"), it.getLong("sessions"), it.getLong("best_score"), it.getInt("best_tile")) } }
    private fun view(rs: java.sql.ResultSet) = GameSessionView(rs.getObject("id").toString(), rs.getString("game_id"), rs.instant("started_at")!!.toEpochMilli(),
        rs.getLong("score"), rs.getInt("lines"), rs.getInt("max_tile"), rs.instant("finished_at")?.toEpochMilli())
}
