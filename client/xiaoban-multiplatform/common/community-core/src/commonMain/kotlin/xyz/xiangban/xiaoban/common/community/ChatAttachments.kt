package xyz.xiangban.xiaoban.common.community

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.*

/** 仅包含用户通过系统选择器明确选择的内容，不读取手机上的任意路径。 */
@Serializable
data class ChatAttachment(val name: String, val text: String? = null, val imageDataUrl: String? = null)

fun validateChatAttachments(attachments: List<ChatAttachment>) {
    require(attachments.size <= 3) { "每条消息最多添加 3 个附件" }
    var bytes = 0
    for (a in attachments) {
        require(a.name.isNotBlank() && a.name.length <= 160) { "附件名称无效" }
        require((a.text != null) != (a.imageDataUrl != null)) { "附件内容无效" }
        a.text?.let { require(it.encodeToByteArray().size <= 256 * 1024) { "文本附件不能超过 256 KiB" } }
        a.imageDataUrl?.let {
            require(it.length <= 2_800_000) { "图片过大，请压缩后重新选择" }
            val prefix = it.substringBefore(',')
            val payload = it.substringAfter(',', "")
            val encoded = payload.trimEnd('=')
            require(prefix in setOf("data:image/png;base64", "data:image/jpeg;base64", "data:image/webp;base64") &&
                payload.isNotEmpty() && payload.length % 4 == 0 && payload.length - encoded.length <= 2 &&
                encoded.all { c -> c in 'A'..'Z' || c in 'a'..'z' || c in '0'..'9' || c == '+' || c == '/' }
            ) { "仅支持有效的本地 PNG、JPEG、WebP 图片" }
        }
        bytes += (a.text ?: a.imageDataUrl.orEmpty()).encodeToByteArray().size
    }
    require(bytes <= 4 * 1024 * 1024) { "附件总量过大，请分开发送" }
}

internal fun attachmentPrompt(prompt: String, attachments: List<ChatAttachment>): String = buildString {
    append(prompt)
    for (a in attachments) if (a.text != null) {
        append("\n\n用户提供的文件：").append(a.name).append("\n<attached_file>\n").append(a.text).append("\n</attached_file>")
    }
}

/** stdio run.parts 和 Chat Completions 分别使用 image/url 与 image_url/image_url。 */
internal fun attachmentParts(prompt: String, attachments: List<ChatAttachment>, chatCompletions: Boolean = false): JsonArray = buildJsonArray {
    add(buildJsonObject { put("type", "text"); put("text", attachmentPrompt(prompt, attachments)) })
    for (a in attachments) a.imageDataUrl?.let { url ->
        add(buildJsonObject {
            if (chatCompletions) {
                put("type", "image_url"); putJsonObject("image_url") { put("url", url) }
            } else { put("type", "image"); put("url", url) }
        })
    }
}
