package xyz.xiaban.server.flags

import kotlinx.coroutines.future.await
import xyz.xiaban.server.redis.Redis
import xyz.xiaban.shared.FlagsUpdate
import xyz.xiaban.shared.FlagsView

/**
 * Runtime switches kept in Redis so every instance sees a change at once without a redeploy (PRD 9.5).
 * Unset fields fall back to the environment defaults.
 */
class FeatureFlags(private val redis: Redis, private val llmDefault: Boolean) {

    suspend fun llmEnabled(): Boolean = read(LLM) ?: llmDefault

    suspend fun view(): FlagsView = FlagsView(llmEnabled = llmEnabled())

    suspend fun update(update: FlagsUpdate): FlagsView {
        update.llmEnabled?.let { write(LLM, it) }
        return view()
    }

    private suspend fun read(field: String): Boolean? = redis.cmd.hget(KEY, field).await()?.let { it == "1" }

    private suspend fun write(field: String, value: Boolean) {
        redis.cmd.hset(KEY, field, if (value) "1" else "0").await()
    }

    private companion object {
        const val KEY = "xb:flags"
        const val LLM = "llm"
    }
}
