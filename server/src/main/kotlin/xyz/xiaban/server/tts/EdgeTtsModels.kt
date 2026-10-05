package xyz.xiaban.server.tts

/** The audio encodings the Edge read-aloud endpoint understands. Only one is currently served. */
enum class EdgeTtsOutputFormat(val wireName: String, val mimeType: String, val bytesPerSecond: Int) {
    Audio24KhZ48KbpsMonoMp3(
        wireName = "audio-24khz-48kbitrate-mono-mp3",
        mimeType = "audio/mpeg",
        bytesPerSecond = 6_000,
    ),
}

enum class EdgeTtsBoundaryType { WordBoundary, SentenceBoundary }

/** Voice and prosody for one synthesis request. */
data class EdgeTtsVoiceOptions(
    val voice: String = "zh-CN-XiaoxiaoNeural",
    val rate: String = "+0%",
    val volume: String = "+0%",
    val pitch: String = "+0Hz",
    val boundary: EdgeTtsBoundaryType = EdgeTtsBoundaryType.WordBoundary,
) {
    init {
        require(rate.matches(Regex("^[+-]\\d+%$"))) { "Invalid rate '$rate'" }
        require(volume.matches(Regex("^[+-]\\d+%$"))) { "Invalid volume '$volume'" }
        require(pitch.matches(Regex("^[+-]\\d+Hz$"))) { "Invalid pitch '$pitch'" }
    }
}

data class EdgeTtsVoice(
    val shortName: String,
    val gender: String,
    val locale: String,
    val friendlyName: String,
)

open class EdgeTtsException(message: String, cause: Throwable? = null) : Exception(message, cause)
