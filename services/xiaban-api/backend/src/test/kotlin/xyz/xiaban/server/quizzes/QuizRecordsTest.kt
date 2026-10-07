package xyz.xiaban.server.quizzes

import kotlin.test.Test
import kotlin.test.assertFailsWith
import xyz.xiaban.server.http.ApiException

class QuizRecordsTest {
    private fun attempt() = QuizAttemptRecord("animal", 1, List(12) { null }, 0, updatedAt = System.currentTimeMillis())
    @Test fun `allows partial and complete valid attempts`() {
        validateQuizAttempt(attempt())
        validateQuizAttempt(attempt().copy(answers = List(12) { -2 }, cursor = 11, completedAt = 1))
        validateQuizAttempt(QuizAttemptRecord("mbanti", 2, List(20) { if (it == 0) 4 else null }, 19, updatedAt = System.currentTimeMillis()))
    }
    @Test fun `rejects mismatched versions invalid answers and premature reports`() {
        for (record in listOf(attempt().copy(version = 2), attempt().copy(answers = List(11) { -2 }),
            attempt().copy(answers = List(12) { 0 }), attempt().copy(cursor = 12),
            attempt().copy(completedAt = 1), attempt().copy(updatedAt = 0))) {
            assertFailsWith<ApiException> { validateQuizAttempt(record) }
        }
    }
}
