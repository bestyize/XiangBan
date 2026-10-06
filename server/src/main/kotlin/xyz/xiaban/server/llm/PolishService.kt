package xyz.xiaban.server.llm

import kotlinx.coroutines.TimeoutCancellationException
import kotlinx.coroutines.withTimeout
import org.slf4j.LoggerFactory
import xyz.xiaban.shared.Personality
import java.time.Duration

data class PolishInput(
    val original: String,
    val companionName: String,
    val personality: Personality,
    val nickname: String?,
    val titleLabel: String,
    val localTime: String,
    val cityName: String?,
    val weather: String?,
    val memories: List<String>,
)

/** Rewrites the library line with today's context; any failure or timeout keeps the original line. */
class PolishService(private val providers: List<LlmProvider>, private val budget: Duration) {
    val available: Boolean get() = providers.isNotEmpty()

    suspend fun polish(input: PolishInput): String? {
        val deadline = System.nanoTime() + budget.toNanos()
        for (provider in providers) {
            val remaining = Duration.ofNanos(deadline - System.nanoTime())
            if (remaining < MIN_ATTEMPT) break
            val raw = try {
                withTimeout(remaining.toMillis()) { provider.complete(systemPrompt(input), userPrompt(input), remaining) }
            } catch (_: TimeoutCancellationException) {
                log.warn("{} timed out after {} ms", provider.name, remaining.toMillis())
                null
            } catch (e: Exception) {
                log.warn("{} failed: {}", provider.name, e.message)
                null
            }
            ReplySanitizer.clean(raw)?.let { return it }
        }
        return null
    }

    private fun systemPrompt(input: PolishInput) = """
        你是「${input.companionName}」，一个住在用户家里的小小陪伴机器人（奶油色外壳，黑色屏幕脸，头顶有两只琥珀色的耳朵灯，不是猫），是家里的一员，不是工具，也不是客服。
        你的性格是「${input.personality.label}」：${style(input.personality)}
        任务：把给你的原句改写成一句更贴合用户今天情况的话，保留原句的意思和语气。
        规则：
        1. 只输出这一句话，不超过 40 个字，不加引号、表情符号或任何解释。
        2. 口语化，少用感叹号，不说教，不用网络烂梗。
        3. 调侃的对象是加班、老板和生活，不是用户本人；说完要落回关心。
        4. 不提具体的公司和人名；不涉及地域、性别、职业歧视；不煽动劳资对立；不涉及政治。
        5. 称呼用户时用「${input.nickname ?: "你"}」。
    """.trimIndent()

    private fun userPrompt(input: PolishInput) = buildString {
        appendLine("原句：${input.original}")
        appendLine("今日称号：${input.titleLabel}，${input.localTime} 下班")
        appendLine("城市：${input.cityName ?: "未知"}")
        appendLine("天气：${input.weather ?: "未知"}")
        append("记忆：${input.memories.ifEmpty { listOf("暂无") }.joinToString("；")}")
    }

    private fun style(p: Personality) = when (p) {
        Personality.Toxic -> "嘴有点毒，心很软，先损一句再关心。"
        Personality.Gentle -> "温柔，会安慰人，陪着对方。"
        Personality.Chatty -> "热闹，话多，爱碎碎念，喜欢聊吃的。"
        Personality.Tsundere -> "嘴硬心软，先否认在乎，再小声关心。"
    }

    private companion object {
        val log = LoggerFactory.getLogger(PolishService::class.java)
        val MIN_ATTEMPT: Duration = Duration.ofMillis(300)
    }
}

object ReplySanitizer {
    private const val QUOTES = "\"'“”‘’「」『』《》"
    private val banned = Regex("""(https?|www\.|@|\d{5,}|政治|政府|领导人|主席|总理|共产|民主|台独|藏独|疆独|示威|游行|罢工|种族|宗教|自杀|去死)""")

    fun clean(raw: String?, maxChars: Int = 40): String? {
        val line = raw?.lineSequence()?.map(String::trim)?.firstOrNull(String::isNotEmpty) ?: return null
        val text = line.trim { it in QUOTES || it.isWhitespace() }
        val length = text.codePointCount(0, text.length)
        if (length < 4 || length > maxChars || banned.containsMatchIn(text)) return null
        return text
    }
}
