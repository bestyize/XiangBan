package xyz.xiangban.xiaoban.common.community

import kotlin.io.encoding.Base64
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import kotlinx.serialization.Serializable
import kotlinx.serialization.builtins.ListSerializer
import kotlinx.serialization.json.*

private fun imageJobHash(value: String) = Sha256().also { it.update(value.encodeToByteArray()) }.hex()

/** Stored beside native session sidecars, so deleting the conversation can remove its receipts. */
fun imageToolJobFile(session: String): String = "agent/image-jobs/${imageJobHash(session)}.json"

@Serializable
private data class AgentImageJob(
    val owner: String,
    val requestId: String,
    val receipt: ImageJobReceipt,
    val parameters: JsonObject,
)

/** Uses the existing official image job API; the Kotlin host retains full images, not model text. */
fun imageAgentTools(
    api: AiGatewayApi,
    files: ChatHistoryFiles,
    accountId: () -> String?,
    workspace: MobileWorkspace? = null,
): List<AgentTool> {
    val service = ImageAgentService(api, files, accountId, workspace)
    return listOf(service.generate, service.list)
}

private class ImageAgentService(
    private val api: AiGatewayApi,
    private val files: ChatHistoryFiles,
    private val accountId: () -> String?,
    private val workspace: MobileWorkspace?,
) {
    private val lock = Mutex()
    private val json = Json { ignoreUnknownKeys = true }
    private val serializer = ListSerializer(AgentImageJob.serializer())

    private suspend fun owner(): Pair<String, String> {
        val context = currentCoroutineContext()
        context.ensureActive()
        context[AiAccountRequest]?.checkCurrent()
        val owner = context[AiAccountRequest]?.identity?.accountId ?: accountId()
        require(!owner.isNullOrBlank()) { "请先登录小伴账号" }
        val session = context[AgentToolContext]?.session
        require(!session.isNullOrBlank()) { "生图工具需要当前对话上下文" }
        return owner to session
    }

    private fun read(session: String): List<AgentImageJob> {
        val saved = files.read(imageToolJobFile(session)) ?: return emptyList()
        // A damaged receipt must not silently become a new paid request.
        return try { json.decodeFromString(serializer, saved) }
        catch (_: Exception) { error("已保存的生图任务记录无法读取，请先核对原任务，暂不重复生成") }
    }

    private fun JsonObject.string(key: String): String? = (this[key] as? JsonPrimitive)?.takeIf { it.isString }?.contentOrNull
    private fun JsonObject.integer(key: String, default: Int): Int =
        if (key !in this) default else (this[key] as? JsonPrimitive)?.takeUnless { it.isString }?.intOrNull ?: error("$key 必须是整数")

    val generate = object : AgentTool {
        override val name = "image_generate"
        override val description = "使用小伴官方生图服务生成一张图片，会消耗生图额度。request_id 是本对话内唯一的任务标记（字母数字或 _-，最多 80 字符）；中断、重试或查回同一任务必须复用原 request_id 和描述，只有用户要求重新生成才换标记。" +
            "返回任务编号并直接在对话中展示图片。可同时传 source_dir=drafts/<作品名> 和 path=assets/hero.png 将图片保存为创作素材；不覆盖已有不同文件。format 支持 png 或服务已开放的 gif。先用 image_jobs 找回中断任务。"
        override val timeoutSeconds = 30 * 60L
        override val parameters = buildJsonObject {
            put("type", "object")
            putJsonObject("properties") {
                putJsonObject("request_id") { put("type", "string"); put("pattern", "^[A-Za-z0-9_-]{1,80}$") }
                putJsonObject("prompt") { put("type", "string"); put("minLength", 1); put("maxLength", 4000) }
                putJsonObject("size") { put("type", "string") }
                putJsonObject("format") { put("type", "string"); put("enum", JsonArray(listOf(JsonPrimitive("png"), JsonPrimitive("gif")))) }
                putJsonObject("frames") { put("type", "integer"); put("minimum", 2); put("maximum", 16) }
                putJsonObject("frame_duration_ms") { put("type", "integer"); put("minimum", 20); put("maximum", 2000) }
                if (workspace != null) {
                    putJsonObject("source_dir") { put("type", "string") }
                    putJsonObject("path") { put("type", "string") }
                }
            }
            put("required", JsonArray(listOf("request_id", "prompt").map(::JsonPrimitive)))
            put("additionalProperties", false)
        }

        override suspend fun run(arguments: JsonObject): AgentToolResult {
            val (owner, session) = owner()
            val requestId = arguments.string("request_id").orEmpty()
            require(requestId.matches(Regex("[A-Za-z0-9_-]{1,80}"))) { "request_id 需要 1–80 个字母、数字、_ 或 -" }
            val format = arguments.string("format") ?: "png"
            val size = arguments.string("size")
            val parameters = api.imageParameters(arguments.string("prompt").orEmpty(), 1, size, format,
                arguments.integer("frames", 9), arguments.integer("frame_duration_ms", 200))
            val target = assetTarget(arguments, format)
            val previous = withContext(Dispatchers.Default) { lock.withLock { read(session).firstOrNull { it.owner == owner && it.requestId == requestId } } }
            require(previous == null || previous.parameters == parameters) { "此 request_id 已用于不同描述或参数；查回原图请使用 image_jobs 中的原参数，重新生成需使用新标记" }

            val job = previous ?: run {
                check(api.officialAiEnabled()) { "小伴 AI 官方服务已关闭，无法新建生图任务" }
                val capability = checkNotNull(api.capabilities().media["image"]) { "小伴 AI 暂未开放生图" }
                check(capability.available) { "小伴 AI 暂未开放生图" }
                check(capability.supportsJobs) { "当前服务尚不支持可恢复的 Agent 生图，请使用对话中的生图入口" }
                require(format in capability.formats) { "当前服务不支持 $format 图片" }
                require(size == null || capability.sizes.isEmpty() || size in capability.sizes) { "当前服务不支持这个图片尺寸，可用尺寸：${capability.sizes.joinToString()}" }
                if (target != null) require(checkNotNull(workspace).list("", recursive = true).none { it.path == target }) {
                    "目标素材已存在，请为新图片选择不同路径；查回已有图片请使用原 request_id"
                }
                // Deterministic UUID also protects against a crash immediately before receipt persistence.
                val hash = imageJobHash("$owner\u0000$session\u0000$requestId")
                val id = "${hash.take(8)}-${hash.substring(8, 12)}-4${hash.substring(13, 16)}-8${hash.substring(17, 20)}-${hash.substring(20, 32)}"
                val new = AgentImageJob(owner, requestId, ImageJobReceipt(id, 1, format), parameters)
                withContext(Dispatchers.Default) {
                    lock.withLock {
                        currentCoroutineContext().ensureActive()
                        currentCoroutineContext()[AiAccountRequest]?.checkCurrent()
                        val all = read(session)
                        val raced = all.firstOrNull { it.owner == owner && it.requestId == requestId }
                        if (raced != null) {
                            require(raced.parameters == parameters) { "此任务标记已用于不同参数" }
                            raced
                        } else {
                            check(all.size < 100) { "此对话已保存 100 个生图任务，请在新对话中生成新图片；原任务仍可查回" }
                            files.write(imageToolJobFile(session), json.encodeToString(serializer, all + new))
                            new
                        }
                    }
                }
            }

            val result = try {
                if (previous == null) api.awaitImageJob(job.receipt, job.parameters)
                else try { api.awaitImageJob(job.receipt) }
                catch (e: ApiException) {
                    // The process may have stopped between saving the receipt and submitting it.
                    if (e.status == 404) api.awaitImageJob(job.receipt, job.parameters) else throw e
                }
            } catch (e: CancellationException) { throw e }
            catch (e: Exception) {
                return AgentToolResult(false, buildJsonObject {
                    put("request_id", requestId); put("job_id", job.receipt.id)
                    put("error", e.message ?: "生图结果暂未取回")
                    put("recovery", "保留此 request_id 和参数再次调用，只查回原任务；不要新建任务标记")
                }.toString())
            }
            currentCoroutineContext().ensureActive()
            currentCoroutineContext()[AiAccountRequest]?.checkCurrent()
            var assetError: String? = null
            var savedPath: String? = null
            if (target != null && result.images.isNotEmpty()) {
                try {
                    val image = result.images.first()
                    val mime = image.substringAfter("data:").substringBefore(';')
                    val extension = when (mime) { "image/png" -> "png"; "image/jpeg" -> "jpg"; "image/webp" -> "webp"; "image/gif" -> "gif"; else -> error("无法识别生成图片的格式") }
                    require(target.lowercase().endsWith(".$extension") || extension == "jpg" && target.lowercase().endsWith(".jpeg")) { "生成结果为 $mime，请用 .$extension 路径查回并保存同一任务" }
                    val bytes = Base64.decode(image.substringAfter(','))
                    MobileGameFormat.validateAsset(target.substringAfter('/', "").substringAfter('/'), bytes)
                    checkNotNull(workspace).gameTransaction {
                        currentCoroutineContext().ensureActive()
                        currentCoroutineContext()[AiAccountRequest]?.checkCurrent()
                        workspace.writeBytes(target, bytes, overwrite = false)
                    }
                    savedPath = target
                } catch (e: CancellationException) { throw e }
                catch (e: Exception) { assetError = e.message ?: "素材保存失败" }
            }
            val error = listOfNotNull(result.error, assetError).joinToString("；").ifEmpty { null }
            return AgentToolResult(error == null && result.images.isNotEmpty(), buildJsonObject {
                put("request_id", requestId); put("job_id", job.receipt.id); put("count", result.images.size)
                savedPath?.let { put("path", it); put("asset_path", arguments.string("path")) }
                put("message", if (result.images.isNotEmpty()) "图片已交给用户查看；本工具未对图片内容做视觉验收" else "尚未取回可用图片")
                error?.let { put("error", it); put("recovery", "复用原 request_id 查回，不要重复生成") }
            }.toString(), generatedImages = result.images)
        }
    }

    private fun assetTarget(arguments: JsonObject, format: String): String? {
        if ("source_dir" !in arguments && "path" !in arguments) return null
        check(workspace != null) { "当前设备不支持保存创作素材" }
        val directory = MobileWorkspaceLimits.path(arguments.string("source_dir").orEmpty())
        require(directory.startsWith("drafts/") && directory.count { it == '/' } == 1) { "source_dir 需要是 drafts/<作品名>" }
        val path = arguments.string("path").orEmpty()
        MobileGameFormat.path(path)
        require(MobileGameFormat.isImageAsset(path) && (path.lowercase().endsWith(".gif") == (format == "gif"))) { "path 需要是与格式一致的图片素材相对路径" }
        return MobileWorkspaceLimits.path("$directory/$path")
    }

    val list = object : AgentTool {
        override val name = "image_jobs"
        override val description = "查询当前账号、当前对话保存的生图任务编号和原参数。用原 request_id 和参数调用 image_generate 可查回结果，不会重复生成。列表中的记录表示已保存回执，不代表已经生成成功。"
        override val risk = "read"
        override val parameters = buildJsonObject {
            put("type", "object")
            putJsonObject("properties") { putJsonObject("offset") { put("type", "integer"); put("minimum", 0) } }
            put("additionalProperties", false)
        }
        override suspend fun run(arguments: JsonObject): AgentToolResult {
            val (owner, session) = owner()
            val offset = arguments.integer("offset", 0)
            require(offset >= 0) { "offset 不能为负" }
            val jobs = withContext(Dispatchers.Default) { lock.withLock { read(session).filter { it.owner == owner } } }
            return AgentToolResult(true, buildJsonObject {
                put("jobs", buildJsonArray {
                    jobs.drop(offset).take(10).forEach { job -> add(buildJsonObject {
                        put("request_id", job.requestId); put("job_id", job.receipt.id)
                        put("arguments", buildJsonObject {
                            put("request_id", job.requestId); put("prompt", job.parameters.getValue("prompt"))
                            job.parameters["size"]?.let { put("size", it) }
                            put("format", job.receipt.format)
                            job.parameters["frame_count"]?.let { put("frames", it) }
                            job.parameters["frame_duration_ms"]?.let { put("frame_duration_ms", it) }
                        })
                    }) }
                })
                put("total", jobs.size)
                if (offset < jobs.size && jobs.size - offset > 10) put("next_offset", offset + 10)
            }.toString())
        }
    }
}
