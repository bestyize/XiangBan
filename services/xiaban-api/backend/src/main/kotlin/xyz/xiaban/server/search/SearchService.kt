package xyz.xiaban.server.search

import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.plugins.timeout
import io.ktor.client.request.get
import io.ktor.client.request.parameter
import io.ktor.http.isSuccess
import kotlinx.serialization.Serializable
import org.slf4j.LoggerFactory
import xyz.xiaban.server.admin.SearchTestResponse

class SearchService(private val http: HttpClient) {
    private val log = LoggerFactory.getLogger(SearchService::class.java)

    suspend fun search(baseUrl: String, query: String, limit: Int = 4): String {
        val root = baseUrl.trim().trimEnd('/')
        if (root.isBlank() || query.isBlank()) return ""

        return try {
            val response = http.get("$root/search") {
                parameter("q", query)
                parameter("format", "json")
                timeout { requestTimeoutMillis = 6000 }
            }
            if (!response.status.isSuccess()) {
                log.warn("SearXNG search failed with status: {}", response.status)
                return "【搜索暂不可用: 状态码 ${response.status}】"
            }
            val res = response.body<SearxngResponse>()
            val items = res.results.take(limit)
            if (items.isEmpty()) {
                return "【未找到关于「$query」的搜索结果】"
            }
            buildString {
                appendLine("关于「$query」的最新网络搜索结果：")
                items.forEachIndexed { i, it ->
                    appendLine("${i + 1}. 【${it.title}】")
                    if (it.content.isNotBlank()) appendLine("   摘要: ${it.content.take(200)}")
                    appendLine("   来源: ${it.url}")
                }
            }
        } catch (e: Exception) {
            log.warn("SearXNG request failed for query {}: {}", query, e.message)
            "【搜索暂不可用: ${e.message}】"
        }
    }

    suspend fun test(baseUrl: String): SearchTestResponse {
        val root = baseUrl.trim().trimEnd('/')
        val response = http.get("$root/search") {
            parameter("q", "测试")
            parameter("format", "json")
            timeout { requestTimeoutMillis = 6000 }
        }
        check(response.status.isSuccess()) { "SearXNG 响应异常: ${response.status}" }
        val res = response.body<SearxngResponse>()
        val firstTitle = res.results.firstOrNull()?.title ?: "无结果"
        return SearchTestResponse(true, res.results.size, firstTitle)
    }

    @Serializable
    private data class SearxngResponse(val results: List<SearxngResult> = emptyList())

    @Serializable
    private data class SearxngResult(
        val title: String = "",
        val content: String = "",
        val url: String = "",
    )
}
