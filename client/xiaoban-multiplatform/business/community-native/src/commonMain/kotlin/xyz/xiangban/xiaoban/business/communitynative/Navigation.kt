package xyz.xiangban.xiaoban.business.communitynative

/** 页面栈里的一页。编码成字符串，方便旋转屏幕后恢复。 */
sealed interface Route {
    data class Pack(val id: Long) : Route
    data class Topic(val id: Long) : Route
    data class Weekly(val week: String) : Route
    data class Project(val slug: String) : Route
    /** AI 工具的详情页，[key] 是后台里的工具标识。 */
    data class Tool(val key: String) : Route
    /** 往期 AI 日报。 */
    data object Daily : Route
    /** 模型天梯。 */
    data object Leaderboard : Route
    /** 智能 vs 价格的横屏全屏散点图。 */
    data object Scatter : Route
    data object Login : Route
    data object Search : Route
    data object NewPost : Route
    /** 发求助：和发帖同一页，一打开就选好「资源求助」并显示悬赏设置。 */
    data object NewHelp : Route
    data object Settings : Route
    data object Models : Route
    /** 设置里的外观（跟随系统 / 浅色 / 深色）和主题色。 */
    data object Appearance : Route
    data object AccentColor : Route
    /** 设置里的声音：语音识别、朗读（来源、音色、语速）、通话。 */
    data object Voice : Route
    /** 云端小伴的对话页。 */
    data object AiChat : Route
    data object Devices : Route
    /** 某台设备上的聊天页：左侧栏是这台设备的项目和对话，具体打开哪个对话由遥控控制器记着。 */
    data class RemoteChat(val peer: String) : Route

    fun encode(): String = when (this) {
        is Pack -> "pack:$id"
        is Topic -> "topic:$id"
        is Weekly -> "weekly:$week"
        is Project -> "project:$slug"
        is Tool -> "tool:$key"
        Daily -> "daily"
        Leaderboard -> "leaderboard"
        Scatter -> "scatter"
        Login -> "login"
        Search -> "search"
        NewPost -> "newpost"
        NewHelp -> "newhelp"
        Settings -> "settings"
        Models -> "models"
        Appearance -> "appearance"
        AccentColor -> "accent"
        Voice -> "voice"
        AiChat -> "aichat"
        Devices -> "devices"
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
                "tool" -> value.takeIf { it.isNotEmpty() }?.let(::Tool)
                "daily" -> Daily
                "leaderboard" -> Leaderboard
                "scatter" -> Scatter
                "login" -> Login
                "search" -> Search
                "newpost" -> NewPost
                "newhelp" -> NewHelp
                "settings" -> Settings
                "models" -> Models
                "appearance" -> Appearance
                "accent" -> AccentColor
                "voice" -> Voice
                "aichat" -> AiChat
                "devices" -> Devices
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
