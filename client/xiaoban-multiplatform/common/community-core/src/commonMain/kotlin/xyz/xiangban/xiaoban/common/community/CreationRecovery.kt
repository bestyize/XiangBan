package xyz.xiangban.xiaoban.common.community

import kotlinx.coroutines.CancellationException
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.contentOrNull

/** Replay only host receipts from this conversation. A streamed tool argument is not a saved file. */
data class CreationProgress(
    val files: Set<String>, val directories: Set<String>, val targetId: String?,
    val installed: MobileGame?, val validated: Boolean, val pendingStep: String?,
    val draftChangedSinceInstall: Boolean = false,
    val lastActionTurnId: Long? = null,
    val useCreationPlan: Boolean = true,
) {
    val resumeId: String? get() = installed?.id ?: targetId
    val nextStep: String get() = when {
        draftChangedSinceInstall -> "安装并校验更新后的作品"
        installed == null -> pendingStep ?: "完成草稿并安装作品"
        !validated -> "校验已安装的作品"
        else -> "核对运行结果"
    }
    val summary: String get() = when {
        installed != null -> "已安装《${installed.name}》" + if (validated) "，校验已通过" else "，尚未确认校验结果"
        draftChangedSinceInstall -> "草稿已更新，等待重新安装"
        files.isEmpty() -> "尚无已保存文件的记录"
        else -> directories.singleOrNull()?.let { "$it " }.orEmpty() +
            "已保存 " + files.take(3).joinToString("、") { it.substringAfterLast('/') } +
            if (files.size > 3) " 等 ${files.size} 个文件" else ""
    }
}

internal val creationTools = setOf("write_file", "edit_file", "copy_file", "delete_file", "asset_import", "lib_add", "image_generate", "app_create", "app_validate")
internal const val CREATION_RESUME = "继续制作"

private fun String.jsonObject() = runCatching { CommunityJson.parseToJsonElement(this) as? JsonObject }.getOrNull()
private fun JsonObject?.field(name: String) = (this?.get(name) as? JsonPrimitive)?.contentOrNull
private fun String?.draftRoot(): String? {
    val path = this ?: return null
    if (!path.startsWith("drafts/") || runCatching { MobileWorkspaceLimits.path(path) }.isFailure) return null
    return path.split('/', limit = 3).take(2).joinToString("/")
}

private class CreationInstallation(val game: MobileGame, val draftRevision: Int, var validated: Boolean = false)

private class CreationTask(val directory: String? = null) {
    val files = linkedSetOf<String>()
    var draftRevision = 0
    var installation: CreationInstallation? = null
    var targetId: String? = null
    var lastActionTurnId: Long? = null
    var pendingStep: String? = null

    fun progress(useCreationPlan: Boolean): CreationProgress {
        val current = installation?.takeIf { it.draftRevision == draftRevision }
        return CreationProgress(files, setOfNotNull(directory), targetId, current?.game,
            current?.validated == true, pendingStep, installation != null && current == null, lastActionTurnId, useCreationPlan)
    }
}

/** Draft directories identify work on files. Installation receipts link each installed ID to its source. */
fun creationProgress(messages: List<ChatTurn>, approved: Boolean = true): CreationProgress {
    // Draft paths contain a slash, while standalone installation IDs do not.
    val tasks = mutableMapOf<String, CreationTask>()
    val installedWorks = mutableMapOf<String, CreationTask>()
    var first: CreationTask? = null
    var active: CreationTask? = null

    for (turn in messages) {
        val calls = turn.tools.filter { it.name in creationTools }
        if (calls.isEmpty()) continue
        val creationTurn = approved || turn.creationInterrupted() || turn.tools.any { it.name == "app_read_format" }
        val plan by lazy { taskPlan(turn.tools)?.let { it.steps.getOrNull(it.current)?.content } }
        fun activate(task: CreationTask) {
            if (first == null) first = task
            task.lastActionTurnId = turn.id
            task.pendingStep = plan
            active = task
        }
        for (call in calls) {
            val receipt = call.output.takeIf { call.status == ToolStatus.Done }?.jsonObject()
            val arguments by lazy { call.arguments.jsonObject() }
            when (call.name) {
                "app_create" -> {
                    val id = receipt.field("id") ?: arguments.field("id")
                    if (id == null || runCatching { MobileGameFormat.id(id) }.isFailure) continue
                    val directory = arguments.field("source_dir").draftRoot()
                    val task = tasks.getOrPut(directory ?: id) { CreationTask(directory) }
                    val game = receipt?.let { runCatching { readMobileGameManifest(call.output) }.getOrNull() }
                    if (game == null) {
                        if (active == null || active === task) {
                            task.targetId = task.installation?.game?.id ?: id
                            activate(task)
                        }
                        continue
                    }
                    task.installation?.let { installedWorks.remove(it.game.id) }
                    val previous = installedWorks.put(game.id, task)
                    if (previous != null && previous !== task) previous.installation = null
                    task.targetId = game.id
                    task.installation = CreationInstallation(game, task.draftRevision)
                    activate(task)
                }
                "app_validate" -> {
                    val id = arguments.field("id") ?: receipt.field("id")
                    val task = installedWorks[id] ?: continue
                    val installation = task.installation ?: continue
                    val game = receipt?.let { runCatching { readMobileGameManifest(call.output) }.getOrNull() }
                    installation.validated = installation.draftRevision == task.draftRevision && game == installation.game
                    if (active === task) activate(task)
                }
                else -> {
                    val path = when (call.name) {
                        "asset_import", "lib_add" -> {
                            val directory = arguments.field("source_dir") ?: continue
                            val relative = receipt.field("path") ?: arguments.field("path") ?: continue
                            "$directory/$relative"
                        }
                        else -> receipt.field("path") ?: arguments.field("path") ?: continue
                    }
                    val directory = path.draftRoot() ?: continue
                    if (path == directory && call.name != "delete_file") continue
                    val task = tasks.getOrPut(directory) { CreationTask(directory) }
                    val changed = when (call.name) {
                        "delete_file" -> receipt.field("deleted") == "true"
                        "asset_import" -> receipt.field("imported") == "true" && receipt.field("path") != null
                        else -> receipt.field("path") != null
                    }
                    if (changed) {
                        if (call.name == "delete_file") task.files.removeAll { it == path || it.startsWith("$path/") }
                        else task.files += path
                        task.draftRevision++
                    }
                    if ((creationTurn || task.lastActionTurnId != null) && (changed || active == null || active === task)) {
                        activate(task)
                    }
                }
            }
        }
    }
    return active?.progress(useCreationPlan = active === first) ?: CreationProgress(emptySet(), emptySet(), null, null, false, null)
}

fun ChatTurn.creationInterrupted(): Boolean = role == "assistant" && !streaming && !imageGeneration &&
    (error != null || cancelled || truncated || tools.any { it.status == ToolStatus.Error } && unresolvedCalls(tools).isNotEmpty())

fun CloudChatState.creationDeliveryProgress(): CreationProgress? {
    if (sending || !nativeAgentAvailable || planningCreation) return null
    if (messages.lastOrNull()?.let { it.role == "assistant" && !it.streaming } != true) return null
    val approved = creation?.approved == true
    return creationProgress(messages, approved).takeIf { approved || it.lastActionTurnId != null }
}

/** Inspect the referenced drafts again at the time of the click, including writes whose result was interrupted. */
internal suspend fun CreationProgress.resumeInput(store: MobileGameStore?, draft: CreationDraft?, plan: ChatTurn?): String = buildString {
    appendLine("结合本会话原需求，从同一作品的已保存进度继续，只补缺失或失败的步骤。")
    directories.singleOrNull()?.let { appendLine("本次续作的草稿目录是 $it。") }
    if (useCreationPlan) {
        draft?.let { appendLine("已确认的需求：${it.request}") }
        plan?.let { appendLine("原方案见 conversation_read(message_id=${it.id})。") }
    }
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
        if (work != null && draftChangedSinceInstall) {
            appendLine("手机上已安装的《${work.name}》是草稿更新前的版本。先按原有授权流程安装当前草稿，再对新安装的作品运行 app_validate。")
        } else if (work != null) {
            appendLine("手机上现有《${work.name}》版本 ${work.version} 已通过文件校验。核对是否本次目标，复用已完成的安装；仅在需要更新时按原有授权流程更新。")
        } else appendLine("尚未核实已安装版本，先检查安装状态。")
    }
    append("保留已有源码和数据，使用同一草稿。需要调整已保存的文件时，用 edit_file 做局部修改。中断的大文件分成可独立保存的小步。")
    append("按需求完成并安装作品，调用 app_validate 校验已安装的作品，再报告可打开的结果。")
}
