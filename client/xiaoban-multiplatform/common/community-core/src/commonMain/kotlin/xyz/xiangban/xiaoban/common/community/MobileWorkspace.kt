package xyz.xiangban.xiaoban.common.community

import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.serialization.json.*

/** Paths are relative to the app's private workspace, never paths in the phone's shared storage. */
data class WorkspaceEntry(val path: String, val directory: Boolean, val size: Long = 0)

interface MobileWorkspace {
    suspend fun list(path: String = "", recursive: Boolean = false): List<WorkspaceEntry>
    suspend fun read(path: String): String
    suspend fun write(path: String, content: String)
    suspend fun remove(path: String, recursive: Boolean = false)
}

object MobileWorkspaceLimits {
    const val FILE_BYTES = 1024 * 1024
    const val TOTAL_BYTES = 32L * 1024 * 1024
    const val ENTRIES = 2048

    fun path(path: String, allowRoot: Boolean = false): String {
        if (allowRoot && path.isEmpty()) return path
        require(path.length <= 512 && path.isNotEmpty()) { "需要工作区相对路径，不能操作工作区根目录" }
        val segments = path.split('/')
        require(segments.size <= 8 && segments.all { segment ->
            segment.length in 1..100 && !segment.startsWith('.') &&
                segment.all { it in 'a'..'z' || it in 'A'..'Z' || it in '0'..'9' || it in "._-" }
        }) { "路径只能用字母、数字、._- 和 /，最多 8 层；不能含绝对路径、隐藏目录或 .." }
        return path
    }

    fun text(content: String): Int {
        require('\u0000' !in content) { "工作区只支持 UTF-8 文本，不支持二进制文件" }
        val bytes = content.encodeToByteArray().size
        require(bytes <= FILE_BYTES) { "单个工作区文件不能超过 1 MB" }
        return bytes
    }
}

/** Platform implementations must refuse symlinks before every filesystem operation. */
abstract class ValidatedMobileWorkspace : MobileWorkspace {
    private val lock = Mutex()
    protected abstract suspend fun entries(path: String, recursive: Boolean): List<WorkspaceEntry>
    protected abstract suspend fun readText(path: String): String
    protected abstract suspend fun writeText(path: String, content: String)
    protected abstract suspend fun removeEntry(path: String, recursive: Boolean)

    final override suspend fun list(path: String, recursive: Boolean): List<WorkspaceEntry> = lock.withLock {
        entries(MobileWorkspaceLimits.path(path, allowRoot = true), recursive).sortedBy { it.path }
    }

    final override suspend fun read(path: String): String = lock.withLock {
        readText(MobileWorkspaceLimits.path(path)).also { MobileWorkspaceLimits.text(it) }
    }

    final override suspend fun write(path: String, content: String) = lock.withLock {
        MobileWorkspaceLimits.path(path)
        val bytes = MobileWorkspaceLimits.text(content)
        val current = entries("", true)
        val old = current.firstOrNull { it.path == path }
        require(old?.directory != true) { "目标是文件夹" }
        require(current.sumOf { it.size } - (old?.size ?: 0) + bytes <= MobileWorkspaceLimits.TOTAL_BYTES) { "工作区总量不能超过 32 MB" }
        val ancestors = path.split('/').dropLast(1).runningReduce { a, b -> "$a/$b" }
        require(current.size + (if (old == null) 1 else 0) + ancestors.count { prefix -> current.none { it.path == prefix } } <= MobileWorkspaceLimits.ENTRIES) {
            "工作区文件和目录数量已达上限"
        }
        writeText(path, content)
    }

    final override suspend fun remove(path: String, recursive: Boolean) = lock.withLock {
        removeEntry(MobileWorkspaceLimits.path(path), recursive)
    }
}

/** Useful for unsupported platforms and deterministic tests; not persistent. */
class MemoryMobileWorkspace : ValidatedMobileWorkspace() {
    private val files = linkedMapOf<String, String>()
    private val directories = linkedSetOf<String>()
    override suspend fun entries(path: String, recursive: Boolean): List<WorkspaceEntry> {
        require(path.isEmpty() || path in directories) { "目录不存在：$path" }
        val prefix = if (path.isEmpty()) "" else "$path/"
        return (directories.map { WorkspaceEntry(it, true) } + files.map { (p, value) -> WorkspaceEntry(p, false, value.encodeToByteArray().size.toLong()) })
            .filter { it.path.startsWith(prefix) && it.path != path && (recursive || '/' !in it.path.removePrefix(prefix)) }
    }
    override suspend fun readText(path: String): String = files[path] ?: error("文件不存在：$path")
    override suspend fun writeText(path: String, content: String) {
        val parents = path.split('/').dropLast(1).runningReduce { a, b -> "$a/$b" }
        require(parents.none { it in files }) { "路径的上级是文件" }
        directories.addAll(parents)
        files[path] = content
    }
    override suspend fun removeEntry(path: String, recursive: Boolean) {
        if (path in directories) {
            val children = files.keys.filter { it.startsWith("$path/") }
            val subdirectories = directories.filter { it.startsWith("$path/") }
            require(recursive || children.isEmpty() && subdirectories.isEmpty()) { "目录非空；删除整个目录需要 recursive=true" }
            children.forEach(files::remove)
            directories.removeAll(subdirectories.toSet())
            directories.remove(path)
        } else require(files.remove(path) != null) { "文件不存在：$path" }
    }
}

private class WorkspaceTool(
    override val name: String,
    override val description: String,
    override val risk: String,
    properties: JsonObject,
    required: List<String> = emptyList(),
    private val action: suspend (JsonObject) -> JsonElement,
) : AgentTool {
    override val parameters = buildJsonObject {
        put("type", "object"); put("properties", properties)
        put("required", JsonArray(required.map(::JsonPrimitive)))
        put("additionalProperties", false)
    }
    override suspend fun run(arguments: JsonObject): AgentToolResult = try {
        AgentToolResult(true, action(arguments).toString())
    } catch (e: CancellationException) { throw e
    } catch (e: Exception) { AgentToolResult(false, e.message ?: "工作区操作失败") }
}

private fun JsonObject.workspaceString(key: String, default: String? = null): String =
    if (key !in this && default != null) default else (this[key] as? JsonPrimitive)?.takeIf { it.isString }?.content ?: error("$key 必须是字符串")
private fun JsonObject.workspaceBoolean(key: String, default: Boolean = false): Boolean =
    this[key]?.let { (it as? JsonPrimitive)?.takeIf { p -> !p.isString }?.booleanOrNull ?: error("$key 必须是布尔值") } ?: default
private fun workspaceProperties(vararg fields: Pair<String, String>): JsonObject = buildJsonObject {
    fields.forEach { (name, type) -> put(name, buildJsonObject { put("type", type) }) }
}
private fun WorkspaceEntry.json() = buildJsonObject { put("path", path); put("directory", directory); put("size", size) }

/** Desktop-style names make projects and tool instructions portable between clients. */
fun mobileWorkspaceTools(workspace: MobileWorkspace): List<AgentTool> = listOf(
    WorkspaceTool("list_dir", "列出手机私有工作区的文件。path 默认根目录；recursive=true 递归列出。路径都相对工作区。", "read",
        workspaceProperties("path" to "string", "recursive" to "boolean")) { args ->
        JsonArray(workspace.list(args.workspaceString("path", ""), args.workspaceBoolean("recursive")).map { it.json() })
    },
    WorkspaceTool("read_file", "读取工作区 UTF-8 文本。offset 为从 0 开始的行号，limit 默认 200、最多 1000；输出 next_offset 可继续读。", "read",
        workspaceProperties("path" to "string", "offset" to "integer", "limit" to "integer"), listOf("path")) { args ->
        val lines = workspace.read(args.workspaceString("path")).split('\n')
        val offset = args["offset"]?.let { (it as? JsonPrimitive)?.intOrNull ?: error("offset 必须是整数") } ?: 0
        val limit = args["limit"]?.let { (it as? JsonPrimitive)?.intOrNull ?: error("limit 必须是整数") } ?: 200
        require(offset >= 0 && limit in 1..1000) { "offset 必须非负，limit 要在 1 到 1000 之间" }
        val selected = mutableListOf<String>()
        var size = 0
        for (line in lines.drop(offset).take(limit)) {
            // A single very long line still progresses; content is capped for tool context size.
            require(line.length <= 65536) { "单行超过 64 KB，请先拆行再读取" }
            if (selected.isNotEmpty() && size + line.length > 65536) break
            selected += line; size += line.length + 1
        }
        buildJsonObject {
            put("path", args.workspaceString("path")); put("content", selected.joinToString("\n"))
            put("offset", offset); put("total_lines", lines.size)
            if (offset + selected.size < lines.size) put("next_offset", offset + selected.size)
        }
    },
    WorkspaceTool("write_file", "创建或覆盖工作区文本文件，自动创建目录。单文件最多 1 MB，工作区总量 32 MB。", "write",
        workspaceProperties("path" to "string", "content" to "string"), listOf("path", "content")) { args ->
        val path = args.workspaceString("path"); workspace.write(path, args.workspaceString("content"))
        buildJsonObject { put("path", path); put("saved", true) }
    },
    WorkspaceTool("edit_file", "精确替换工作区文件中的文本。默认要求 old_text 仅匹配一次；replace_all=true 时替换所有匹配。", "write",
        workspaceProperties("path" to "string", "old_text" to "string", "new_text" to "string", "replace_all" to "boolean"), listOf("path", "old_text", "new_text")) { args ->
        val path = args.workspaceString("path"); val old = args.workspaceString("old_text"); val replacement = args.workspaceString("new_text")
        require(old.isNotEmpty()) { "old_text 不能为空" }
        val content = workspace.read(path)
        var count = 0
        var cursor = 0
        while (true) {
            val match = content.indexOf(old, cursor)
            if (match < 0) break
            count += 1; cursor = match + old.length
        }
        require(count > 0) { "没有找到 old_text，文件未修改" }
        require(count == 1 || args.workspaceBoolean("replace_all")) { "old_text 有多个匹配，请提供更准确的上下文或 replace_all=true" }
        workspace.write(path, content.replace(old, replacement))
        buildJsonObject { put("path", path); put("replacements", count) }
    },
    WorkspaceTool("grep", "在工作区文本文件中搜索字面文本，返回路径、行号和内容；最多 100 个匹配。path 默认为根目录。", "read",
        workspaceProperties("query" to "string", "path" to "string"), listOf("query")) { args ->
        val query = args.workspaceString("query"); require(query.isNotEmpty() && query.length <= 1000) { "query 长度须为 1 到 1000" }
        val matches = mutableListOf<JsonElement>()
        var truncated = false
        for (entry in workspace.list(args.workspaceString("path", ""), true).filterNot { it.directory }) {
            workspace.read(entry.path).lineSequence().forEachIndexed { index, line ->
                if (line.contains(query)) {
                    if (matches.size < 100) matches += buildJsonObject { put("path", entry.path); put("line", index + 1); put("text", line.take(500)) }
                    else truncated = true
                }
            }
            if (truncated) break
        }
        buildJsonObject { put("matches", JsonArray(matches)); put("truncated", truncated) }
    },
    WorkspaceTool("delete_file", "删除工作区文件或空目录；递归删除目录必须显式传 recursive=true。", "write",
        workspaceProperties("path" to "string", "recursive" to "boolean"), listOf("path")) { args ->
        val path = args.workspaceString("path"); workspace.remove(path, args.workspaceBoolean("recursive"))
        buildJsonObject { put("path", path); put("deleted", true) }
    },
)
