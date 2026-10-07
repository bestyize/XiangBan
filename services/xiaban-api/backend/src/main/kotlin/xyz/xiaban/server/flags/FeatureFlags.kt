package xyz.xiaban.server.flags

import kotlinx.coroutines.future.await
import xyz.xiaban.contracts.admin.settings.FlagsUpdate
import xyz.xiaban.contracts.admin.settings.FlagsView
import xyz.xiaban.contracts.common.Features
import xyz.xiaban.server.redis.Redis

/**
 * Ordinary feature flags live in Redis. When an LLM settings store is supplied, both the legacy
 * flags API and model calls use its single switch. Old Redis LLM values are only bootstrap defaults.
 */
class FeatureFlags internal constructor(
    private val readStored: suspend (String) -> Boolean?,
    private val writeStored: suspend (String, Boolean) -> Unit,
    private val llmDefault: Boolean,
    private val goHomeDefault: Boolean,
    private val appDownloadUrl: String?,
    private val llmSwitch: LlmSwitch? = null,
) {
    constructor(redis: Redis, llmDefault: Boolean, goHomeDefault: Boolean, appDownloadUrl: String?, llmSwitch: LlmSwitch? = null) : this(
        readStored = { field -> redis.cmd.hget(KEY, field).await()?.let { it == "1" } },
        writeStored = { field, value -> redis.cmd.hset(KEY, field, if (value) "1" else "0").await(); Unit },
        llmDefault = llmDefault,
        goHomeDefault = goHomeDefault,
        appDownloadUrl = appDownloadUrl,
        llmSwitch = llmSwitch,
    )

    suspend fun llmEnabled(): Boolean = llmSwitch?.enabled() ?: (readStored(LLM) ?: llmDefault)

    /** 带小伴回家 (PRD 8.4): off until phase three, and useless without somewhere to send people. */
    suspend fun goHomeEnabled(): Boolean = appDownloadUrl != null && (readStored(GO_HOME) ?: goHomeDefault)

    suspend fun view(): FlagsView = FlagsView(llmEnabled = llmEnabled(), goHomeEnabled = goHomeEnabled())

    /** What the web page needs to know. */
    suspend fun features(): Features = Features(goHome = goHomeEnabled(), appDownloadUrl = appDownloadUrl)

    suspend fun update(update: FlagsUpdate): FlagsView {
        update.llmEnabled?.let { value ->
            if (llmSwitch != null) llmSwitch.setEnabled(value) else writeStored(LLM, value)
        }
        update.goHomeEnabled?.let { writeStored(GO_HOME, it) }
        return view()
    }

    companion object {
        private const val KEY = "xb:flags"
        private const val LLM = "llm"
        private const val GO_HOME = "go_home"

        /** Preserve existing deployments until an explicit admin LLM configuration is saved. */
        suspend fun legacyLlmEnabled(redis: Redis, environmentDefault: Boolean): Boolean =
            redis.cmd.hget(KEY, LLM).await()?.let { it == "1" } ?: environmentDefault
    }
}
