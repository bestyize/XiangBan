package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.PathEffect
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import kotlin.time.Instant
import xyz.xiangban.xiaoban.common.community.*

private enum class WorkFilter(val label: String) {
    All("全部"), Draft("草稿"), Live("已上线"), Review("审核中"), Removed("已下架");
}

private sealed interface OwnWork {
    val key: String
    val title: String
    val kind: String
    val status: WorkFilter
    val metadata: String

    data class Draft(val game: MobileGame, val editDate: String?) : OwnWork {
        override val key get() = "draft:${game.id}"
        override val title get() = game.name
        override val kind get() = game.kind
        override val status get() = WorkFilter.Draft
        override val metadata get() = listOfNotNull(kindLabel(kind), "仅本机", editDate?.let { "编辑于 $it" }).joinToString(" · ")
    }

    data class Published(val pack: Pack) : OwnWork {
        override val key get() = "pack:${pack.id}"
        override val title get() = pack.title
        override val kind get() = pack.kind
        override val status get() = when {
            pack.status == "removed" -> WorkFilter.Removed
            pack.status in listOf("pending", "rejected") || pack.inReview -> WorkFilter.Review
            else -> WorkFilter.Live
        }
        override val metadata get() = listOfNotNull(kindLabel(kind), "v${pack.version}",
            pack.updatedAt.take(10).takeIf { it.length == 10 }?.let { "更新于 ${it.takeLast(5)}" }).joinToString(" · ")
    }
}

private val OwnWork.statusLabel: String
    get() = if (this is OwnWork.Published && pack.status == "rejected") "未通过" else status.label

private data class WorkConfirmation(val work: OwnWork, val takeDown: Boolean = false)

@Composable
internal fun MyContentScreen(env: CommunityEnv, chat: CloudChatController, back: () -> Unit, open: (Route) -> Unit) {
    val session by env.session.state.collectAsState()
    key(session.generation) {
        MyWorksPage(env, chat, session.loggedIn, back, open)
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun MyWorksPage(
    env: CommunityEnv, chat: CloudChatController, loggedIn: Boolean, back: () -> Unit, open: (Route) -> Unit,
) {
    val scope = rememberCoroutineScope()
    val games = env.games
    val revision = games?.revision?.collectAsState()?.value
    val draftsLoader = remember(games) { Loader(scope) { games?.scan() ?: MobileGameScan(emptyList(), emptyList()) } }
    LaunchedEffect(draftsLoader, revision) { draftsLoader.reload() }
    val packsLoader = if (loggedIn) rememberLoader(env, "my-works") { env.content.myPacks() } else null
    val draftsState by draftsLoader.state.collectAsState()
    val packsState = packsLoader?.state?.collectAsState()?.value
    var knownDrafts by remember { mutableStateOf<MobileGameScan?>(null) }
    var knownPacks by remember { mutableStateOf(emptyList<Pack>()) }
    val scanned = (draftsState as? Load.Ready)?.value
    val loadedPacks = (packsState as? Load.Ready)?.value?.items
    LaunchedEffect(scanned) { if (scanned != null) knownDrafts = scanned }
    LaunchedEffect(loadedPacks) { if (loadedPacks != null) knownPacks = loadedPacks }
    val drafts = scanned ?: knownDrafts
    val packs = loadedPacks ?: knownPacks
    val entries = remember(drafts, packs) {
        val publishedIds = packs.map { it.kind to it.slug }.toSet()
        drafts?.games.orEmpty()
            .filterNot { (it.kind to it.id) in publishedIds }
            .sortedByDescending { drafts?.editedAtMillis?.get(it.id) }
            .map { game ->
                val date = drafts?.editedAtMillis?.get(game.id)?.let {
                    Instant.fromEpochMilliseconds(it + env.utcOffsetMinutes * 60_000L).toString().substring(5, 10)
                }
                OwnWork.Draft(game, date)
            } + packs.map { OwnWork.Published(it) }
    }
    var filter by rememberSaveable { mutableStateOf(WorkFilter.All.name) }
    var menu by remember { mutableStateOf<OwnWork?>(null) }
    var confirmation by remember { mutableStateOf<WorkConfirmation?>(null) }
    var busy by remember { mutableStateOf(false) }
    var actionError by remember { mutableStateOf<String?>(null) }
    var toast by remember { mutableStateOf<String?>(null) }
    val selected = WorkFilter.valueOf(filter)
    val visible = if (selected == WorkFilter.All) entries else entries.filter { it.status == selected }
    val loading = draftsState == Load.Loading || packsState == Load.Loading
    val failure = (packsState as? Load.Failed)?.message ?: (draftsState as? Load.Failed)?.message
        ?: drafts?.errors?.firstOrNull()

    fun refresh() {
        draftsLoader.reload()
        packsLoader?.reload()
    }

    fun openWork(work: OwnWork) {
        when (work) {
            is OwnWork.Draft -> if (env.platform?.openGame(work.game.id) != true) toast = "当前无法打开这件作品"
            is OwnWork.Published -> open(if (work.kind in CapabilityPackKinds) Route.Ability(work.pack.id) else Route.Pack(work.pack.id))
        }
    }

    fun confirm(work: OwnWork, takeDown: Boolean = false) {
        menu = null
        actionError = null
        confirmation = WorkConfirmation(work, takeDown)
    }

    Box(Modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize().navigationBarsPadding()) {
            RefinedScreenBar("我的作品", back, divider = entries.isEmpty()) {
                if (entries.isNotEmpty()) RefinedBarAction("发布", { open(Route.PublishPack()) }, accent = true)
            }
            if (entries.isNotEmpty()) {
                RefinedTabBar(WorkFilter.entries.map { tab ->
                    val count = if (tab == WorkFilter.All) entries.size else entries.count { it.status == tab }
                    RefinedTab(tab.name, tab.label, count.toLong())
                }, filter, compact = true) { filter = it }
            }
            if (failure != null) MineInlineRetry(failure, "重试", ::refresh)
            PullToRefreshBox(
                isRefreshing = loading && entries.isNotEmpty(),
                onRefresh = { if (!busy) refresh() },
                modifier = Modifier.weight(1f).fillMaxWidth(),
            ) {
                when {
                    visible.isNotEmpty() -> key(filter) {
                        LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(bottom = 24.dp)) {
                            if (loggedIn) item {
                                Row(
                                    Modifier.fillMaxWidth().height(28.dp),
                                    horizontalArrangement = Arrangement.spacedBy(5.dp, Alignment.CenterHorizontally),
                                    verticalAlignment = Alignment.CenterVertically,
                                ) {
                                    val ink = LocalExtraColors.current.ink3
                                    XIcon(XIcons.Refresh, tint = ink, size = 12.dp)
                                    Text("下拉刷新已发布作品审核状态", style = XType.Caption.copy(fontSize = 11.5.sp), color = ink)
                                }
                            }
                            itemsIndexed(visible, key = { _, work -> work.key }) { index, work ->
                                Column(Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface)) {
                                    if (index > 0) Hairline(Modifier.padding(start = 76.dp))
                                    OwnWorkRow(work, { openWork(work) }, { menu = work })
                                }
                            }
                        }
                    }
                    loading && entries.isEmpty() -> Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) { Spinner() }
                    failure != null && entries.isEmpty() -> CenterMessage("暂时无法读取作品")
                    entries.isEmpty() -> MineSubpageEmptyState(
                        XIcons.Folder, "还没有作品",
                        "在手机或电脑创作的小程序、游戏与桌面装扮，未发布草稿与已上线版本都会在这里管理",
                    ) {
                        MineEmptyAction("去创作", { open(Route.AiCreate()) }, XIcons.Plus)
                        Text(
                            "已有作品包？直接发布",
                            Modifier.padding(top = 4.dp).clip(CircleShape).clickable(role = Role.Button) { open(Route.PublishPack()) }
                                .padding(horizontal = 12.dp, vertical = 6.dp),
                            style = XType.Secondary, fontWeight = FontWeight.Medium, color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                    else -> MineSubpageEmptyState(
                        XIcons.Folder,
                        if (selected == WorkFilter.Draft) "暂无未发布的草稿" else "该分类下暂无作品",
                        if (selected == WorkFilter.Draft) "创作完暂不公开的作品会先保存在本机草稿中，随时可继续修改或发布"
                        else "切换其他状态标签查看你的作品",
                    )
                }
            }
        }
        menu?.let { work ->
            WorkMenu(work, onDismiss = { menu = null }, onOpen = { menu = null; openWork(work) },
                onEdit = { game -> menu = null; open(Route.AiCreate(game.id)) },
                onPublish = {
                    menu = null
                    open(when (work) {
                        is OwnWork.Draft -> Route.PublishPack(localGameId = work.game.id)
                        is OwnWork.Published -> Route.PublishPack(packId = work.pack.id)
                    })
                },
                onRemove = { confirm(work, takeDown = true) },
                onDelete = { confirm(work) },
            )
        }
        confirmation?.let { action ->
            WorkConfirmationDialog(action, busy, actionError, { confirmation = null }) {
                if (busy) return@WorkConfirmationDialog
                busy = true
                actionError = null
                scope.launch {
                    try {
                        when (val work = action.work) {
                            is OwnWork.Draft -> {
                                check(chat.state.value.runningSessions.isEmpty()) { "请先结束正在运行的创作任务" }
                                checkNotNull(games) { "当前设备无法删除本机草稿" }.delete(work.game.id)
                                val remaining = drafts?.copy(games = drafts.games.filterNot { it.id == work.game.id })
                                knownDrafts = remaining
                                if (remaining != null) draftsLoader.updateReady { remaining }
                                toast = "已删除草稿「${work.title}」"
                            }
                            is OwnWork.Published -> {
                                if (action.takeDown) env.content.removeOwn(work.pack.id) else env.content.deleteOwn(work.pack.id)
                                knownPacks = if (action.takeDown) packs.map {
                                    if (it.id == work.pack.id) it.copy(status = "removed", inReview = false) else it
                                } else packs.filterNot { it.id == work.pack.id }
                                packsLoader?.reload()
                                toast = if (action.takeDown) "已下架「${work.title}」" else "已删除作品「${work.title}」"
                            }
                        }
                        confirmation = null
                    } catch (e: CancellationException) {
                        throw e
                    } catch (e: Exception) {
                        if (e is ApiException && e.status == 404) {
                            packsLoader?.reload()
                            confirmation = null
                            toast = "作品状态已变化，正在刷新列表"
                        } else {
                            actionError = if (e is ApiException && e.status == 405) "论坛服务暂未支持永久删除"
                            else e.message ?: "操作失败，请重试"
                        }
                    } finally {
                        busy = false
                    }
                }
            }
        }
        Toast(toast) { toast = null }
    }
}

@Composable
private fun OwnWorkRow(work: OwnWork, onClick: () -> Unit, onMore: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    val extra = LocalExtraColors.current
    val tileShape = RoundedCornerShape(14.dp)
    val icon = when (work.kind) {
        "game" -> XIcons.Gamepad
        "app", "tool" -> XIcons.Grid
        else -> XIcons.SparkleSmall
    }
    Row(
        Modifier.fillMaxWidth().clickable(role = Role.Button, onClick = onClick)
            .padding(start = 16.dp, end = 10.dp, top = 14.dp, bottom = 14.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Box(Modifier.size(48.dp).background(extra.fill, tileShape).border(0.5.dp, extra.hairline, tileShape),
            contentAlignment = Alignment.Center) {
            XIcon(icon, tint = colors.onSurfaceVariant, size = 22.dp)
        }
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(7.dp)) {
                Text(work.title, Modifier.weight(1f, fill = false), style = XType.Body.copy(fontSize = 15.5.sp),
                    fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                WorkStatusPill(work)
            }
            Text(work.metadata, style = XType.Caption.copy(fontSize = 12.5.sp, lineHeight = 17.sp),
                color = extra.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
        }
        Box(Modifier.size(36.dp).clip(RoundedCornerShape(10.dp)).clickable(role = Role.Button, onClick = onMore),
            contentAlignment = Alignment.Center) {
            XIcon(XIcons.More, tint = extra.ink3, size = 18.dp, description = "更多操作")
        }
    }
}

@Composable
private fun WorkStatusPill(work: OwnWork) {
    val colors = MaterialTheme.colorScheme
    val extra = LocalExtraColors.current
    val draftLine = Color(if (ThemeState.dark) 0xFF4D4D54 else 0xFFC9CCD2)
    val modifier = when (work.status) {
        WorkFilter.Draft -> Modifier.background(colors.background, CircleShape).drawBehind {
            val stroke = 1.dp.toPx()
            drawRoundRect(draftLine, Offset(stroke / 2, stroke / 2), Size(size.width - stroke, size.height - stroke),
                CornerRadius(size.height / 2), style = Stroke(stroke, pathEffect = PathEffect.dashPathEffect(floatArrayOf(3.dp.toPx(), 2.dp.toPx()))))
        }
        WorkFilter.Removed -> Modifier.background(extra.fill, CircleShape)
        else -> Modifier
    }
    val tint = when {
        work.statusLabel == "未通过" -> Danger
        work.status == WorkFilter.Live -> Color(if (ThemeState.dark) 0xFF45BE83 else 0xFF2FA36B)
        work.status == WorkFilter.Review -> Color(if (ThemeState.dark) 0xFFF0B65A else 0xFFB26A00)
        work.status == WorkFilter.Removed -> extra.ink3
        else -> colors.onSurfaceVariant
    }
    Box(modifier.heightIn(min = 20.dp).padding(horizontal = 7.dp), contentAlignment = Alignment.Center) {
        Text(work.statusLabel, style = XType.Caption.copy(fontSize = 11.sp), fontWeight = FontWeight(550), color = tint, maxLines = 1)
    }
}

@Composable
private fun WorkMenu(
    work: OwnWork, onDismiss: () -> Unit, onOpen: () -> Unit, onEdit: (MobileGame) -> Unit,
    onPublish: () -> Unit, onRemove: () -> Unit, onDelete: () -> Unit,
) {
    BottomSheet(onDismiss, flush = true, cornerRadius = 20.dp, topPadding = 0.dp, scrim = Color.Black.copy(alpha = .34f)) {
        SheetDragHandle()
        Column(Modifier.weight(1f, fill = false).verticalScroll(rememberScrollState())) {
            Column(Modifier.padding(start = 16.dp, end = 16.dp, top = 4.dp, bottom = 10.dp)) {
                Text(work.title, style = XType.Section, fontWeight = FontWeight(650), maxLines = 2, overflow = TextOverflow.Ellipsis)
                val detail = when (work) {
                    is OwnWork.Draft -> "草稿 · 仅本机"
                    is OwnWork.Published -> "v${work.pack.version} · ${work.statusLabel}"
                }
                Text("${kindLabel(work.kind)} · $detail", Modifier.padding(top = 2.dp),
                    style = XType.Caption.copy(fontSize = 12.5.sp), color = LocalExtraColors.current.ink3)
            }
            if (work is OwnWork.Draft) {
                val isGame = work.kind == "game"
                SheetMenuRow(if (isGame) XIcons.Play else XIcons.Eye, if (isGame) "试玩作品" else "预览作品", first = true, onClick = onOpen)
                SheetMenuRow(XIcons.Pen, "继续修改") { onEdit(work.game) }
                SheetMenuRow(XIcons.Upload, "发布到社区", onClick = onPublish)
                SheetMenuRow(XIcons.Trash, "删除草稿", danger = true, onClick = onDelete)
            } else {
                SheetMenuRow(XIcons.Eye, "查看作品详情", first = true, onClick = onOpen)
                if (work.status != WorkFilter.Removed) {
                    SheetMenuRow(XIcons.Upload, "发布新版本", onClick = onPublish)
                    SheetMenuRow(XIcons.EyeOff, "下架作品", onClick = onRemove)
                }
                SheetMenuRow(XIcons.Trash, "删除作品", danger = true, onClick = onDelete)
            }
        }
        SheetCancelFooter(onDismiss)
    }
}

@Composable
private fun WorkConfirmationDialog(action: WorkConfirmation, busy: Boolean, error: String?, onDismiss: () -> Unit, onConfirm: () -> Unit) {
    val work = action.work
    val local = work is OwnWork.Draft
    val verb = if (action.takeDown) "下架" else if (local) "删除草稿" else "删除作品"
    val description = when {
        action.takeDown -> "下架后，作品只在你的「已下架」列表中显示。已应用的副本仍可使用。"
        local -> "将清理本机草稿文件与试玩存档，删除后无法恢复。已导出和已发布的副本仍可使用。"
        work.status == WorkFilter.Removed -> "将从你的作品列表中永久移除。删除后无法恢复，本机源文件仍可使用。"
        else -> "将从社区和你的作品列表中永久移除。删除后无法恢复，本机源文件和已应用的副本仍可使用。"
    }
    val colors = MaterialTheme.colorScheme
    val extra = LocalExtraColors.current
    Dialog(onDismissRequest = { if (!busy) onDismiss() },
        properties = DialogProperties(dismissOnBackPress = !busy, dismissOnClickOutside = !busy, usePlatformDefaultWidth = false)) {
        Column(
            Modifier.padding(horizontal = 24.dp, vertical = 14.dp).widthIn(max = 318.dp).fillMaxWidth()
                .dialogCardSurface(colors.surface).verticalScroll(rememberScrollState())
                .padding(start = 20.dp, end = 20.dp, top = 24.dp, bottom = 14.dp),
        ) {
            Text("$verb「${work.title}」？", style = XType.Title.copy(fontSize = 17.sp), fontWeight = FontWeight(650))
            Text(description, Modifier.padding(top = 8.dp), style = XType.Secondary.copy(fontSize = 13.5.sp, lineHeight = 20.sp),
                color = colors.onSurfaceVariant)
            error?.let { Text(it, Modifier.padding(top = 8.dp), style = XType.Secondary, color = Danger) }
            Spacer(Modifier.height(20.dp))
            Box(
                Modifier.fillMaxWidth().heightIn(min = 44.dp).clip(CircleShape)
                    .background(if (action.takeDown || busy) extra.fill else Danger)
                    .clickable(enabled = !busy, role = Role.Button, onClick = onConfirm).padding(6.dp),
                contentAlignment = Alignment.Center,
            ) {
                Text(if (busy) "处理中…" else if (action.takeDown) "确认下架" else if (local) "删除草稿" else "永久删除",
                    style = XType.Body, fontWeight = FontWeight.SemiBold, color = if (action.takeDown || busy) colors.onSurface else Color.White)
            }
            Spacer(Modifier.height(6.dp))
            Box(Modifier.fillMaxWidth().heightIn(min = 40.dp).clip(CircleShape)
                .clickable(enabled = !busy, role = Role.Button, onClick = onDismiss), contentAlignment = Alignment.Center) {
                Text("取消", style = XType.Body.copy(fontSize = 14.5.sp), color = colors.onSurfaceVariant.copy(alpha = if (busy) .38f else 1f))
            }
        }
    }
}
