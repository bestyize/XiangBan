package xyz.xiaban.server.quizzes

import kotlinx.serialization.Serializable
import kotlinx.serialization.builtins.ListSerializer
import kotlinx.serialization.builtins.nullable
import kotlinx.serialization.builtins.serializer
import xyz.xiaban.server.db.*
import xyz.xiaban.server.http.AppJson
import xyz.xiaban.server.http.badRequest

@Serializable data class QuizAttemptRecord(
    val quizId: String, val version: Int, val answers: List<Int?>, val cursor: Int,
    val completedAt: Long? = null, val updatedAt: Long,
)

/** Answers are stored by account and definition version; reports are derived from these answers. */
class QuizRecords(private val db: Db) {
    suspend fun all(userId: Long): List<QuizAttemptRecord> = db.read { c ->
        c.query("select * from quiz_attempts where user_id=?", userId, map = ::view)
    }

    suspend fun save(userId: Long, record: QuizAttemptRecord): QuizAttemptRecord {
        validateQuizAttempt(record)
        return db.tx { c ->
            c.execute("""insert into quiz_attempts(user_id,quiz_id,version,answers,cursor,completed_at,client_updated_at)
                values (?,?,?,?::jsonb,?,?,?) on conflict (user_id,quiz_id,version) do update set
                answers=excluded.answers,cursor=excluded.cursor,completed_at=excluded.completed_at,
                client_updated_at=excluded.client_updated_at,updated_at=now()
                where quiz_attempts.client_updated_at < excluded.client_updated_at""",
                userId, record.quizId, record.version, AppJson.encodeToString(answerSerializer, record.answers),
                record.cursor, record.completedAt, record.updatedAt)
            c.queryOne("select * from quiz_attempts where user_id=? and quiz_id=? and version=?", userId, record.quizId, record.version, map = ::view)!!
        }
    }

    private fun view(rs: java.sql.ResultSet) = QuizAttemptRecord(
        rs.getString("quiz_id"), rs.getInt("version"), AppJson.decodeFromString(answerSerializer, rs.getString("answers")),
        rs.getInt("cursor"), rs.longOrNull("completed_at"), rs.getLong("client_updated_at"),
    )
    private companion object { val answerSerializer = ListSerializer(Int.serializer().nullable) }
}

internal fun validateQuizAttempt(record: QuizAttemptRecord) {
    val count = when (record.quizId to record.version) {
        "mbti" to 1 -> 40
        "mbanti" to 1 -> 25
        "mbanti" to 2 -> 20
        "animal" to 1, "afterhours" to 1 -> 12
        else -> badRequest("没有找到这套测试版本")
    }
    val values = when (record.quizId) {
        "mbti" -> setOf(1, 2, 3, 4, 5)
        "mbanti" -> setOf(0, 1, 2, 3, 4)
        else -> setOf(-2, -1, 1, 2)
    }
    if (record.answers.size != count || record.answers.any { it != null && it !in values } || record.cursor !in 0 until count)
        badRequest("测试答案或题号不正确")
    if (record.quizId == "mbanti") {
        val omitted = if (record.version == 1) List(25) { it % 5 }
            else listOf(0, 1, 2, 3, 4, 2, 3, 4, 0, 1, 0, 1, 2, 3, 4, 0, 1, 2, 3, 4)
        if (record.answers.withIndex().any { (index, value) -> value == omitted[index] })
            badRequest("请选择题目中的选项")
    }
    if (record.updatedAt !in 1..(System.currentTimeMillis() + 300_000L)) badRequest("保存时间不正确")
    if (record.completedAt != null && (record.completedAt <= 0 || record.completedAt > record.updatedAt || record.answers.any { it == null }))
        badRequest("请先完成所有题目")
}
