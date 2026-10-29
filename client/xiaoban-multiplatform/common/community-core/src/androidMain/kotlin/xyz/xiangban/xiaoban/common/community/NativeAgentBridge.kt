package xyz.xiangban.xiaoban.common.community

/**
 * Rust 内核（libs/agent-mobile）的 JNI 入口。函数名要和 `libs/agent-mobile/src/jni_api.rs` 里的符号一致，
 * 所以这个类的包名和类名不能改。
 */
object AgentNative {
    init {
        System.loadLibrary("agent_mobile")
    }

    @JvmStatic external fun nativeStart(config: String): Long
    @JvmStatic external fun nativeSend(handle: Long, line: String)
    @JvmStatic external fun nativePoll(handle: Long, timeoutMs: Long): String?
    @JvmStatic external fun nativeStop(handle: Long)
}

private class NativeAgentBridge : AgentBridge {
    override fun start(config: String): Long = AgentNative.nativeStart(config)
    override fun send(handle: Long, line: String) = AgentNative.nativeSend(handle, line)
    override fun poll(handle: Long, timeoutMs: Long): String? = AgentNative.nativePoll(handle, timeoutMs)
    override fun stop(handle: Long) = AgentNative.nativeStop(handle)
}

/** 装了内核库就返回桥；没有（比如本地没编 Rust 库的调试包）返回 null，界面退回纯云端聊天。 */
fun nativeAgentBridgeOrNull(): AgentBridge? = try {
    AgentNative.nativeStop(0) // 触发加载；句柄 0 什么也不做
    NativeAgentBridge()
} catch (_: LinkageError) {
    null
}
