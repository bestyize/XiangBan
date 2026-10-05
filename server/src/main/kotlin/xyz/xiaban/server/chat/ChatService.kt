package xyz.xiaban.server.chat

import org.slf4j.LoggerFactory
import xyz.xiaban.server.flags.FeatureFlags
import xyz.xiaban.server.llm.LlmProvider
import xyz.xiaban.server.memory.MemoryService
import xyz.xiaban.server.users.ProfileService
import xyz.xiaban.shared.ChatMessageView
import xyz.xiaban.shared.Personality
import java.time.Duration

class ChatService(
    private val repository: ChatRepository,
    private val profiles: ProfileService,
    private val memories: MemoryService,
    private val providers: List<LlmProvider>,
    private val flags: FeatureFlags,
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
            val fallback = "小伴正在打盹呢，等它醒来再陪你聊吧～"
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
        val messages = mutableListOf<Pair<String, String>>()
        messages.add("system" to systemPrompt)
        for (h in history.dropLast(1)) {
            if (h.content.isNotBlank()) messages.add(h.role to h.content)
        }
        messages.add("user" to trimmed)

        val provider = providers.first()
        val full = try {
            provider.stream(
                messages = messages,
                timeout = STREAM_TIMEOUT,
                maxTokens = 300,
                temperature = 0.8,
                onChunk = onChunk,
            )
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

    private fun buildSystemPrompt(
        companionName: String,
        personality: Personality,
        nickname: String?,
        memoryNotes: List<String>,
    ): String = buildString {
        appendLine("你是「$companionName」，一只住在用户家里的猫，是家里的一员，不是AI工具，也不是客服。")
        appendLine("你的性格是「${personality.label}」：${style(personality)}")
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
        appendLine("2. 每次回复控制在 30 ~ 120 字，不要长篇大论。")
        appendLine("3. 如果用户感到疲惫、工作受挫或向你倾诉，优先提供温柔的情绪价值与同理心，坚定地站在用户这一边。")
        appendLine("4. 适当自然地融入猫猫的小习惯（如蹭蹭、伸懒腰、趴在身边打呼噜），但不要每句都刻意喵喵叫。")
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
