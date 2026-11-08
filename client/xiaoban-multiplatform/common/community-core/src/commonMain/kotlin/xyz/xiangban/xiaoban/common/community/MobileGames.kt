package xyz.xiangban.xiaoban.common.community

import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.*

/** 一个协议 2 作品（游戏或工具）。清单字段与 docs/architecture/app-runtime 的第 3 节一致。 */
data class MobileGame(
    val id: String,
    val name: String,
    val description: String,
    val orientation: String = "portrait",
    val kind: String = "game",
    val category: String = "casual",
    val input: List<String> = listOf("touch", "mouse", "keyboard"),
    val capabilities: List<String> = emptyList(),
    val version: String = "1.0.0",
)

data class MobileGameFile(val path: String, val content: String)
data class MobileGameScan(val games: List<MobileGame>, val errors: List<String>)

object MobileGameFormat {
    const val FILES = 100
    const val FILE_BYTES = 1024 * 1024
    const val TOTAL_BYTES = 8 * 1024 * 1024
    const val GAMES = 40
    private const val MAX_DEPTH = 6
    private val builtin = setOf("falling-blocks", "merge-2048", "block-puzzle", "water-sort", "snake-arena", "ink-blade", "dusk-glider", "pro-cube", "jade-gomoku", "mbti")
    private val extensions = setOf("html", "js", "mjs", "css", "json", "svg", "md", "txt")
    private val gameCategories = setOf("action", "puzzle", "board", "casual", "story")
    private val appCategories = setOf("tool", "quiz")
    private val inputs = setOf("touch", "mouse", "keyboard")
    private val allCapabilities = setOf("haptics", "share", "clipboard", "screen", "notify", "profile", "camera", "microphone", "contacts", "files")
    private val versionPattern = Regex("[a-zA-Z0-9.-]{1,32}")
    val required = listOf("index.html", "icon.svg", "cover.svg")

    fun id(id: String) {
        require(id.length in 2..40 && id.first() in "abcdefghijklmnopqrstuvwxyz0123456789" && id.last() != '-' &&
            id.all { it in "abcdefghijklmnopqrstuvwxyz0123456789-" }) {
            "作品 id 需要 2 到 40 个小写字母、数字或 -，以字母或数字开头和结尾"
        }
        require(id !in builtin) { "作品 id 与小伴内置作品重名：$id" }
    }

    fun metadata(game: MobileGame): MobileGame {
        id(game.id)
        // Rust str::trim also strips NEXT LINE (U+0085), which Kotlin's JVM trim does not.
        val normalized = game.copy(name = game.name.trim { it.isWhitespace() || it == '\u0085' }, description = game.description.trim { it.isWhitespace() || it == '\u0085' },
            version = game.version.trim())
        fun length(text: String) = text.count { !it.isLowSurrogate() }
        require(normalized.name.isNotEmpty() && length(normalized.name) <= 24) { "作品名称需要 1 到 24 个字" }
        require(normalized.description.isNotEmpty() && length(normalized.description) <= 80) { "作品介绍需要 1 到 80 个字" }
        require(normalized.version.matches(versionPattern)) { "version 只能含字母、数字、点和连字符，最长 32 位" }
        require(game.orientation in setOf("portrait", "landscape", "any")) { "orientation 只能是 portrait、landscape 或 any" }
        require(game.kind == "game" || game.kind == "app") { "kind 只能是 game 或 app" }
        val categories = if (game.kind == "game") gameCategories else appCategories
        require(game.category in categories) { "${game.kind} 的 category 只能取 ${categories.joinToString("、")}" }
        require(game.input.isNotEmpty() && game.input.all { it in inputs } && "touch" in game.input && game.input.distinct().size == game.input.size) {
            "input 只能取 touch、mouse、keyboard，且手机作品必须含 touch"
        }
        require(game.capabilities.all { it in allCapabilities } && game.capabilities.distinct().size == game.capabilities.size) {
            "capabilities 只能取 ${allCapabilities.joinToString("、")}"
        }
        return normalized
    }

    fun path(path: String) {
        val segments = path.split('/')
        require(segments.size <= MAX_DEPTH && segments.all { segment ->
            segment.isNotEmpty() && !segment.startsWith('.') && segment.all { it in 'a'..'z' || it in 'A'..'Z' || it in '0'..'9' || it in "._-" }
        }) { "文件路径最多 $MAX_DEPTH 层，只能用字母、数字、._-，不能以 . 开头" }
        require(segments.first() != "__MACOSX") { "__MACOSX/ 是压缩包常见的垃圾目录，不能用于作品文件" }
        require(!segments.first().equals("manifest.json", ignoreCase = true) || segments.size == 1) { "manifest.json 保留给作品清单，不能作为目录" }
        require(path.substringAfterLast('.', "").lowercase() in extensions) { "手机目前只能写 html/js/mjs/css/json/svg/md/txt 文本文件；图片请用 SVG 或 canvas 绘制" }
    }

    fun files(files: List<MobileGameFile>) {
        require(files.size in 1..FILES) { "作品需要 1 到 $FILES 个文本文件" }
        val seen = mutableSetOf<String>()
        var total = 0L
        for (file in files) {
            path(file.path)
            require(!file.path.equals("manifest.json", ignoreCase = true)) { "manifest.json 由 app_create 自动生成，草稿或 files 不能包含它" }
            require(seen.add(file.path.lowercase())) { "文件路径不能重名（忽略大小写）：${file.path}" }
            require('\u0000' !in file.content) { "作品文件必须是 UTF-8 文本" }
            val size = file.content.encodeToByteArray().size
            require(size <= FILE_BYTES) { "单个文件不能超过 1 MB：${file.path}" }
            total += size
        }
        require(total <= TOTAL_BYTES) { "作品文本合计不能超过 8 MB" }
        for (name in required) require(files.any { it.path == name }) { "作品必须有 $name（入口、1:1 图标、16:9 封面）" }
        require(seen.none { parent -> seen.any { it.startsWith("$parent/") } }) { "文件路径冲突：同一名字不能既是文件又是目录" }
    }

    val instructions = """
        小伴应用规范（运行时协议 2，手机创作版）：
        1. 你在做一个作品：游戏（kind=game）或工具、测评（kind=app），在手机上全屏运行。先给用户一个简短方案，写明类型、竖屏还是横屏、玩法或功能，确认后再动手。
        2. 免构建：写的源码就是最终运行的代码，没有 npm 和打包。用原生 JavaScript，可以用 <script type="module"> 和相对路径 import；不要依赖 CDN，手机上没有库可复制，需要的代码自己写。用 write_file 在专用草稿目录（例如 drafts/my-app）逐步写文件，再 app_create(source_dir=该目录) 安装；也可传 files=[{path,content}]，两者只能选一种。manifest.json 由工具生成，不要自己写。
        3. 文件：必须有 index.html（入口）、icon.svg（1:1 图标，至少 192×192 的比例）、cover.svg（16:9 封面，640×360），风格与作品一致。其余文件只能是 html/js/mjs/css/json/svg/md/txt 文本，最多 100 个，单个不超过 1 MB，合计不超过 8 MB；路径最多 6 层，每段只用字母数字和 ._-，不能以 . 开头。手机目前不能写图片、音频等二进制，图形请用 SVG、canvas 绘制，音效用 WebAudio 合成。
        4. 清单信息写在 app_create 的参数里：id（2 到 40 位小写字母、数字、-，不能与内置作品重名）、name（最多 24 字）、description（最多 80 字）、kind（game 或 app，默认 game）、category（游戏取 action、puzzle、board、casual、story，应用取 tool、quiz；默认游戏 casual、应用 tool）、orientation（portrait、landscape 或 any）、input（touch、mouse、keyboard，必须含 touch，默认三个都有）、capabilities（用到哪些系统能力，默认没有）、version（默认 1.0.0）。已有 id 需要 overwrite=true。
        5. 运行环境：作品是一个普通网站，有自己独立的 https 源。可以用 fetch、WebSocket、localStorage、IndexedDB、Worker、WebAssembly、Canvas、WebGL、WebAudio；只能访问 https/wss，不能访问 localhost 和局域网；不能用 alert、confirm、prompt、全屏 API、屏幕方向 API。联网只在确实需要实时数据时用，并处理断网：没网时给出可见提示，不要白屏。不要把密钥写进代码。
        6. 布局：页面占满 100vw × 100vh。右上角有宿主的“··· | ✕”胶囊，四周有状态栏和挖孔。用 CSS 变量 --xb-safe-top、--xb-safe-right、--xb-safe-bottom、--xb-safe-left 留出安全边距，用 --xb-capsule-top、--xb-capsule-right、--xb-capsule-width、--xb-capsule-height 计算胶囊区域，所有可点的控件（选关、暂停、结算、滚动后的控件）都要避开它。canvas 按 devicePixelRatio 绘制并监听 resize。按钮至少 44×44 像素。不要自己做左上角返回按钮，退出由宿主的胶囊负责。
        7. SDK：宿主已注入全局对象 xb，不要自己打包，也不要自己写 postMessage。第一屏画好后调用 xb.ready()（10 秒内必须调用，否则宿主判定启动失败）。xb.on('hide', fn) 暂停并停掉声音，xb.on('show', fn) 恢复。作品里有多个界面时用 history.pushState 记录、在 popstate 里切回上一个界面，系统返回键会逐层退回；不要拦截系统返回。xb.exit() 退出作品。xb.info 里有 safeArea、capsule、orientation、theme。
        8. 存档：重要数据用 await xb.storage.set(key, value) 和 await xb.storage.get(key)，值是 JSON，单个作品合计最多 10 MB；数据一变就存，不要等退出时才存，手机随时可能挂起作品。
        9. 游戏成绩（可选）：开始一局调 xb.game.start()，过程中 xb.game.progress({score})，结束时 xb.game.result({score})，score 是非负整数。
        10. 系统能力：capabilities 里写 camera、microphone 后，用标准的 navigator.mediaDevices.getUserMedia 使用，宿主会请用户授权；使用前先 if (xb.can('camera'))，不支持就隐藏对应按钮，作品照样可用。其余能力（haptics、share、clipboard、screen、notify、profile、contacts、files）手机宿主暂未开放，不要依赖。
        11. 声音和操作：用户第一次点击时调用 AudioContext.resume()，收到 show 后再调用一次，并提供静音开关。同时支持触控和键鼠；游戏里的运动都乘以帧间隔 dt，dt 最多取 1/30 秒。
        12. 内容：不卖东西、不引导外部付款，不收集手机号等隐私，不做色情、赌博、血腥内容，不在后台偷偷上传用户数据。
        13. 质量：游戏要有开始、进行、暂停、结算和重开的完整流程，随机局面保证有解，难度先低后高；工具和测评打开就能用；主要操作都有动画或震动之类的反馈；配色 4 到 6 种，主角、目标、危险物一眼能分清。
        14. 试用：安装后运行 app_validate，再请用户试玩（app_open），之后用 app_runtime_read 读加载、脚本报错和 xb.ready 的记录；没有实际试玩记录不能声称测试通过。作品安装在 games/<id>/，改完文件重新 app_validate。导出或分享给他人时，打包成的 ZIP 以 manifest.json 在最外层。
    """.trimIndent()
}

private val MobileGameJson = Json { ignoreUnknownKeys = true }
private fun JsonObject.gameString(key: String, default: String? = null): String =
    if (key !in this && default != null) default else (this[key] as? JsonPrimitive)?.takeIf { it.isString }?.content ?: throw IllegalArgumentException("$key 必须是字符串")
private fun JsonObject.gameStrings(key: String, default: List<String>): List<String> {
    val value = this[key] ?: return default
    val array = value as? JsonArray ?: throw IllegalArgumentException("$key 必须是列表")
    return array.map { (it as? JsonPrimitive)?.takeIf { p -> p.isString }?.content ?: throw IllegalArgumentException("$key 必须是字符串列表") }
}
private fun JsonObject.gameBoolean(key: String, default: Boolean = false): Boolean =
    this[key]?.let { (it as? JsonPrimitive)?.takeIf { p -> !p.isString }?.booleanOrNull ?: throw IllegalArgumentException("$key 必须是布尔值") } ?: default
private fun JsonObject.gameInt(key: String): Int? = (this[key] as? JsonPrimitive)?.takeIf { !it.isString }?.intOrNull

private fun MobileGame.json(): JsonObject = buildJsonObject {
    put("schemaVersion", 3); put("runtimeProtocol", 2); put("kind", kind); put("id", id); put("name", name); put("description", description)
    put("version", version); put("category", category); put("entry", "index.html"); put("icon", "icon.svg"); put("cover", "cover.svg")
    put("orientation", orientation); put("input", JsonArray(input.map(::JsonPrimitive)))
    if (capabilities.isNotEmpty()) put("capabilities", JsonArray(capabilities.map(::JsonPrimitive)))
}

fun readMobileGameManifest(text: String): MobileGame {
    require(text.encodeToByteArray().size <= 64 * 1024) { "manifest.json 不能超过 64 KB" }
    val json = MobileGameJson.parseToJsonElement(text) as? JsonObject ?: throw IllegalArgumentException("manifest.json 需要是 JSON 对象")
    require(json.gameInt("runtimeProtocol") == 2 && json.gameInt("schemaVersion") == 3) {
        "这是旧版（协议 1）作品，现在已不再支持；请让小伴读取源码，按协议 2 规范重新生成（app_create，overwrite=true）"
    }
    require(json.gameString("entry", "index.html") == "index.html" && json.gameString("icon") == "icon.svg" && json.gameString("cover") == "cover.svg") {
        "手机作品的入口、图标、封面必须是 index.html、icon.svg、cover.svg"
    }
    require("layer" !in json) { "手机不支持桌面专用作品" }
    return MobileGameFormat.metadata(MobileGame(
        id = json.gameString("id"), name = json.gameString("name"), description = json.gameString("description"),
        orientation = json.gameString("orientation", "portrait"), kind = json.gameString("kind"), category = json.gameString("category"),
        input = json.gameStrings("input", emptyList()), capabilities = json.gameStrings("capabilities", emptyList()),
        version = json.gameString("version"),
    ))
}

/** Keeps the pack manifest in games/<id>/; run logs and storage belong outside that folder. */
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
            } catch (e: Exception) { errors += "${entry.path}：${e.message ?: "作品校验失败"}" }
        }
        return MobileGameScan(games.sortedBy { it.name }, errors)
    }

    suspend fun read(id: String): MobileGame {
        MobileGameFormat.id(id)
        val text = workspace.read("games/$id/manifest.json")
        val game = readMobileGameManifest(text)
        require(game.id == id) { "manifest id 与作品目录不一致" }
        return game
    }

    private suspend fun contentFiles(id: String): List<MobileGameFile> {
        val entries = workspace.list("games/$id", recursive = true)
        require(entries.size <= MobileGameFormat.FILES * 4 + 1) { "作品目录条目过多" }
        val files = entries.filterNot { it.directory || it.path == "games/$id/manifest.json" }
        require(files.size in 1..MobileGameFormat.FILES && files.sumOf { it.size } <= MobileGameFormat.TOTAL_BYTES) {
            "作品需要 1 到 ${MobileGameFormat.FILES} 个文本文件，合计不超过 8 MB"
        }
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

    /** Deletes only one user work and its run log. Old protocol-1 works can still be deleted. A failed deletion restores its files. */
    suspend fun delete(id: String) = lock.withLock {
        MobileGameFormat.id(id) // Also refuses every reserved official id.
        val manifest = runCatching { MobileGameJson.parseToJsonElement(workspace.read("games/$id/manifest.json")) as? JsonObject }.getOrNull()
        require(manifest?.let { (it["id"] as? JsonPrimitive)?.contentOrNull } == id) { "找不到这件作品" }
        val roots = workspace.list().filter { it.directory }.map { it.path }.toSet()
        val auxiliary = listOf("saves/$id.json", "logs/$id.txt").filter { path ->
            val parent = path.substringBefore('/')
            parent in roots && workspace.list(parent).any { it.path == path && !it.directory }
        }
        val files = workspace.list("games/$id", recursive = true).filterNot { it.directory }.map { it.path } + auxiliary
        val backup = files.map { it to workspace.read(it) }
        withContext(NonCancellable) {
            try {
                workspace.remove("games/$id", recursive = true)
                auxiliary.forEach { workspace.remove(it) }
            } catch (failure: Exception) {
                try { backup.forEach { (path, content) -> workspace.write(path, content) } }
                catch (rollback: Exception) { failure.addSuppressed(rollback) }
                throw failure
            }
        }
    }

    suspend fun createFromDirectory(game: MobileGame, sourceDir: String, overwrite: Boolean = false): MobileGame {
        MobileWorkspaceLimits.path(sourceDir)
        val entries = workspace.list(sourceDir, recursive = true)
        require(entries.size <= MobileGameFormat.FILES * 4) { "作品草稿目录条目过多" }
        val sourceFiles = entries.filterNot { it.directory }
        require(sourceFiles.size in 1..MobileGameFormat.FILES && sourceFiles.sumOf { it.size } <= MobileGameFormat.TOTAL_BYTES) {
            "作品草稿需要 1 到 ${MobileGameFormat.FILES} 个文本文件，合计不超过 8 MB"
        }
        val files = sourceFiles.map {
            MobileGameFile(it.path.removePrefix("$sourceDir/"), workspace.read(it.path))
        }
        return create(game, files, overwrite)
    }

    suspend fun create(game: MobileGame, files: List<MobileGameFile>, overwrite: Boolean = false): MobileGame = lock.withLock {
        val normalized = MobileGameFormat.metadata(game)
        MobileGameFormat.files(files)
        val hadOld = exists(game.id)
        require(!hadOld || overwrite) { "作品 id 已存在；修改已有作品请显式设置 overwrite=true" }
        if (!hadOld) require(workspace.list().none { it.path == "games" } || workspace.list("games").count { it.directory } < MobileGameFormat.GAMES) {
            "最多保存 ${MobileGameFormat.GAMES} 件自创作品"
        }
        // Snapshot before any mutation, so invalid replacements and failed writes retain the original.
        val previous = if (hadOld) workspace.list("games/${game.id}", true).filterNot { it.directory }.map {
            MobileGameFile(it.path.removePrefix("games/${game.id}/"), workspace.read(it.path))
        } else emptyList()
        try {
            if (hadOld) workspace.remove("games/${game.id}", recursive = true)
            files.forEach { workspace.write("games/${game.id}/${it.path}", it.content) }
            // A directory is only discoverable as a valid work when its final manifest is present.
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
    } catch (e: Exception) { AgentToolResult(false, e.message ?: "作品操作失败") }
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
    MobileGameTool("app_read_format", "读取小伴应用规范（运行时协议 2）的完整创作规则，制作游戏或工具前先读取。", gameParameters()) {
        JsonPrimitive(MobileGameFormat.instructions)
    },
    MobileGameTool("app_list", "列出手机工作区中校验通过的作品（游戏和工具）。", gameParameters()) {
        val scan = store.scan()
        buildJsonObject { put("games", JsonArray(scan.games.map { it.json() })); put("errors", JsonArray(scan.errors.map(::JsonPrimitive))) }
    },
    MobileGameTool("app_validate", "校验作品清单、入口、图标、封面、路径、文件类型与大小，试用或导出前使用。", gameParameters("id" to "string", required = listOf("id"))) {
        store.validate(it.gameString("id")).json()
    },
    MobileGameTool("app_create", "按协议 2 安装一个作品（游戏或工具）。source_dir（工作区内专用草稿目录）或 files 数组必须二选一；manifest 自动生成。已有 id 需要 overwrite=true。", gameParameters(
        "id" to "string", "name" to "string", "description" to "string", "kind" to "string", "category" to "string", "orientation" to "string",
        "input" to "array", "capabilities" to "array", "version" to "string", "source_dir" to "string", "files" to "array", "overwrite" to "boolean",
        required = listOf("id", "name", "description"),
    ), "write") { args ->
        val kind = args.gameString("kind", "game")
        val game = MobileGame(
            id = args.gameString("id"), name = args.gameString("name"), description = args.gameString("description"),
            orientation = args.gameString("orientation", "portrait"), kind = kind,
            category = args.gameString("category", if (kind == "app") "tool" else "casual"),
            input = args.gameStrings("input", listOf("touch", "mouse", "keyboard")), capabilities = args.gameStrings("capabilities", emptyList()),
            version = args.gameString("version", "1.0.0"),
        )
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
