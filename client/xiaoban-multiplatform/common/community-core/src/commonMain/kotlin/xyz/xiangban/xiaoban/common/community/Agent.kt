package xyz.xiangban.xiaoban.common.community

import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.booleanOrNull
import kotlinx.serialization.json.buildJsonArray
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.put

// ───────────── 手机里的 Agent 内核（libs/agent-mobile） ─────────────
//
// 内核负责“对话循环、调用工具、问用户同不同意”，网络请求和工具都留给这里：
//   内核 → 手机：model_request（请你把这个请求发给 server-ai）、tool_request（请你执行这个工具）、approval_request（请问用户）
//   手机 → 内核：model_line / model_end（模型的流式响应）、tool_result、approval、run、cancel
// 每条都是一行 JSON，细节见 libs/agent-mobile/src/lib.rs 顶部的说明。

/** 平台提供的原生桥：Android 上是 JNI（libagent_mobile.so），没有时界面退回纯云端聊天。 */
interface AgentBridge {
    /** 启动一个引擎，返回句柄；配置不对时抛异常。 */
    fun start(config: String): Long
    fun send(handle: Long, line: String)
    /** 取内核的下一行输出，最多等 [timeoutMs] 毫秒；没有返回 null。 */
    fun poll(handle: Long, timeoutMs: Long): String?
    fun stop(handle: Long)
}

data class AgentToolResult(val ok: Boolean, val output: String)

/** 手机侧实现的一个工具。风险 `read` 的直接执行，其余的先问用户。 */
interface AgentTool {
    val name: String
    val description: String
    /** 参数的 JSON Schema。 */
    val parameters: JsonObject
    val risk: String get() = "write"
    suspend fun run(arguments: JsonObject): AgentToolResult
}

sealed interface AgentEvent {
    data class Text(val run: String, val text: String) : AgentEvent
    data class ToolStart(val run: String, val id: String, val name: String, val arguments: String) : AgentEvent
    data class ToolEnd(val run: String, val id: String, val output: String, val isError: Boolean) : AgentEvent
    /** 内核请用户确认一个会改动东西的工具调用；用 [AgentRuntime.answerApproval] 回答。 */
    data class Approval(val id: String, val tool: String, val risk: String, val arguments: JsonObject, val danger: String?) : AgentEvent
    data class RunEnd(val run: String, val ok: Boolean, val text: String?, val error: String?) : AgentEvent
    /** 引擎自己出了问题（异常退出等）。 */
    data class Failure(val message: String) : AgentEvent
}

private val AgentJson = kotlinx.serialization.json.Json { ignoreUnknownKeys = true }

private fun JsonElement?.str(): String? = this?.takeIf { it !is JsonNull }?.let { runCatching { (it as JsonPrimitive).contentOrNull }.getOrNull() }

/**
 * 管一个内核引擎：启动、把内核的请求交给 [model] 和 [tools] 去办、把事件抛给界面。
 * 不依赖 Compose；在 JVM 测试里可以接真正的内核库。
 */
class AgentRuntime(
    private val bridge: AgentBridge,
    private val model: suspend (body: String, onLine: suspend (String) -> Unit) -> ModelResult,
    private val tools: List<AgentTool>,
    private val sessionsDir: String,
    private val systemPrompt: String,
    private val scope: CoroutineScope,
) {
    private val _events = MutableSharedFlow<AgentEvent>(extraBufferCapacity = 256)
    val events: SharedFlow<AgentEvent> = _events.asSharedFlow()

    private val lock = Mutex()
    private var handle = 0L
    private var pump: Job? = null
    private var currentModel = ""
    private val requests = HashMap<String, Job>()
    private val requestsLock = Mutex()
    private var nextRun = 0L

    private fun line(block: kotlinx.serialization.json.JsonObjectBuilder.() -> Unit): String = buildJsonObject(block).toString()

    private fun send(text: String) {
        val h = handle
        if (h != 0L) runCatching { bridge.send(h, text) }
    }

    /** 引擎没在跑就启动它，已经在跑就只换模型。 */
    suspend fun ensureStarted(modelName: String) = lock.withLock {
        if (handle == 0L) {
            val config = buildJsonObject {
                put("model", modelName)
                put("sessions_dir", sessionsDir)
                put("system_prompt", systemPrompt)
                put("tools", buildJsonArray {
                    for (t in tools) add(buildJsonObject {
                        put("name", t.name)
                        put("description", t.description)
                        put("parameters", t.parameters)
                        put("risk", t.risk)
                    })
                })
            }.toString()
            handle = bridge.start(config)
            currentModel = modelName
            val started = handle
            pump = scope.launch(Dispatchers.Default) { pumpLoop(started) }
        } else if (modelName != currentModel) {
            currentModel = modelName
            send(line { put("type", "set_model"); put("model", modelName) })
        }
    }

    /** 在会话 [session] 里开始一轮；返回这一轮的编号，用来取消。 */
    fun nextRunId(): String = "r${++nextRun}"

    fun run(session: String, text: String, id: String = nextRunId()): String {
        send(line { put("type", "run"); put("id", id); put("session", session); put("input", text) })
        return id
    }

    fun cancel(run: String) = send(line { put("type", "cancel"); put("id", run) })

    /** `decision`：once（允许这一次）、always（这次对话里一直允许）、deny（拒绝）。 */
    fun answerApproval(id: String, decision: String) = send(line { put("type", "approval"); put("id", id); put("decision", decision) })

    suspend fun stop() = lock.withLock {
        val h = handle
        handle = 0L
        pump?.cancel()
        pump = null
        requestsLock.withLock {
            requests.values.forEach { it.cancel() }
            requests.clear()
        }
        if (h != 0L) runCatching { bridge.stop(h) }
    }

    private suspend fun pumpLoop(h: Long) {
        try {
            while (scope.isActive) {
                val text = withBlocking { bridge.poll(h, 400) } ?: continue
                val obj = runCatching { AgentJson.parseToJsonElement(text).jsonObject }.getOrNull() ?: continue
                if (obj["type"].str() == "closed") break
                dispatch(obj)
            }
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            _events.emit(AgentEvent.Failure(e.message ?: "智能体引擎出错"))
        }
    }

    private suspend fun <T> withBlocking(block: () -> T): T = kotlinx.coroutines.withContext(Dispatchers.IO) { block() }

    private suspend fun dispatch(obj: JsonObject) {
        when (obj["type"].str()) {
            "model_request" -> {
                val id = obj["id"].str() ?: return
                val body = obj["body"].str() ?: return
                val h = handle
                val job = scope.launch(Dispatchers.Default) {
                    val result = try {
                        model(body) { l -> if (h != 0L) bridge.send(h, line { put("type", "model_line"); put("id", id); put("line", l) }) }
                    } catch (e: CancellationException) {
                        throw e
                    } catch (e: Exception) {
                        ModelResult(0, error = e.message ?: "请求失败")
                    }
                    requestsLock.withLock { requests.remove(id) }
                    if (h != 0L) runCatching {
                        bridge.send(h, line {
                            put("type", "model_end"); put("id", id); put("status", result.status)
                            result.error?.let { put("error", it) }
                            result.body?.let { put("body", it) }
                        })
                    }
                }
                requestsLock.withLock { requests[id] = job }
            }
            "model_cancel" -> obj["id"].str()?.let { id -> requestsLock.withLock { requests.remove(id) }?.cancel() }
            "tool_request" -> {
                val id = obj["id"].str() ?: return
                val name = obj["name"].str().orEmpty()
                val args = obj["arguments"] as? JsonObject ?: JsonObject(emptyMap())
                scope.launch(Dispatchers.Default) {
                    val result = try {
                        tools.firstOrNull { it.name == name }?.run(args) ?: AgentToolResult(false, "没有这个工具：$name")
                    } catch (e: CancellationException) {
                        throw e
                    } catch (e: Exception) {
                        AgentToolResult(false, e.message ?: "工具执行失败")
                    }
                    send(line { put("type", "tool_result"); put("id", id); put("ok", result.ok); put("output", result.output) })
                }
            }
            "approval_request" -> _events.tryEmit(
                AgentEvent.Approval(
                    id = obj["id"].str().orEmpty(), tool = obj["tool"].str().orEmpty(), risk = obj["risk"].str() ?: "write",
                    arguments = obj["arguments"] as? JsonObject ?: JsonObject(emptyMap()), danger = obj["danger"].str(),
                ),
            )
            "run_end" -> _events.tryEmit(
                AgentEvent.RunEnd(obj["run"].str().orEmpty(), obj["ok"].let { (it as? JsonPrimitive)?.booleanOrNull } == true, obj["text"].str(), obj["error"].str()),
            )
            "event" -> {
                val run = obj["run"].str().orEmpty()
                val ev = obj["event"] as? JsonObject ?: return
                when (ev["type"].str()) {
                    "text_delta" -> ev["text"].str()?.let { _events.tryEmit(AgentEvent.Text(run, it)) }
                    "tool_start" -> _events.tryEmit(AgentEvent.ToolStart(run, ev["id"].str().orEmpty(), ev["name"].str().orEmpty(), ev["arguments"].str().orEmpty()))
                    "tool_end" -> _events.tryEmit(
                        AgentEvent.ToolEnd(run, ev["id"].str().orEmpty(), ev["output"].str().orEmpty(), (ev["is_error"] as? JsonPrimitive)?.booleanOrNull == true),
                    )
                }
            }
            "error" -> _events.tryEmit(AgentEvent.Failure(obj["message"].str() ?: "智能体引擎出错"))
        }
    }
}
