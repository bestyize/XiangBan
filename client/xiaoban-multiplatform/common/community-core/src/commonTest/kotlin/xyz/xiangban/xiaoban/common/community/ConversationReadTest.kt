package xyz.xiangban.xiaoban.common.community

import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class ConversationReadTest {
    @Test fun executionHandoffIncludesCorrectionsAndEveryAttachmentIndex() {
        val messages = listOf(
            ChatTurn(1, "user", "原始需求"),
            ChatTurn(2, "assistant", "旧方案", creationPlan = true),
            ChatTurn(3, "user", "更正：不要红色，保留导出功能", attachments = listOf(ChatAttachment("规格.txt", text = "完整规格"), ChatAttachment("参考.png", imageDataUrl = "data:image/png;base64,YQ=="))),
            ChatTurn(4, "assistant", "新方案", creationPlan = true),
        )
        val input = creationExecutionInput(CreationDraft("作品", "原始需求"), messages, messages.last())
        assertTrue(input.contains("更正：不要红色，保留导出功能"))
        assertTrue(input.contains("规格.txt"))
        assertTrue(input.contains("参考.png"))
        assertTrue(input.contains("\"message_id\":3"))
        assertTrue(input.contains("\"index\":1"))
        assertTrue(input.contains("新方案"))
        assertFalse(input.contains("旧方案"))
    }

    @Test fun longOriginalAndAttachmentCanBeReadWithoutDroppingAnyCharacters() = runTest {
        val original = "开头" + "内容🙂".repeat(3000) + "末尾必须保留"
        val saved = StoredChat("own-session", listOf(ChatTurn(7, "user", original, attachments = listOf(ChatAttachment("完整.txt", text = original)))))
        for (attachment in listOf(false, true)) {
            val recovered = StringBuilder()
            var offset = 0
            do {
                val result = readConversation(saved, buildJsonObject {
                    put("message_id", 7); put("offset", offset); put("limit", 111)
                    if (attachment) put("attachment_index", 0)
                }) { error("text must not resolve an image") }
                assertTrue(result.ok)
                val page = CommunityJson.parseToJsonElement(result.output).jsonObject
                recovered.append(page.getValue("text").jsonPrimitive.content)
                val next = page.getValue("next_offset")
                if (next == JsonNull) break
                offset = next.jsonPrimitive.content.toInt()
            } while (true)
            assertEquals(original, recovered.toString())
        }
    }

    @Test fun imageReadUsesHostSessionAndHonorsModelVisionCapability() = runTest {
        val image = "data:image/png;base64,YQ=="
        val saved = StoredChat("own-session", listOf(ChatTurn(7, "user", "看参考图", attachments = listOf(ChatAttachment("参考.png", imageDataUrl = image)))))
        val tool = conversationReadTool { session, args ->
            assertEquals("own-session", session)
            readConversation(saved, args) { it }
        }
        val args = buildJsonObject { put("message_id", 7); put("attachment_index", 0); put("session", "another-session") }
        val result = withContext(AgentToolContext("own-session", "call-1")) { tool.run(args) }
        assertTrue(result.ok)
        assertEquals(image, result.parts!!.single().jsonObject.getValue("url").jsonPrimitive.content)
        assertFalse(result.output.contains("base64"))
        val unsupported = withContext(AgentToolContext("own-session", "call-2", supportsVision = false)) { tool.run(args) }
        assertFalse(unsupported.ok)
        assertEquals(null, unsupported.parts)
    }

    @Test fun indexAndMissingMessagesHaveExplicitBoundaries() = runTest {
        val saved = StoredChat("s", listOf(ChatTurn(1, "user", "一"), ChatTurn(2, "assistant", "二")))
        val page = readConversation(saved, buildJsonObject { put("limit", 1) }) { it }
        val json = CommunityJson.parseToJsonElement(page.output).jsonObject
        assertEquals("1", json.getValue("next_from").jsonPrimitive.content)
        assertEquals(1, json.getValue("messages").jsonArray.size)
        assertFalse(readConversation(saved, buildJsonObject { put("message_id", 99) }) { it }.ok)
        assertFalse(readConversation(saved, buildJsonObject { put("message_id", 1); put("offset", 9) }) { it }.ok)
    }
}
