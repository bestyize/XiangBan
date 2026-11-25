package xyz.xiangban.xiaoban.business.communitynative

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.*
import xyz.xiangban.xiaoban.common.community.*

/** 只有宿主具备相应能力才向模型注册，避免把按钮文案当作真实工具。 */
internal fun mobileGameActionTools(store: MobileGameStore?, workspace: MobileWorkspace?, platform: PlatformActions?): List<AgentTool> {
    if (store == null || workspace == null) return emptyList()
    fun tool(toolName: String, help: String, readOnly: Boolean = false, action: suspend (MobileGame) -> String) = object : AgentTool {
        override val name = toolName
        override val description = help
        override val risk = if (readOnly) "read" else "write"
        override val parameters = buildJsonObject {
            put("type", "object")
            putJsonObject("properties") { putJsonObject("id") { put("type", "string") } }
            put("required", buildJsonArray { add(JsonPrimitive("id")) })
            put("additionalProperties", false)
        }
        override suspend fun run(arguments: JsonObject): AgentToolResult {
            val id = arguments["id"]?.jsonPrimitive?.content.orEmpty()
            return AgentToolResult(true, action(store.validate(id)))
        }
    }
    return buildList {
        add(tool("app_runtime_read", "读取本版本的启动、脚本错误和首局成绩证据。启用成绩上报时核对 first_round 与实际结算分数。", true) { work ->
            val logFile = "logs/${work.id}.txt"
            val logs = workspace.list().any { it.path == "logs" && it.directory } &&
                workspace.list("logs").any { it.path == logFile }
            mobileGameRuntimeReport(work, if (logs) workspace.read(logFile) else "").toString()
        })
        if (platform != null) {
            if (platform.gamesSupported) addAll(mobileGameAssetTools(
                workspace, platform::creationThreeLibrary,
                if (platform.forumFilePickerSupported) platform::pickForumFile else null,
            ))
            add(tool("app_open", "在手机运行宿主中全屏打开已校验的作品供用户试玩。") { work ->
                check(withContext(Dispatchers.Main) { platform.openGame(work.id) }) { "这个平台暂不支持作品试玩，或系统 WebView 版本过低" }
                "已请求打开作品。用户在作品卡片记录试玩结论。"
            })
            add(tool("app_export", "把作品导出为协议 2 的 ZIP 并保存到手机的下载目录，可用于备份或分享。") { work ->
                check(withContext(Dispatchers.Main) { platform.exportGame(work.id) }) { "无法导出作品，当前平台可能尚未接入导出能力" }
                "已把作品包保存到下载目录的 Xiaoban 文件夹。"
            })
        }
    }
}
