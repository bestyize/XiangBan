package xyz.xiangban.xiaoban.common.community

import kotlinx.serialization.json.*

/** Convert the bounded, plain-chat request. Agent tool requests are encoded by agent-kernel instead. */
internal fun customPlainBody(chatBody: String, config: CustomModelConfig): String {
    val chat = CommunityJson.parseToJsonElement(chatBody).jsonObject
    val messages = chat["messages"]!!.jsonArray
    fun parts(content: JsonElement, responses: Boolean): JsonElement {
        if (content is JsonPrimitive) return content
        return JsonArray(content.jsonArray.map { part ->
            val obj = part.jsonObject
            when (obj["type"]?.jsonPrimitive?.contentOrNull) {
                "text" -> buildJsonObject { put("type", if (responses) "input_text" else "text"); put("text", obj["text"] ?: JsonPrimitive("")) }
                "image_url" -> {
                    val url = obj["image_url"]!!.jsonObject["url"]!!.jsonPrimitive.content
                    if (responses) buildJsonObject { put("type", "input_image"); put("image_url", url) }
                    else {
                        val meta = url.substringBefore(',', "")
                        require(meta.startsWith("data:") && meta.endsWith(";base64")) { "图片格式不受此接口支持" }
                        buildJsonObject { put("type", "image"); put("source", buildJsonObject {
                            put("type", "base64"); put("media_type", meta.removePrefix("data:").removeSuffix(";base64")); put("data", url.substringAfter(','))
                        }) }
                    }
                }
                else -> throw IllegalArgumentException("附件格式不受此接口支持")
            }
        })
    }
    return if (config.protocol == "responses") buildJsonObject {
        put("model", config.model); put("stream", true); put("store", false)
        config.maxOutputTokens?.let { put("max_output_tokens", it) }
        put("input", JsonArray(messages.map { m -> buildJsonObject {
            val obj = m.jsonObject
            put("role", obj["role"]!!); put("content", parts(obj["content"]!!, true))
        } }))
    }.toString() else {
        val system = messages.map { it.jsonObject }.filter { it["role"]?.jsonPrimitive?.contentOrNull == "system" }
            .joinToString("\n\n") { it["content"]?.jsonPrimitive?.contentOrNull.orEmpty() }
        val turns = mutableListOf<Pair<String, MutableList<JsonElement>>>()
        messages.map { it.jsonObject }.filterNot { it["role"]?.jsonPrimitive?.contentOrNull == "system" }.forEach { message ->
            val role = if (message["role"]?.jsonPrimitive?.contentOrNull == "assistant") "assistant" else "user"
            val converted = parts(message["content"]!!, false)
            val blocks = if (converted is JsonArray) converted.toList() else listOf(buildJsonObject { put("type", "text"); put("text", converted) })
            if (turns.lastOrNull()?.first == role) turns.last().second.addAll(blocks) else turns += role to blocks.toMutableList()
        }
        buildJsonObject {
            put("model", config.model); put("stream", true); put("max_tokens", config.maxOutputTokens ?: 8192)
            if (system.isNotBlank()) put("system", system)
            put("messages", JsonArray(turns.map { (role, blocks) -> buildJsonObject { put("role", role); put("content", JsonArray(blocks)) } }))
        }.toString()
    }
}

/** Plain-chat stream decoder. A protocol terminal event is required before accepting the response. */
internal class CustomPlainStream(private val protocol: String, private val onText: (String) -> Unit) {
    private val full = StringBuilder()
    private var terminal = false
    private var stopReason: String? = null
    fun payload(raw: String) {
        if (raw == "[DONE]") return
        val event = CommunityJson.parseToJsonElement(raw).jsonObject
        val type = event["type"]?.jsonPrimitive?.contentOrNull.orEmpty()
        if (type == "error") throw ApiException(0, "模型服务返回错误，请检查服务配置")
        if (protocol == "anthropic" && type == "message_delta") {
            stopReason = event["delta"]?.jsonObject?.get("stop_reason")?.jsonPrimitive?.contentOrNull ?: stopReason
        }
        val text = when {
            protocol == "responses" && type == "response.output_text.delta" -> event["delta"]?.jsonPrimitive?.contentOrNull
            protocol == "anthropic" && type == "content_block_start" && event["content_block"]?.jsonObject?.get("type")?.jsonPrimitive?.contentOrNull == "text" -> event["content_block"]?.jsonObject?.get("text")?.jsonPrimitive?.contentOrNull
            protocol == "anthropic" && type == "content_block_delta" && event["delta"]?.jsonObject?.get("type")?.jsonPrimitive?.contentOrNull == "text_delta" -> event["delta"]?.jsonObject?.get("text")?.jsonPrimitive?.contentOrNull
            else -> null
        }
        if (!text.isNullOrEmpty()) { full.append(text); onText(text) }
        if (protocol == "responses" && type in setOf("response.completed", "response.incomplete", "response.failed")) {
            event["response"]?.let { response -> customSingleText(response.toString(), protocol) { snapshot ->
                if (!snapshot.startsWith(full.toString())) throw ApiException(0, "模型返回的内容不一致，请重试")
                val rest = snapshot.substring(full.length)
                if (rest.isNotEmpty()) { full.append(rest); onText(rest) }
            } }
            if (type == "response.incomplete") throw truncatedCustomReply()
            if (type == "response.failed") throw ApiException(0, "模型回复未完成，请重试", "stream_interrupted")
            terminal = true
        }
        if (protocol == "anthropic" && type == "message_stop") {
            checkCustomStopReason(stopReason)
            terminal = true
        }
    }
    fun finish(): String {
        if (!terminal) throw ApiException(0, "模型回复未完整结束，请重试")
        return full.toString()
    }
}

internal fun customSingleText(body: String, protocol: String, onText: (String) -> Unit = {}): String {
    val obj = runCatching { CommunityJson.parseToJsonElement(body).jsonObject }.getOrElse { throw ApiException(0, "模型返回了无法识别的内容") }
    if (obj["error"] is JsonObject) throw ApiException(0, "模型服务返回错误，请检查服务配置")
    val blocks = if (protocol == "responses") {
        obj["output"]?.jsonArray.orEmpty().flatMap { item ->
            if (item.jsonObject["type"]?.jsonPrimitive?.contentOrNull == "message") item.jsonObject["content"]?.jsonArray.orEmpty() else emptyList()
        }
    } else obj["content"]?.jsonArray.orEmpty()
    val text = blocks.mapNotNull { block ->
        val item = block.jsonObject
        if (item["type"]?.jsonPrimitive?.contentOrNull in setOf("text", "output_text")) item["text"]?.jsonPrimitive?.contentOrNull else null
    }.joinToString("")
    // Preserve useful partial output before rejecting an incomplete terminal snapshot.
    if (text.isNotEmpty()) onText(text)
    if (protocol == "responses") {
        when (obj["status"]?.jsonPrimitive?.contentOrNull) {
            "incomplete" -> throw truncatedCustomReply()
            "failed", "cancelled" -> throw ApiException(0, "模型回复未完成，请重试", "stream_interrupted")
            null, "completed" -> Unit // Some compatible providers omit status in their final JSON snapshot.
            else -> throw ApiException(0, "模型回复尚未结束，请重试", "stream_interrupted")
        }
    } else checkCustomStopReason(obj["stop_reason"]?.jsonPrimitive?.contentOrNull)
    return text
}

private fun truncatedCustomReply() =
    ApiException(0, "模型回复因输出上限而未完成，已保留收到的内容；请继续生成或调整输出上限。", "response_truncated")

private fun checkCustomStopReason(reason: String?) {
    when (reason) {
        "max_tokens", "model_context_window_exceeded" -> throw truncatedCustomReply()
        null, "end_turn", "stop_sequence", "refusal" -> Unit
        else -> throw ApiException(0, "模型回复未完整结束，已保留收到的内容。", "stream_interrupted")
    }
}
