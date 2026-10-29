package xyz.xiangban.xiaoban

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import android.media.AudioAttributes
import android.media.AudioFormat
import android.media.AudioDeviceInfo
import android.media.AudioFocusRequest
import android.media.AudioManager
import android.media.audiofx.AcousticEchoCanceler
import android.media.audiofx.NoiseSuppressor
import android.media.AudioRecord
import android.media.MediaPlayer
import android.media.MediaRecorder
import android.content.Intent
import android.os.Build
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.speech.RecognitionListener
import android.speech.RecognizerIntent
import android.speech.SpeechRecognizer
import java.io.ByteArrayOutputStream
import java.io.File

/** 手机录音（16kHz 单声道 16 位 PCM，最长 20 秒）和播放电脑合成好的语音。 */
class AndroidVoice(private val context: Context, private val requestMic: suspend () -> Boolean) {
    @Volatile private var record: AudioRecord? = null
    private var thread: Thread? = null
    private var buffer: ByteArrayOutputStream? = null
    private var player: MediaPlayer? = null

    suspend fun start(): Boolean {
        if (record != null) return true
        if (context.checkSelfPermission(Manifest.permission.RECORD_AUDIO) != PackageManager.PERMISSION_GRANTED && !requestMic()) return false
        val minimum = AudioRecord.getMinBufferSize(SampleRate, AudioFormat.CHANNEL_IN_MONO, AudioFormat.ENCODING_PCM_16BIT)
        if (minimum <= 0) return false
        val rec = try {
            AudioRecord(MediaRecorder.AudioSource.MIC, SampleRate, AudioFormat.CHANNEL_IN_MONO, AudioFormat.ENCODING_PCM_16BIT, minimum * 4)
        } catch (e: SecurityException) {
            return false
        }
        if (rec.state != AudioRecord.STATE_INITIALIZED) { rec.release(); return false }
        val out = ByteArrayOutputStream()
        buffer = out
        record = rec
        rec.startRecording()
        thread = Thread {
            val chunk = ByteArray(minimum)
            while (record === rec) {
                val n = rec.read(chunk, 0, chunk.size)
                if (n < 0) break
                if (n > 0) synchronized(out) { if (out.size() < MaxBytes) out.write(chunk, 0, n) }
            }
        }.also { it.start() }
        return true
    }

    fun stop(): ByteArray? {
        val rec = record ?: return null
        record = null
        runCatching { rec.stop() }
        thread?.join(500)
        thread = null
        rec.release()
        val out = buffer ?: return null
        buffer = null
        return synchronized(out) { out.toByteArray() }
    }

    /**
     * 流式录音：每 20ms（640 字节）一帧回调；返回停止函数，不能录返回 null。
     * 按“通话”的方式录：麦克风用 VOICE_COMMUNICATION，打开系统的回声消除和降噪，音频走通话模式并外放（有耳机就不强制外放），
     * 这样小伴说话时麦克风里它自己的声音会被大幅压掉，用户才能直接开口打断。
     */
    suspend fun stream(onFrame: (ByteArray) -> Unit): (() -> Unit)? {
        if (context.checkSelfPermission(Manifest.permission.RECORD_AUDIO) != PackageManager.PERMISSION_GRANTED && !requestMic()) return null
        val minimum = AudioRecord.getMinBufferSize(SampleRate, AudioFormat.CHANNEL_IN_MONO, AudioFormat.ENCODING_PCM_16BIT)
        if (minimum <= 0) return null
        val rec = try {
            AudioRecord(MediaRecorder.AudioSource.VOICE_COMMUNICATION, SampleRate, AudioFormat.CHANNEL_IN_MONO, AudioFormat.ENCODING_PCM_16BIT, maxOf(minimum, FrameBytes * 8))
        } catch (e: SecurityException) {
            return null
        }
        if (rec.state != AudioRecord.STATE_INITIALIZED) { rec.release(); return null }
        val effects = ArrayList<android.media.audiofx.AudioEffect>()
        if (AcousticEchoCanceler.isAvailable()) AcousticEchoCanceler.create(rec.audioSessionId)?.let { it.enabled = true; effects += it }
        if (NoiseSuppressor.isAvailable()) NoiseSuppressor.create(rec.audioSessionId)?.let { it.enabled = true; effects += it }
        val audio = context.getSystemService(Context.AUDIO_SERVICE) as AudioManager
        val callAttributes = AudioAttributes.Builder().setUsage(AudioAttributes.USAGE_MEDIA).setContentType(AudioAttributes.CONTENT_TYPE_SPEECH).build()
        // 占用音频焦点：别的应用在放的音乐/视频会暂停，通话结束再恢复。
        var focus: AudioFocusRequest? = null
        if (Build.VERSION.SDK_INT >= 26) {
            focus = AudioFocusRequest.Builder(AudioManager.AUDIOFOCUS_GAIN_TRANSIENT).setAudioAttributes(callAttributes).setOnAudioFocusChangeListener { }.build()
            audio.requestAudioFocus(focus)
        } else {
            @Suppress("DEPRECATION")
            audio.requestAudioFocus(null, AudioManager.STREAM_MUSIC, AudioManager.AUDIOFOCUS_GAIN_TRANSIENT)
        }
        val running = java.util.concurrent.atomic.AtomicBoolean(true)
        rec.startRecording()
        val worker = Thread {
            val frame = ByteArray(FrameBytes)
            while (running.get()) {
                var filled = 0
                while (filled < FrameBytes && running.get()) {
                    val n = rec.read(frame, filled, FrameBytes - filled)
                    if (n < 0) { running.set(false); break }
                    filled += n
                }
                if (filled == FrameBytes && running.get()) onFrame(frame.copyOf())
            }
            runCatching { rec.stop() }
            effects.forEach { runCatching { it.release() } }
            rec.release()
            if (Build.VERSION.SDK_INT >= 26) focus?.let { audio.abandonAudioFocusRequest(it) }
            else {
                @Suppress("DEPRECATION")
                audio.abandonAudioFocus(null)
            }
        }
        worker.start()
        return { running.set(false) }
    }

    /** 系统的回声消除可用，或者戴着耳机：小伴自己的声音不会被当成用户开口。 */
    fun echoSafe(): Boolean {
        val audio = context.getSystemService(Context.AUDIO_SERVICE) as AudioManager
        val headset = audio.getDevices(AudioManager.GET_DEVICES_OUTPUTS).any { it.type in HeadsetTypes }
        return AcousticEchoCanceler.isAvailable() || headset
    }

    /** 用系统自带的语音识别听一句话（本地识别和云端都用不了时的最后备选）。 */
    fun listenOnce(onPartial: (String) -> Unit, onResult: (String?, String?) -> Unit): (() -> Unit)? {
        if (!SpeechRecognizer.isRecognitionAvailable(context)) return null
        if (context.checkSelfPermission(Manifest.permission.RECORD_AUDIO) != PackageManager.PERMISSION_GRANTED) {
            onResult(null, "没有麦克风权限")
            return {}
        }
        val main = Handler(Looper.getMainLooper())
        var recognizer: SpeechRecognizer? = null
        var finished = false
        fun finish(text: String?, error: String?) {
            if (finished) return
            finished = true
            recognizer?.destroy()
            recognizer = null
            onResult(text, error)
        }
        main.post {
            val r = SpeechRecognizer.createSpeechRecognizer(context)
            recognizer = r
            r.setRecognitionListener(object : RecognitionListener {
                override fun onResults(results: Bundle?) = finish(results?.getStringArrayList(SpeechRecognizer.RESULTS_RECOGNITION)?.firstOrNull().orEmpty(), null)
                override fun onPartialResults(partialResults: Bundle?) {
                    partialResults?.getStringArrayList(SpeechRecognizer.RESULTS_RECOGNITION)?.firstOrNull()?.let(onPartial)
                }
                override fun onError(error: Int) {
                    if (error == SpeechRecognizer.ERROR_NO_MATCH || error == SpeechRecognizer.ERROR_SPEECH_TIMEOUT) finish("", null)
                    else finish(null, "手机自带的语音识别出错了（$error）")
                }
                override fun onReadyForSpeech(params: Bundle?) {}
                override fun onBeginningOfSpeech() {}
                override fun onRmsChanged(rmsdB: Float) {}
                override fun onBufferReceived(buffer: ByteArray?) {}
                override fun onEndOfSpeech() {}
                override fun onEvent(eventType: Int, params: Bundle?) {}
            })
            r.startListening(Intent(RecognizerIntent.ACTION_RECOGNIZE_SPEECH).apply {
                putExtra(RecognizerIntent.EXTRA_LANGUAGE_MODEL, RecognizerIntent.LANGUAGE_MODEL_FREE_FORM)
                putExtra(RecognizerIntent.EXTRA_LANGUAGE, "zh-CN")
                putExtra(RecognizerIntent.EXTRA_PARTIAL_RESULTS, true)
            })
        }
        return {
            main.post {
                finished = true
                recognizer?.cancel()
                recognizer?.destroy()
                recognizer = null
            }
        }
    }

    private val playSeq = java.util.concurrent.atomic.AtomicInteger()

    fun play(bytes: ByteArray, onDone: () -> Unit): Boolean {
        stopPlayback()
        return try {
            val file = File(context.cacheDir, "reply-voice-${playSeq.incrementAndGet() % 4}.mp3")
            file.writeBytes(bytes)
            val mp = MediaPlayer()
            // 使用媒体流属性播放，避免电话通道硬件降噪与窄带滤波导致的破音与沙沙声
            mp.setAudioAttributes(AudioAttributes.Builder().setUsage(AudioAttributes.USAGE_MEDIA).setContentType(AudioAttributes.CONTENT_TYPE_SPEECH).build())
            mp.setDataSource(file.absolutePath)
            mp.setOnCompletionListener { stopPlayback(); onDone() }
            mp.setOnErrorListener { _, _, _ -> stopPlayback(); onDone(); true }
            mp.prepare()
            mp.start()
            player = mp
            true
        } catch (e: Exception) {
            stopPlayback()
            false
        }
    }

    fun stopPlayback() {
        player?.let { runCatching { it.stop() }; it.release() }
        player = null
    }

    private companion object {
        const val SampleRate = 16_000
        const val FrameBytes = 640
        val HeadsetTypes = setOf(
            AudioDeviceInfo.TYPE_WIRED_HEADSET, AudioDeviceInfo.TYPE_WIRED_HEADPHONES, AudioDeviceInfo.TYPE_USB_HEADSET,
            AudioDeviceInfo.TYPE_BLUETOOTH_SCO, AudioDeviceInfo.TYPE_BLUETOOTH_A2DP, AudioDeviceInfo.TYPE_BLE_HEADSET,
        )
        const val MaxBytes = 20 * SampleRate * 2
    }
}
