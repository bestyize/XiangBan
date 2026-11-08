package xyz.xiangban.xiaoban

import org.json.JSONObject
import java.io.File
import java.util.concurrent.Executors

/** 手机里自己做的作品的试玩记录，写在 workspace/logs/<id>.txt，供 Agent 的 app_runtime_read 读取。 */
internal class RuntimeV2Log(private val file: File) {
    private val executor = Executors.newSingleThreadExecutor { Thread(it, "runtime-v2-log").apply { isDaemon = true } }
    private var windowStart = 0L
    private var windowCount = 0

    fun reset() = submit { write(emptyList()) }

    fun add(type: String, vararg fields: Pair<String, Any?>) {
        val now = System.currentTimeMillis()
        val allowed = synchronized(this) {
            if (now - windowStart >= 1000) { windowStart = now; windowCount = 0 }
            ++windowCount <= 10
        }
        if (!allowed) return
        val line = JSONObject().put("type", type).put("time", now).also { json -> fields.forEach { (key, value) -> json.put(key, value) } }.toString()
        if (line.toByteArray(Charsets.UTF_8).size > MAX_LINE_BYTES) return
        submit {
            val old = if (file.isFile && file.length() <= MAX_BYTES) file.readLines(Charsets.UTF_8) else emptyList()
            val lines = (old + line).takeLast(MAX_LINES).toMutableList()
            while (lines.sumOf { it.toByteArray(Charsets.UTF_8).size + 1 } > MAX_BYTES) lines.removeAt(0)
            write(lines)
        }
    }

    fun close() = executor.shutdown()

    private fun submit(block: () -> Unit) {
        runCatching { executor.execute { runCatching(block) } }
    }

    private fun write(lines: List<String>) {
        val parent = file.parentFile ?: return
        if (!parent.isDirectory && !parent.mkdirs()) return
        val temporary = File.createTempFile("xb-log-", ".tmp", parent)
        try {
            temporary.writeText(lines.joinToString("") { "$it\n" }, Charsets.UTF_8)
            check(temporary.renameTo(file))
        } finally { temporary.delete() }
    }

    private companion object {
        const val MAX_LINES = 100
        const val MAX_BYTES = 64 * 1024
        const val MAX_LINE_BYTES = 8 * 1024
    }
}
