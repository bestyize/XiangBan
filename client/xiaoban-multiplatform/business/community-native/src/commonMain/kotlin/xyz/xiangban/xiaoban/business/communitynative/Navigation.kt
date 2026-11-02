package xyz.xiangban.xiaoban.business.communitynative

import io.ktor.http.encodeURLParameter
import io.ktor.http.decodeURLPart

/** 页面栈里的一页。编码成字符串，方便旋转屏幕后恢复。 */
sealed interface Route {
    data class Pack(val id: Long) : Route
    data class Ability(val id: Long) : Route
    data class Scheme(val id: Long) : Route
    data object SchemeBackups : Route
    data object MyWorks : Route
    data object Notifications : Route
    data object Challenges : Route
    data class Challenge(val key: String) : Route
    data object Bookmarks : Route
    data object Blocks : Route
    data class User(val uid: String, val tab: String = "topics") : Route
    data class PublishPack(val packId: Long? = null, val localGameId: String? = null) : Route
    data class Topic(val id: Long) : Route
    data class Weekly(val week: String) : Route
    data class Project(val slug: String) : Route
    /** AI 工具的详情页，[key] 是后台里的工具标识。 */
    data class Tool(val key: String) : Route
    /** 往期 AI 日报。 */
    data object Daily : Route
    data class News(val key: String) : Route
    /** 开源热榜项目的站内详情：仓库卡片、中文 README、评论。 */
    data class Repo(val owner: String, val repo: String) : Route
    data class OpenSource(val owner: String, val repo: String) : Route
    data class OpenSourceCollection(val key: String) : Route
    data object OpenSourceShares : Route
    /** 模型天梯。 */
    data object Leaderboard : Route
    /** OpenRouter 调用量 Top 50 与历史完整日榜。 */
    data object UsageLeaderboard : Route
    /** 智能 vs 价格的横屏全屏散点图。 */
    data object Scatter : Route
    data object Login : Route
    data object Search : Route
    data object NewPost : Route
    data class CreationPost(val title: String, val body: String) : Route
    /** 发求助：和发帖同一页，一打开就选好「资源求助」并显示悬赏设置。 */
    data object NewHelp : Route
    data object Settings : Route
    data object Models : Route
    /** 设置里的外观（跟随系统 / 浅色 / 深色）和主题色。 */
    data object Appearance : Route
    /** 设置里的声音：语音识别、朗读（来源、音色、语速）、通话。 */
    data object Voice : Route
    /** 云端小伴的对话页。 */
    data object AiChat : Route
    data object Devices : Route
    /** 某台设备上的聊天页：左侧栏是这台设备的项目和对话，具体打开哪个对话由遥控控制器记着。 */
    data class RemoteChat(val peer: String, val draft: String = "") : Route

    fun encode(): String = when (this) {
        is Pack -> "pack:$id"
        is Ability -> "ability:$id"
        is Scheme -> "scheme:$id"
        SchemeBackups -> "schemebackups"
        MyWorks -> "myworks"
        Notifications -> "notifications"
        Challenges -> "challenges"
        is Challenge -> "challenge:$key"
        Bookmarks -> "bookmarks"
        Blocks -> "blocks"
        is User -> "user:$uid:$tab"
        is PublishPack -> localGameId?.let { "publishgame:$it" } ?: packId?.let { "publishpack:$it" } ?: "publishpack"
        is Topic -> "topic:$id"
        is Weekly -> "weekly:$week"
        is Project -> "project:$slug"
        is Tool -> "tool:$key"
        Daily -> "daily"
        is News -> "news:${key.encodeURLParameter()}"
        is Repo -> "repo:${owner.encodeURLParameter()}/${repo.encodeURLParameter()}"
        is OpenSource -> "opensource:${owner.encodeURLParameter()}/${repo.encodeURLParameter()}"
        is OpenSourceCollection -> "opensource-collection:${key.encodeURLParameter()}"
        OpenSourceShares -> "opensource-shares"
        Leaderboard -> "leaderboard"
        UsageLeaderboard -> "usage-leaderboard"
        Scatter -> "scatter"
        Login -> "login"
        Search -> "search"
        NewPost -> "newpost"
        is CreationPost -> "creationpost:${title.encodeURLParameter()}:${body.encodeURLParameter()}"
        NewHelp -> "newhelp"
        Settings -> "settings"
        Models -> "models"
        Appearance -> "appearance"
        Voice -> "voice"
        AiChat -> "aichat"
        Devices -> "devices"
        is RemoteChat -> "rchat:$peer" + if (draft.isNotBlank()) ":${draft.encodeURLParameter()}" else ""
    }

    companion object {
        fun decode(text: String): Route? {
            val key = text.substringBefore(':')
            val value = text.substringAfter(':', "")
            return when (key) {
                "pack" -> value.toLongOrNull()?.let(::Pack)
                "ability" -> value.toLongOrNull()?.takeIf { it > 0 }?.let(::Ability)
                "scheme" -> value.toLongOrNull()?.takeIf { it > 0 }?.let(::Scheme)
                "schemebackups" -> SchemeBackups
                "myworks" -> MyWorks
                "notifications" -> Notifications
                "challenges" -> Challenges
                "challenge" -> value.takeIf { it.isNotEmpty() }?.let(::Challenge)
                "bookmarks" -> Bookmarks
                "blocks" -> Blocks
                "user" -> value.substringBefore(':').takeIf { it.isNotEmpty() }?.let { User(it, value.substringAfter(':', "topics")) }
                "publishpack" -> if (value.isEmpty()) PublishPack() else value.toLongOrNull()?.takeIf { it > 0 }?.let(::PublishPack)
                "topic" -> value.toLongOrNull()?.let(::Topic)
                "weekly" -> value.takeIf { it.isNotEmpty() }?.let(::Weekly)
                "project" -> value.takeIf { it.isNotEmpty() }?.let(::Project)
                "tool" -> value.takeIf { it.isNotEmpty() }?.let(::Tool)
                "daily" -> Daily
                "news" -> runCatching { value.decodeURLPart().takeIf { it.isNotBlank() }?.let(::News) }.getOrNull()
                "opensource-collection" -> runCatching { value.decodeURLPart().takeIf { it.isNotBlank() }?.let(::OpenSourceCollection) }.getOrNull()
                "opensource-shares" -> OpenSourceShares
                "repo", "opensource" -> runCatching {
                    val owner = value.substringBefore('/').decodeURLPart()
                    val repo = value.substringAfter('/', "").decodeURLPart()
                    if (owner.isNotBlank() && repo.isNotBlank() && '/' !in owner && '/' !in repo) {
                        if (key == "opensource") OpenSource(owner, repo) else Repo(owner, repo)
                    } else null
                }.getOrNull()
                "leaderboard" -> Leaderboard
                "usage-leaderboard" -> UsageLeaderboard
                "scatter" -> Scatter
                "login" -> Login
                "search" -> Search
                "newpost" -> NewPost
                "creationpost" -> runCatching { CreationPost(value.substringBefore(':').decodeURLPart(), value.substringAfter(':', "").decodeURLPart()) }.getOrNull()
                "publishgame" -> value.takeIf { it.matches(Regex("[a-z0-9][a-z0-9-]{1,39}")) }?.let { PublishPack(localGameId = it) }
                "newhelp" -> NewHelp
                "settings" -> Settings
                "models" -> Models
                "appearance", "accent" -> Appearance
                "voice" -> Voice
                "aichat" -> AiChat
                "devices" -> Devices
                "rchat" -> value.substringBefore(':').takeIf { it.isNotEmpty() }?.let { peer -> runCatching { RemoteChat(peer, value.substringAfter(':', "").decodeURLPart()) }.getOrNull() }
                else -> null
            }
        }

        /** 栈用换行分隔存成一个字符串。周刊期号、项目标识、设备编号里不会有换行。 */
        fun decodeStack(text: String): List<Route> = buildList {
            text.split('\n').forEach { encoded ->
                val route = if (encoded.isEmpty()) null else decode(encoded)
                if (route != null && !(encoded == "accent" && lastOrNull() == Appearance)) add(route)
            }
        }
        fun encodeStack(stack: List<Route>): String = stack.joinToString("\n") { it.encode() }
    }
}

/** 悬浮导航里的四个去处（中间的“发布”是弹层，不是页面）。 */
enum class Tab(val label: String) { Ai("小伴AI"), Games("创造"), Community("社区"), Mine("我的") }
