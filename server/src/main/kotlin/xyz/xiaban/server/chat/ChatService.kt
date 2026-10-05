package xyz.xiaban.server.chat

import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.add
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import kotlinx.serialization.json.putJsonArray
import kotlinx.serialization.json.putJsonObject
import org.slf4j.LoggerFactory
import xyz.xiaban.server.admin.LlmSettingsStore
import xyz.xiaban.server.flags.FeatureFlags
import xyz.xiaban.server.http.AppJson
import xyz.xiaban.server.llm.ChatMessage
import xyz.xiaban.server.llm.FunctionDefinition
import xyz.xiaban.server.llm.LlmChatResult
import xyz.xiaban.server.llm.LlmProvider
import xyz.xiaban.server.llm.ToolDefinition
import xyz.xiaban.server.memory.MemoryService
import xyz.xiaban.server.search.SearchService
import xyz.xiaban.server.users.ProfileService
import xyz.xiaban.shared.ChatMessageView
import xyz.xiaban.shared.Personality
import java.time.Clock
import java.time.Duration
import java.time.LocalDateTime
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import java.util.Locale

class ChatService(
    private val repository: ChatRepository,
    private val profiles: ProfileService,
    private val memories: MemoryService,
    private val providers: List<LlmProvider>,
    private val flags: FeatureFlags,
    private val searchService: SearchService,
    private val llmSettings: LlmSettingsStore,
    private val clock: Clock = Clock.systemUTC(),
) {
    val available: Boolean get() = providers.isNotEmpty()

    suspend fun list(userId: Long, limit: Int = 30): List<ChatMessageView> =
        repository.list(userId, limit)

    suspend fun clear(userId: Long): Int =
        repository.clear(userId)

    suspend fun stream(
        userId: Long?,
        userMessage: String,
        guestHistory: List<ChatMessageView> = emptyList(),
        onChunk: suspend (String) -> Unit,
    ): String {
        val trimmed = userMessage.trim().take(MAX_MESSAGE_CHARS)
        require(trimmed.isNotEmpty()) { "消息不能为空" }

        if (userId != null) {
            repository.append(userId, "user", trimmed)
        }

        if (!available || !flags.llmEnabled()) {
            val fallback = "小伴的聊天暂时关闭了，稍后再来找它吧。"
            if (userId != null) repository.append(userId, "assistant", fallback)
            onChunk(fallback)
            return fallback
        }

        val companionName: String
        val personality: Personality
        val nickname: String?
        val memoryNotes: List<String>
        val history: List<ChatMessageView>

        if (userId != null) {
            val user = profiles.user(userId)
            companionName = user.companionName
            personality = user.personality
            nickname = user.nickname
            memoryNotes = memories.recent(userId, limit = 5)
            history = repository.list(userId, limit = 10)
        } else {
            companionName = "小伴"
            personality = Personality.Gentle
            nickname = null
            memoryNotes = emptyList()
            history = (guestHistory + ChatMessageView(role = "user", content = trimmed)).takeLast(10)
        }

        val systemPrompt = buildSystemPrompt(companionName, personality, nickname, memoryNotes)
        val provider = providers.first()
        val llmConf = llmSettings.current()
        val webSearchEnabled = llmConf.webSearchEnabled && llmConf.webSearchBaseUrl.isNotBlank()

        val full = try {
            if (webSearchEnabled) {
                val tools = listOf(
                    ToolDefinition(
                        function = FunctionDefinition(
                            name = "web_search",
                            description = "进行实时网络搜索。当用户询问最新时事、新闻热点、今天/当下发生的事情、近期上映的电影或外部生活资讯时必须调用此工具获取最新客观事实。",
                            parameters = buildJsonObject {
                                put("type", "object")
                                putJsonObject("properties") {
                                    putJsonObject("query") {
                                        put("type", "string")
                                        put("description", "要搜索的关键词或简短搜索短语，例如「2026年最近热门电影」")
                                    }
                                }
                                putJsonArray("required") {
                                    add("query")
                                }
                            },
                        ),
                    ),
                )
                val chatMessages = mutableListOf<ChatMessage>()
                chatMessages.add(ChatMessage("system", systemPrompt))
                for (h in history.dropLast(1)) {
                    if (h.content.isNotBlank()) chatMessages.add(ChatMessage(h.role, h.content))
                }
                chatMessages.add(ChatMessage("user", trimmed))

                val turn = provider.chatWithTools(
                    messages = chatMessages,
                    tools = tools,
                    timeout = STREAM_TIMEOUT,
                    maxTokens = 300,
                    temperature = 0.8,
                )

                when (turn) {
                    is LlmChatResult.NeedToolCalls -> {
                        chatMessages.add(turn.rawAssistantMessage)
                        for (call in turn.toolCalls) {
                            if (call.function.name == "web_search") {
                                val query = extractQuery(call.function.arguments)
                                onChunk("🔍 小伴正在帮你搜「$query」…\n\n")
                                val searchResults = searchService.search(llmConf.webSearchBaseUrl, query)
                                chatMessages.add(
                                    ChatMessage(
                                        role = "tool",
                                        toolCallId = call.id,
                                        content = searchResults,
                                    ),
                                )
                            }
                        }
                        provider.streamMessages(
                            messages = chatMessages,
                            timeout = STREAM_TIMEOUT,
                            maxTokens = 300,
                            temperature = 0.8,
                            onChunk = onChunk,
                        )
                    }
                    is LlmChatResult.Content -> {
                        onChunk(turn.text)
                        turn.text
                    }
                }
            } else {
                val messages = mutableListOf<Pair<String, String>>()
                messages.add("system" to systemPrompt)
                for (h in history.dropLast(1)) {
                    if (h.content.isNotBlank()) messages.add(h.role to h.content)
                }
                messages.add("user" to trimmed)
                provider.stream(
                    messages = messages,
                    timeout = STREAM_TIMEOUT,
                    maxTokens = 300,
                    temperature = 0.8,
                    onChunk = onChunk,
                )
            }
        } catch (e: Exception) {
            log.warn("Chat stream failed from {}: {}", provider.name, e.message)
            val errReply = "小伴晃了晃脑袋，刚才走神了，再说一遍好不好？"
            onChunk(errReply)
            errReply
        }

        if (userId != null && full.isNotBlank()) {
            repository.append(userId, "assistant", full)
        }

        return full
    }

    private fun extractQuery(arguments: String): String {
        val parsed = runCatching { AppJson.decodeFromString<JsonObject>(arguments) }.getOrNull()
        val q = parsed?.get("query")?.toString()?.trim('"')
        if (!q.isNullOrBlank()) return q
        val match = Regex(""""query"\s*:\s*"([^"]+)"""").find(arguments)
        return match?.groupValues?.get(1)?.trim() ?: arguments.trim().take(30)
    }

    private fun buildSystemPrompt(
        companionName: String,
        personality: Personality,
        nickname: String?,
        memoryNotes: List<String>,
    ): String = buildString {
        appendLine("你是「$companionName」，一只住在用户家里的猫，是家里的一员，不是AI工具，也不是客服。")
        appendLine("你的性格是「${personality.label}」：${style(personality)}")
        val now = LocalDateTime.ofInstant(clock.instant(), ZoneId.of("Asia/Shanghai"))
        val dateStr = now.format(DateTimeFormatter.ofPattern("yyyy年M月d日 EEEE HH:mm", Locale.CHINESE))
        appendLine("当前现实世界时间是：$dateStr。")
        if (!nickname.isNullOrBlank()) {
            appendLine("用户的称呼是「$nickname」。")
        }
        if (memoryNotes.isNotEmpty()) {
            appendLine("你记得关于用户的这些事：")
            for (m in memoryNotes) appendLine("- $m")
        }
        appendLine()
        appendLine("对话规则：")
        appendLine("1. 像真实室友发消息一样口语化、生活化。多用暖心自然的短句，不用说教，少用感叹号，绝不用官方腔和网络烂梗。")
        appendLine("2. 每次回复控制在 30 ~ 140 字，不要长篇大论。")
        appendLine("3. 如果用户感到疲惫、工作受挫或向你倾诉，优先提供温柔的情绪价值与同理心，坚定地站在用户这一边。")
        appendLine("4. 适当自然地融入猫猫的小习惯（如蹭蹭、伸懒腰、趴在身边打呼噜），但不要每句都刻意喵喵叫。")
        appendLine("5. 若调用了网络搜索，请用小伴自己的口吻自然分享找到的事实，不要生硬复制粘贴网址或网页源代码。")
    }

    private fun style(p: Personality) = when (p) {
        Personality.Toxic -> "嘴有点毒，心很软，先损一句打工人，再细心关心。"
        Personality.Gentle -> "温柔治愈，会体贴人，像一杯温热牛奶静静陪着对方。"
        Personality.Chatty -> "活泼元气，话多爱碎碎念，喜欢分享各种好吃的好玩的。"
        Personality.Tsundere -> "嘴硬心软，假装满不在乎，其实偷偷把用户的每一句话都放在心上。"
    }

    companion object {
        private val log = LoggerFactory.getLogger(ChatService::class.java)
        private const val MAX_MESSAGE_CHARS = 500
        private val STREAM_TIMEOUT: Duration = Duration.ofSeconds(20)
    }
}
