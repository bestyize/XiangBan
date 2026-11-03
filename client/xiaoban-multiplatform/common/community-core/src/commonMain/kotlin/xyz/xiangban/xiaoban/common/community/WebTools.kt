package xyz.xiangban.xiaoban.common.community

import io.ktor.http.Url
import kotlinx.coroutines.CancellationException
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonArray
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.intOrNull
import kotlinx.serialization.json.put
import kotlinx.serialization.json.putJsonObject

@Serializable
internal data class WebSearchReply(val results: List<WebSearchHit>)

@Serializable
internal data class WebSearchHit(val title: String, val url: String, val snippet: String, val date: String? = null)

@Serializable
internal data class WebFetchReply(val title: String, val url: String, val content: String, val truncated: Boolean = false)

fun gatewayWebTools(api: AiGatewayApi): List<AgentTool> = listOf(
    GatewayWebTool("web_search", "搜索公开网页。使用简短关键词，按需读取原文，并在回答中附上实际来源链接。", "query") { args ->
        val query = (args["query"] as? JsonPrimitive)?.contentOrNull.orEmpty().trim()
        require(query.isNotEmpty() && query.length <= 128) { "搜索关键词需要 1 到 128 字" }
        val limit = (args["limit"] as? JsonPrimitive)?.intOrNull?.coerceIn(1, 10) ?: 5
        val hits = api.webSearch(query, limit).results.take(limit)
        if (hits.isEmpty()) "没有找到相关网页。"
        else hits.joinToString("\n\n") { hit ->
            "${hit.title.take(300)}\n${hit.url}\n${hit.date.orEmpty()}\n${hit.snippet.take(1200)}"
        }
    },
    GatewayWebTool("web_fetch", "读取指定公开网页的正文。根据正文回答，并附上实际来源链接。", "url") { args ->
        val raw = (args["url"] as? JsonPrimitive)?.contentOrNull.orEmpty().trim()
        require(raw.length in 1..8192) { "请输入有效的网页地址" }
        val url = runCatching { Url(raw) }.getOrNull()
        require(url != null && url.protocol.name in setOf("http", "https") &&
            url.host.isNotBlank() && url.user == null && url.password == null &&
            (raw.startsWith("https://", true) || raw.startsWith("http://", true))) { "请输入不含账号密码的 HTTP(S) 地址" }
        val page = api.webFetch(raw)
        val content = page.content.take(12_000)
        "${page.title.take(300)}\n${page.url}\n\n$content" +
            if (page.truncated || content.length < page.content.length) "\n\n[正文已截断]" else ""
    },
)

private class GatewayWebTool(
    override val name: String,
    override val description: String,
    parameter: String,
    private val execute: suspend (JsonObject) -> String,
) : AgentTool {
    override val risk = "network"
    override val timeoutSeconds = 65L
    override val untrustedOutput = true
    override val parameters = buildJsonObject {
        put("type", "object")
        putJsonObject("properties") {
            putJsonObject(parameter) { put("type", "string") }
            if (parameter == "query") putJsonObject("limit") {
                put("type", "integer")
                put("minimum", 1)
                put("maximum", 10)
            }
        }
        put("required", buildJsonArray { add(JsonPrimitive(parameter)) })
    }

    override suspend fun run(arguments: JsonObject): AgentToolResult = try {
        AgentToolResult(true, execute(arguments))
    } catch (e: CancellationException) {
        throw e
    } catch (e: ApiException) {
        AgentToolResult(false, e.message)
    } catch (e: IllegalArgumentException) {
        AgentToolResult(false, e.message ?: "联网工具参数无效")
    }
}
