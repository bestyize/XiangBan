package xyz.xiangban.xiaoban

import android.os.Handler
import android.os.Looper
import android.util.AtomicFile
import android.util.Log
import org.json.JSONArray
import org.json.JSONObject
import java.io.File
import java.io.FileNotFoundException
import java.util.concurrent.Executors

/** One queue across host instances so reloads and concurrent opens cannot overwrite pending writes. */
internal object RuntimeV2Storage {
    const val MAX_MESSAGE_BYTES = 5 * 1024 * 1024
    private const val MAX_STORAGE_BYTES = 10 * 1024 * 1024
    private val queue = Executors.newSingleThreadExecutor()
    private val main = Handler(Looper.getMainLooper())

    fun submit(file: File, method: String, params: JSONObject, payload: String, reply: (Any?, String?) -> Unit) {
        queue.execute {
            val result = try {
                if (payload.toByteArray(Charsets.UTF_8).size > MAX_MESSAGE_BYTES) throw StorageError("quota")
                val key = if (method == "storage.keys") null else (params.opt("key") as? String)
                    ?.takeIf { it.codePointCount(0, it.length) <= 128 } ?: throw StorageError("invalid")
                val atomic = AtomicFile(file)
                val values = try {
                    atomic.openRead().bufferedReader().use { JSONObject(it.readText()) }
                } catch (error: FileNotFoundException) {
                    if (file.exists()) throw error
                    JSONObject()
                }
                when (method) {
                    "storage.get" -> values.opt(checkNotNull(key)) ?: JSONObject.NULL
                    "storage.keys" -> JSONArray(values.keys().asSequence().toList())
                    "storage.set" -> {
                        if (!params.has("value")) throw StorageError("invalid")
                        values.put(checkNotNull(key), params.get("value"))
                        write(atomic, values)
                        JSONObject.NULL
                    }
                    "storage.remove" -> {
                        if (values.remove(checkNotNull(key)) != null) write(atomic, values)
                        JSONObject.NULL
                    }
                    else -> throw StorageError("unsupported")
                } to null
            } catch (error: StorageError) {
                null to error.code
            } catch (error: Exception) {
                Log.w("RuntimeV2Storage", "作品存储操作失败", error)
                null to "busy"
            }
            main.post { reply(result.first, result.second) }
        }
    }

    fun afterWrites(callback: () -> Unit) { queue.execute { main.post(callback) } }

    private fun write(atomic: AtomicFile, values: JSONObject) {
        val bytes = values.toString().toByteArray(Charsets.UTF_8)
        if (bytes.size > MAX_STORAGE_BYTES) throw StorageError("quota")
        val stream = atomic.startWrite()
        try {
            stream.write(bytes)
            atomic.finishWrite(stream)
        } catch (error: Exception) {
            atomic.failWrite(stream)
            throw error
        }
    }

    private class StorageError(val code: String) : Exception()
}
