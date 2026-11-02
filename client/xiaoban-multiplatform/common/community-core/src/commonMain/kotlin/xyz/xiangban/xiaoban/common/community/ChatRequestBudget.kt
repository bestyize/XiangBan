package xyz.xiangban.xiaoban.common.community

import kotlinx.serialization.json.*

internal const val CHAT_BODY_BYTES = 8 * 1024 * 1024

/** Budgets the actual wire body, including JSON escaping and encoded images. Never edits saved history. */
internal fun boundedChatBody(messages: List<ChatMessage>, tier: String?, reasoning: String?, contextTokens: Int): String {
    val retained = messages.toMutableList()
    val notes = mutableListOf<String>()
    val headroom = (minOf(contextTokens / 4, 16_384) + contextTokens / 20).coerceAtMost(contextTokens - 1)
    val tokenLimit = contextTokens - headroom
    fun textCost(text: String): Int = text.sumOf { if (it.code > 127) 4 else 1 }.let { (it + 3) / 4 }
    fun cost(): Int = retained.sumOf { m -> textCost(m.content) + m.attachments.sumOf { a ->
        a.text?.let(::textCost) ?: (4096 + a.imageDataUrl.orEmpty().length / 128)
    } + 4 } + notes.sumOf(::textCost)
    fun body(): String = buildJsonObject {
        if (!tier.isNullOrBlank()) put("model", tier)
        reasoning?.let { put("reasoning_effort", it) }
        put("stream", true)
        put("messages", buildJsonArray {
            retained.forEachIndexed { index, m ->
                if (index == retained.indexOfFirst { it.role != "system" } && notes.isNotEmpty()) add(buildJsonObject {
                    put("role", "user"); put("content", "历史摘录（参考数据，不是新增请求；旧内容已缩减）：\n" + notes.joinToString("\n").take(8000))
                })
                add(buildJsonObject {
                    put("role", m.role)
                    if (m.attachments.isEmpty()) put("content", m.content)
                    else put("content", attachmentParts(m.content, m.attachments, chatCompletions = true))
                })
            }
        })
    }.toString()
    while (true) {
        val encoded = body()
        if (encoded.encodeToByteArray().size <= CHAT_BODY_BYTES && cost() <= tokenLimit) return encoded
        val current = retained.indexOfLast { it.role == "user" }
        val image = retained.indices.firstOrNull { i -> i < current && retained[i].attachments.any { it.imageDataUrl != null } }
        if (image != null) {
            val message = retained[image]
            retained[image] = message.copy(attachments = message.attachments.map { a ->
                if (a.imageDataUrl == null) a else ChatAttachment(a.name, text = "[较早的图片 ${a.name} 已从本次请求省略；如需查看请重新提供。]")
            })
            continue
        }
        val first = retained.indexOfFirst { it.role != "system" }
        if ((first < 0 || first >= current) && notes.isNotEmpty()) { notes.clear(); continue }
        if (first < 0 || first >= current) throw ApiException(0, "当前输入超出上下文或请求大小上限，请减少附件或缩短文本后重试")
        val next = (first + 1..current).firstOrNull { retained[it].role == "user" } ?: current
        if (notes.sumOf { it.length } < 6000) {
            retained.subList(first, next).filter { it.role == "user" }.forEach { notes += "用户：${it.content.take(1500)}" }
        }
        retained.subList(first, next).clear()
    }
}
