package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.selection.SelectionContainer
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Checkbox
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.backhandler.BackHandler
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.JsonObject
import xyz.xiangban.xiaoban.common.community.*

private val ContentSpacing = Arrangement.spacedBy(8.dp)

@Composable
private fun ManifestSection(title: String, value: String?, collapsible: Boolean = false) {
    WhiteCard(padding = 14.dp) {
        Text(title, style = XType.Body, fontWeight = FontWeight.SemiBold)
        if (collapsible && value != null) ExpandableDescription(value)
        else SelectionContainer { Text(value ?: "发布者未提供此项说明", Modifier.padding(top = 8.dp), style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant) }
    }
}

@Composable
fun AbilityDetailScreen(env: CommunityEnv, id: Long, back: () -> Unit, open: ((Route) -> Unit)? = null) = CommunityDetail {
    val loader = rememberLoader(env, "ability-$id") { env.forum.pack(id) }
    val state by loader.state.collectAsState()
    Column(Modifier.fillMaxSize()) {
        if (state !is Load.Ready<*>) ScreenBar("能力详情", onBack = back)
        Box(Modifier.weight(1f)) { LoadBox(state, loader::reload) { pack -> CapabilityDetailContent(env, pack, back, open = open) } }
    }
}

/** 能力详情同时服务于能力和通用作品入口。 */
@Composable
internal fun CapabilityDetailContent(env: CommunityEnv, pack: Pack, back: () -> Unit, open: ((Route) -> Unit)? = null) {
    var sharing by rememberSaveable(pack.id) { mutableStateOf(false) }
    val manifest = pack.versions.firstOrNull { it.version == pack.version }?.manifest ?: JsonObject(emptyMap())
    val prompt = manifestValue(manifest, "prompt", "promptText", "systemPrompt", "system_prompt")
    val manualOnly = manifest["manualOnly"]?.toString() == "true"
    Column(Modifier.fillMaxSize()) {
        val author = pack.author
        DetailAuthorBar(author?.nickname ?: "社区作品", back, onShare = { sharing = true },
            onAuthor = if (author != null && open != null) ({ open(Route.User(author.uid)) }) else null) {
            author?.let { DetailAuthorFollow(env, it.uid, open) }
        }
        WorkInteractionLayout(env, "pack", pack.id, open, { sharing = true }, enabled = pack.status == "published",
            linkedTopicId = pack.topicId) { comments ->
            Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState())) {
                DetailGallery(listOfNotNull(pack.coverUrl?.takeIf(String::isNotBlank)?.let(env.forum::imageUrl)), pack.title)
                DetailBody {
                    DetailHeading(pack.title)
                    if (pack.summary.isNotBlank()) Text(pack.summary, style = XType.DetailBody)
                    DetailTags(pack.tags)
                    if (manualOnly) BlueTag("手动模板 · 不自动执行", Modifier.padding(top = 8.dp))
                    DetailMetadata("${kindLabel(pack.kind)} · v${pack.version}")
                    if (pack.status != "published") Text(contentWorkState(pack), Modifier.padding(top = 8.dp), color = Danger, style = XType.Secondary)
                }
                Column(Modifier.fillMaxWidth().padding(horizontal = 16.dp), verticalArrangement = ContentSpacing) {
                    if (pack.status == "published") NativeReportAction(env, "pack", pack.id, open)
                    ManifestSection("能做", manifestValue(manifest, "canDo", "can_do", "capabilities"))
                    ManifestSection("不能做", manifestValue(manifest, "cannotDo", "cannot_do", "limitations"))
                    ManifestSection("所需权限", manifestValue(manifest, "permissions", "requiredPermissions"))
                    ManifestSection("运行位置", manifestValue(manifest, "runLocation", "run_location", "runtime", "executionLocation"))
                    ManifestSection("完整提示词", prompt, collapsible = true)
                    if (prompt != null) GhostPillButton("复制提示词", { env.platform?.copy(prompt) }, Modifier.fillMaxWidth(), enabled = env.platform != null)
                    var expanded by rememberSaveable { mutableStateOf(false) }
                    GhostPillButton(if (expanded) "收起完整清单" else "查看完整清单", { expanded = !expanded }, Modifier.fillMaxWidth())
                    if (expanded) ManifestSection("资源清单原文", CommunityJson.encodeToString(manifest))
                    FootNote(if (manualOnly) "这是可下载的手动模板，按说明使用或复制提示词；不会自动执行任务。" else "手机用于浏览和分享。实际运行与授权在电脑端确认，导入不会自动执行；未提供权限说明不代表无需权限。")
                    GhostPillButton("复制电脑端打开链接", { env.platform?.copy(env.forum.packUrl(pack.id)) }, Modifier.fillMaxWidth(), enabled = env.platform != null && pack.status == "published")
                }
                comments()
            }
        }
    }
    OverlayTransition(sharing) { panel ->
        CommunityShareSheet(
            env = env,
            url = env.forum.packUrl(pack.id),
            heading = "分享能力",
            onDismiss = { sharing = false },
            panelModifier = panel,
            author = pack.author?.nickname.orEmpty(),
            dateLabel = pack.updatedAt.take(10),
            coverUrl = pack.coverUrl?.takeIf(String::isNotBlank)?.let(env.forum::imageUrl),
        )
    }
}

@Composable
fun SchemeDetailScreen(env: CommunityEnv, id: Long, back: () -> Unit, open: (Route) -> Unit) = CommunityDetail {
    var sharing by rememberSaveable(id) { mutableStateOf(false) }
    val loader = rememberLoader(env, "scheme-$id") { env.content.scheme(id) }
    val state by loader.state.collectAsState()
    Column(Modifier.fillMaxSize()) {
        if (state !is Load.Ready<*>) ScreenBar("桌面方案", onBack = back)
        LoadBox(state, loader::reload) { scheme ->
            val author = scheme.author
            DetailAuthorBar(author.nickname ?: "社区作者", back, onShare = { sharing = true }, onAuthor = { open(Route.User(author.uid)) }) {
                DetailAuthorFollow(env, author.uid, open)
            }
            WorkInteractionLayout(env, "scheme", scheme.id, open, { sharing = true }) { comments ->
                LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(bottom = 16.dp), verticalArrangement = ContentSpacing) {
                    item { DetailBody {
                        DetailHeading(scheme.title)
                        if (scheme.summary.isNotBlank()) Text(scheme.summary, style = XType.DetailBody)
                        DetailTags(listOf("社区方案"))
                        DetailMetadata("${scheme.createdAt.take(10)} 发布 · ${scheme.items.size} 项作品")
                    } }
                    item { Box(Modifier.padding(horizontal = 16.dp)) { NativeReportAction(env, "scheme", scheme.id, open) } }
                    item { Text("包含的内容", Modifier.padding(horizontal = 16.dp), style = XType.Section, fontWeight = FontWeight.SemiBold) }
                    itemsIndexed(scheme.items, key = { index, item -> "$index-${item.packId}-${item.version}" }) { _, item -> WhiteCard(Modifier.padding(horizontal = 16.dp), padding = 14.dp) {
                        Text(if (item.available) item.title ?: item.slug else "内容不可用", style = XType.Body, fontWeight = FontWeight.Medium)
                        Text(if (item.available) "${kindLabel(item.kind)} · 方案版本 v${item.version}" else "此项已下架或当前不可访问", Modifier.padding(top = 6.dp), style = XType.Caption, color = LocalExtraColors.current.ink3)
                        if (item.available) GhostPillButton("查看内容", { open(if (item.kind in CapabilityPackKinds) Route.Ability(item.packId) else Route.Pack(item.packId)) }, Modifier.padding(top = 8.dp))
                    } }
                    item { Box(Modifier.padding(horizontal = 16.dp)) { FootNote("方案是一份内容清单。手机不会应用桌面内容；在电脑上打开后，逐项检查版本、可用性和权限再确认。") } }
                    item { comments() }
                }
            }
        }
    }
    OverlayTransition(sharing) { panel ->
        val scheme = (state as? Load.Ready)?.value
        CommunityShareSheet(
            env = env,
            url = "${env.config.forumBase.trimEnd('/')}/schemes/$id",
            heading = "分享方案",
            onDismiss = { sharing = false },
            panelModifier = panel,
            author = scheme?.author?.nickname.orEmpty(),
            dateLabel = scheme?.createdAt.orEmpty().take(10),
        )
    }
}

/** 创作栏的“方案”读取 schemes，不把它作为压缩包 kind。 */
@Composable
internal fun NativeSchemesFeed(env: CommunityEnv, open: (Route) -> Unit) {
    var page by rememberSaveable { mutableStateOf(1) }
    val loader = rememberLoader(env, "scheme-page-$page") { env.content.schemes(page) }
    val state by loader.state.collectAsState()
    LoadBox(state, loader::reload) { result ->
        if (result.items.isEmpty()) CenterMessage("这里还没有桌面方案", "刷新", loader::reload)
        else LazyColumn(Modifier.fillMaxSize(), contentPadding = FeedPadding, verticalArrangement = ContentSpacing) {
            items(result.items, key = { it.id }) { scheme -> WhiteCard(Modifier.clickable { open(Route.Scheme(scheme.id)) }, padding = 14.dp) {
                Text(scheme.title, style = XType.Section, fontWeight = FontWeight.SemiBold)
                Text("${scheme.items.size} 项内容 · ${scheme.author.nickname ?: "作者"}", Modifier.padding(top = 8.dp), style = XType.Caption, color = LocalExtraColors.current.ink3)
            } }
            item { Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                GhostPillButton("上一页", { page-- }, Modifier.weight(1f), enabled = page > 1)
                GhostPillButton("下一页", { page++ }, Modifier.weight(1f), enabled = result.hasMore)
            } }
        }
    }
}

private fun packFilterKey(pack: Pack): String = when {
    pack.status == "removed" -> "removed"
    pack.status == "rejected" || pack.inReview -> "review"
    else -> "live"
}

private fun packStatusBadge(pack: Pack, dark: Boolean, extra: ExtraColors): Triple<String, Color, Color> = when {
    pack.status == "removed" -> Triple("已下架", extra.fill, extra.ink3)
    pack.status == "rejected" -> Triple("未通过", DangerSoft, Danger)
    pack.inReview -> Triple("审核中", if (dark) Color(0xFF362916) else Color(0xFFFDF3E3), if (dark) Color(0xFFF0B65A) else Color(0xFFB26A00))
    pack.trustLevel == 0 -> Triple("官方", if (dark) Color(0xFF1A3126) else Color(0xFFEAF6F0), Success)
    else -> Triple("已上线", if (dark) Color(0xFF1A3126) else Color(0xFFEAF6F0), Success)
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MyContentScreen(env: CommunityEnv, back: () -> Unit, open: (Route) -> Unit) {
    val session by env.session.state.collectAsState()
    val extra = LocalExtraColors.current
    val dark = ThemeState.dark

    if (!session.loggedIn) {
        Column(Modifier.fillMaxSize()) {
            ScreenBar("我的作品", onBack = back)
            RefinedEmptyState(XIcons.Folder, "登录后查看自己的作品", "在手机或电脑创作的小程序、游戏与桌面装扮，发布后都会在这里管理版本", "登录", { open(Route.Login) })
        }
        return
    }

    val loader = rememberLoader(env, "my-packs-${session.accountId}") { env.content.myPacks() }
    val state by loader.state.collectAsState()
    val scope = rememberCoroutineScope()
    var filter by rememberSaveable(session.accountId) { mutableStateOf("all") }
    var actionPack by remember { mutableStateOf<Pack?>(null) }
    var removing by remember { mutableStateOf<Pack?>(null) }
    var busy by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    val allItems = (state as? Load.Ready)?.value?.items.orEmpty()

    Box(Modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize()) {
            ScreenBar(
                "我的作品",
                onBack = back,
                trailing = {
                    Text(
                        "发布",
                        Modifier.clip(CircleShape)
                            .clickable { open(Route.PublishPack()) }
                            .padding(horizontal = 10.dp, vertical = 6.dp),
                        style = XType.Secondary,
                        fontWeight = FontWeight.SemiBold,
                        color = extra.brandInk,
                    )
                },
            )

            if (allItems.isNotEmpty()) {
                RefinedTabBar(
                    listOf(
                        RefinedTab("all", "全部", allItems.size.toLong()),
                        RefinedTab("live", "已上线", allItems.count { packFilterKey(it) == "live" }.toLong()),
                        RefinedTab("review", "审核中", allItems.count { packFilterKey(it) == "review" }.toLong()),
                        RefinedTab("removed", "已下架", allItems.count { packFilterKey(it) == "removed" }.toLong()),
                    ),
                    filter,
                ) { filter = it }
            }

            PullToRefreshBox(
                isRefreshing = state is Load.Loading && allItems.isNotEmpty(),
                onRefresh = loader::reload,
                modifier = Modifier.weight(1f).fillMaxWidth(),
            ) {
                LoadBox(state, loader::reload) { result ->
                    val filtered = if (filter == "all") result.items else result.items.filter { packFilterKey(it) == filter }
                    if (result.items.isEmpty()) {
                        Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState())) {
                            RefinedEmptyState(
                                icon = XIcons.Folder,
                                title = "还没有发布过作品",
                                desc = "在手机或电脑创作的小程序、游戏与桌面装扮，发布后都会在这里管理版本",
                                actionLabel = "发布作品",
                                onAction = { open(Route.PublishPack()) },
                            )
                        }
                    } else if (filtered.isEmpty()) {
                        Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState())) {
                            RefinedEmptyState(
                                icon = XIcons.Folder,
                                title = "该分类下暂无作品",
                                desc = "切换其他状态标签查看你的已发布作品",
                            )
                        }
                    } else {
                        LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(top = 8.dp, bottom = 24.dp)) {
                            itemsIndexed(filtered, key = { _, it -> it.id }) { idx, pack ->
                                val (statusLabel, statusBg, statusTint) = packStatusBadge(pack, dark, extra)
                                val (kindIcon, kindBg, kindTint) = when (pack.kind) {
                                    "game" -> Triple(XIcons.Gamepad, if (dark) Color(0xFF36221B) else Color(0xFFFFF2ED), if (dark) Color(0xFFFF9C78) else Color(0xFFD95A2B))
                                    "app", "tool" -> Triple(XIcons.Grid, if (dark) Color(0xFF1E2A42) else Color(0xFFEEF4FF), if (dark) Color(0xFF8DB6FF) else Color(0xFF2F6FDB))
                                    else -> Triple(XIcons.SparkleSmall, if (dark) Color(0xFF2A2342) else Color(0xFFF2EEFC), if (dark) Color(0xFFBBA8FF) else Color(0xFF7152D8))
                                }
                                val openDetail = { open(if (pack.kind in CapabilityPackKinds) Route.Ability(pack.id) else Route.Pack(pack.id)) }
                                Column(Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface)) {
                                    if (idx > 0) Hairline(Modifier.padding(start = 76.dp))
                                    Row(
                                        Modifier.fillMaxWidth().clickable(onClick = openDetail)
                                            .padding(start = 16.dp, end = 8.dp, top = 14.dp, bottom = 14.dp),
                                        verticalAlignment = Alignment.CenterVertically,
                                        horizontalArrangement = Arrangement.spacedBy(12.dp),
                                    ) {
                                        Box(
                                            Modifier.size(48.dp).clip(RoundedCornerShape(14.dp)).background(kindBg),
                                            contentAlignment = Alignment.Center,
                                        ) {
                                            XIcon(kindIcon, tint = kindTint, size = 22.dp)
                                        }
                                        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                                            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(7.dp)) {
                                                Text(
                                                    pack.title,
                                                    Modifier.weight(1f, fill = false),
                                                    style = XType.Body,
                                                    fontWeight = FontWeight.SemiBold,
                                                    maxLines = 1,
                                                    overflow = TextOverflow.Ellipsis,
                                                )
                                                Box(
                                                    Modifier.height(20.dp).clip(CircleShape).background(statusBg).padding(horizontal = 7.dp),
                                                    contentAlignment = Alignment.Center,
                                                ) {
                                                    Text(statusLabel, style = XType.Caption, fontWeight = FontWeight.Medium, color = statusTint)
                                                }
                                            }
                                            Text(
                                                "${kindLabel(pack.kind)} · v${pack.version} · 更新于 ${pack.updatedAt.take(10)}",
                                                style = XType.Caption,
                                                color = extra.ink3,
                                                maxLines = 1,
                                                overflow = TextOverflow.Ellipsis,
                                            )
                                        }
                                        IconTap(XIcons.More, "更多操作", { actionPack = pack }, size = 18.dp, tint = extra.ink3)
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        actionPack?.let { pack ->
            val (statusLabel, _, _) = packStatusBadge(pack, dark, extra)
            BottomSheet(onDismiss = { actionPack = null }, flush = true, cornerRadius = 20.dp) {
                Column(Modifier.padding(start = 16.dp, end = 16.dp, bottom = 10.dp)) {
                    Text(pack.title, style = XType.Section, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                    Text("${kindLabel(pack.kind)} · v${pack.version} · $statusLabel", Modifier.padding(top = 2.dp), style = XType.Caption, color = extra.ink3)
                }
                SheetMenuRow(XIcons.Eye, "查看作品详情", first = true) {
                    actionPack = null
                    open(if (pack.kind in CapabilityPackKinds) Route.Ability(pack.id) else Route.Pack(pack.id))
                }
                if (pack.status != "removed") {
                    SheetMenuRow(XIcons.Upload, "发布新版本") {
                        actionPack = null
                        open(Route.PublishPack(pack.id))
                    }
                    SheetMenuRow(XIcons.Trash, "下架作品", danger = true) {
                        actionPack = null
                        removing = pack
                        error = null
                    }
                }
                SheetCancelFooter { actionPack = null }
            }
        }

        OverlayTransition(removing != null, direction = "center") {
            CenterDialog({ if (!busy) removing = null }) {
                Text("下架「${removing?.title.orEmpty()}」？", style = XType.Title, fontWeight = FontWeight.SemiBold)
                Text("下架后不再出现在公开入口，已经应用过的内容不受影响。", Modifier.padding(top = 10.dp), style = XType.Secondary, color = extra.ink3)
                error?.let { Text(it, Modifier.padding(top = 8.dp), color = Danger, style = XType.Secondary) }
                Row(Modifier.padding(top = 16.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    GhostPillButton("取消", { removing = null }, Modifier.weight(1f), enabled = !busy)
                    PillButton(if (busy) "处理中…" else "确认下架", {
                        if (busy) return@PillButton
                        val id = removing?.id ?: return@PillButton
                        busy = true
                        scope.launch {
                            try { env.content.removeOwn(id); removing = null; loader.reload() }
                            catch (e: CancellationException) { throw e }
                            catch (e: Exception) { error = messageOf(e) }
                            finally { busy = false }
                        }
                    }, Modifier.weight(1f), enabled = !busy)
                }
            }
        }
    }
}
@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun PublishContentScreen(
    env: CommunityEnv, packId: Long?, back: () -> Unit, open: (Route) -> Unit, localGameId: String? = null,
    onPublished: ((Long, String) -> Unit)? = null,
) {
    val session by env.session.state.collectAsState()
    val accountGeneration by rememberUpdatedState(session.generation)
    val scope = rememberCoroutineScope()
    val forPost = onPublished != null
    var kind by rememberSaveable(packId, session.accountId) { mutableStateOf(if (localGameId != null) "game" else "effect") }
    var title by rememberSaveable(packId, session.accountId) { mutableStateOf("") }
    var summary by rememberSaveable(packId, session.accountId) { mutableStateOf("") }
    var tags by rememberSaveable(packId, session.accountId) { mutableStateOf("") }
    var changelog by rememberSaveable(packId, session.accountId) { mutableStateOf("") }
    var rights by rememberSaveable(packId, session.accountId) { mutableStateOf(false) }
    var fileKey by rememberSaveable(packId, session.accountId) { mutableStateOf("") }
    var file by remember(packId, session.accountId) { mutableStateOf<ForumUploadFile?>(null) }
    var selectedName by remember(session.accountId) { mutableStateOf("") }
    var suggestedTitle by rememberSaveable(session.accountId) { mutableStateOf("") }
    var suggestedSummary by rememberSaveable(session.accountId) { mutableStateOf("") }
    var busy by remember { mutableStateOf(false) }
    var step by remember { mutableStateOf("") }
    var error by remember { mutableStateOf<String?>(null) }
    var published by rememberSaveable(packId, session.accountId) { mutableStateOf<Long?>(null) }
    var importedGame by rememberSaveable(localGameId, session.accountId) { mutableStateOf(false) }
    LaunchedEffect(localGameId, session.accountId) {
        if (localGameId != null && session.loggedIn && file == null && fileKey.isEmpty()) {
            busy = true; step = "正在读取本地作品…"
            try {
                val game = env.games?.read(localGameId) ?: error("本地作品不可用")
                file = env.platform?.gamePackage(localGameId) ?: error("当前平台不支持作品打包")
                if (!importedGame) { kind = game.kind; title = game.name; summary = game.description; importedGame = true }
            } catch (e: CancellationException) { throw e }
            catch (e: Exception) { error = e.message ?: "读取本地作品失败" }
            finally { busy = false; step = "" }
        }
    }
    BackHandler(enabled = busy || forPost) { if (!busy) back() }
    Column(Modifier.fillMaxSize().imePadding()) {
        val heading = if (forPost) "上传作品" else if (packId == null) "发布内容包" else "发布新版本"
        ScreenBar(heading, onBack = { if (!busy) back() })
        if (!session.loggedIn) { CenterMessage("登录后发布作品", "登录", { open(Route.Login) }); return@Column }
        val platform = env.platform
        val canPick = platform != null && if (forPost) platform.forumWorkPickerSupported else platform.forumFilePickerSupported
        val webPublisher = "${env.config.forumBase.trimEnd('/')}/publish${packId?.let { "?pack=$it" }.orEmpty()}"
        if (forPost && !canPick) {
            CenterMessage("在网页版发布后，回到发帖页从「作品」中选取。", "打开网页版", { platform?.openUrl(webPublisher) })
            return@Column
        }
        val existingLoader = rememberLoader(env, "publish-original-$packId-${session.accountId}") { packId?.let { env.forum.pack(it) } }
        val existingState by existingLoader.state.collectAsState()
        val existing = (existingState as? Load.Ready)?.value
        val actualKind = existing?.kind ?: kind
        val versionReady = packId == null || (existing?.author?.uid == session.accountId && existing?.status != "removed")
        if (published != null) {
            CenterMessage("已提交发布\n审核与上架状态可在我的作品查看", "我的作品", { open(Route.MyWorks) })
            return@Column
        }
        Column(Modifier.weight(1f).fillMaxWidth().verticalScroll(rememberScrollState()).padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            if (forPost && (file != null || fileKey.isNotEmpty())) {
                Text(kindLabel(actualKind), style = XType.Secondary, color = LocalExtraColors.current.ink3)
            } else if (packId == null && !forPost) {
                Text("资源类型", style = XType.Secondary, color = LocalExtraColors.current.ink3)
                if (localGameId != null) Text("${kindLabel(actualKind)} · 本机作品", style = XType.Body, fontWeight = FontWeight.Medium)
                else PageChipRow(ContentPackLimits.keys.map { it to kindLabel(it) }, actualKind, { if (!busy) { kind = it; fileKey = "" } })
            } else if (packId != null) {
                when (val state = existingState) {
                    Load.Loading -> Text("正在读取原作品…", style = XType.Body)
                    is Load.Failed -> Banner(state.message, warn = true, actionText = "重试", onAction = existingLoader::reload)
                    is Load.Ready -> Text(existing?.title ?: "作品不可用", style = XType.Title, fontWeight = FontWeight.SemiBold)
                }
                if (existing != null && !versionReady) Text("只有作者可以发布新版本", color = Danger, style = XType.Body)
            }
            if (!canPick) {
                Text("当前设备请在网页版发布内容包。", style = XType.Secondary, color = LocalExtraColors.current.ink3)
                GhostPillButton("打开网页版", { platform?.openUrl(webPublisher) }, enabled = platform != null)
            } else {
                val pickLabel = if (forPost) {
                    if (file == null && fileKey.isEmpty()) "选择作品文件" else "更换作品文件"
                } else if (file == null) "选择内容包 .zip" else "更换内容包"
                GhostPillButton(pickLabel, {
                    if (busy) return@GhostPillButton
                    val generation = session.generation
                    busy = true; error = null
                    scope.launch {
                        try {
                            if (forPost) {
                                val prepared = platform?.pickForumWork() ?: return@launch
                                check(accountGeneration == generation) { "账号已切换，请重新选择" }
                                kind = prepared.kind
                                selectedName = prepared.originalName
                                if (title.isBlank() || title == suggestedTitle) title = prepared.title
                                if (summary.isBlank() || summary == suggestedSummary) summary = prepared.summary
                                suggestedTitle = prepared.title
                                suggestedSummary = prepared.summary
                                file = prepared.file
                            } else {
                                val selected = platform?.pickForumFile(false, ContentPackLimits[actualKind] ?: 0) ?: return@launch
                                check(accountGeneration == generation) { "账号已切换，请重新选择" }
                                file = selected
                            }
                            fileKey = ""
                        } catch (e: CancellationException) { throw e }
                        catch (e: Exception) {
                            error = when (e) {
                                is ApiException, is IllegalArgumentException -> e.message
                                else -> null
                            } ?: "无法读取文件，请重新选择"
                        }
                        finally { busy = false }
                    }
                }, Modifier.fillMaxWidth(), enabled = !busy && versionReady)
                val chosenFile = file
                if (chosenFile != null) {
                    val name = selectedName.ifBlank { chosenFile.name }
                    Text("$name · ${formatSize(chosenFile.bytes.size.toLong())}", style = XType.Secondary)
                }
                val formats = if (forPost) "支持 ZIP、HTML 和 HTM。ZIP 根目录需包含 manifest.json。"
                    else "${kindLabel(actualKind)}上限 ${formatSize(ContentPackLimits[actualKind] ?: 0)}，ZIP 根目录需包含 manifest.json；由服务器检查内容清单。"
                Text(formats, style = XType.Caption, color = LocalExtraColors.current.ink3)
            }
            if (packId == null) {
                TextBox(title, { title = it }, placeholder = "标题（最多 60 字）", enabled = !busy)
                AreaBox(summary, { summary = it }, placeholder = "简介（最多 500 字）", minHeight = 120.dp, enabled = !busy)
                TextBox(tags, { tags = it }, placeholder = "标签，空格或逗号分隔（最多 8 个）", enabled = !busy)
            } else AreaBox(changelog, { changelog = it }, placeholder = "新版本更新说明（最多 500 字）", minHeight = 120.dp, enabled = !busy)
            Row(Modifier.fillMaxWidth(), verticalAlignment = androidx.compose.ui.Alignment.CenterVertically) {
                Checkbox(rights, { rights = it }, enabled = !busy)
                Text("我确认这是我的原创或已获授权，并同意按社区规则发布。", style = XType.Secondary)
            }
            if (fileKey.isNotEmpty()) Text("文件已上传，可继续校验与发布。", style = XType.Caption, color = LocalExtraColors.current.ink3)
            Text(if (forPost) "作品发布成功后会添加到当前帖子草稿。" else "上传和导入不会执行内容。发布后会出现在社区作品里。",
                style = XType.Caption, color = LocalExtraColors.current.ink3)
            error?.let { Text(it, color = Danger, style = XType.Body) }
            if (busy) Text(step.ifEmpty { "正在选择文件…" }, style = XType.Secondary, color = LocalExtraColors.current.ink3)
            PillButton(if (busy) "处理中…" else if (fileKey.isEmpty()) "上传并提交发布" else "继续校验并发布", {
                if (busy) return@PillButton
                val draft = ContentPublishDraft(actualKind, title, summary, tags.split(Regex("[,，、\\s]+")).filter { it.isNotBlank() }, rights, changelog)
                val problem = contentDraftError(draft, packId != null)
                if (problem != null) { error = problem; return@PillButton }
                if (fileKey.isEmpty() && file == null) {
                    error = if (forPost) "请选择作品文件" else "请选择内容包 .zip"
                    return@PillButton
                }
                val generation = session.generation
                busy = true; error = null
                scope.launch {
                    try {
                        if (fileKey.isEmpty()) {
                            step = "正在计算校验值并上传内容包…"
                            fileKey = env.content.upload(actualKind, file!!).fileKey
                        }
                        check(accountGeneration == generation) { "账号已切换，请重新选择文件" }
                        step = "正在等待资源服务器校验…"
                        if (!env.content.awaitVerified(fileKey)) {
                            error = "文件校验尚未完成，已保留上传记录和表单，请稍后点继续。"
                            return@launch
                        }
                        step = "正在提交发布…"
                        check(accountGeneration == generation) { "账号已切换，请重新选择文件" }
                        val result = env.content.publish(draft, fileKey, packId)
                        check(accountGeneration == generation) { "账号已切换，请在我的作品核对发布结果" }
                        file = null
                        if (onPublished != null) onPublished(result.id, draft.title.trim()) else published = result.id
                    } catch (e: CancellationException) { throw e }
                    catch (e: Exception) { error = messageOf(e) }
                    finally { busy = false; step = "" }
                }
            }, Modifier.fillMaxWidth(), enabled = !busy && rights && versionReady && canPick)
        }
    }
}
