package xyz.xiaban.server.quizzes

import io.ktor.client.call.body
import io.ktor.http.HttpStatusCode
import kotlin.test.*
import kotlinx.serialization.json.JsonObject
import xyz.xiaban.contracts.account.auth.AccountApiPaths
import xyz.xiaban.server.account.bind
import xyz.xiaban.server.support.apiTest
class QuizRecordsFlowTest {
    @Test fun `only registered accounts can save and records stay isolated and delete with account`() = apiTest {
        val guest = session()
        val record = QuizAttemptRecord("animal", 1, List(12) { -2 }, 11, completedAt = 1, updatedAt = System.currentTimeMillis())
        assertEquals(HttpStatusCode.Unauthorized, get("/api/quizzes/progress", null).status)
        assertEquals(HttpStatusCode.Forbidden, put("/api/quizzes/progress", guest.token, record).status)
        val a = bind(guest.token, "quiz-a@example.com")
        val b = bind(session().token, "quiz-b@example.com")
        assertEquals(record, put("/api/quizzes/progress", a.token, record).body<QuizAttemptRecord>())
        assertEquals(listOf(record), get("/api/quizzes/progress", a.token).body<List<QuizAttemptRecord>>())
        assertTrue(get("/api/quizzes/progress", b.token).body<List<QuizAttemptRecord>>().isEmpty())
        val stale = record.copy(answers = List(12) { null }, completedAt = null, updatedAt = record.updatedAt - 1)
        assertEquals(record, put("/api/quizzes/progress", a.token, stale).body<QuizAttemptRecord>())
        assertEquals(HttpStatusCode.BadRequest, put("/api/quizzes/progress", a.token, record.copy(cursor = 12)).status)
        assertTrue(get(AccountApiPaths.ACCOUNT_EXPORT, a.token).body<JsonObject>().containsKey("quizAttempts"))
        delete(AccountApiPaths.ACCOUNT, a.token)
        assertTrue(services.quizzes.all(a.profile.userId).isEmpty())
    }
}
