package xyz.xiangban.xiaoban.common.community

import kotlin.random.Random

/** Context for the model only; the visible user message remains exactly what the user said. */
fun voiceInterruptionNote(spoken: String?): String {
    val heard = spoken?.replace(Regex("\\s+"), " ")?.trim().orEmpty()
    val excerpt = if (heard.length > 140) heard.take(40) + "……" + heard.takeLast(100) else heard
    val detail = if (spoken == null) "具体听到哪里不详。" else if (excerpt.isEmpty()) "用户还没听到任何内容就打断了。"
        else "用户听到的是：「$excerpt」，最后一句可能没听完，后面的内容没有听到。"
    return "【语音打断】你上一条回答被用户开口打断了。${detail}请直接回应用户这次说的话；没听到的部分如果仍然重要，再简短补上。\n"
}

fun voiceSpokenNote(phrase: String) = "【语音接话】你已经先对用户说出了「$phrase」，不要重复这句话，直接接着往下回应。\n"
fun withoutVoiceNotes(text: String) = text.replace(Regex("^(?:【语音(?:打断|接话)】[^\\n]*\\n)+"), "")

private fun speechWords(text: String) = text.lowercase().filter { it.isLetterOrDigit() }
fun isVoiceFiller(text: String): Boolean {
    val words = speechWords(text)
    return words.isEmpty() || words == "嗯哼" || words.all { it in "嗯啊哦噢喔哎唉呃额欸诶嘿哈呵" }
}

/** Suppress recognized speech that matches the answer currently being spoken. */
fun isVoiceEcho(text: String, playback: String): Boolean {
    val query = speechWords(text); val source = speechWords(playback)
    if (query.isEmpty()) return true
    if (source.isEmpty()) return false
    if (source.contains(query)) return true
    if (query.length !in 5..256) return false
    val allowance = minOf(10, query.length / 5)
    var previous = IntArray(source.length + 1)
    var current = IntArray(source.length + 1)
    for (i in 1..query.length) {
        current[0] = i
        for (j in 1..source.length) current[j] = minOf(previous[j] + 1, current[j - 1] + 1, previous[j - 1] + if (query[i - 1] == source[j - 1]) 0 else 1)
        val tmp = previous; previous = current; current = tmp
    }
    return previous.any { it <= allowance }
}

/**
 * 识别器碰到噪音、呼吸、没说完的音节或小伴自己的回声时，常猜成一两个英文单词（最常见的是 "The"）。
 * 这不是用户说的话：既不能打断朗读，也不能当问题发出去。与桌面 `voice/stray.ts` 一致。
 */
private val StrayWords = setOf(
    "the", "a", "an", "i", "it", "is", "and", "so", "to", "of", "in", "on", "at", "he", "she", "we", "they", "you", "that", "this",
    "but", "or", "as", "be", "if", "uh", "um", "er", "ah", "oh", "hmm", "huh", "mm", "yeah", "yes", "no", "well", "what", "there",
)
private val RecognizerTag = Regex("<\\|[^>]*\\|>")
private val NoiseLabel = Regex("^\\s*[（(\\[]\\s*(?:noise|music|silence|杂音|噪音|音乐|静音)\\s*[）)\\]]\\s*$", RegexOption.IGNORE_CASE)

private fun isCjk(c: Char) = c in '㐀'..'鿿' || c in '぀'..'ヿ' || c in '가'..'힯'

/** 只有几个别的语言的口头词或虚词，一个字也不是用户自己说的。 */
fun isVoiceStray(text: String): Boolean {
    val plain = RecognizerTag.replace(text, "").lowercase()
    if (plain.any(::isCjk)) return false
    val words = ArrayList<String>()
    val current = StringBuilder()
    for (c in plain) {
        if (c.isLetterOrDigit() || c == '\'') current.append(c)
        else if (current.isNotEmpty()) { words += current.toString(); current.clear() }
    }
    if (current.isNotEmpty()) words += current.toString()
    return words.isNotEmpty() && words.size <= 4 && words.all { it in StrayWords }
}

/** 不算一句话：杂音、标签、没有字的标点、只是“嗯”“哦”，或识别器乱猜的英文单词。 */
fun isVoiceNoise(text: String): Boolean =
    isVoiceStray(text) || NoiseLabel.matches(RecognizerTag.replace(text, "")) || isVoiceFiller(text)

/**
 * 小伴先开口说的那一句，要贴合用户说的话：招呼回招呼，请求接下来办，问题想一想，其他的什么都不说——
 * 宁可不说，也不说错。与桌面 `voice/ack.ts` 一致。
 */
enum class VoiceAckKind { Hello, Morning, Noon, Afternoon, Evening, Present, HowAreYou, Thanks, GoodNight, Bye, Task, Question }

val VoiceAckPhrases: Map<VoiceAckKind, List<String>> = mapOf(
    VoiceAckKind.Hello to listOf("你好呀，我在", "嗨，我在呢"),
    VoiceAckKind.Morning to listOf("早上好呀"),
    VoiceAckKind.Noon to listOf("中午好呀"),
    VoiceAckKind.Afternoon to listOf("下午好呀"),
    VoiceAckKind.Evening to listOf("晚上好呀"),
    VoiceAckKind.Present to listOf("在的，我在"),
    VoiceAckKind.HowAreYou to listOf("我挺好的呀"),
    VoiceAckKind.Thanks to listOf("不客气", "不用谢，随时找我"),
    VoiceAckKind.GoodNight to listOf("晚安，好梦"),
    VoiceAckKind.Bye to listOf("再见，下次再聊"),
    VoiceAckKind.Task to listOf("好的，我来看看", "好，我来处理"),
    VoiceAckKind.Question to listOf("我想想", "嗯，我想想"),
)

/** 全部接话，通话一开始就预先合成好。 */
val AllVoiceAckPhrases: List<String> = VoiceAckPhrases.values.flatten()

/** 招呼、道谢这类不用想的立即回；请求和问题等 0.7 秒，回答还没开始才说。 */
fun voiceAckDelayMs(kind: VoiceAckKind): Long = if (kind == VoiceAckKind.Task || kind == VoiceAckKind.Question) 700L else 0L

private const val AckAddress = "(?:小伴|你|您|呀|啊|啦|哦|呢|哈|吧|了|的|大家|啦啦|嘿|喂)*"
private fun social(words: String) = Regex("^$AckAddress(?:$words)$AckAddress$")

private val AckGreetings: List<Pair<VoiceAckKind, Regex>> = listOf(
    VoiceAckKind.Hello to social("你好|您好|嗨|哈喽|hello|hi|hey|哈啰"),
    VoiceAckKind.Morning to social("早安|早上好|早|早啊|早晨"),
    VoiceAckKind.Noon to social("中午好|午安"),
    VoiceAckKind.Afternoon to social("下午好"),
    VoiceAckKind.Evening to social("晚上好"),
    VoiceAckKind.Present to social("在吗|在不在|在么"),
    VoiceAckKind.HowAreYou to social("你好吗|你怎么样|最近好吗|最近怎么样|你还好吗"),
    VoiceAckKind.Thanks to social("谢谢|多谢|感谢|谢啦|谢谢你|辛苦了|辛苦"),
    VoiceAckKind.GoodNight to social("晚安"),
    VoiceAckKind.Bye to social("再见|拜拜|回头见|下次聊|先这样"),
)
private val AckAgreement = social("好的|好吧|好|行|可以|对|是的|嗯|哦|嗯嗯|哦哦|哈哈|呵呵")
private val AckFeeling = Regex("难过|伤心|郁闷|好烦|烦死|崩溃|焦虑|压力|好累|累死|孤独|想哭|难受|不开心|失眠|害怕|生气")
private val AckTask = Regex("^(?:请|麻烦|能不能|能否|可以|帮我|帮忙|给我|替我|把|打开|关闭|查|搜索|搜|找|写|整理|改|修改|创建|新建|删除|生成|翻译|总结|分析|设置|设定|切换|播放|发送|下载|安装|运行|读|看看|看一下|提醒)|帮我|给我|替我")
private val AckQuestion = Regex("[吗么呢]$|什么|怎么|为啥|如何|多少|是不是|有没有|会不会|哪(?:个|里|些|儿)|谁|区别|原理|意思")

private val HangUpLead = "(?:小伴|好|好的|行|那|那就|就|嗯|先|请|麻烦|帮我|给我|你|我|我们|咱们|可以|现在)*"
private val HangUpCore = "挂断|挂掉|挂了|挂|结束|退出|关闭|关掉|关了|不聊了|不说了"
private val HangUpTail = "(?:电话|通话|语音|对话|实时对话|聊天|吧|了|啦|呀|啊|哦|嘛|一下|好了|就好|谢谢|再见|拜拜)*"
private val HangUp = Regex("^$HangUpLead(?:$HangUpCore)$HangUpTail$")
/** 光说“结束”“退出”“关闭”不够，要带上它们结束的是什么，免得把别的事当成挂断（“退出”“关掉”可能是叫小伴关别的东西）。 */
private val HangUpBare = Regex("^$HangUpLead(?:结束|退出|关闭|关掉|关了)(?:吧|了|啦|呀|啊|哦|嘛|一下|好了|就好)*$")

/**
 * 用户是不是在让小伴挂断这通电话：“挂断”“挂了吧”“结束通话”“退出实时对话”“好，不聊了”这类整句的话。
 * 只认整句，“不要挂断”“怎么挂断电话”“挂了吗”这种都不算；“再见”“拜拜”也不算，那是在道别，可能还要再说两句。
 */
fun isVoiceHangUp(text: String): Boolean {
    val said = speechWords(text)
    return said.isNotEmpty() && HangUp.matches(said) && !HangUpBare.matches(said)
}

/** 这句话适合先回哪一类；什么都不该说返回 null。 */
fun voiceAckKind(text: String): VoiceAckKind? {
    val said = speechWords(text)
    if (said.isEmpty()) return null
    for ((kind, pattern) in AckGreetings) if (pattern.matches(said)) return kind
    if (said.length < 3 || AckAgreement.matches(said)) return null
    if (AckTask.containsMatchIn(said)) return VoiceAckKind.Task
    if (AckFeeling.containsMatchIn(said)) return null
    // 很短的一句（“好吗”）不值得先回应。
    if (said.length >= 5 && AckQuestion.containsMatchIn(said)) return VoiceAckKind.Question
    return null
}

/** 选一句，不重复上一次说的。[pick] 返回 [0, 1)。 */
fun chooseVoiceAck(ready: List<String>, last: String?, pick: () -> Double = { Random.nextDouble() }): String? {
    val options = if (ready.size > 1) ready.filter { it != last } else ready
    return options.getOrNull(minOf(options.size - 1, (pick() * options.size).toInt()))
}
