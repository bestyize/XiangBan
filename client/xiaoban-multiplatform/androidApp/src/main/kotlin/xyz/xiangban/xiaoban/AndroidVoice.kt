package xyz.xiangban.xiaoban

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import android.media.AudioAttributes
import android.media.AudioFormat
import android.media.AudioDeviceInfo
import android.media.AudioDeviceCallback
import android.media.AudioFocusRequest
import android.media.AudioManager
import android.media.audiofx.AcousticEchoCanceler
import android.media.audiofx.NoiseSuppressor
import android.media.AudioRecord
import android.media.AudioRecordingConfiguration
import android.media.AudioRouting
import android.media.audiofx.AudioEffect
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
import java.util.concurrent.atomic.AtomicBoolean
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow

/** 手机录音（16kHz 单声道 16 位 PCM，最长 20 秒）和播放电脑合成好的语音。 */
class AndroidVoice(private val context: Context, private val requestMic: suspend () -> Boolean) {
    private val captureLock = Any()
    private val main = Handler(Looper.getMainLooper())
    private val visibility = MutableStateFlow(false)
    val foreground = visibility.asStateFlow()
    private var pushToTalk: PushToTalk? = null
    private var callCapture: CallCapture? = null
    private var player: MediaPlayer? = null

    fun setForeground(active: Boolean) {
        visibility.value = active
        if (!active) {
            stop()
            synchronized(captureLock) { callCapture }?.halt("小伴已退到后台，通话已结束。回到前台可重新开始。")
            stopPlayback()
        }
    }

    private suspend fun canRecord(): Boolean =
        visibility.value && (context.checkSelfPermission(Manifest.permission.RECORD_AUDIO) == PackageManager.PERMISSION_GRANTED || requestMic())

    private fun createRecord(source: Int, bufferSize: Int): AudioRecord? {
        val rec = try {
            AudioRecord(source, SampleRate, AudioFormat.CHANNEL_IN_MONO, AudioFormat.ENCODING_PCM_16BIT, bufferSize)
        } catch (_: RuntimeException) { return null }
        if (rec.state == AudioRecord.STATE_INITIALIZED) return rec
        rec.release()
        return null
    }

    suspend fun start(): Boolean {
        if (!canRecord()) return false
        val minimum = AudioRecord.getMinBufferSize(SampleRate, AudioFormat.CHANNEL_IN_MONO, AudioFormat.ENCODING_PCM_16BIT)
        if (minimum <= 0) return false
        return synchronized(captureLock) {
            if (!visibility.value || callCapture != null) return@synchronized false
            if (pushToTalk != null) return@synchronized true
            val rec = createRecord(MediaRecorder.AudioSource.MIC, minimum * 4) ?: return@synchronized false
            val capture = PushToTalk(rec, minimum)
            pushToTalk = capture
            try {
                capture.begin()
                true
            } catch (_: RuntimeException) {
                capture.release()
                false
            }
        }
    }

    fun stop(): ByteArray? = synchronized(captureLock) { pushToTalk }?.finish()

    private inner class PushToTalk(private val record: AudioRecord, private val chunkSize: Int) {
        private val running = AtomicBoolean(true)
        private val bytes = ByteArrayOutputStream()

        fun begin() {
            record.startRecording()
            check(record.recordingState == AudioRecord.RECORDSTATE_RECORDING)
            Thread({
                try {
                    val chunk = ByteArray(chunkSize)
                    while (running.get()) {
                        val n = record.read(chunk, 0, chunk.size)
                        if (n < 0) break
                        if (n > 0) synchronized(bytes) {
                            if (running.get()) bytes.write(chunk, 0, minOf(n, MaxBytes - bytes.size()))
                        }
                    }
                } catch (_: RuntimeException) {
                    // The caller sees an empty recording after a revoked or failed input.
                } finally { release() }
            }, "xiaoban-voice-input").start()
        }

        fun finish(): ByteArray? {
            if (!running.compareAndSet(true, false)) return null
            runCatching { record.stop() }
            return synchronized(bytes) { bytes.toByteArray() }
        }

        fun release() {
            running.set(false)
            runCatching { record.stop() }
            runCatching { record.release() }
            synchronized(captureLock) { if (pushToTalk === this) pushToTalk = null }
        }
    }

    /** The stream owns its input, focus, and callbacks until it releases the device. */
    suspend fun stream(onFrame: (ByteArray) -> Unit, onFailure: (String) -> Unit): (() -> Unit)? {
        if (!canRecord()) return null
        val minimum = AudioRecord.getMinBufferSize(SampleRate, AudioFormat.CHANNEL_IN_MONO, AudioFormat.ENCODING_PCM_16BIT)
        if (minimum <= 0) return null
        return synchronized(captureLock) {
            if (!visibility.value || callCapture != null || pushToTalk != null) return@synchronized null
            val rec = createRecord(MediaRecorder.AudioSource.VOICE_COMMUNICATION, maxOf(minimum, FrameBytes * 8))
                ?: return@synchronized null
            val capture = CallCapture(rec, onFrame, onFailure)
            callCapture = capture
            try {
                capture.begin()
                val stop: () -> Unit = { capture.halt() }
                stop
            } catch (e: RuntimeException) {
                capture.release()
                onFailure(e.message?.takeIf { it == FocusUnavailable } ?: "无法启动麦克风，请检查权限后重试。")
                null
            }
        }
    }

    private inner class CallCapture(
        private val record: AudioRecord,
        private val onFrame: (ByteArray) -> Unit,
        private val onFailure: (String) -> Unit,
    ) {
        private val running = AtomicBoolean(true)
        private val audio = context.getSystemService(AudioManager::class.java)
        private val effects = mutableListOf<AudioEffect>()
        private val focusListener = AudioManager.OnAudioFocusChangeListener { focus ->
            if (focus < 0) halt("音频焦点已被其他应用占用，通话已结束。")
        }
        private var focusRequest: AudioFocusRequest? = null
        private var hasFocus = false
        private var deviceId: Int? = null
        private var headsetIds = emptySet<Int>()
        private val routing = AudioRouting.OnRoutingChangedListener { route ->
            val next = route.routedDevice?.id
            if (deviceId != null && next != deviceId) halt(RouteChanged)
            deviceId = next
        }
        private val devices = object : AudioDeviceCallback() {
            override fun onAudioDevicesAdded(addedDevices: Array<out AudioDeviceInfo>) = checkHeadsets()
            override fun onAudioDevicesRemoved(removedDevices: Array<out AudioDeviceInfo>) = checkHeadsets()
        }
        private val recording = object : AudioManager.AudioRecordingCallback() {
            override fun onRecordingConfigChanged(configs: List<AudioRecordingConfiguration>) {
                if (Build.VERSION.SDK_INT >= 29 && configs.any { it.clientAudioSessionId == record.audioSessionId && it.isClientSilenced }) {
                    halt("系统暂停了麦克风输入，通话已结束。")
                }
            }
        }

        private fun currentHeadsets(): Set<Int> = audio.getDevices(AudioManager.GET_DEVICES_ALL)
            .filter { it.type in HeadsetTypes }.map { it.id }.toSet()

        private fun checkHeadsets() {
            if (running.get() && currentHeadsets() != headsetIds) halt(RouteChanged)
        }

        @Suppress("DEPRECATION")
        fun begin() {
            val callAttributes = AudioAttributes.Builder().setUsage(AudioAttributes.USAGE_MEDIA)
                .setContentType(AudioAttributes.CONTENT_TYPE_SPEECH).build()
            val granted = if (Build.VERSION.SDK_INT >= 26) {
                val request = AudioFocusRequest.Builder(AudioManager.AUDIOFOCUS_GAIN_TRANSIENT)
                    .setAudioAttributes(callAttributes).setOnAudioFocusChangeListener(focusListener, main).build()
                focusRequest = request
                audio.requestAudioFocus(request)
            } else audio.requestAudioFocus(focusListener, AudioManager.STREAM_MUSIC, AudioManager.AUDIOFOCUS_GAIN_TRANSIENT)
            check(granted == AudioManager.AUDIOFOCUS_REQUEST_GRANTED) { FocusUnavailable }
            hasFocus = true
            if (AcousticEchoCanceler.isAvailable()) enableEffect { AcousticEchoCanceler.create(record.audioSessionId) }
            if (NoiseSuppressor.isAvailable()) enableEffect { NoiseSuppressor.create(record.audioSessionId) }
            record.startRecording()
            check(running.get() && record.recordingState == AudioRecord.RECORDSTATE_RECORDING)
            deviceId = record.routedDevice?.id
            headsetIds = currentHeadsets()
            record.addOnRoutingChangedListener(routing, main)
            audio.registerAudioDeviceCallback(devices, main)
            if (Build.VERSION.SDK_INT >= 29) record.registerAudioRecordingCallback(context.mainExecutor, recording)
            Thread(::read, "xiaoban-voice-call").start()
        }

        private fun enableEffect(create: () -> AudioEffect?) {
            val effect = runCatching(create).getOrNull() ?: return
            effects += effect
            runCatching { effect.enabled = true }
        }

        private fun read() {
            try {
                val frame = ByteArray(FrameBytes)
                var filled = 0
                var emptyReads = 0
                while (running.get()) {
                    val n = record.read(frame, filled, frame.size - filled)
                    if (n < 0 || n == 0 && ++emptyReads > 100) {
                        halt("麦克风输入已中断，通话已结束。")
                        break
                    }
                    if (n == 0) { Thread.sleep(10); continue }
                    emptyReads = 0
                    filled += n
                    if (filled == frame.size && running.get()) {
                        onFrame(frame.copyOf())
                        filled = 0
                    }
                }
            } catch (_: Exception) {
                halt("麦克风输入已中断，通话已结束。")
            } finally { release() }
        }

        fun halt(reason: String? = null) {
            if (!running.compareAndSet(true, false)) return
            runCatching { record.stop() } // Unblocks a pending read.
            if (reason != null) onFailure(reason)
        }

        @Suppress("DEPRECATION")
        fun release() {
            running.set(false)
            runCatching { record.stop() }
            if (Build.VERSION.SDK_INT >= 29) runCatching { record.unregisterAudioRecordingCallback(recording) }
            runCatching { record.removeOnRoutingChangedListener(routing) }
            runCatching { audio.unregisterAudioDeviceCallback(devices) }
            effects.forEach { runCatching { it.release() } }
            runCatching { record.release() }
            if (hasFocus) runCatching {
                if (Build.VERSION.SDK_INT >= 26) focusRequest?.let { audio.abandonAudioFocusRequest(it) }
                else audio.abandonAudioFocus(focusListener)
            }
            synchronized(captureLock) { if (callCapture === this) callCapture = null }
        }
    }

    /** 系统的回声消除可用，或者戴着耳机：小伴自己的声音不会被当成用户开口。 */
    fun echoSafe(): Boolean {
        val audio = context.getSystemService(Context.AUDIO_SERVICE) as AudioManager
        val headset = audio.getDevices(AudioManager.GET_DEVICES_OUTPUTS).any { it.type in HeadsetTypes }
        return AcousticEchoCanceler.isAvailable() || headset
    }

    /** 用系统自带的语音识别听一句话（本地识别和云端都用不了时的最后备选）。 */
    fun listenOnce(onPartial: (String) -> Unit, onResult: (String?, String?) -> Unit): (() -> Unit)? {
        if (!visibility.value || !SpeechRecognizer.isRecognitionAvailable(context)) return null
        if (context.checkSelfPermission(Manifest.permission.RECORD_AUDIO) != PackageManager.PERMISSION_GRANTED) {
            onResult(null, "没有麦克风权限")
            return {}
        }
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
            if (finished) return@post
            if (!visibility.value) { finish(null, "小伴已退到后台，通话已结束。"); return@post }
            val r = try { SpeechRecognizer.createSpeechRecognizer(context) } catch (_: RuntimeException) {
                finish(null, "无法启动手机语音识别")
                return@post
            }
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
            val request = Intent(RecognizerIntent.ACTION_RECOGNIZE_SPEECH).apply {
                putExtra(RecognizerIntent.EXTRA_LANGUAGE_MODEL, RecognizerIntent.LANGUAGE_MODEL_FREE_FORM)
                putExtra(RecognizerIntent.EXTRA_LANGUAGE, "zh-CN")
                putExtra(RecognizerIntent.EXTRA_PARTIAL_RESULTS, true)
            }
            try { r.startListening(request) }
            catch (_: RuntimeException) { finish(null, "无法启动手机语音识别") }
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

    /** 正在播的这段播完要告诉谁；被别人停掉也要告诉，不然等它的通话会一直等下去。只说一次。 */
    private var finishCurrent: (() -> Unit)? = null

    @Synchronized
    fun play(bytes: ByteArray, onDone: () -> Unit): Boolean {
        if (!visibility.value) return false
        stopPlayback()
        var told = false
        val finish = { if (!told) { told = true; onDone() } }
        return try {
            val file = File(context.cacheDir, "reply-voice-${playSeq.incrementAndGet() % 4}.mp3")
            file.writeBytes(bytes)
            val mp = MediaPlayer()
            player = mp
            // 使用媒体流属性播放，避免电话通道硬件降噪与窄带滤波导致的破音与沙沙声
            mp.setAudioAttributes(AudioAttributes.Builder().setUsage(AudioAttributes.USAGE_MEDIA).setContentType(AudioAttributes.CONTENT_TYPE_SPEECH).build())
            mp.setDataSource(file.absolutePath)
            mp.setOnCompletionListener { if (player === mp) stopPlayback() else finish() }
            mp.setOnErrorListener { _, _, _ -> if (player === mp) stopPlayback() else finish(); true }
            mp.prepare()
            mp.start()
            finishCurrent = finish
            true
        } catch (e: Exception) {
            stopPlayback()
            false
        }
    }

    @Synchronized
    fun stopPlayback() {
        player?.let { runCatching { it.stop() }; it.release() }
        player = null
        finishCurrent?.let { finishCurrent = null; it() }
    }

    private companion object {
        const val FocusUnavailable = "无法取得音频焦点，请结束其他通话后重试。"
        const val RouteChanged = "耳机或麦克风已切换，通话已结束。可以重新开始。"
        const val SampleRate = 16_000
        const val FrameBytes = 640
        val HeadsetTypes = setOf(
            AudioDeviceInfo.TYPE_WIRED_HEADSET, AudioDeviceInfo.TYPE_WIRED_HEADPHONES, AudioDeviceInfo.TYPE_USB_HEADSET,
            AudioDeviceInfo.TYPE_BLUETOOTH_SCO, AudioDeviceInfo.TYPE_BLUETOOTH_A2DP, AudioDeviceInfo.TYPE_BLE_HEADSET,
        )
        const val MaxBytes = 20 * SampleRate * 2
    }
}
