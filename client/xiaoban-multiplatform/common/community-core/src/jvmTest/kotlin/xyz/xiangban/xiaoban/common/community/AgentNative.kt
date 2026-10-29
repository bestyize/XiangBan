package xyz.xiangban.xiaoban.common.community

/** 测试用：JNI 类和 Android 上同名同包，从环境变量 AGENT_MOBILE_LIB 指向的本机库加载。 */
object AgentNative {
    init {
        System.load(System.getenv("AGENT_MOBILE_LIB"))
    }

    @JvmStatic external fun nativeStart(config: String): Long
    @JvmStatic external fun nativeSend(handle: Long, line: String)
    @JvmStatic external fun nativePoll(handle: Long, timeoutMs: Long): String?
    @JvmStatic external fun nativeStop(handle: Long)
}
