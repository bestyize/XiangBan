package xyz.xiangban.xiaoban.common.community

import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
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
    val icon: String = "icon.svg",
    val cover: String = "cover.svg",
)

data class MobileGameFile(val path: String, val bytes: ByteArray) {
    constructor(path: String, content: String) : this(path, content.encodeToByteArray())
    val content: String get() = bytes.decodeToString(throwOnInvalidSequence = true)
    override fun equals(other: Any?): Boolean = other is MobileGameFile && path == other.path && bytes.contentEquals(other.bytes)
    override fun hashCode(): Int = 31 * path.hashCode() + bytes.contentHashCode()
}
data class MobileGameScan(val games: List<MobileGame>, val errors: List<String>)

object MobileGameFormat {
    const val FILES = 100
    const val FILE_BYTES = MobileWorkspaceLimits.FILE_BYTES
    const val TOTAL_BYTES = 24 * 1024 * 1024
    const val GAMES = 40
    private const val MAX_DEPTH = 6
    private val builtin = BuiltinMobileGames.ids.toSet()
    private val textExtensions = setOf("html", "js", "mjs", "css", "json", "svg", "md", "txt", "gltf")
    private val images = setOf("png", "jpg", "jpeg", "webp", "gif")
    private val assets = images + setOf("mp3", "wav", "ogg", "m4a", "glb", "woff2")
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
        for (image in listOf(game.icon, game.cover)) {
            path(image)
            require(extension(image) == "svg" || isImageAsset(image)) { "图标和封面需要是 SVG、PNG、JPEG、WebP 或 GIF" }
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
        require(extension(path) in textExtensions || isAssetPath(path)) { "手机创作暂不支持这个文件类型：$path" }
    }

    private fun extension(path: String) = path.substringAfterLast('/').substringAfterLast('.', "").lowercase()
    fun isImageAsset(path: String) = extension(path) in images
    fun isAssetPath(path: String) = extension(path) in assets

    fun validateAsset(path: String, bytes: ByteArray) {
        path(path)
        require(isAssetPath(path) && bytes.size in 1..MobileWorkspaceLimits.ASSET_BYTES) {
            "请选择支持的素材，单个文件不能超过 10 MB：$path"
        }
        fun matches(offset: Int, signature: String) = signature.withIndex().all { (index, char) ->
            bytes.getOrNull(offset + index)?.toInt()?.and(255) == char.code
        }
        val valid = when (extension(path)) {
            "png" -> matches(0, "\u0089PNG\r\n\u001a\n")
            "jpg", "jpeg" -> matches(0, "\u00ff\u00d8\u00ff")
            "webp" -> matches(0, "RIFF") && matches(8, "WEBP")
            "gif" -> matches(0, "GIF87a") || matches(0, "GIF89a")
            "wav" -> matches(0, "RIFF") && matches(8, "WAVE")
            "ogg" -> matches(0, "OggS")
            "mp3" -> matches(0, "ID3") || (bytes.size >= 4 && matches(0, "\u00ff") && (bytes[1].toInt() and 0xe0) == 0xe0)
            "m4a" -> bytes.size >= 12 && matches(4, "ftyp")
            "glb" -> bytes.size >= 12 && matches(0, "glTF")
            "woff2" -> matches(0, "wOF2")
            else -> false
        }
        require(valid) { "所选文件与素材路径的扩展名不匹配：$path" }
    }

    fun files(files: List<MobileGameFile>, requiredPaths: List<String> = required) {
        require(files.size in 1..FILES) { "作品需要 1 到 $FILES 个文件" }
        val seen = mutableSetOf<String>()
        val directories = mutableMapOf<String, String>()
        var total = 0L
        for (file in files) {
            path(file.path)
            require(!file.path.equals("manifest.json", ignoreCase = true)) { "manifest.json 由 app_create 自动生成，草稿或 files 不能包含它" }
            require(seen.add(file.path.lowercase())) { "文件路径不能重名（忽略大小写）：${file.path}" }
            var directory = file.path.substringBeforeLast('/', "")
            while (directory.isNotEmpty()) {
                val previous = directories.put(directory.lowercase(), directory)
                require(previous == null || previous == directory) { "目录大小写不一致：$directory" }
                directory = directory.substringBeforeLast('/', "")
            }
            if (isAssetPath(file.path)) {
                validateAsset(file.path, file.bytes)
            } else {
                require(file.bytes.size <= FILE_BYTES) { "单个文本文件不能超过 1 MB：${file.path}" }
                require('\u0000' !in file.content) { "作品源码必须是 UTF-8 文本" }
            }
            total += file.bytes.size
        }
        require(total <= TOTAL_BYTES) { "作品文件合计不能超过 24 MB" }
        for (name in requiredPaths) require(files.any { it.path == name && it.bytes.isNotEmpty() }) {
            "作品必须有 $name（入口、1:1 图标、16:9 封面）"
        }
        require(seen.none { it in directories }) { "文件路径冲突：同一名字不能既是文件又是目录" }
    }

    val instructions = """
        小伴应用规范（运行时协议 2，手机创作版）：
        1. 你在做一个作品：游戏（kind=game）或工具、测评（kind=app），在手机上全屏运行。先给用户一个简短方案，写明类型、竖屏还是横屏、玩法或功能，确认后再动手。
        2. 作品免构建，可用原生 JavaScript 和相对路径 import。在 drafts/<作品名> 内用 write_file 写源码。需要 3D 时，用 lib_add(library="three", source_dir=草稿目录) 加入 Three.js r128，再用 <script src="lib/three.min.js"></script> 加载全局 THREE。优先使用包内文件，确保没有 CDN 也能运行。
        3. 入口是 index.html，图标默认用 1:1 的 icon.svg，封面默认用 16:9 的 cover.svg。使用 PNG、JPEG、WebP 或 GIF 时，在 app_create 的 icon、cover 参数里写包内相对路径。用 asset_import 请用户选择图片、音频、GLB 或 WOFF2，再用 source_dir 安装以保留素材。纯文本作品也可传 files，两个参数二选一。工具自动生成 manifest.json。最多 100 个文件，单个源码最多 1 MB、单个素材最多 10 MB、合计最多 24 MB。相对路径最多 6 层，每段只用字母数字和 ._-。
        4. 清单信息写在 app_create 的参数里：id（2 到 40 位小写字母、数字、-，不能与内置作品重名）、name（最多 24 字）、description（最多 80 字）、kind（game 或 app，默认 game）、category（游戏取 action、puzzle、board、casual、story，应用取 tool、quiz；默认游戏 casual、应用 tool）、orientation（portrait、landscape 或 any）、input（touch、mouse、keyboard，必须含 touch，默认三个都有）、capabilities（用到哪些系统能力，默认没有）、version（默认 1.0.0）。已有 id 需要 overwrite=true。
        5. 运行环境：作品是一个普通网站，有自己独立的 https 源。可以用 fetch、WebSocket、localStorage、IndexedDB、Worker、WebAssembly、Canvas、WebGL、WebAudio；只能访问 https/wss，不能访问 localhost 和局域网；不能用 alert、confirm、prompt、全屏 API、屏幕方向 API。联网只在确实需要实时数据时用，并处理断网：没网时给出可见提示，不要白屏。不要把密钥写进代码。
        6. 布局：viewport meta 加 viewport-fit=cover，页面用 100vw × 100dvh（100vh 兜底）。背景延伸到状态栏、挖孔和手势区后方；只给内容与操作控件按 --xb-safe-top/right/bottom/left 留系统安全边距。
           右上角竖三点由宿主绘制。用 --xb-capsule-top/right/width/height 计算实际触控矩形（左边为视口宽减 right 和 width）；只挪开相交的控件，至少留 8 CSS 像素间隔。左侧标题和中间内容可以同排，不要用胶囊底边抬高整页或整个滚动区域。检查首页、选关、暂停、结算、弹窗及滚动后的操作。
           canvas 按 devicePixelRatio 绘制；监听 resize 和 xb.on('safearea', ...) 后用最新的 xb.info.safeArea、xb.info.capsule 更新布局和命中范围。按钮至少 44×44 像素。Android 游戏运行页不另设左上角返回按钮，退出由宿主管理。
        7. SDK：宿主已注入全局对象 xb，不要自己打包，也不要自己写 postMessage。第一屏画好后调用 xb.ready()（10 秒内必须调用，否则宿主判定启动失败）。xb.on('hide', fn) 暂停并停掉声音，xb.on('show', fn) 恢复。作品里有多个界面时用 history.pushState 记录、在 popstate 里切回上一个界面，系统返回键会逐层退回；不要拦截系统返回。xb.exit() 退出作品。xb.info 里有 safeArea、capsule、orientation、theme。
        8. 存档：重要数据用 await xb.storage.set(key, value) 和 await xb.storage.get(key)，值是 JSON，单个作品合计最多 10 MB；数据一变就存，不要等退出时才存，手机随时可能挂起作品。
        9. 游戏成绩（可选）：开始一局调 xb.game.start()，过程中 xb.game.progress({score})，结束时 xb.game.result({score})，score 是非负整数。
        10. 系统能力：capabilities 里写 camera、microphone 后，用标准的 navigator.mediaDevices.getUserMedia 使用，宿主会请用户授权；使用前先 if (xb.can('camera'))，不支持就隐藏对应按钮，作品照样可用。其余能力（haptics、share、clipboard、screen、notify、profile、contacts、files）手机宿主暂未开放，不要依赖。
        11. 声音和操作：用户第一次点击时调用 AudioContext.resume()，收到 show 后再调用一次，并提供静音开关。同时支持触控和键鼠；游戏里的运动都乘以帧间隔 dt，dt 最多取 1/30 秒。
        12. 内容：不卖东西、不引导外部付款，不收集手机号等隐私，不做色情、赌博、血腥内容，不在后台偷偷上传用户数据。
        13. 质量：游戏要有开始、进行、暂停、结算和重开的完整流程，随机局面保证有解，难度先低后高；工具和测评打开就能用；主要操作都有动画或震动之类的反馈；配色 4 到 6 种，主角、目标、危险物一眼能分清。
        14. 安装后用 app_validate 校验，请用户用 app_open 试玩，再用 app_runtime_read 检查加载、脚本报错和 xb.ready 的记录。有实际试玩记录才能声称测试通过。修改已安装作品时，用 copy_file 将 games/<id>/ 的源码和素材复制进新草稿，保留路径并跳过 manifest.json。修改后用 app_create 安装新版。导出或分享的 ZIP 将 manifest.json 放在最外层。
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
    put("version", version); put("category", category); put("entry", "index.html"); put("icon", icon); put("cover", cover)
    put("orientation", orientation); put("input", JsonArray(input.map(::JsonPrimitive)))
    if (capabilities.isNotEmpty()) put("capabilities", JsonArray(capabilities.map(::JsonPrimitive)))
}

fun readMobileGameManifest(text: String): MobileGame {
    require(text.encodeToByteArray().size <= 64 * 1024) { "manifest.json 不能超过 64 KB" }
    val json = MobileGameJson.parseToJsonElement(text) as? JsonObject ?: throw IllegalArgumentException("manifest.json 需要是 JSON 对象")
    require(json.gameInt("runtimeProtocol") == 2 && json.gameInt("schemaVersion") == 3) {
        "这是旧版（协议 1）作品，现在已不再支持；请让小伴读取源码，按协议 2 规范重新生成（app_create，overwrite=true）"
    }
    require(json.gameString("entry", "index.html") == "index.html") { "手机作品的入口必须是 index.html" }
    require("layer" !in json) { "手机不支持桌面专用作品" }
    return MobileGameFormat.metadata(MobileGame(
        id = json.gameString("id"), name = json.gameString("name"), description = json.gameString("description"),
        orientation = json.gameString("orientation", "portrait"), kind = json.gameString("kind"), category = json.gameString("category"),
        input = json.gameStrings("input", emptyList()), capabilities = json.gameStrings("capabilities", emptyList()),
        version = json.gameString("version"), icon = json.gameString("icon"), cover = json.gameString("cover"),
    ))
}

/** Keeps the pack manifest in games/<id>/; run logs and storage belong outside that folder. */
class MobileGameStore(val workspace: MobileWorkspace) {
    private val lock = Mutex()
    private val changes = MutableStateFlow(0)
    val revision = changes.asStateFlow()

    private suspend fun <T> access(action: suspend () -> T): T = lock.withLock {
        workspace.gameTransaction(action)
    }

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

    private suspend fun contentFiles(directory: String, installed: Boolean): List<MobileGameFile> {
        val entries = workspace.list(directory, recursive = true)
        require(entries.size <= MobileGameFormat.FILES * 4 + 1) { "作品目录条目过多" }
        val files = entries.filterNot { it.directory || (installed && it.path == "$directory/manifest.json") }
        require(files.size in 1..MobileGameFormat.FILES && files.sumOf { it.size } <= MobileGameFormat.TOTAL_BYTES) {
            "作品需要 1 到 ${MobileGameFormat.FILES} 个文件，合计不超过 24 MB"
        }
        return files.map { file ->
            val path = file.path.removePrefix("$directory/")
            MobileGameFormat.path(path)
            if (MobileGameFormat.isAssetPath(path)) MobileGameFile(path, workspace.readBytes(file.path))
            else MobileGameFile(path, workspace.read(file.path))
        }
    }

    suspend fun validate(id: String): MobileGame = access {
        val game = read(id)
        MobileGameFormat.files(contentFiles("games/$id", installed = true), listOf("index.html", game.icon, game.cover))
        game
    }

    suspend fun readFile(id: String, path: String): String {
        MobileGameFormat.id(id)
        if (path != "manifest.json") MobileGameFormat.path(path)
        return workspace.read("games/$id/$path")
    }

    suspend fun readFileBytes(id: String, path: String): ByteArray {
        MobileGameFormat.id(id)
        MobileGameFormat.path(path)
        return workspace.readBytes("games/$id/$path")
    }

    /** Snapshot includes the normalized resource manifest, ready to write at the ZIP root. */
    suspend fun exportFiles(id: String): List<MobileGameFile> = access {
        val game = read(id)
        val files = contentFiles("games/$id", installed = true)
        MobileGameFormat.files(files, listOf("index.html", game.icon, game.cover))
        listOf(MobileGameFile("manifest.json", game.json().toString())) + files
    }

    /** Deletes only one user work and its run log. Old protocol-1 works can still be deleted. A failed deletion restores its files. */
    suspend fun delete(id: String) = access {
        MobileGameFormat.id(id) // Also refuses every reserved official id.
        val manifest = runCatching { MobileGameJson.parseToJsonElement(workspace.read("games/$id/manifest.json")) as? JsonObject }.getOrNull()
        require(manifest?.let { (it["id"] as? JsonPrimitive)?.contentOrNull } == id) { "找不到这件作品" }
        val roots = workspace.list().filter { it.directory }.map { it.path }.toSet()
        val auxiliary = listOf("saves/$id.json", "logs/$id.txt").filter { path ->
            val parent = path.substringBefore('/')
            parent in roots && workspace.list(parent).any { it.path == path && !it.directory }
        }
        val files = workspace.list("games/$id", recursive = true).filterNot { it.directory }.map { it.path } + auxiliary
        val backup = files.map { it to workspace.readBytes(it) }
        withContext(NonCancellable) {
            try {
                workspace.remove("games/$id", recursive = true)
                auxiliary.forEach { workspace.remove(it) }
            } catch (failure: Exception) {
                try { backup.forEach { (path, bytes) -> workspace.writeBytes(path, bytes) } }
                catch (rollback: Exception) { failure.addSuppressed(rollback) }
                throw failure
            }
        }
        changes.value++
    }

    suspend fun createFromDirectory(game: MobileGame, sourceDir: String, overwrite: Boolean = false): MobileGame = access {
        MobileWorkspaceLimits.path(sourceDir)
        install(game, contentFiles(sourceDir, installed = false), overwrite)
    }

    suspend fun create(game: MobileGame, files: List<MobileGameFile>, overwrite: Boolean = false): MobileGame =
        access { install(game, files, overwrite) }

    private suspend fun install(game: MobileGame, files: List<MobileGameFile>, overwrite: Boolean): MobileGame {
        val normalized = MobileGameFormat.metadata(game)
        MobileGameFormat.files(files, listOf("index.html", normalized.icon, normalized.cover))
        val hadOld = exists(game.id)
        require(!hadOld || overwrite) { "作品 id 已存在；修改已有作品请显式设置 overwrite=true" }
        if (!hadOld) require(workspace.list().none { it.path == "games" } || workspace.list("games").count { it.directory } < MobileGameFormat.GAMES) {
            "最多保存 ${MobileGameFormat.GAMES} 件自创作品"
        }
        // Snapshot before any mutation, so invalid replacements and failed writes retain the original.
        val directory = "games/${game.id}"
        val previous = if (hadOld) workspace.list(directory, true).filterNot { it.directory }.map {
            it.path to workspace.readBytes(it.path)
        } else emptyList()
        try {
            if (hadOld) workspace.remove(directory, recursive = true)
            for (file in files) {
                val path = "$directory/${file.path}"
                if (MobileGameFormat.isAssetPath(file.path)) workspace.writeBytes(path, file.bytes)
                else workspace.write(path, file.content)
            }
            // A directory is only discoverable as a valid work when its final manifest is present.
            workspace.write("$directory/manifest.json", normalized.json().toString())
        } catch (error: Exception) {
            withContext(NonCancellable) {
                try {
                    if (exists(game.id)) workspace.remove(directory, recursive = true)
                    previous.forEach { (path, bytes) -> workspace.writeBytes(path, bytes) }
                } catch (rollback: Exception) { error.addSuppressed(rollback) }
            }
            throw error
        }
        changes.value++
        return normalized
    }
}

private class MobileGameTool(
    override val name: String,
    override val description: String,
    override val parameters: JsonObject,
    override val risk: String = "read",
    override val timeoutSeconds: Long = 60,
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
    MobileGameTool("app_create", "按协议 2 安装作品。source_dir（工作区内草稿目录，保留图片、音频等素材）和纯文本 files 数组二选一；icon 和 cover 是包内相对路径，清单自动生成。已有 id 需要 overwrite=true。", gameParameters(
        "id" to "string", "name" to "string", "description" to "string", "kind" to "string", "category" to "string", "orientation" to "string",
        "input" to "array", "capabilities" to "array", "version" to "string", "icon" to "string", "cover" to "string",
        "source_dir" to "string", "files" to "array", "overwrite" to "boolean",
        required = listOf("id", "name", "description"),
    ), "write") { args ->
        val kind = args.gameString("kind", "game")
        val game = MobileGame(
            id = args.gameString("id"), name = args.gameString("name"), description = args.gameString("description"),
            orientation = args.gameString("orientation", "portrait"), kind = kind,
            category = args.gameString("category", if (kind == "app") "tool" else "casual"),
            input = args.gameStrings("input", listOf("touch", "mouse", "keyboard")), capabilities = args.gameStrings("capabilities", emptyList()),
            version = args.gameString("version", "1.0.0"), icon = args.gameString("icon", "icon.svg"), cover = args.gameString("cover", "cover.svg"),
        )
        require(("source_dir" in args) != ("files" in args)) { "source_dir 和 files 必须二选一" }
        val created = if ("source_dir" in args) store.createFromDirectory(game, args.gameString("source_dir"), args.gameBoolean("overwrite")) else {
            val array = args["files"] as? JsonArray ?: throw IllegalArgumentException("files 必须是数组")
            store.create(game, array.map { value ->
                val file = value as? JsonObject ?: throw IllegalArgumentException("files 的每项需要 path 与 content")
                val path = file.gameString("path")
                require(!MobileGameFormat.isAssetPath(path)) { "素材需要用 asset_import 导入草稿，再使用 source_dir 安装" }
                MobileGameFile(path, file.gameString("content"))
            }, args.gameBoolean("overwrite"))
        }
        created.json()
    },
)

/** Only registered on a host with a bundled library or a user-mediated system file picker. */
fun mobileGameAssetTools(
    workspace: MobileWorkspace,
    loadThree: (suspend () -> ByteArray?)?,
    pickAsset: (suspend (image: Boolean, maxBytes: Long) -> ForumUploadFile?)?,
): List<AgentTool> = buildList {
    if (loadThree != null) add(MobileGameTool(
        "lib_add", "将内置 Three.js r128 复制进手机创作草稿。设 library=three，通过 lib/three.min.js 加载全局 THREE。",
        gameParameters("library" to "string", "source_dir" to "string", required = listOf("library", "source_dir")), "write",
    ) { args ->
        val directory = args.draftDirectory()
        require(args.gameString("library").lowercase() in setOf("three", "three.js")) { "当前手机内置库支持 three（Three.js r128）" }
        val entry = "lib/three.min.js"
        val bytes = checkNotNull(loadThree()) { "当前安装包中找不到 Three.js" }
        check(bytes.size in 1..MobileGameFormat.FILE_BYTES) { "内置库文件大小无效" }
        workspace.gameTransaction { workspace.writeBytes("$directory/$entry", bytes, overwrite = false) }
        buildJsonObject { put("library", "three"); put("version", "0.128.0"); put("path", entry); put("global", "THREE") }
    })
    if (pickAsset != null) add(MobileGameTool(
        "asset_import", "请用户通过系统选择器将一个本地素材复制进创作草稿。path 是相对草稿的目标文件名，例如 assets/hero.webp，扩展名必须与用户选择的文件一致。支持 PNG、JPEG、WebP、GIF、MP3、WAV、OGG、M4A、GLB 和 WOFF2，最多 10 MB。",
        gameParameters("source_dir" to "string", "path" to "string", required = listOf("source_dir", "path")), "write",
        timeoutSeconds = 300,
    ) { args ->
        val directory = args.draftDirectory()
        val path = args.gameString("path")
        MobileGameFormat.path(path)
        require(MobileGameFormat.isAssetPath(path)) { "请选择受支持的图片、音频、GLB 或 WOFF2 素材路径" }
        val target = MobileWorkspaceLimits.path("$directory/$path")
        val file = pickAsset(MobileGameFormat.isImageAsset(path), MobileWorkspaceLimits.ASSET_BYTES.toLong())
        if (file == null) {
            buildJsonObject { put("imported", false); put("cancelled", true) }
        } else {
            MobileGameFormat.validateAsset(path, file.bytes)
            workspace.gameTransaction { workspace.writeBytes(target, file.bytes, overwrite = false) }
            buildJsonObject { put("imported", true); put("path", path); put("size", file.bytes.size) }
        }
    })
}

private fun JsonObject.draftDirectory(): String {
    val path = MobileWorkspaceLimits.path(gameString("source_dir"))
    require(path.startsWith("drafts/") && path.count { it == '/' } == 1) { "source_dir 需要是 drafts/<作品名> 的独立草稿目录" }
    return path
}
