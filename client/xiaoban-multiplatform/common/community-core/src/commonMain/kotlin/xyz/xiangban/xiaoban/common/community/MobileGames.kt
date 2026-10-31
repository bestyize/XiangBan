package xyz.xiangban.xiaoban.common.community

import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.*

data class MobileGame(
    val id: String,
    val name: String,
    val description: String,
    val save: Boolean = false,
    val orientation: String = "portrait",
    val layer: String = "space",
    val capabilities: List<String> = emptyList(),
)

data class MobileGameFile(val path: String, val content: String)
data class MobileGameScan(val games: List<MobileGame>, val errors: List<String>)

object MobileGameFormat {
    const val FILES = 40
    const val FILE_BYTES = 1024 * 1024
    const val TOTAL_BYTES = 4 * 1024 * 1024
    const val GAMES = 40
    private val builtin = setOf("falling-blocks", "merge-2048", "block-puzzle", "water-sort", "snake-arena", "ink-blade", "dusk-glider", "pro-cube", "jade-gomoku")
    private val extensions = setOf("html", "js", "css", "json", "svg", "md", "txt")

    fun id(id: String) {
        require(id.length in 2..40 && id.first() in "abcdefghijklmnopqrstuvwxyz0123456789" && id.all { it in "abcdefghijklmnopqrstuvwxyz0123456789-" }) {
            "游戏 id 需要 2 到 40 个小写字母、数字或 -，以字母或数字开头"
        }
        require(id !in builtin) { "游戏 id 与桌面内置游戏重名：$id" }
    }

    fun metadata(game: MobileGame): MobileGame {
        id(game.id)
        // Rust str::trim also strips NEXT LINE (U+0085), which Kotlin's JVM trim does not.
        val normalized = game.copy(name = game.name.trim { it.isWhitespace() || it == '\u0085' }, description = game.description.trim { it.isWhitespace() || it == '\u0085' })
        fun length(text: String) = text.count { !it.isLowSurrogate() }
        require(normalized.name.isNotEmpty() && length(normalized.name) <= 24) { "游戏名称需要 1 到 24 个字" }
        require(normalized.description.isNotEmpty() && length(normalized.description) <= 80) { "游戏介绍需要 1 到 80 个字" }
        require(game.orientation in setOf("portrait", "landscape")) { "orientation 只能是 portrait 或 landscape" }
        require(game.layer == "space" && game.capabilities.isEmpty()) { "手机当前只支持普通游戏（layer=space）；桌面层与系统能力需要桌面宿主和 v2 资源包" }
        return normalized
    }

    fun path(path: String) {
        val segments = path.split('/')
        require(segments.size <= 4 && segments.all { segment ->
            segment.isNotEmpty() && !segment.startsWith('.') && segment.all { it in 'a'..'z' || it in 'A'..'Z' || it in '0'..'9' || it in "._-" }
        }) { "游戏路径最多 4 层，只能用字母、数字、._-，不能以 . 开头" }
        require(segments.first() != "__MACOSX") { "__MACOSX/ 是桌面 ZIP 导入时忽略的保留目录，不能用于游戏文件" }
        require(!segments.first().equals("manifest.json", ignoreCase = true) || segments.size == 1) { "manifest.json 保留给资源清单，不能作为目录" }
        require(path.substringAfterLast('.', "").lowercase() in extensions) { "首阶段只支持 html/js/css/json/svg/md/txt 文本；不支持二进制素材，请使用内联 SVG 或 data: 资源" }
    }

    fun files(files: List<MobileGameFile>) {
        require(files.size in 1..FILES) { "游戏需要 1 到 40 个文本文件" }
        val seen = mutableSetOf<String>()
        var total = 0L
        for (file in files) {
            path(file.path)
            require(!file.path.equals("manifest.json", ignoreCase = true)) { "manifest.json 由 game_create 自动生成，草稿或 files 不能包含它" }
            require(seen.add(file.path.lowercase())) { "游戏路径不能重名（忽略大小写）：${file.path}" }
            require('\u0000' !in file.content) { "游戏文件必须是 UTF-8 文本" }
            val size = file.content.encodeToByteArray().size
            require(size <= FILE_BYTES) { "游戏单文件不能超过 1 MB：${file.path}" }
            total += size
        }
        require(total <= TOTAL_BYTES) { "游戏文本合计不能超过 4 MB" }
        require(files.any { it.path == "index.html" }) { "游戏必须有入口 index.html" }
        require(seen.none { parent -> seen.any { it.startsWith("$parent/") } }) { "文件路径冲突：同一名字不能既是文件又是目录" }
    }

    val instructions = """
        小伴手机与小伴桌面兼容的普通小游戏格式（文本包 v1）：
        1. 用 write_file 在专用草稿目录（例如 drafts/my-game）逐步写文件，再 game_create(source_dir=该目录) 安装；也可传 files=[{path,content}]。两种来源只能选一种。manifest.json 由工具自动生成，草稿和 files 不要放它。
        2. id 长度 2~40，只用小写字母、数字、-，以字母数字开头；不能使用内置游戏 id。name 最多 24 字，description 必填且最多 80 字；save 默认 false；orientation 是 portrait（默认）或 landscape。手机目前只支持 layer=space，不支持桌面层或 capabilities。
        3. 必须包含 index.html，建议同时创作匹配玩法的 cover.svg（640×360）。文件仅 html/js/css/json/svg/md/txt，最多 40 个，单个最多 1 MB，合计最多 4 MB。路径最多 4 层，每段仅字母数字及 ._-，不能以 . 开头，不能忽略大小写重名。当前不能导入二进制素材，图形优先用 CSS、canvas 或内联 SVG。
        4. HTML/CSS/JS 离线自包含，相对路径引用脚本和样式。使用普通 script，不用 ES modules/import/export；不联网，不引用 CDN；不用 localStorage/cookie/IndexedDB，不访问系统权限。画布适配 100vw × 100vh，提供触控操作和可点击的重开按钮，不仅依赖键盘。
        5. 宿主协议：顶层同步监听 window.message，校验 event.source===window.parent、channel==='xiaban.game-host'、protocol===1、nonce===new URLSearchParams(location.search).get('hostNonce')。收到 host.init 后，用 window.parent.postMessage({channel:'xiaban.game-pack',protocol:1,nonce,type:'game.ready'},'*') 回复。host.init 的 save 是上次存档或 null，safeArea 为安全边距。
        6. game.ready 后可发 game.start、game.progress、game.result，均使用 channel:'xiaban.game-pack',protocol:1,nonce；成绩 score 为非负整数。save=true 时发 game.save，data 为普通 JSON 对象，序列化不超过 32 KiB；进度变化就保存，不等退出。不要自写 Esc 处理或引用 desktop-host.js。
        7. 安装后文件位于 games/<id>/；改文件后运行 game_validate 再试玩和导出。打包 ZIP 时 manifest.json 在最外层，字段为 schemaVersion:1,kind:'game',id,name,description,save,orientation,layer:'space'，其余游戏文本原样保留。存档与运行日志不进入资源包。
    """.trimIndent()
}

private val MobileGameJson = Json { ignoreUnknownKeys = true }
private fun JsonObject.gameString(key: String, default: String? = null): String =
    if (key !in this && default != null) default else (this[key] as? JsonPrimitive)?.takeIf { it.isString }?.content ?: throw IllegalArgumentException("$key 必须是字符串")
private fun JsonObject.gameBoolean(key: String, default: Boolean = false): Boolean =
    this[key]?.let { (it as? JsonPrimitive)?.takeIf { p -> !p.isString }?.booleanOrNull ?: throw IllegalArgumentException("$key 必须是布尔值") } ?: default
private fun MobileGame.json(): JsonObject = buildJsonObject {
    put("schemaVersion", 1); put("kind", "game"); put("id", id); put("name", name); put("description", description)
    put("save", save); put("orientation", orientation); put("layer", layer)
}

/** Keeps the desktop ZIP manifest in games/<id>/; saves and host logs belong outside that folder. */
class MobileGameStore(val workspace: MobileWorkspace) {
    private val lock = Mutex()

    private suspend fun exists(id: String): Boolean = workspace.list().any { it.path == "games" && it.directory } &&
        workspace.list("games").any { it.path == "games/$id" }

    suspend fun list(): List<MobileGame> = scan().games

    suspend fun scan(): MobileGameScan {
        val root = workspace.list().firstOrNull { it.path == "games" } ?: return MobileGameScan(emptyList(), emptyList())
        if (!root.directory) return MobileGameScan(emptyList(), listOf("games 必须是目录"))
        val games = mutableListOf<MobileGame>()
        val errors = mutableListOf<String>()
        for (entry in workspace.list("games").filter { it.directory }) {
            try { games += validate(entry.path.substringAfterLast('/'))
            } catch (e: CancellationException) { throw e
            } catch (e: Exception) { errors += "${entry.path}：${e.message ?: "游戏校验失败"}" }
        }
        return MobileGameScan(games.sortedBy { it.name }, errors)
    }

    suspend fun read(id: String): MobileGame {
        MobileGameFormat.id(id)
        val text = workspace.read("games/$id/manifest.json")
        require(text.encodeToByteArray().size <= 64 * 1024) { "manifest.json 不能超过 64 KB" }
        val json = MobileGameJson.parseToJsonElement(text) as? JsonObject ?: throw IllegalArgumentException("manifest.json 需要是 JSON 对象")
        require((json["schemaVersion"] as? JsonPrimitive)?.let { !it.isString && it.intOrNull == 1 } == true && json.gameString("kind") == "game") {
            "手机当前只支持 schemaVersion=1、kind=game 的文本游戏包"
        }
        val capabilities = json["capabilities"]?.let { value ->
            val array = value as? JsonArray ?: throw IllegalArgumentException("capabilities 必须是列表")
            array.map { (it as? JsonPrimitive)?.takeIf { p -> p.isString }?.content ?: throw IllegalArgumentException("capabilities 必须是字符串列表") }
        } ?: emptyList()
        val game = MobileGameFormat.metadata(MobileGame(
            json.gameString("id"), json.gameString("name"), json.gameString("description"), json.gameBoolean("save"),
            json.gameString("orientation", "portrait"), json.gameString("layer", "space"), capabilities,
        ))
        require(game.id == id) { "manifest id 与游戏目录不一致" }
        return game
    }

    private suspend fun contentFiles(id: String): List<MobileGameFile> {
        val entries = workspace.list("games/$id", recursive = true)
        require(entries.size <= MobileGameFormat.FILES * 4 + 1) { "游戏目录条目过多" }
        val files = entries.filterNot { it.directory || it.path == "games/$id/manifest.json" }
        require(files.size in 1..MobileGameFormat.FILES && files.sumOf { it.size } <= MobileGameFormat.TOTAL_BYTES) { "游戏需要 1 到 40 个文本文件，合计不超过 4 MB" }
        return files.map { entry ->
            MobileGameFile(entry.path.removePrefix("games/$id/"), workspace.read(entry.path))
        }
    }

    suspend fun validate(id: String): MobileGame {
        val game = read(id)
        MobileGameFormat.files(contentFiles(id))
        return game
    }

    suspend fun readFile(id: String, path: String): String {
        MobileGameFormat.id(id)
        if (path != "manifest.json") MobileGameFormat.path(path)
        return workspace.read("games/$id/$path")
    }

    /** Snapshot includes the normalized resource manifest, ready to write at the ZIP root. */
    suspend fun exportFiles(id: String): List<MobileGameFile> {
        val game = read(id)
        val files = contentFiles(id)
        MobileGameFormat.files(files)
        return listOf(MobileGameFile("manifest.json", game.json().toString())) + files.sortedBy { it.path }
    }

    suspend fun createFromDirectory(game: MobileGame, sourceDir: String, overwrite: Boolean = false): MobileGame {
        MobileWorkspaceLimits.path(sourceDir)
        val entries = workspace.list(sourceDir, recursive = true)
        require(entries.size <= MobileGameFormat.FILES * 4) { "游戏草稿目录条目过多" }
        val sourceFiles = entries.filterNot { it.directory }
        require(sourceFiles.size in 1..MobileGameFormat.FILES && sourceFiles.sumOf { it.size } <= MobileGameFormat.TOTAL_BYTES) { "游戏草稿需要 1 到 40 个文本文件，合计不超过 4 MB" }
        val files = sourceFiles.map {
            MobileGameFile(it.path.removePrefix("$sourceDir/"), workspace.read(it.path))
        }
        return create(game, files, overwrite)
    }

    suspend fun create(game: MobileGame, files: List<MobileGameFile>, overwrite: Boolean = false): MobileGame = lock.withLock {
        val normalized = MobileGameFormat.metadata(game)
        MobileGameFormat.files(files)
        val hadOld = exists(game.id)
        require(!hadOld || overwrite) { "游戏 id 已存在；修改已有游戏请显式设置 overwrite=true" }
        if (!hadOld) require(workspace.list().none { it.path == "games" } || workspace.list("games").count { it.directory } < MobileGameFormat.GAMES) {
            "最多保存 40 款自创游戏"
        }
        // Snapshot before any mutation, so invalid replacements and failed writes retain the original.
        val previous = if (hadOld) workspace.list("games/${game.id}", true).filterNot { it.directory }.map {
            MobileGameFile(it.path.removePrefix("games/${game.id}/"), workspace.read(it.path))
        } else emptyList()
        try {
            if (hadOld) workspace.remove("games/${game.id}", recursive = true)
            files.forEach { workspace.write("games/${game.id}/${it.path}", it.content) }
            // A directory is only discoverable as a valid game when its final manifest is present.
            workspace.write("games/${game.id}/manifest.json", normalized.json().toString())
        } catch (error: Exception) {
            withContext(NonCancellable) {
                try {
                    if (exists(game.id)) workspace.remove("games/${game.id}", recursive = true)
                    previous.forEach { workspace.write("games/${game.id}/${it.path}", it.content) }
                } catch (rollback: Exception) { error.addSuppressed(rollback) }
            }
            throw error
        }
        normalized
    }
}

private class MobileGameTool(
    override val name: String,
    override val description: String,
    override val parameters: JsonObject,
    override val risk: String = "read",
    private val action: suspend (JsonObject) -> JsonElement,
) : AgentTool {
    override suspend fun run(arguments: JsonObject): AgentToolResult = try {
        AgentToolResult(true, action(arguments).toString())
    } catch (e: CancellationException) { throw e
    } catch (e: Exception) { AgentToolResult(false, e.message ?: "游戏操作失败") }
}

private fun gameParameters(vararg fields: Pair<String, String>, required: List<String> = emptyList()): JsonObject = buildJsonObject {
    put("type", "object")
    put("properties", buildJsonObject { fields.forEach { (name, type) ->
        put(name, buildJsonObject {
            put("type", type)
            if (type == "array") put("items", if (name == "files") buildJsonObject {
                put("type", "object"); put("properties", buildJsonObject {
                    put("path", buildJsonObject { put("type", "string") }); put("content", buildJsonObject { put("type", "string") })
                }); put("required", JsonArray(listOf(JsonPrimitive("path"), JsonPrimitive("content"))))
            } else buildJsonObject { put("type", "string") })
        })
    } })
    put("required", JsonArray(required.map(::JsonPrimitive))); put("additionalProperties", false)
}

fun mobileGameTools(store: MobileGameStore): List<AgentTool> = listOf(
    MobileGameTool("game_read_format", "读取与小伴桌面兼容的手机小游戏完整创作规范，创作前先读取。", gameParameters()) {
        JsonPrimitive(MobileGameFormat.instructions)
    },
    MobileGameTool("game_list", "列出手机工作区中校验通过的小游戏。", gameParameters()) {
        val scan = store.scan()
        buildJsonObject { put("games", JsonArray(scan.games.map { it.json() })); put("errors", JsonArray(scan.errors.map(::JsonPrimitive))) }
    },
    MobileGameTool("game_validate", "校验游戏 manifest、入口、路径、文件类型与大小，试玩或导出前使用。", gameParameters("id" to "string", required = listOf("id"))) {
        store.validate(it.gameString("id")).json()
    },
    MobileGameTool("game_create", "安装一个桌面兼容的普通小游戏。source_dir（工作区内专用草稿目录）或 files 数组必须二选一；manifest 自动生成。已有 id 需要 overwrite=true。", gameParameters(
        "id" to "string", "name" to "string", "description" to "string", "save" to "boolean", "orientation" to "string",
        "layer" to "string", "capabilities" to "array", "source_dir" to "string", "files" to "array", "overwrite" to "boolean",
        required = listOf("id", "name", "description"),
    ), "write") { args ->
        val capabilities = (args["capabilities"] as? JsonArray)?.map {
            (it as? JsonPrimitive)?.takeIf { value -> value.isString }?.content ?: throw IllegalArgumentException("capabilities 必须是字符串列表")
        } ?: if ("capabilities" in args) throw IllegalArgumentException("capabilities 必须是列表") else emptyList()
        val game = MobileGame(args.gameString("id"), args.gameString("name"), args.gameString("description"), args.gameBoolean("save"),
            args.gameString("orientation", "portrait"), args.gameString("layer", "space"), capabilities)
        require(("source_dir" in args) != ("files" in args)) { "source_dir 和 files 必须二选一" }
        val created = if ("source_dir" in args) store.createFromDirectory(game, args.gameString("source_dir"), args.gameBoolean("overwrite")) else {
            val array = args["files"] as? JsonArray ?: throw IllegalArgumentException("files 必须是数组")
            store.create(game, array.map { value ->
                val file = value as? JsonObject ?: throw IllegalArgumentException("files 的每项需要 path 与 content")
                MobileGameFile(file.gameString("path"), file.gameString("content"))
            }, args.gameBoolean("overwrite"))
        }
        created.json()
    },
)
