package xyz.xiangban.xiaoban.business.communitynative

/** 页面栈里的一页。编码成字符串，方便旋转屏幕后恢复。 */
sealed interface Route {
    data class Pack(val id: Long) : Route
    data class Topic(val id: Long) : Route
    data class Weekly(val week: String) : Route
    data class Project(val slug: String) : Route
    data object Login : Route

    fun encode(): String = when (this) {
        is Pack -> "pack:$id"
        is Topic -> "topic:$id"
        is Weekly -> "weekly:$week"
        is Project -> "project:$slug"
        Login -> "login"
    }

    companion object {
        fun decode(text: String): Route? {
            val key = text.substringBefore(':')
            val value = text.substringAfter(':', "")
            return when (key) {
                "pack" -> value.toLongOrNull()?.let(::Pack)
                "topic" -> value.toLongOrNull()?.let(::Topic)
                "weekly" -> value.takeIf { it.isNotEmpty() }?.let(::Weekly)
                "project" -> value.takeIf { it.isNotEmpty() }?.let(::Project)
                "login" -> Login
                else -> null
            }
        }

        /** 栈用换行分隔存成一个字符串。周刊期号和项目标识里不会有换行。 */
        fun decodeStack(text: String): List<Route> = text.split('\n').mapNotNull { if (it.isEmpty()) null else decode(it) }
        fun encodeStack(stack: List<Route>): String = stack.joinToString("\n") { it.encode() }
    }
}

enum class Tab(val label: String) { Community("社区"), Discuss("讨论"), Ai("小伴"), Mine("我的") }
