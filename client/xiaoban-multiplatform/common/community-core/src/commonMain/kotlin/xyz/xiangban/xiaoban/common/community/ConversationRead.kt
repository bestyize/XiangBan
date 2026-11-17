package xyz.xiangban.xiaoban.common.community

import kotlinx.coroutines.currentCoroutineContext
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonArray
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.put

/** Host history includes original planning messages and attachments absent from the native session. */
internal fun conversationReadTool(reader: suspend (String, JsonObject) -> AgentToolResult): AgentTool = object : AgentTool {
    override val name = "conversation_read"
    override val description = "读取当前会话的原始消息、制作方案和附件。无message_id时分页列出索引；指定message_id读取完整原文，正文用offset/limit继续分页；指定attachment_index读取该附件文字或图片。只能读取本次任务所属会话。"
    override val risk = "read"
    override val untrustedOutput = true
    override val parameters = buildJsonObject {
        put("type", "object")
        put("properties", buildJsonObject {
            put("message_id", buildJsonObject { put("type", "integer"); put("description", "消息索引中的id") })
            put("attachment_index", buildJsonObject { put("type", "integer"); put("minimum", 0) })
            put("from", buildJsonObject { put("type", "integer"); put("minimum", 0); put("description", "索引起点，默认0") })
            put("offset", buildJsonObject { put("type", "integer"); put("minimum", 0); put("description", "原文字符偏移，以返回的next_offset为准") })
            put("limit", buildJsonObject { put("type", "integer"); put("minimum", 1); put("maximum", 12000) })
        })
        put("additionalProperties", false)
    }
    override suspend fun run(arguments: JsonObject): AgentToolResult {
        val context = currentCoroutineContext()[AgentToolContext]
            ?: return AgentToolResult(false, "当前任务没有绑定会话，无法读取原文。")
        val result = reader(context.session, arguments)
        return if (!context.supportsVision && result.parts?.any { (it as? JsonObject)?.get("type") == JsonPrimitive("image") } == true) {
            result.copy(ok = false, output = "当前模型不支持读取图片，请选择支持图片的模型后继续。\n${result.output}", parts = null)
        } else result
    }
}

private fun attachmentIndex(attachments: List<ChatAttachment>) = buildJsonArray {
    attachments.forEachIndexed { index, attachment -> add(buildJsonObject {
        put("index", index); put("name", attachment.name)
        put("text_chars", attachment.text.orEmpty().length)
        put("image", attachment.imageDataUrl != null)
    }) }
}

/** All original IDs and attachment indexes are passed up front; large bodies remain losslessly readable. */
internal fun creationExecutionInput(draft: CreationDraft, messages: List<ChatTurn>, plan: ChatTurn): String {
    val originals = messages.filter { it.role == "user" && it.text != "确认方案，开始制作" }
    val inlineLimit = (32_000 / originals.size.coerceAtLeast(1)).coerceAtMost(12_000)
    return buildString {
        append("用户已通过宿主按钮确认方案。以下原始需求和补充按时间顺序排列，后续更正优先。\n")
        if (originals.none { it.text == draft.request }) append("作品需求：\n${draft.request}\n\n")
        append("完整原文与附件索引：\n")
        (originals + plan).forEach { turn ->
            append(buildJsonObject {
                put("message_id", turn.id); put("role", turn.role); put("text_chars", turn.text.length)
                put("attachments", attachmentIndex(turn.attachments))
            })
            append('\n')
        }
        append("\n用户需求和更正原文：\n")
        originals.forEach { turn ->
            append("[message_id=${turn.id}]\n${turn.text.take(inlineLimit)}\n")
            if (turn.text.length > inlineLimit) append("[正文未完整内联；必须用 conversation_read(message_id=${turn.id}, offset=$inlineLimit) 继续读取。]\n")
        }
        append("\n用户确认的方案 [message_id=${plan.id}]：\n${plan.text.take(24_000)}\n")
        if (plan.text.length > 24_000) append("[方案未完整内联；必须用 conversation_read(message_id=${plan.id}, offset=24000) 继续读取。]\n")
        append("\n附件内容没有内联。制作前用 conversation_read 的 message_id 和 attachment_index 读取相关附件；")
        append("文字按 next_offset 继续读取，图片以工具返回的多模态内容为准。不要用方案摘要替代用户原文。\n")
        append("先读取 app_read_format，再按确认方案制作。先完成可运行的包并安装，只有文件变更或有新证据时才重读相同目录。")
        append("检查实际运行日志，需等待用户验收时用 ask_user 明确交还，区分静态检查和实测。\n")
    }
}

internal suspend fun readConversation(
    saved: StoredChat,
    arguments: JsonObject,
    resolveImage: suspend (String) -> String?,
): AgentToolResult {
    fun number(name: String): Int? = (arguments[name] as? JsonPrimitive)?.contentOrNull?.toIntOrNull()
    fun failure(message: String) = AgentToolResult(false, message)
    val id = (arguments["message_id"] as? JsonPrimitive)?.contentOrNull?.toLongOrNull()
    if (arguments.containsKey("message_id") && id == null) return failure("message_id 必须是索引中的整数。")
    if (id == null) {
        if (arguments.containsKey("attachment_index")) return failure("读取附件时必须同时提供message_id。")
        val from = number("from") ?: 0
        val limit = (number("limit") ?: 30).coerceIn(1, 100)
        if (from !in 0..saved.messages.size) return failure("from 超出消息范围。")
        val page = saved.messages.drop(from).take(limit)
        return AgentToolResult(true, buildJsonObject {
            put("session", saved.id); put("total", saved.messages.size); put("from", from)
            put("messages", buildJsonArray { page.forEach { turn -> add(buildJsonObject {
                put("message_id", turn.id); put("role", turn.role); put("text_chars", turn.text.length)
                put("attachments", attachmentIndex(turn.attachments))
            }) } })
            put("next_from", if (from + page.size < saved.messages.size) JsonPrimitive(from + page.size) else JsonNull)
        }.toString())
    }
    val turn = saved.messages.firstOrNull { it.id == id } ?: return failure("当前会话不存在这条消息。")
    val attachment = if (arguments.containsKey("attachment_index")) {
        val index = number("attachment_index") ?: return failure("attachment_index 必须是整数。")
        turn.attachments.getOrNull(index) ?: return failure("附件索引超出范围。")
    } else null
    val source = if (attachment == null) turn.text else attachment.text.orEmpty()
    val requestedOffset = number("offset") ?: 0
    if (requestedOffset !in 0..source.length) return failure("offset 超出原文范围。")
    // Do not split a UTF-16 surrogate pair. Consumers continue with the exact returned offset.
    val offset = if (requestedOffset > 0 && requestedOffset < source.length && source[requestedOffset].code in 0xDC00..0xDFFF && source[requestedOffset - 1].code in 0xD800..0xDBFF) requestedOffset - 1 else requestedOffset
    var end = (offset + (number("limit") ?: 6000).coerceIn(1, 12_000)).coerceAtMost(source.length)
    if (end < source.length && end > offset && source[end - 1].code in 0xD800..0xDBFF && source[end].code in 0xDC00..0xDFFF) end++
    val image = attachment?.imageDataUrl?.let { reference ->
        resolveImage(reference) ?: return failure("附件图片暂不可用，请重新选择该图片。")
    }
    if (image != null && (!image.startsWith("data:image/") || image.length > CHAT_IMAGE_DATA_URL_LIMIT)) return failure("附件图片超过模型读取上限，请先缩小图片。")
    return AgentToolResult(true, buildJsonObject {
        put("session", saved.id); put("message_id", turn.id); put("role", turn.role)
        attachment?.let { put("attachment_index", number("attachment_index")); put("name", it.name) }
        put("text", source.substring(offset, end)); put("offset", offset); put("total_chars", source.length)
        put("truncated", end < source.length)
        put("next_offset", if (end < source.length) JsonPrimitive(end) else JsonNull)
        if (attachment == null) put("attachments", attachmentIndex(turn.attachments))
        if (image != null) put("image_returned", true)
    }.toString(), parts = image?.let { JsonArray(listOf(buildJsonObject { put("type", "image"); put("url", it) })) })
}
