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
        add(tool("game_runtime_read", "读取手机试玩记录（加载、脚本报错和成绩事件）；没有记录不代表已经验证。", true) { id ->
            val logs = workspace.list().any { it.path == "logs" && it.directory } &&
                workspace.list("logs").any { it.path == "logs/$id.txt" }
            if (logs) workspace.read("logs/$id.txt") else "还没有这款游戏的试玩记录，请用户试玩后再检查。"
        })
        if (platform != null) {
            add(tool("game_open", "在手机独立沙箱中打开已校验的游戏供用户试玩；这不代表自动测试通过。") { id ->
                check(withContext(Dispatchers.Main) { platform.openGame(id) }) { "这个平台暂不支持游戏试玩" }
                "已请求打开游戏，请用户实际试玩，再用 game_runtime_read 检查运行记录。"
            })
            add(tool("game_export", "导出兼容小伴桌面的游戏 ZIP 并打开系统保存窗口，由用户选择保存位置。") { id ->
                check(withContext(Dispatchers.Main) { platform.exportGame(id) }) { "无法导出游戏，当前平台可能尚未接入导出能力" }
                "已打开游戏导出窗口，请用户选择保存位置；保存完成后可发送到电脑，在小伴桌面导入。"
            })
        }
    }
}
