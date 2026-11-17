package xyz.xiangban.xiaoban.common.community

import kotlin.coroutines.AbstractCoroutineContextElement
import kotlin.coroutines.CoroutineContext

/** Host-supplied identity; never accept a session selected by model arguments. */
class AgentToolContext(val session: String, val callId: String, val supportsVision: Boolean = true) : AbstractCoroutineContextElement(Key) {
    companion object Key : CoroutineContext.Key<AgentToolContext>
}
