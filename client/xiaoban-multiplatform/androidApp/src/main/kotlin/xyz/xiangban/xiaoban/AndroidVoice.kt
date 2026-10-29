package xyz.xiangban.xiaoban

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import android.media.AudioFormat
import android.media.AudioRecord
import android.media.MediaPlayer
import android.media.MediaRecorder
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

    fun play(bytes: ByteArray, onDone: () -> Unit): Boolean {
        stopPlayback()
        return try {
            val file = File(context.cacheDir, "reply-voice.mp3")
            file.writeBytes(bytes)
            val mp = MediaPlayer()
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
        const val MaxBytes = 20 * SampleRate * 2
    }
}
