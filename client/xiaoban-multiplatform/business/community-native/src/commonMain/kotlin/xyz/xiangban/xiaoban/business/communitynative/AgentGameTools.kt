package xyz.xiangban.xiaoban.business.communitynative

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.*
import xyz.xiangban.xiaoban.common.community.*

/** 只有宿主具备相应能力才向模型注册，避免把按钮文案当作真实工具。 */
internal fun mobileGameActionTools(store: MobileGameStore?, workspace: MobileWorkspace?, platform: PlatformActions?): List<AgentTool> {
    if (store == null || workspace == null) return emptyList()
    fun tool(toolName: String, help: String, readOnly: Boolean = false, action: suspend (String) -> String) = object : AgentTool {
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
            store.validate(id)
            return AgentToolResult(true, action(id))
        }
    }
    return buildList {
        add(tool("app_runtime_read", "读取手机试玩记录（打开、xb.ready 用时、脚本报错、访问过的域名和成绩事件）；没有记录不代表已经验证。", true) { id ->
            val logs = workspace.list().any { it.path == "logs" && it.directory } &&
                workspace.list("logs").any { it.path == "logs/$id.txt" }
            if (logs) workspace.read("logs/$id.txt") else "还没有这件作品的试玩记录，请用户试玩后再检查。"
        })
        if (platform != null) {
            if (platform.gamesSupported) addAll(mobileGameAssetTools(
                workspace, platform::creationThreeLibrary,
                if (platform.forumFilePickerSupported) platform::pickForumFile else null,
            ))
            add(tool("app_open", "在手机运行宿主中全屏打开已校验的作品供用户试玩；这不代表自动测试通过。") { id ->
                check(withContext(Dispatchers.Main) { platform.openGame(id) }) { "这个平台暂不支持作品试玩，或系统 WebView 版本过低" }
                "已请求打开作品，请用户实际试玩，再用 app_runtime_read 检查运行记录。"
            })
            add(tool("app_export", "把作品导出为协议 2 的 ZIP 并保存到手机的下载目录，可用于备份或分享。") { id ->
                check(withContext(Dispatchers.Main) { platform.exportGame(id) }) { "无法导出作品，当前平台可能尚未接入导出能力" }
                "已把作品包保存到下载目录的 Xiaoban 文件夹。"
            })
        }
    }
}
