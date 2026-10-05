package xyz.xiaban.server.flags

/** One authoritative switch shared by provider selection and legacy feature-flag APIs. */
interface LlmSwitch {
    suspend fun enabled(): Boolean
    suspend fun setEnabled(enabled: Boolean)
}
