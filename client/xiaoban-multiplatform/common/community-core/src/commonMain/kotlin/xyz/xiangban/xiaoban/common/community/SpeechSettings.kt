package xyz.xiangban.xiaoban.common.community

/** 朗读的声音从哪来：Edge 在线音色，或手机自带的系统朗读（离线）。 */
enum class SpeechSource(val id: String, val label: String, val desc: String) {
    Edge("edge", "Edge 在线", "微软 Edge 在线音色，免费；连不上时自动改用系统朗读。"),
    System("system", "系统朗读", "用手机自带的语音，不联网、无延迟，音色较普通。");

    companion object {
        fun byId(id: String?): SpeechSource = entries.firstOrNull { it.id == id } ?: Edge
    }
}

class EdgeVoice(val id: String, val name: String, val gender: String)

/** 手机上内置的几个常用中文音色，和电脑端默认的一致。 */
object EdgeVoices {
    val All: List<EdgeVoice> = listOf(
        EdgeVoice("zh-CN-XiaoxiaoNeural", "晓晓", "女声"),
        EdgeVoice("zh-CN-XiaoyiNeural", "晓伊", "女声"),
        EdgeVoice("zh-CN-YunxiNeural", "云希", "男声"),
        EdgeVoice("zh-CN-YunyangNeural", "云扬", "男声"),
    )
    val Default: EdgeVoice get() = All.first()
    fun byId(id: String?): EdgeVoice = All.firstOrNull { it.id == id } ?: Default
}

/** 语速五档：很慢、较慢、正常、较快、很快，对应 Edge 的语速百分比。 */
object SpeechRates {
    val Names = listOf("很慢", "较慢", "正常", "较快", "很快")
    private val Percents = listOf(-40, -20, 0, 25, 50)
    const val DefaultStep = 2
    fun clamp(step: Int): Int = step.coerceIn(0, Names.lastIndex)
    fun percent(step: Int): Int = Percents[clamp(step)]
}

/**
 * 声音设置：朗读来源、自动朗读、音色、语速，以及“直接开口打断”（没设置过就是 null，按手机能不能消回声来定）。
 * 用键值对存取，存在哪里由平台的偏好存储决定。
 */
data class SpeechSettings(
    val source: SpeechSource = SpeechSource.Edge,
    val autoRead: Boolean = false,
    val voice: String = EdgeVoices.Default.id,
    val rateStep: Int = SpeechRates.DefaultStep,
    val bargeIn: Boolean? = null,
) {
    fun encode(): Map<String, String> = buildMap {
        put("speech.source", source.id)
        put("speech.autoRead", autoRead.toString())
        put("speech.voice", voice)
        put("speech.rate", rateStep.toString())
        put("speech.barge", bargeIn?.toString().orEmpty())
    }

    companion object {
        fun decode(get: (String) -> String?): SpeechSettings = SpeechSettings(
            source = SpeechSource.byId(get("speech.source")),
            autoRead = get("speech.autoRead") == "true",
            voice = EdgeVoices.byId(get("speech.voice")).id,
            rateStep = SpeechRates.clamp(get("speech.rate")?.toIntOrNull() ?: SpeechRates.DefaultStep),
            bargeIn = get("speech.barge")?.toBooleanStrictOrNull(),
        )
    }
}
