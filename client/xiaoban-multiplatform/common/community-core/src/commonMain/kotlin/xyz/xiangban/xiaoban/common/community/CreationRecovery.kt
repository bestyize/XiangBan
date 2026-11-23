package xyz.xiangban.xiaoban.common.community

import kotlinx.coroutines.CancellationException
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.contentOrNull

/** Replay only host receipts from this conversation. A streamed tool argument is not a saved file. */
data class CreationProgress(
    val files: Set<String>, val directories: Set<String>, val targetId: String?,
    val installed: MobileGame?, val validated: Boolean, val pendingStep: String?,
) {
    val resumeId: String? get() = installed?.id ?: targetId
    val nextStep: String get() = pendingStep ?: when {
        installed == null -> "完成草稿并安装作品"
        !validated -> "校验已安装的作品"
        else -> "核对运行结果"
    }
    val summary: String get() = when {
        installed != null -> "已安装《${installed.name}》" + if (validated) "，校验已通过" else "，尚未确认校验结果"
        files.isEmpty() -> "尚无已保存文件的记录"
        else -> "已保存 " + files.take(3).joinToString("、") { it.substringAfterLast('/') } +
            if (files.size > 3) " 等 ${files.size} 个文件" else ""
    }
}

internal val creationTools = setOf("write_file", "edit_file", "copy_file", "delete_file", "asset_import", "lib_add", "app_create", "app_validate")

private fun String.jsonObject() = runCatching { CommunityJson.parseToJsonElement(this) as? JsonObject }.getOrNull()
private fun JsonObject?.field(name: String) = (this?.get(name) as? JsonPrimitive)?.contentOrNull
private fun String?.draftRoot(): String? {
    val path = this ?: return null
    if (!path.startsWith("drafts/") || runCatching { MobileWorkspaceLimits.path(path) }.isFailure) return null
    return path.split('/', limit = 3).take(2).joinToString("/")
}

fun creationProgress(messages: List<ChatTurn>): CreationProgress {
    val files = linkedSetOf<String>()
    val directories = linkedSetOf<String>()
    var installed: MobileGame? = null
    var target: String? = null
    var validated = false
    for (turn in messages) for (call in turn.tools) {
        if (call.name !in creationTools) continue
        val receipt = if (call.status == ToolStatus.Done) call.output.jsonObject() else null
        val arguments by lazy { call.arguments.jsonObject() }
        when (call.name) {
            "app_create", "app_validate" -> {
                val id = receipt.field("id") ?: arguments.field("id")
                if (id == null || runCatching { MobileGameFormat.id(id) }.isFailure) continue
                if (call.name == "app_create") {
                    target = id
                    arguments.field("source_dir").draftRoot()?.let(directories::add)
                    if (receipt != null) {
                        installed = runCatching { readMobileGameManifest(call.output) }.getOrNull() ?: installed
                        validated = false
                    }
                } else if (installed?.id == id) validated = receipt.field("id") == id
            }
            else -> {
                val path = if (call.name == "lib_add" || call.name == "asset_import") {
                    val root = arguments.field("source_dir") ?: continue
                    val relative = receipt.field("path") ?: arguments.field("path") ?: continue
                    "$root/$relative"
                } else receipt.field("path") ?: arguments.field("path")
                val file = path ?: continue
                val root = file.draftRoot() ?: continue
                directories += root
                if (file == root && call.name != "delete_file") continue
                if (receipt == null) continue
                when (call.name) {
                    "delete_file" -> if (receipt.field("deleted") == "true") files.removeAll { it == file || it.startsWith("$file/") }
                    "asset_import" -> if (receipt.field("imported") == "true") files += file
                    else -> if (receipt.field("path") != null) files += file
                }
            }
        }
    }
    val plan = messages.asReversed().firstNotNullOfOrNull { taskPlan(it.tools) }
    val pending = plan?.steps?.getOrNull(plan.current)?.content
    return CreationProgress(files, directories, target, installed, validated, pending)
}

fun ChatTurn.creationInterrupted(): Boolean = role == "assistant" && !streaming && !imageGeneration &&
    (error != null || cancelled || truncated || tools.any { it.status == ToolStatus.Error } && unresolvedCalls(tools).isNotEmpty())

/** Inspect the referenced drafts again at the time of the click, including writes whose result was interrupted. */
internal suspend fun CreationProgress.resumeInput(store: MobileGameStore?, draft: CreationDraft, plan: ChatTurn?): String = buildString {
    appendLine("从同一作品的已保存进度继续，只补缺失或失败的步骤。已确认的需求：${draft.request}")
    plan?.let { appendLine("原方案见 conversation_read(message_id=${it.id})。") }
    appendLine("本会话的操作回执：$summary。尚待完成：$nextStep。")
    for (root in directories) {
        val entries = try { store?.workspace?.list(root, recursive = true) }
            catch (e: CancellationException) { throw e } catch (_: Exception) { null }
        appendLine(if (entries == null) "$root：当前未能核实，请先检查该目录。"
            else "$root 当前文件：" + entries.filterNot { it.directory }.take(MobileGameFormat.FILES).joinToString { "${it.path}（${it.size} 字节）" }.ifEmpty { "无" })
    }
    val id = resumeId
    if (id != null) {
        appendLine("沿用原作品 ID $id，不创建新的 ID。")
        val work = try { store?.validate(id) } catch (e: CancellationException) { throw e } catch (_: Exception) { null }
        if (work != null) appendLine("手机上现有《${work.name}》版本 ${work.version} 已通过文件校验。核对是否本次目标，已完成安装的直接打开验收；仅在需要更新时按原有授权流程更新。")
        else appendLine("尚未核实已安装版本，先检查安装状态。")
    }
    append("保留已有源码和数据，使用同一草稿。需要调整已保存的文件时，用 edit_file 做局部修改。中断的大文件分成可独立保存的小步。")
}
