package xyz.xiaban.server.tts

import java.security.MessageDigest
import kotlin.random.Random

/**
 * Microsoft Edge "read aloud" wire protocol, as used by the open-source edge-tts projects.
 *
 * This is an undocumented endpoint that Microsoft does not support. The client identifier,
 * trusted token and version string may change without notice, so [EdgeTtsService] treats any
 * failure as "no audio" rather than an error the user has to see.
 */
internal object EdgeTtsProtocol {
    const val BASE_HOST = "speech.platform.bing.com"
    const val BASE_PATH = "/consumer/speech/synthesize/readaloud"
    const val TRUSTED_CLIENT_TOKEN = "6A5AA1D4EAFF4E9FB37E23D68491D6F4"
    const val CHROMIUM_FULL_VERSION = "143.0.3650.75"
    const val SEC_MS_GEC_VERSION = "1-$CHROMIUM_FULL_VERSION"
    const val WIN_EPOCH_SECONDS = 11_644_473_600L
    const val TICKS_PER_SECOND = 10_000_000L

    const val ORIGIN = "chrome-extension://jdiccldimpdaibmpdkjnbmckianbfold"
    const val USER_AGENT =
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 " +
            "(KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36 Edg/143.0.0.0"

    fun webSocketUrl(connectionId: String, unixSeconds: Long = nowSeconds()): String =
        "wss://$BASE_HOST$BASE_PATH/edge/v1?TrustedClientToken=$TRUSTED_CLIENT_TOKEN" +
            "&ConnectionId=$connectionId" +
            "&Sec-MS-GEC=${secMsGec(unixSeconds)}" +
            "&Sec-MS-GEC-Version=$SEC_MS_GEC_VERSION"

    fun voicesUrl(unixSeconds: Long = nowSeconds()): String =
        "https://$BASE_HOST$BASE_PATH/voices/list?trustedclienttoken=$TRUSTED_CLIENT_TOKEN" +
            "&Sec-MS-GEC=${secMsGec(unixSeconds)}" +
            "&Sec-MS-GEC-Version=$SEC_MS_GEC_VERSION"

    /**
     * The short-lived token both endpoints require. The service rounds the timestamp to a
     * five-minute window and hashes it with the public trusted-client token. This is not an
     * API key and is not secret.
     */
    fun secMsGec(unixSeconds: Long = nowSeconds()): String {
        val windowsSeconds = unixSeconds + WIN_EPOCH_SECONDS
        val roundedSeconds = windowsSeconds - windowsSeconds.mod(300L)
        val ticks = roundedSeconds * TICKS_PER_SECOND
        return sha256HexUpper("$ticks$TRUSTED_CLIENT_TOKEN")
    }

    fun connectionId(): String = randomHex(16)

    fun muid(): String = randomHex(16)

    fun speechConfig(voice: EdgeTtsVoiceOptions): String {
        val wordBoundary = if (voice.boundary == EdgeTtsBoundaryType.WordBoundary) "true" else "false"
        val sentenceBoundary = if (voice.boundary == EdgeTtsBoundaryType.SentenceBoundary) "true" else "false"
        return "X-Timestamp:${serviceTimestamp()}\r\n" +
            "Content-Type:application/json; charset=utf-8\r\n" +
            "Path:speech.config\r\n\r\n" +
            """{"context":{"synthesis":{"audio":{"metadataoptions":{"sentenceBoundaryEnabled":"$sentenceBoundary","wordBoundaryEnabled":"$wordBoundary"},"outputFormat":"${EdgeTtsOutputFormat.Audio24KhZ48KbpsMonoMp3.wireName}"}}}}"""
    }

    fun ssmlRequest(requestId: String, voice: EdgeTtsVoiceOptions, text: String): String =
        "X-RequestId:$requestId\r\n" +
            "Content-Type:application/ssml+xml\r\n" +
            "X-Timestamp:${serviceTimestamp()}Z\r\n" +
            "Path:ssml\r\n\r\n" +
            ssml(voice, text)

    fun ssml(options: EdgeTtsVoiceOptions, text: String): String {
        val voice = normalizeVoice(options.voice).escapeXml()
        return "<speak version='1.0' xmlns='http://www.w3.org/2001/10/synthesis' xml:lang='en-US'>" +
            "<voice name='$voice'>" +
            "<prosody pitch='${options.pitch.escapeXml()}' rate='${options.rate.escapeXml()}' " +
            "volume='${options.volume.escapeXml()}'>${text.escapeXml()}</prosody></voice></speak>"
    }

    fun normalizeVoice(voice: String): String {
        val value = voice.trim()
        if (value.startsWith("Microsoft Server Speech Text to Speech Voice")) return value
        val match = Regex("^([a-z]{2,})-([A-Z]{2,})-(.+Neural)$").find(value) ?: return value
        val language = match.groupValues[1]
        var region = match.groupValues[2]
        var name = match.groupValues[3]
        val hyphen = name.indexOf('-')
        if (hyphen >= 0) {
            region = "$region-${name.substring(0, hyphen)}"
            name = name.substring(hyphen + 1)
        }
        return "Microsoft Server Speech Text to Speech Voice ($language-$region, $name)"
    }

    fun serviceTimestamp(unixSeconds: Long = nowSeconds()): String =
        java.time.format.DateTimeFormatter
            .ofPattern("EEE MMM dd yyyy HH:mm:ss 'GMT+0000 (Coordinated Universal Time)'", java.util.Locale.US)
            .withZone(java.time.ZoneOffset.UTC)
            .format(java.time.Instant.ofEpochSecond(unixSeconds))

    private fun nowSeconds(): Long = System.currentTimeMillis() / 1_000

    private fun sha256HexUpper(input: String): String =
        MessageDigest.getInstance("SHA-256")
            .digest(input.toByteArray(Charsets.UTF_8))
            .joinToString("") { (it.toInt() and 0xff).toString(16).padStart(2, '0').uppercase() }

    private fun randomHex(byteCount: Int): String =
        Random.nextBytes(byteCount).joinToString("") { (it.toInt() and 0xff).toString(16).padStart(2, '0').uppercase() }
}

internal fun String.escapeXml(): String = buildString(length) {
    for (char in this@escapeXml) {
        when (char) {
            '&' -> append("&amp;")
            '<' -> append("&lt;")
            '>' -> append("&gt;")
            '"' -> append("&quot;")
            '\'' -> append("&apos;")
            else -> append(char)
        }
    }
}

/** Strips control characters the service rejects, and splits long text on natural boundaries. */
internal object Utf8TextChunker {
    fun split(text: String, maxBytes: Int = 4_096): List<String> {
        val sanitized = text.sanitizeControlCharacters()
        if (sanitized.isBlank()) return emptyList()

        val result = mutableListOf<String>()
        var start = 0
        while (start < sanitized.length) {
            var end = start
            var usedBytes = 0
            while (end < sanitized.length) {
                val width = sanitized.codePointWidth(end)
                val bytes = sanitized.utf8Width(end, width)
                if (usedBytes + bytes > maxBytes) break
                usedBytes += bytes
                end += width
            }
            if (end == start) end += sanitized.codePointWidth(start)

            // Prefer a whitespace break, but never make a tiny leading chunk.
            val preferred = (end - 1 downTo start + maxOf(1, (end - start) / 2))
                .firstOrNull { sanitized[it].isWhitespace() }
            if (preferred != null) end = preferred

            val chunk = sanitized.substring(start, end).trim()
            if (chunk.isNotEmpty()) result += chunk
            start = end
        }
        return result
    }

    private fun String.codePointWidth(index: Int): Int {
        val current = this[index]
        return if (current.isHighSurrogate() && index + 1 < length && this[index + 1].isLowSurrogate()) 2 else 1
    }

    private fun String.utf8Width(index: Int, width: Int): Int {
        val code = if (width == 2) {
            0x10000 + ((this[index].code - 0xD800) shl 10) + (this[index + 1].code - 0xDC00)
        } else {
            this[index].code
        }
        return when {
            code < 0x80 -> 1
            code < 0x800 -> 2
            code < 0x10000 -> 3
            else -> 4
        }
    }
}

internal fun String.sanitizeControlCharacters(): String = buildString(length) {
    for (char in this@sanitizeControlCharacters) {
        val code = char.code
        append(if (code in 0..8 || code in 11..12 || code in 14..31) ' ' else char)
    }
}

internal data class BinaryAudioFrame(val headers: Map<String, String>, val data: ByteArray)

internal fun parseBinaryAudioFrame(frame: ByteArray): BinaryAudioFrame {
    if (frame.size < 2) throw EdgeTtsException("Binary response is missing its header length")
    val headerLength = ((frame[0].toInt() and 0xff) shl 8) or (frame[1].toInt() and 0xff)
    // The service closes an audio stream with a two-byte zero-length binary frame.
    if (frame.size == 2 && headerLength == 0) return BinaryAudioFrame(emptyMap(), ByteArray(0))
    // Unlike ordinary HTTP framing, the length includes its own two-byte prefix.
    if (headerLength < 4 || headerLength + 2 > frame.size ||
        frame[headerLength] != '\r'.code.toByte() ||
        frame[headerLength + 1] != '\n'.code.toByte()
    ) {
        throw EdgeTtsException("Binary response has an invalid header length")
    }
    val headers = parseHeaders(frame.decodeToString(2, headerLength))
    return BinaryAudioFrame(headers, frame.copyOfRange(headerLength + 2, frame.size))
}

internal data class TextResponse(val path: String, val headers: Map<String, String>, val payload: String)

internal fun parseTextResponse(text: String): TextResponse {
    val separator = text.indexOf("\r\n\r\n")
    if (separator < 0) throw EdgeTtsException("Text response is missing the header separator")
    val headers = parseHeaders(text.substring(0, separator))
    val path = headers["Path"] ?: headers["path"] ?: throw EdgeTtsException("Text response is missing Path")
    return TextResponse(path, headers, text.substring(separator + 4))
}

private fun parseHeaders(raw: String): Map<String, String> =
    raw.lineSequence().mapNotNull { line ->
        val colon = line.indexOf(':')
        if (colon <= 0) null else line.substring(0, colon).trim() to line.substring(colon + 1).trim()
    }.toMap()
