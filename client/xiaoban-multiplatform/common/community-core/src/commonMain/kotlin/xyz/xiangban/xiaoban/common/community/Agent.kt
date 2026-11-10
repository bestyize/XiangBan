package xyz.xiangban.xiaoban.common.community

import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.IO
import kotlin.concurrent.Volatile
import kotlinx.coroutines.Job
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.isActive
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
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
    val timeoutSeconds: Long get() = 60
    val alwaysAsk: Boolean get() = false
    val untrustedOutput: Boolean get() = false
    suspend fun run(arguments: JsonObject): AgentToolResult
}

sealed interface AgentEvent {
    data class Text(val run: String, val text: String) : AgentEvent
    data class ToolStart(val run: String, val id: String, val name: String, val arguments: String) : AgentEvent
    data class ToolEnd(val run: String, val id: String, val output: String, val isError: Boolean) : AgentEvent
    /** 内核请用户确认一个会改动东西的工具调用；用 [AgentRuntime.answerApproval] 回答。 */
    data class Approval(val id: String, val tool: String, val risk: String, val arguments: JsonObject, val danger: String?, val session: String = "", val run: String = "", val canRemember: Boolean = true) : AgentEvent
    data class Question(val id: String, val session: String, val question: String, val options: List<String>, val run: String = "") : AgentEvent
    data class RunEnd(val run: String, val ok: Boolean, val text: String?, val error: String?, val finish: String? = null, val promptTokens: Long = 0, val completionTokens: Long = 0) : AgentEvent
    data class Progress(val run: String, val kind: String, val detail: JsonObject) : AgentEvent
    /** 引擎自己出了问题（异常退出等）。 */
    data class Failure(val message: String, val runs: Set<String>? = null) : AgentEvent
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
    private val utcOffsetMinutes: Int = 0,
) {
    private val _events = MutableSharedFlow<AgentEvent>(extraBufferCapacity = 256)
    val events: SharedFlow<AgentEvent> = _events.asSharedFlow()

    private val lock = Mutex()
    @Volatile private var handle = 0L
    @Volatile private var generation = 0L
    private var pump: Job? = null
    private var workers: Job? = null
    private var currentModel = ""
    @Volatile private var currentReasoning: String? = null
    private val requests = MutableStateFlow<Map<Pair<Long, String>, Job>>(emptyMap())
    private val mutationRequests = MutableStateFlow<Set<Pair<Long, String>>>(emptySet())
    private val requestRuns = MutableStateFlow<Map<Pair<Long, String>, String>>(emptyMap())
    private var nextRun = 0L
    private data class ActiveRun(val id: String, val generation: Long, val cancelled: Boolean = false, val options: ChatOptions = ChatOptions())
    private val activeRuns = MutableStateFlow<Map<String, ActiveRun>>(emptyMap())

    private fun line(block: kotlinx.serialization.json.JsonObjectBuilder.() -> Unit): String = buildJsonObject(block).toString()

    private fun send(text: String) {
        val h = handle
        check(h != 0L) { "智能体引擎未启动" }
        bridge.send(h, text)
    }

    /** 引擎没在跑就启动它，已经在跑就只换模型。 */
    suspend fun ensureStarted(modelName: String, reasoningEffort: String? = null) = lock.withLock {
        currentReasoning = reasoningEffort
        check(scope.isActive) { "智能体运行环境已关闭" }
        if (handle == 0L) {
            val config = buildJsonObject {
                put("model", modelName)
                put("sessions_dir", sessionsDir)
                put("system_prompt", systemPrompt)
                put("utc_offset_minutes", utcOffsetMinutes)
                put("tools", buildJsonArray {
                    for (t in tools) add(buildJsonObject {
                        put("name", t.name)
                        put("description", t.description)
                        put("parameters", t.parameters)
                        put("risk", t.risk)
                        put("timeout_secs", t.timeoutSeconds)
                        put("always_ask", t.alwaysAsk)
                        put("untrusted_output", t.untrustedOutput)
                    })
                })
            }.toString()
            handle = bridge.start(config).also { check(it != 0L) { "智能体引擎启动失败" } }
            currentModel = modelName
            val started = handle
            val epoch = ++generation
            val job = SupervisorJob(scope.coroutineContext[Job])
            workers = job
            val requestScope = CoroutineScope(scope.coroutineContext + job)
            pump = scope.launch(Dispatchers.Default, start = CoroutineStart.UNDISPATCHED) { pumpLoop(started, epoch, requestScope) }
        } else if (modelName != currentModel) {
            send(line { put("type", "set_model"); put("model", modelName) })
            currentModel = modelName
        }
    }

    /** 在会话 [session] 里开始一轮；返回这一轮的编号，用来取消。 */
    fun nextRunId(): String = "r${++nextRun}"

    fun run(session: String, text: String, id: String = nextRunId(), attachments: List<ChatAttachment> = emptyList(), options: ChatOptions? = null, tier: AiTier? = null, retry: Boolean = false): String {
        validateChatAttachments(attachments)
        check(!hasStoppingMutation()) { "上一项本机操作仍在停止，请稍后核对结果再继续" }
        val snapshot = options ?: ChatOptions(tier = currentModel, reasoningByTier = currentReasoning?.let { mapOf(currentModel to it) }.orEmpty())
        val run = ActiveRun(id, generation, options = snapshot)
        activeRuns.update {
            check(session !in it) { "上一轮仍在停止，请稍后重试" }
            it + (session to run)
        }
        try {
            configureSession(session, snapshot, tier)
            send(line {
                put("type", "run"); put("id", id); put("session", session); put("isolate", true)
                if (retry) put("retry", true)
                else if (attachments.isEmpty()) put("input", text) else put("parts", attachmentParts(text, attachments))
                snapshot.budget.maxToolCalls?.let { put("max_tool_calls", it) }
                snapshot.budget.maxMinutes?.let { put("max_millis", it * 60_000) }
                snapshot.budget.maxTokens?.let { put("max_tokens", it) }
            })
        } catch (e: Exception) {
            activeRuns.update { if (it[session] == run) it - session else it }
            throw e
        }
        return id
    }

    fun cancel(run: String) {
        activeRuns.update { running -> running.mapValues { (_, value) -> if (value.id == run) value.copy(cancelled = true) else value } }
        runCatching { send(line { put("type", "cancel"); put("id", run) }) }
        requestRuns.value.filterValues { it == run }.keys.forEach { requests.value[it]?.cancel() }
    }

    fun configureSession(session: String, options: ChatOptions, tier: AiTier? = null) {
        options.validate()
        if (handle == 0L) return
        send(line {
            put("type", "configure_session"); put("session", session)
            put("settings", buildJsonObject {
                put("model", options.tier); put("context_tokens", options.contextTokens)
                tier?.contextWindowTokens?.takeIf { it > 0 }?.let { put("model_window", it) }
                tier?.maxOutputTokens?.takeIf { it > 0 }?.let { put("max_output_tokens", it) }
                put("supports_tools", tier?.supportsTools ?: true)
                options.budget.maxTurns?.let { put("max_turns", it) }
                put("permission", options.permissionMode)
                put("grants", JsonArray(options.grants.map(::JsonPrimitive)))
            })
        })
        if (session in reviewed.value) {
            send(line { put("type", "reviewed_session"); put("session", session) })
            reviewed.update { it - session }
        }
    }

    private val reviewed = MutableStateFlow<Set<String>>(emptySet())

    private fun hasStoppingMutation(): Boolean = mutationRequests.value.any { key ->
        val run = requestRuns.value[key]
        run != null && activeRuns.value.values.none { it.id == run && !it.cancelled }
    }

    fun acknowledgeOutcome(session: String) {
        check(!hasStoppingMutation()) { "本机操作仍在停止，请稍后核对实际结果" }
        reviewed.update { it + session }
        if (handle != 0L) { send(line { put("type", "reviewed_session"); put("session", session) }); reviewed.update { it - session } }
    }

    /** `decision`：once（允许这一次）、always（这次对话里一直允许）、deny（拒绝）。 */
    fun answerApproval(id: String, decision: String) {
        val rawId = currentQuestionId(id) ?: return
        send(line { put("type", "approval"); put("id", rawId); put("decision", decision) })
    }

    fun answerQuestion(id: String, answer: String?) {
        val rawId = currentQuestionId(id) ?: return
        send(line { put("type", "answer"); put("id", rawId); put("answer", answer?.let(::JsonPrimitive) ?: JsonNull) })
    }

    // UI IDs are opaque and scoped to an engine lifetime; q1/a1 may be reused after restart.
    private fun currentQuestionId(id: String): String? = when {
        handle == 0L -> null
        ':' !in id -> id // Compatibility with hosts that retained a raw protocol ID.
        id.substringBefore(':').toLongOrNull() == generation -> id.substringAfter(':')
        else -> null
    }

    suspend fun stop() = lock.withLock {
        val h = handle
        handle = 0L
        ++generation
        activeRuns.value = emptyMap()
        pump?.cancel()
        pump = null
        workers?.cancel()
        workers = null
        requests.value.values.forEach { it.cancel() }
        requests.value = emptyMap()
        if (h != 0L) withBlocking { runCatching { bridge.stop(h) } }
    }

    private suspend fun pumpLoop(h: Long, epoch: Long, requestScope: CoroutineScope) {
        try {
            while (kotlinx.coroutines.currentCoroutineContext().isActive && handle == h && generation == epoch) {
                val text = withBlocking { bridge.poll(h, 400) } ?: continue
                val obj = runCatching { AgentJson.parseToJsonElement(text).jsonObject }.getOrNull() ?: continue
                if (obj["type"].str() == "closed") throw IllegalStateException("智能体引擎意外退出，请重试")
                dispatch(obj, h, epoch, requestScope)
            }
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            val failedRuns = lock.withLock {
                if (handle == h && generation == epoch) {
                    val affected = activeRuns.value.values.map { it.id }.toSet()
                    handle = 0L
                    ++generation
                    activeRuns.value = emptyMap()
                    pump = null
                    workers?.cancel()
                    workers = null
                    requests.value.values.forEach { it.cancel() }; requests.value = emptyMap()
                    withBlocking { runCatching { bridge.stop(h) } }
                    affected
                } else null
            }
            if (failedRuns != null) _events.emit(AgentEvent.Failure(e.message ?: "智能体引擎出错", failedRuns))
        } finally {
            // Parent-scope cancellation must also release the native engine and its pending calls.
            withContext(NonCancellable) {
                lock.withLock {
                    if (handle == h && generation == epoch) {
                        handle = 0L
                        ++generation
                        activeRuns.value = emptyMap()
                        workers?.cancel()
                        workers = null
                        requests.value.values.forEach { it.cancel() }; requests.value = emptyMap()
                        withBlocking { runCatching { bridge.stop(h) } }
                    }
                }
            }
        }
    }

    private suspend fun <T> withBlocking(block: () -> T): T = kotlinx.coroutines.withContext(Dispatchers.IO) { block() }

    private fun sendFor(h: Long, epoch: Long, text: String) {
        if (handle == h && generation == epoch) bridge.send(h, text)
    }

    private fun pendingRun(session: String, epoch: Long): ActiveRun? {
        val running = activeRuns.value
        val run = if (session.isEmpty()) running.values.filter { it.generation == epoch && !it.cancelled }.singleOrNull() else running[session]
        return run?.takeIf { it.generation == epoch && !it.cancelled }
    }

    private fun request(id: String, epoch: Long, requestScope: CoroutineScope, run: String? = null, mutation: Boolean = false, onSkipped: (() -> Unit)? = null, action: suspend () -> Unit) {
        val key = epoch to id
        val began = MutableStateFlow(false)
        val job = requestScope.launch(Dispatchers.Default, start = CoroutineStart.LAZY) { began.value = true; action() }
        if (run != null) requestRuns.update { it + (key to run) }
        if (mutation) mutationRequests.update { it + key }
        requests.update { it + (key to job) }
        // Also runs for a LAZY coroutine cancelled before its body begins.
        job.invokeOnCompletion {
            requests.update { if (it[key] == job) it - key else it }
            mutationRequests.update { it - key }
            requestRuns.update { it - key }
            if (!began.value) runCatching { onSkipped?.invoke() }
        }
        job.start()
    }

    private suspend fun dispatch(obj: JsonObject, h: Long, epoch: Long, requestScope: CoroutineScope) {
        if (handle != h || generation != epoch) return
        when (obj["type"].str()) {
            "model_request" -> {
                val id = obj["id"].str() ?: return
                val running = pendingRun(obj["session"].str().orEmpty(), epoch)
                val body = withReasoningEffort(obj["body"].str() ?: return, if (running != null) running.options.let { it.reasoningByTier[it.tier] } else currentReasoning)
                val affected = activeRuns.value.values.map { it.id }.toSet()
                request(id, epoch, requestScope, running?.id) {
                    val result = try {
                        if (running == null || pendingRun(obj["session"].str().orEmpty(), epoch)?.id != running.id) throw CancellationException("run cancelled")
                        model(body) { l -> sendFor(h, epoch, line { put("type", "model_line"); put("id", id); put("line", l) }) }
                    } catch (e: CancellationException) {
                        throw e
                    } catch (e: Exception) {
                        ModelResult(0, error = e.message ?: "请求失败")
                    }
                    try {
                        sendFor(h, epoch, line {
                            put("type", "model_end"); put("id", id); put("status", result.status)
                            result.error?.let { put("error", it) }
                            result.body?.let { put("body", it) }
                            put("verified_tool_terminal", result.verifiedToolTerminal)
                        })
                    } catch (e: Exception) {
                        if (handle == h && generation == epoch) _events.emit(AgentEvent.Failure(e.message ?: "向智能体发送模型结果失败", affected))
                    }
                }
            }
            "model_cancel", "tool_cancel" -> obj["id"].str()?.let { id -> requests.value[epoch to id]?.cancel() }
            "tool_request" -> {
                val id = obj["id"].str() ?: return
                val name = obj["name"].str().orEmpty()
                val args = obj["arguments"] as? JsonObject ?: JsonObject(emptyMap())
                val session = obj["session"].str().orEmpty()
                val running = pendingRun(session, epoch)
                val tool = tools.firstOrNull { it.name == name }
                request(id, epoch, requestScope, running?.id, mutation = tool?.risk != "read", onSkipped = {
                    sendFor(h, epoch, line { put("type", "tool_result"); put("id", id); put("ok", false); put("cancelled", true); put("outcome_unknown", false); put("output", "已取消，工具未开始执行") })
                }) {
                    var entered = false
                    val result = try {
                        currentCoroutineContext().ensureActive()
                        if (running == null || pendingRun(session, epoch)?.id != running.id) throw CancellationException("run cancelled")
                        entered = true
                        tool?.run(args) ?: AgentToolResult(false, "没有这个工具：$name")
                    } catch (e: CancellationException) {
                        withContext(NonCancellable) {
                            sendFor(h, epoch, line {
                                put("type", "tool_result"); put("id", id); put("ok", false); put("cancelled", true)
                                put("outcome_unknown", entered && tool?.risk != "read")
                                put("output", if (entered && tool?.risk != "read") "操作被中断，结果需要核对" else "已取消，未确认执行副作用")
                            })
                        }
                        return@request
                    } catch (e: Exception) { AgentToolResult(false, e.message ?: "工具执行失败") }
                    // If the operation completed, report its actual outcome even if cancellation raced the reply.
                    withContext(NonCancellable) {
                        sendFor(h, epoch, line { put("type", "tool_result"); put("id", id); put("ok", result.ok); put("output", result.output) })
                    }
                }
            }
            "approval_request" -> {
                val session = obj["session"].str().orEmpty()
                val run = pendingRun(session, epoch) ?: return
                _events.emit(
                AgentEvent.Approval(
                    id = "$epoch:${obj["id"].str().orEmpty()}", tool = obj["tool"].str().orEmpty(), risk = obj["risk"].str() ?: "write",
                    arguments = obj["arguments"] as? JsonObject ?: JsonObject(emptyMap()), danger = obj["danger"].str(),
                    session = session, run = run.id, canRemember = (obj["can_remember"] as? JsonPrimitive)?.booleanOrNull ?: (obj["danger"].str() == null),
                ),
                )
            }
            "approval_resolved" -> {
                val session = obj["session"].str().orEmpty()
                val run = activeRuns.value[session] ?: return
                _events.emit(AgentEvent.Progress(run.id, "approval_resolved", buildJsonObject { put("id", "$epoch:${obj["id"].str().orEmpty()}") }))
            }
            "question" -> {
                val session = obj["session"].str().orEmpty()
                val run = pendingRun(session, epoch) ?: return
                _events.emit(AgentEvent.Question(
                    "$epoch:${obj["id"].str().orEmpty()}", session, obj["question"].str().orEmpty(),
                    (obj["options"] as? JsonArray).orEmpty().mapNotNull { it.str() }, run.id,
                ))
            }
            "run_end" -> {
                val run = obj["run"].str().orEmpty()
                activeRuns.update { running -> running.filterValues { it.id != run || it.generation != epoch } }
                _events.emit(
                AgentEvent.RunEnd(obj["run"].str().orEmpty(), obj["ok"].let { (it as? JsonPrimitive)?.booleanOrNull } == true, obj["text"].str(), obj["error"].str(), obj["finish"].str(), obj["prompt_tokens"].str()?.toLongOrNull() ?: 0, obj["completion_tokens"].str()?.toLongOrNull() ?: 0),
                )
            }
            "event" -> {
                val run = obj["run"].str().orEmpty()
                val ev = obj["event"] as? JsonObject ?: return
                when (ev["type"].str()) {
                    "text_delta" -> ev["text"].str()?.let { _events.emit(AgentEvent.Text(run, it)) }
                    "tool_start" -> _events.emit(AgentEvent.ToolStart(run, ev["id"].str().orEmpty(), ev["name"].str().orEmpty(), ev["arguments"].str().orEmpty()))
                    "reasoning_delta", "tool_input", "tool_waiting", "usage", "retrying", "truncated", "continuing" -> _events.emit(AgentEvent.Progress(run, ev["type"].str().orEmpty(), ev))
                    "tool_end" -> _events.emit(
                        AgentEvent.ToolEnd(run, ev["id"].str().orEmpty(), ev["output"].str().orEmpty(), (ev["is_error"] as? JsonPrimitive)?.booleanOrNull == true),
                    )
                }
            }
            "error" -> {
                val failed = activeRuns.value.values.filter { it.generation == epoch }.map { it.id }.toSet()
                activeRuns.update { running -> running.filterValues { it.generation != epoch || it.id !in failed } }
                failed.forEach { runId ->
                    runCatching { sendFor(h, epoch, line { put("type", "cancel"); put("id", runId) }) }
                    requestRuns.value.filterValues { it == runId }.keys.forEach { requests.value[it]?.cancel() }
                }
                _events.emit(AgentEvent.Failure(obj["message"].str() ?: "智能体引擎出错", failed))
            }
        }
    }
}

/** Host-owned gateway option; leave the native kernel protocol unchanged. */
internal fun withReasoningEffort(body: String, effort: String?): String {
    val payload = AgentJson.parseToJsonElement(body) as JsonObject
    return JsonObject((payload - "reasoning_effort") +
        (effort?.let { mapOf("reasoning_effort" to JsonPrimitive(it)) } ?: emptyMap())).toString()
}
