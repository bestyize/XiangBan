package xyz.xiaban.server.tts

import java.security.MessageDigest
import java.util.Base64
import kotlinx.coroutines.future.await
import org.slf4j.LoggerFactory
import xyz.xiaban.contracts.companion.personality.Personality
import xyz.xiaban.server.redis.Redis

/** Audio plus the MIME type it should be served as. */
data class SpeechAudio(val bytes: ByteArray, val mimeType: String) {
    override fun equals(other: Any?) = other is SpeechAudio && bytes.contentEquals(other.bytes) && mimeType == other.mimeType
    override fun hashCode() = 31 * bytes.contentHashCode() + mimeType.hashCode()
}

/**
 * Text to speech for 小伴.
 *
 * Audio is cached in Redis by (voice, text) because replies repeat often — greetings, the
 * scripted fallbacks — and the upstream service is unofficial. Any failure returns null so
 * the caller can simply keep the text reply.
 */
class TtsService(
    private val client: EdgeTtsClient,
    private val redis: Redis,
    private val enabled: suspend () -> Boolean = { true },
    /** Voice chosen in the admin console; empty means "use the personality's own voice". */
    private val configuredVoice: suspend () -> String = { "" },
) {
    suspend fun speak(text: String, personality: Personality): SpeechAudio? {
        if (!enabled()) return null
        val trimmed = text.trim().take(MAX_CHARS)
        if (trimmed.isEmpty()) return null

        val options = voiceFor(personality, configuredVoice())
        val key = cacheKey(options.voice, trimmed)
        redis.cmd.get(key).await()?.let { cached ->
            return runCatching { Base64.getDecoder().decode(cached) }.getOrNull()?.let { SpeechAudio(it, MP3) }
        }

        val audio = try {
            client.synthesize(trimmed, options)
        } catch (e: Exception) {
            log.warn("edge-tts synthesis failed for voice {}: {}", options.voice, e.message)
            return null
        }
        if (audio.isEmpty()) return null

        runCatching { redis.cmd.setex(key, CACHE_SECONDS, Base64.getEncoder().encodeToString(audio)).await() }
        return SpeechAudio(audio, MP3)
    }

    /** Voices offered by the service, for the admin picker. */
    suspend fun voices(): List<EdgeTtsVoice> = client.voices().filter { it.locale.startsWith("zh") }

    /** Renders [text] in an explicitly chosen voice, bypassing the cache; used by the admin preview. */
    suspend fun preview(voice: String, text: String): SpeechAudio? {
        val chosen = voice.trim().ifEmpty { "zh-CN-XiaoxiaoNeural" }
        if (!chosen.matches(Regex("^[A-Za-z0-9-]{1,64}$"))) return null
        val audio = try {
            client.synthesize(text.take(MAX_CHARS), EdgeTtsVoiceOptions(voice = chosen))
        } catch (e: Exception) {
            log.warn("edge-tts preview failed for voice {}: {}", chosen, e.message)
            return null
        }
        return audio.takeIf { it.isNotEmpty() }?.let { SpeechAudio(it, MP3) }
    }

    /**
     * One voice per personality so 小伴 sounds like the character the user picked, unless the
     * admin pinned a specific voice, in which case that voice wins for everyone.
     * Xiaoxiao is warm, Xiaoyi bright, Yunxi a younger male voice.
     */
    private fun voiceFor(personality: Personality, pinnedVoice: String): EdgeTtsVoiceOptions {
        val pinned = pinnedVoice.trim()
        if (pinned.isNotEmpty()) return EdgeTtsVoiceOptions(voice = pinned)
        return when (personality) {
            Personality.Gentle -> EdgeTtsVoiceOptions(voice = "zh-CN-XiaoxiaoNeural", rate = "-5%", pitch = "+10Hz")
            Personality.Chatty -> EdgeTtsVoiceOptions(voice = "zh-CN-XiaoyiNeural", rate = "+8%", pitch = "+5Hz")
            Personality.Toxic -> EdgeTtsVoiceOptions(voice = "zh-CN-YunxiNeural", rate = "+2%", pitch = "-5Hz")
            Personality.Tsundere -> EdgeTtsVoiceOptions(voice = "zh-CN-YunxiNeural", rate = "-3%", pitch = "-10Hz")
        }
    }

    private fun cacheKey(voice: String, text: String): String {
        val digest = MessageDigest.getInstance("SHA-256").digest("$voice\u0000$text".toByteArray())
        return "xb:tts:" + digest.joinToString("") { "%02x".format(it) }
    }

    companion object {
        private val log = LoggerFactory.getLogger(TtsService::class.java)
        private const val MP3 = "audio/mpeg"
        private const val MAX_CHARS = 300
        private const val CACHE_SECONDS = 7L * 24 * 3600
    }
}
