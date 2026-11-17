package xyz.xiangban.xiaoban.common.community

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith

class CustomPlainCompletionTest {
    @Test fun anthropicTokenLimitKeepsPartialTextAndRejectsMessageStop() {
        val text = StringBuilder()
        val stream = CustomPlainStream("anthropic") { text.append(it) }
        stream.payload("""{"type":"content_block_delta","delta":{"type":"text_delta","text":"方案只写到一半"}}""")
        stream.payload("""{"type":"message_delta","delta":{"stop_reason":"max_tokens"}}""")
        assertEquals("response_truncated", assertFailsWith<ApiException> {
            stream.payload("""{"type":"message_stop"}""")
        }.code)
        assertEquals("方案只写到一半", text.toString())
    }

    @Test fun normalAnthropicTerminalCompletes() {
        val stream = CustomPlainStream("anthropic") { }
        stream.payload("""{"type":"content_block_delta","delta":{"type":"text_delta","text":"完整方案"}}""")
        stream.payload("""{"type":"message_delta","delta":{"stop_reason":"end_turn"}}""")
        stream.payload("""{"type":"message_stop"}""")
        assertEquals("完整方案", stream.finish())
    }

    @Test fun incompleteJsonRepliesKeepTheirContentBeforeFailure() {
        for ((protocol, body) in listOf(
            "anthropic" to """{"stop_reason":"max_tokens","content":[{"type":"text","text":"部分内容"}]}""",
            "responses" to """{"status":"incomplete","output":[{"type":"message","content":[{"type":"output_text","text":"部分内容"}]}]}""",
        )) {
            var text = ""
            assertEquals("response_truncated", assertFailsWith<ApiException> {
                customSingleText(body, protocol) { text += it }
            }.code)
            assertEquals("部分内容", text)
        }
    }

    @Test fun incompleteResponsesSnapshotRetainsOnlyTheMissingSuffix() {
        val text = StringBuilder()
        val stream = CustomPlainStream("responses") { text.append(it) }
        stream.payload("""{"type":"response.output_text.delta","delta":"部分"}""")
        assertEquals("response_truncated", assertFailsWith<ApiException> {
            stream.payload("""{"type":"response.incomplete","response":{"status":"incomplete","output":[{"type":"message","content":[{"type":"output_text","text":"部分内容"}]}]}}""")
        }.code)
        assertEquals("部分内容", text.toString())
    }

    @Test fun failedAndInProgressJsonAreNotSuccessfulReplies() {
        for (status in listOf("failed", "cancelled", "in_progress")) {
            assertEquals("stream_interrupted", assertFailsWith<ApiException> {
                customSingleText("""{"status":"$status","output":[]}""", "responses")
            }.code)
        }
    }
}
