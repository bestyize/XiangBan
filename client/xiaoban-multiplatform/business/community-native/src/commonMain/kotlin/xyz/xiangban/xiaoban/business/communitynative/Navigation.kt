package xyz.xiangban.xiaoban.business.communitynative

/** 页面栈里的一页。编码成字符串，方便旋转屏幕后恢复。 */
sealed interface Route {
    data class Pack(val id: Long) : Route
    data class Topic(val id: Long) : Route
    data class Weekly(val week: String) : Route
    data class Project(val slug: String) : Route
    data object Login : Route
    data object Search : Route
    data object NewPost : Route
    data object Settings : Route
    data object Models : Route
    /** 云端小伴的对话页。 */
    data object AiChat : Route
    data object Devices : Route
    /** 某台电脑上的会话列表。 */
    data class RemoteList(val peer: String) : Route
    /** 某台电脑上的一个会话（具体会话由遥控控制器记着）。 */
    data class RemoteChat(val peer: String) : Route

    fun encode(): String = when (this) {
        is Pack -> "pack:$id"
        is Topic -> "topic:$id"
        is Weekly -> "weekly:$week"
        is Project -> "project:$slug"
        Login -> "login"
        Search -> "search"
        NewPost -> "newpost"
        Settings -> "settings"
        Models -> "models"
        AiChat -> "aichat"
        Devices -> "devices"
        is RemoteList -> "rlist:$peer"
        is RemoteChat -> "rchat:$peer"
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
                "search" -> Search
                "newpost" -> NewPost
                "settings" -> Settings
                "models" -> Models
                "aichat" -> AiChat
                "devices" -> Devices
                "rlist" -> value.takeIf { it.isNotEmpty() }?.let(::RemoteList)
                "rchat" -> value.takeIf { it.isNotEmpty() }?.let(::RemoteChat)
                else -> null
            }
        }

        /** 栈用换行分隔存成一个字符串。周刊期号、项目标识、设备编号里不会有换行。 */
        fun decodeStack(text: String): List<Route> = text.split('\n').mapNotNull { if (it.isEmpty()) null else decode(it) }
        fun encodeStack(stack: List<Route>): String = stack.joinToString("\n") { it.encode() }
    }
}

/** 悬浮导航里的四个去处（中间的“发布”是弹层，不是页面）。 */
enum class Tab(val label: String) { Ai("小伴AI"), Games("游戏"), Community("社区"), Mine("我的") }
