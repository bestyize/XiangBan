package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.content.TextContent
import io.ktor.http.headersOf
import io.ktor.http.encodedPath
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.awaitCancellation
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeout
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue

private class AgentAwareFiles : ChatHistoryFiles {
    val texts = mutableMapOf<String, String>()
    override fun read(name: String): String? = texts[name]
    override fun write(name: String, text: String) { texts[name] = text }
    override fun delete(name: String) { texts.remove(name) }
    override fun agentCopy(id: String): String = "agent/s-$id.jsonl"
    override fun agentProgressCopy(id: String): String = "agent/progress/s-$id.jsonl"
}

private suspend fun <T> StateFlow<T>.await(predicate: (T) -> Boolean): T =
    withContext(Dispatchers.Default) { withTimeout(5_000) { first(predicate) } }

class ChatHistoryTest {
    private fun scope() = CoroutineScope(Dispatchers.Unconfined)

    /** 写文件在后台线程，等它落盘。 */
    private suspend fun eventually(condition: () -> Boolean) = withContext(Dispatchers.Default) {
        withTimeout(5_000) { while (!condition()) delay(10) }
    }

    private fun gateway(bodies: MutableList<String>): AiGatewayApi {
        var n = 0
        val client = HttpClient(MockEngine { request ->
            bodies += (request.body as TextContent).text
            n++
            respond("data: {\"choices\":[{\"delta\":{\"content\":\"回答$n\"}}]}\n\ndata: [DONE]\n", HttpStatusCode.OK, headersOf(HttpHeaders.ContentType, "text/event-stream"))
        })
        return AiGatewayApi(client, CommunityConfig()) { "tok" }
    }

    @Test
    fun `workspace survives replies and account switching and deletion keeps chats`() {
        val store = ChatHistory(InMemoryChatHistoryFiles(), scope())
        store.useAccount("a")
        store.addWorkspace("工作")
        val turns = listOf(ChatTurn(id = 1, role = "user", text = "你好"))
        store.save("one", turns, 1)
        store.move("one", "工作")
        store.save("one", turns, 2)
        assertEquals("工作", store.list().single().workspace)
        store.useAccount("b")
        assertTrue(store.workspaces().isEmpty())
        store.useAccount("a")
        assertEquals(listOf("工作"), store.workspaces())
        store.renameWorkspace("工作", "学习")
        assertEquals("学习", store.list().single().workspace)
        store.deleteWorkspace("学习")
        assertNull(store.list().single().workspace)
        assertEquals(turns, store.load("one"))
    }

    @Test
    fun `new conversation ids do not repeat across launches`() {
        val a = newChatId(1_700_000_000_000, Random(1))
        val b = newChatId(1_700_000_000_000, Random(2))
        val later = newChatId(1_700_000_000_001, Random(1))
        assertNotEquals(a, b)
        assertNotEquals(a, later)
        assertTrue(a.all { it in 'a'..'z' || it in '0'..'9' }, a)
    }

    @Test
    fun `conversations are kept, listed newest first and reopened after a restart`() = runTest {
        val files = AgentAwareFiles()
        val bodies = mutableListOf<String>()
        val api = gateway(bodies)
        var now = 1_000L
        val first = CloudChatController(api, scope(), history = ChatHistory(files, scope()), clock = { now })
        first.useAccount("u1")

        first.send("第一个问题")
        first.state.await { !it.sending && it.messages.lastOrNull()?.text == "回答1" }
        val firstId = first.state.value.conversationId
        now = 2_000L
        first.clearConversation()
        first.send("第二个问题")
        first.state.await { !it.sending && it.messages.lastOrNull()?.text == "回答2" }
        assertEquals(listOf("第二个问题", "第一个问题"), first.state.value.conversations.map { it.title })
        eventually { files.texts["chats/u1/index.json"]?.contains("第一个问题") == true && files.texts["chats/u1/$firstId.json"]?.contains("回答1") == true }

        // 重新打开应用：列表还在，点开旧对话接着聊，模型能看到之前的内容。
        val again = CloudChatController(api, scope(), history = ChatHistory(files, scope()), clock = { now })
        again.useAccount("u1")
        assertEquals(listOf("第二个问题", "第一个问题"), again.state.value.conversations.map { it.title })
        again.openConversation(firstId)
        assertEquals(firstId, again.state.value.conversationId)
        assertEquals(listOf("第一个问题", "回答1"), again.state.value.messages.map { it.text })
        now = 3_000L
        again.send("接着问")
        again.state.await { !it.sending && it.messages.size == 4 }
        assertTrue(bodies.last().contains("第一个问题") && bodies.last().contains("回答1"), bodies.last())
        assertEquals(firstId, again.state.value.conversations.first().id)
        // 消息编号接着往后排，不和旧的撞上。
        assertEquals(again.state.value.messages.size, again.state.value.messages.map { it.id }.toSet().size)

        // 别的账号看不到这些对话。
        again.useAccount("u2")
        assertTrue(again.state.value.conversations.isEmpty())
    }

    @Test
    fun `deletion waits for the agent and ignores late saves`() = runTest {
        val files = AgentAwareFiles()
        val stopped = CompletableDeferred<Unit>()
        val history = ChatHistory(files, scope()) { stopped.await() }.apply { useAccount("owner") }
        history.save("one", listOf(ChatTurn(1, "user", "hi")), 1)
        files.texts["agent/s-one.jsonl"] = "active"
        history.delete("one")
        assertEquals("chats/owner", files.texts["chats/owner/deleted/one"])
        assertEquals("active", files.texts["agent/s-one.jsonl"])
        assertTrue(history.save("one", listOf(ChatTurn(2, "user", "late")), 2).isEmpty())
        assertNull(history.load("one"))
        stopped.complete(Unit)
        eventually { "agent/s-one.jsonl" !in files.texts }
    }

    @Test
    fun `deleting a conversation removes its files and the agent's copy`() = runTest {
        val files = AgentAwareFiles()
        val chat = CloudChatController(gateway(mutableListOf()), scope(), history = ChatHistory(files, scope()), clock = { 1L })
        chat.useAccount("u1")
        chat.send("要删的")
        chat.state.await { !it.sending && it.messages.size == 2 }
        val id = chat.state.value.conversationId
        files.texts["agent/s-$id.jsonl"] = "{}"
        files.texts["agent/progress/s-$id.jsonl"] = "{\"handoff\":\"恢复这一步\"}"
        eventually { "chats/u1/$id.json" in files.texts }

        chat.deleteConversation(id)
        assertTrue(chat.state.value.conversations.isEmpty())
        assertTrue(chat.state.value.messages.isEmpty())
        assertNotEquals(id, chat.state.value.conversationId)
        eventually { "chats/u1/$id.json" !in files.texts && "agent/s-$id.jsonl" !in files.texts && "agent/progress/s-$id.jsonl" !in files.texts }
        assertNull(ChatHistory(files, scope()).apply { useAccount("u1") }.load(id))
    }

    @Test
    fun `restart restores the interrupted response and scoped media`() = runTest {
        val files = InMemoryChatHistoryFiles()
        val history = ChatHistory(files, scope()).apply { useAccount("owner") }
        val image = "data:image/png;base64,iVBORw0KGgo="
        val user = ChatTurn(1, "user", "你好", attachments = listOf(ChatAttachment("画面.png", imageDataUrl = image)))
        val reply = ChatTurn(2, "assistant", "", streaming = true)
        history.save("one", listOf(user, reply), 1)
        history.checkpoint("one", reply.copy(text = "已收到部分文字"))
        eventually { files.read("chats/owner/one.stream.json") != null && files.read("chats/owner/one.json") != null }
        val stored = checkNotNull(files.read("chats/owner/one.json"))
        assertTrue("media:" in stored && image !in stored)
        val restarted = ChatHistory(files, scope()).apply { useAccount("owner") }
        val messages = checkNotNull(restarted.load("one"))
        assertEquals("已收到部分文字", messages.last().text)
        assertEquals(INTERRUPTED_REPLY, messages.last().error)
        val reference = checkNotNull(messages.first().attachments.single().imageDataUrl)
        assertEquals(image, restarted.mediaFor("one").resolve(reference))
        restarted.useAccount("other")
        assertNull(restarted.mediaFor("one").resolve(reference))
        restarted.useAccount("owner")
        restarted.delete("one")
        eventually { files.read("chats/owner/one.stream.json") == null }
        assertNull(restarted.mediaFor("one").resolve(reference))
    }

    @Test
    fun `image recovery after a lost response uses the original job and never posts again`() = runTest {
        val files = InMemoryChatHistoryFiles()
        val submitted = CompletableDeferred<String>()
        var postCount = 0
        val client = HttpClient(MockEngine { request ->
            val path = request.url.encodedPath
            val response = when {
                path == "/v1/images/jobs" -> {
                    postCount++
                    val body = (request.body as TextContent).text
                    val id = Regex("""["]job_id["]\s*:\s*["]([^"]+)["]""").find(body)!!.groupValues[1]
                    assertTrue(files.read("chats/u1/index.json").orEmpty().contains("画一只猫"))
                    submitted.complete(id)
                    awaitCancellation()
                }
                path.startsWith("/v1/images/jobs/") -> """{"job_id":"${path.substringAfterLast('/')}","status":"succeeded","result":{"data":[{"b64_json":"iVBORw0KGgo="}]}}"""
                else -> """{}"""
            }
            respond(response, HttpStatusCode.OK, headersOf(HttpHeaders.ContentType, "application/json"))
        })
        val api = AiGatewayApi(client, CommunityConfig()) { "tok" }
        val store = ChatHistory(files, scope()).apply { useAccount("u1") }
        val first = CloudChatController(api, scope(), history = store).apply { useAccount("u1") }
        assertTrue(first.generateImages("画一只猫", 1, null))
        val id = withContext(Dispatchers.Default) { withTimeout(5_000) { submitted.await() } }
        val conversation = first.state.value.conversationId
        first.stop()
        eventually { files.read("chats/u1/$conversation.json").orEmpty().contains(id) }
        val again = CloudChatController(api, scope(), history = ChatHistory(files, scope())).apply {
            useAccount("u1")
            openConversation(conversation)
        }
        val reply = again.state.value.messages.last()
        assertEquals(id, reply.imageJob?.id)
        again.recoverImageJob(reply.id)
        val result = again.state.await { !it.sending && it.messages.lastOrNull()?.generatedImages?.size == 1 }
        assertEquals(1, postCount)
        assertEquals(id, result.messages.last().imageJob?.id)
        assertTrue(!result.messages.last().imageJobRecoverable)
        val image = result.messages.last().generatedImages.single()
        assertEquals("data:image/png;base64,iVBORw0KGgo=", again.resolveImage(conversation, image))
    }

    @Test
    fun `an empty new conversation is not saved`() {
        val history = ChatHistory(InMemoryChatHistoryFiles(), scope())
        history.useAccount(null)
        assertTrue(history.save("c1", emptyList(), 1).isEmpty())
        assertTrue(history.save("c1", listOf(ChatTurn(1, "assistant", "你好")), 1).isEmpty())
        assertEquals("问点事", history.save("c1", listOf(ChatTurn(1, "user", "问点事")), 1).single().title)
    }
}
