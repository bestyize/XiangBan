package xyz.xiaban.server.support

import kotlinx.coroutines.delay
import xyz.xiaban.server.llm.LlmProvider
import java.time.Duration
import java.util.concurrent.atomic.AtomicInteger

class FakeLlm : LlmProvider {
    override val name = "fake"

    @Volatile var reply: String? = "改写过的一句话，路上慢点。"
    @Volatile var delayMs = 0L
    val calls = AtomicInteger()
    @Volatile var lastUserPrompt: String? = null

    fun reset() {
        reply = "改写过的一句话，路上慢点。"
        delayMs = 0
        calls.set(0)
        lastUserPrompt = null
    }

    override suspend fun complete(system: String, user: String, timeout: Duration): String {
        calls.incrementAndGet()
        lastUserPrompt = user
        delay(delayMs)
        return reply ?: error("fake model is down")
    }
}
