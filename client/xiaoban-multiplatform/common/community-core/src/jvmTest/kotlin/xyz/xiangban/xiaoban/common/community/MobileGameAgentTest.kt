package xyz.xiangban.xiaoban.common.community

import kotlinx.coroutines.*
import kotlinx.serialization.json.*
import kotlin.test.*

/** Real JNI harness + the actual phone tool schemas and implementations, without a paid model call. */
class MobileGameAgentTest {
    @Test
    fun `the phone agent writes a draft and installs a desktop compatible game after approval`() = runBlocking {
        if (System.getenv("AGENT_MOBILE_LIB").isNullOrBlank()) return@runBlocking
        val bridge = object : AgentBridge {
            override fun start(config: String) = AgentNative.nativeStart(config)
            override fun send(handle: Long, line: String) = AgentNative.nativeSend(handle, line)
            override fun poll(handle: Long, timeoutMs: Long) = AgentNative.nativePoll(handle, timeoutMs)
            override fun stop(handle: Long) = AgentNative.nativeStop(handle)
        }
        val workspace = MemoryMobileWorkspace()
        val store = MobileGameStore(workspace)
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        val calls = listOf(
            "game_read_format" to buildJsonObject {},
            "write_file" to buildJsonObject { put("path", "drafts/garden/index.html"); put("content", "<!doctype html><button>开始游戏</button>") },
            "game_create" to buildJsonObject { put("id", "touch-garden"); put("name", "触控花园"); put("description", "点击种花"); put("source_dir", "drafts/garden"); put("orientation", "landscape") },
            "game_validate" to buildJsonObject { put("id", "touch-garden") },
        )
        var step = 0
        val model: suspend (String, suspend (String) -> Unit) -> ModelResult = { body, line ->
            assertTrue(body.contains("game_create"), "actual tool schemas must reach the model")
            val call = calls.getOrNull(step++)
            val delta = buildJsonObject {
                if (call == null) put("content", "已制作游戏")
                else put("tool_calls", buildJsonArray {
                    add(buildJsonObject {
                        put("index", 0); put("id", "call$step"); put("type", "function")
                        putJsonObject("function") { put("name", call.first); put("arguments", call.second.toString()) }
                    })
                })
            }
            val payload = buildJsonObject { put("choices", buildJsonArray {
                add(buildJsonObject { put("index", 0); put("delta", delta); put("finish_reason", if (call == null) "stop" else "tool_calls") })
            }) }
            line("data: $payload"); line(""); line("data: [DONE]"); line("")
            ModelResult(200)
        }
        val directory = java.nio.file.Files.createTempDirectory("mobile-game-agent").toFile()
        val runtime = AgentRuntime(bridge, model, mobileWorkspaceTools(workspace) + mobileGameTools(store), directory.path, "制作兼容游戏", scope)
        val ended = CompletableDeferred<AgentEvent.RunEnd>()
        val approvals = mutableListOf<String>()
        val errors = mutableListOf<String>()
        scope.launch(start = CoroutineStart.UNDISPATCHED) {
            runtime.events.collect { event -> when (event) {
                is AgentEvent.Approval -> { approvals += event.tool; runtime.answerApproval(event.id, "once") }
                is AgentEvent.ToolEnd -> if (event.isError) errors += event.output
                is AgentEvent.RunEnd -> ended.complete(event)
                else -> Unit
            } }
        }
        try {
            runtime.ensureStarted("standard")
            runtime.run("game-session", "制作一款触控游戏")
            val result = withTimeout(10_000) { ended.await() }
            assertTrue(result.ok, result.error)
            assertTrue(errors.isEmpty(), errors.joinToString())
            assertEquals(listOf("write_file", "game_create"), approvals)
            val files = store.exportFiles("touch-garden")
            val manifest = Json.parseToJsonElement(files.single { it.path == "manifest.json" }.content).jsonObject
            assertEquals(1, manifest.getValue("schemaVersion").jsonPrimitive.int)
            assertEquals("game", manifest.getValue("kind").jsonPrimitive.content)
            assertEquals("landscape", manifest.getValue("orientation").jsonPrimitive.content)
            assertEquals(listOf("index.html", "manifest.json"), files.map { it.path }.sorted())
        } finally { runtime.stop(); scope.cancel(); directory.deleteRecursively() }
    }
}
