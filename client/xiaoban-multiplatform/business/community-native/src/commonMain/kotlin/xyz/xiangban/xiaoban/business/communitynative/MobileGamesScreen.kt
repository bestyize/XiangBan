package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.tween
import androidx.compose.animation.core.updateTransition
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.backhandler.BackHandler
import androidx.compose.ui.draw.blur
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.luminance
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.input.pointer.PointerEventPass
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.hideFromAccessibility
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.sp
import coil3.ImageLoader
import coil3.compose.AsyncImage
import coil3.compose.LocalPlatformContext
import coil3.svg.SvgDecoder
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.ApiException
import xyz.xiangban.xiaoban.common.community.AppShortcut
import xyz.xiangban.xiaoban.common.community.BuiltinMobileGames
import xyz.xiangban.xiaoban.common.community.CloudChatController
import xyz.xiangban.xiaoban.common.community.CommunityJson
import xyz.xiangban.xiaoban.common.community.FeaturedToolsCache
import xyz.xiangban.xiaoban.common.community.MobileGame
import xyz.xiangban.xiaoban.common.community.MobileGameStore
import kotlin.time.Clock

private val LocalCreationTypes = listOf("游戏", "小程序")
private val CreationTypes = listOf("全部") + LocalCreationTypes + listOf("壁纸", "小组件", "特效", "伙伴")
private enum class FeaturedPage { Games, Tools }
private data class CreationItem(val id: String, val name: String, val description: String, val kind: String,
    val game: MobileGame? = null, val owned: Boolean = false) {
    val key get() = (if (owned) "mine:" else "builtin:") + id
    val device get() = when (game?.kind) { "game" -> "手机可玩"; "app" -> "手机可用"; else -> "电脑创作灵感" }
}
// These are reference ideas, not installed mobile desktop resources. Actual creation runs on the selected desktop.
private val DesktopIdeas = listOf(
    CreationItem("soft-hills", "柔软山丘", "制作一张层层山丘随光线缓慢变化的动画壁纸，色调柔和，适合长时间工作。", "壁纸"),
    CreationItem("daylight-clock", "日光时钟", "制作一个简洁的桌面时钟小组件，显示真实时间与日期，配温暖的日光色调。", "小组件"),
    CreationItem("fireflies", "流萤之夜", "制作一个夜间桌面萤火特效，光点缓慢浮动，不影响点击和正常工作。", "特效"),
    CreationItem("sunflower", "向日葵小伴", "制作一位向日葵桌面伙伴，工作时安静陪伴，点击时轻轻摇摆回应。", "伙伴"),
)

@OptIn(androidx.compose.ui.ExperimentalComposeUiApi::class)
@Composable
internal fun GamesScreen(
    env: CommunityEnv,
    chat: CloudChatController,
    open: (Route) -> Unit,
    mine: Boolean,
    onMineChanged: (Boolean) -> Unit,
    active: Boolean,
    onOverlayChanged: (Boolean) -> Unit,
    createRequest: String?,
    onCreateRequestHandled: () -> Unit,
) {
    var official by remember { mutableStateOf(emptyList<MobileGame>()) }
    var games by remember { mutableStateOf(emptyList<MobileGame>()) }
    var errors by remember { mutableStateOf(emptyList<String>()) }
    var loading by remember { mutableStateOf(true) }
    var reload by remember { mutableStateOf(0) }
    var category by rememberSaveable { mutableStateOf("全部") }
    var detail by remember { mutableStateOf<CreationItem?>(null) }
    // One sheet at a time; submenus retain the selected work without stacking focus locks.
    var panel by remember { mutableStateOf("actions") }
    var creating by remember { mutableStateOf(false) }
    var remixSource by remember { mutableStateOf<MobileGame?>(null) }
    var featuredPage by rememberSaveable { mutableStateOf<FeaturedPage?>(null) }
    var createKind by rememberSaveable { mutableStateOf("游戏") }
    var idea by rememberSaveable { mutableStateOf("") }
    var remotePrompt by remember { mutableStateOf<String?>(null) }
    var busy by remember { mutableStateOf(false) }
    var message by remember { mutableStateOf<String?>(null) }
    val scope = rememberCoroutineScope()
    val session by env.session.state.collectAsState()
    val featuredToolsCacheKey = "featured-tools:${env.config.forumBase.trimEnd('/')}:${env.config.runtimeV2}"
    var featuredShortcuts by remember(env) {
        mutableStateOf(FeaturedToolsCache.restore(env.prefs.get(featuredToolsCacheKey), Clock.System.now().toEpochMilliseconds()))
    }
    val featuredToolsLoader = rememberLoader(env, "featured-tools") {
        val packs = try {
            env.forum.featuredTools()
        } catch (e: ApiException) {
            if (e.status != 0 && e.status != 408 && e.status !in 500..599) throw e
            delay(700)
            env.forum.featuredTools()
        }
        val shortcuts = packs.map { AppShortcut("community", it.id.toString(), it.title, it.coverUrl, it.slug) }
        val cache = FeaturedToolsCache(Clock.System.now().toEpochMilliseconds(), shortcuts)
        env.prefs.put(featuredToolsCacheKey, CommunityJson.encodeToString(cache))
        featuredShortcuts = shortcuts
    }
    val featuredTools by key(featuredToolsLoader) { featuredToolsLoader.state.collectAsState() }
    val cloud by chat.state.collectAsState()
    val remote by env.remoteChat.state.collectAsState()
    val workRevision = env.games?.revision?.collectAsState()?.value
    val creationTypes = if (env.games != null) CreationTypes else CreationTypes.filterNot { it == "小程序" }
    val context = LocalPlatformContext.current
    val imageLoader = remember(context) { ImageLoader.Builder(context).components { add(SvgDecoder.Factory()) }.build() }
    DisposableEffect(imageLoader) { onDispose { imageLoader.shutdown() } }
    val covers = remember { mutableStateMapOf<String, ByteArray>() }
    val overlay = detail != null || creating || remotePrompt != null || remixSource != null
    val featuredTransition = updateTransition(featuredPage.takeUnless { mine }, label = "精选列表切换")
    val featuredVisible = featuredTransition.currentState != null || featuredTransition.targetState != null
    val plazaShift by featuredTransition.animateFloat(
        transitionSpec = { tween(PageMillis, easing = FastOutSlowInEasing) },
        label = "广场让位",
    ) { if (it == null) 0f else 1f }
    LaunchedEffect(overlay, featuredVisible) { onOverlayChanged(overlay || featuredVisible) }
    DisposableEffect(Unit) { onDispose { onOverlayChanged(false) } }
    fun close() { if (!busy) { detail = null; creating = false; remotePrompt = null; remixSource = null; panel = "actions" } }
    fun openOfficial(game: MobileGame) {
        if (env.platform?.openGame(game.id) != true) message = "当前平台无法打开作品"
    }
    fun showOfficialDetails(game: MobileGame) {
        detail = CreationItem(game.id, game.name, game.description, if (game.kind == "app") "小程序" else "游戏", game)
        panel = "actions"
    }
    BackHandler(enabled = active && (overlay || featuredVisible)) {
        if (!overlay) featuredPage = null
        else if (!busy && detail != null && panel != "actions") panel = "actions" else close()
    }
    fun createLocal(title: String, brief: String, request: String) {
        if (!session.loggedIn) { open(Route.Login); return }
        if (env.agent == null) { message = "当前安装包不支持手机本地创作，可连接电脑创作。"; return }
        if (!chat.startCreation(title, brief, request)) return
        close()
        open(Route.AiChat)
    }
    fun createOnComputer(prompt: String) {
        if (!session.loggedIn) { open(Route.Login); return }
        creating = false; detail = null; remotePrompt = prompt
        scope.launch { env.remoteChat.refreshPeers() }
    }
    fun startCreation() {
        if (createKind in LocalCreationTypes) {
            val work = if (createKind == "小程序") "小程序（kind=app，category 取 tool 或 quiz）" else "离线小游戏（kind=game）"
            val brief = idea.trim()
            createLocal(xyz.xiangban.xiaoban.common.community.creationTitle(brief, createKind), brief,
                "制作支持手机触控和电脑键鼠的$work。需求：$brief")
        } else createOnComputer("请在这台电脑上帮我创作小伴桌面的${createKind}。我的想法：${idea.trim()}。先读取对应的资源规范，给出简短方案让我确认，再制作并预览。")
    }
    fun prepare(item: CreationItem? = null) {
        detail = null
        createKind = item?.kind ?: category.takeUnless { it == "全部" } ?: "游戏"
        idea = item?.description.orEmpty()
        creating = true
    }
    LaunchedEffect(active, createRequest) {
        if (active && createRequest != null) {
            prepare()
            createKind = createRequest
            onCreateRequestHandled()
        }
    }
    fun runAction(action: suspend () -> Unit) {
        if (busy) return
        busy = true
        scope.launch {
            try { action() }
            catch (e: CancellationException) { throw e }
            catch (e: Exception) { message = e.message ?: "操作失败，请重试" }
            finally { busy = false }
        }
    }
    LaunchedEffect(env.games, workRevision, reload) {
        loading = true
        val failures = mutableListOf<String>()
        try {
            try { official = env.platform?.builtinGames().orEmpty() }
            catch (e: CancellationException) { throw e }
            catch (_: Exception) { failures += "官方游戏读取失败，请重试" }
            try { val scan = env.games?.scan(); games = scan?.games.orEmpty(); failures += scan?.errors.orEmpty() }
            catch (e: CancellationException) { throw e }
            catch (_: Exception) { failures += "本机作品读取失败，请重试" }
            covers.clear()
            official.forEach { game ->
                try { env.platform?.builtinGameCover(game.id)?.let { covers["builtin:${game.id}"] = it } }
                catch (e: CancellationException) { throw e } catch (_: Exception) { }
            }
            DesktopIdeas.forEach { item ->
                try { env.platform?.creationCover(item.id)?.let { covers[item.key] = it } }
                catch (e: CancellationException) { throw e } catch (_: Exception) { }
            }
        } finally { errors = failures; loading = false }
    }
    val myItems = games.map { CreationItem(it.id, it.name, it.description, if (it.kind == "app") "小程序" else "游戏", it, true) }
    val visible = myItems.filter { category == "全部" || it.kind == category }
    // Ignore touches while a page is moving or leaving the active tab.
    val blockTouches = if (active && !featuredTransition.isRunning) Modifier else Modifier.pointerInput(Unit) {
        awaitPointerEventScope {
            while (true) awaitPointerEvent(PointerEventPass.Initial).changes.forEach { it.consume() }
        }
    }
    Box(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.background).then(blockTouches)) {
        Column(
            Modifier.fillMaxSize()
                .graphicsLayer { translationX = -size.width * plazaShift }
                .semantics { if (featuredVisible) hideFromAccessibility() }
                .statusBarsPadding(),
        ) {
            Row(Modifier.fillMaxWidth().padding(start = 16.dp, end = 8.dp), verticalAlignment = Alignment.CenterVertically) {
                Row(Modifier.weight(1f), horizontalArrangement = Arrangement.spacedBy(24.dp)) {
                    listOf(false to "广场", true to "我的作品").forEach { (value, label) ->
                        Column(Modifier.semantics { selected = mine == value }.clickable(role = Role.Tab) { onMineChanged(value) }.padding(vertical = 8.dp)) {
                            Text(label, style = XType.Title, fontWeight = if (mine == value) FontWeight.SemiBold else FontWeight.Normal, color = if (mine == value) MaterialTheme.colorScheme.onSurface else MaterialTheme.colorScheme.onSurfaceVariant)
                            Box(Modifier.padding(top = 8.dp).width(20.dp).height(3.dp).background(if (mine == value) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.background, CircleShape))
                        }
                    }
                }
                IconTap(XIcons.Search, "搜索应用", {}) // Search destination is reserved for a later version.
                if (mine) IconTap(XIcons.Plus, "新建作品", { prepare() })
            }
            Box(Modifier.fillMaxSize()) {
                RetainedTabPage(selected = !mine) {
                    AppPlazaScreen(env, official, games, covers, imageLoader, loading, errors, { reload++ },
                        featuredShortcuts = featuredShortcuts, featuredTools = featuredTools, retryFeaturedTools = featuredToolsLoader::reload,
                        onOfficial = ::openOfficial, onOfficialDetails = ::showOfficialDetails,
                        onAllGames = { featuredPage = FeaturedPage.Games },
                        onAllTools = { featuredPage = FeaturedPage.Tools }, open = open)
                }
                RetainedTabPage(selected = mine) {
                    LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 12.dp, bottom = NavInset), verticalArrangement = Arrangement.spacedBy(16.dp)) {
                        item {
                            Row(Modifier.fillMaxWidth().outlinedCard(16.dp).clickable { prepare() }.padding(16.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                                XIcon(XIcons.Sparkle, tint = MaterialTheme.colorScheme.primary, size = 24.dp)
                                Column(Modifier.weight(1f)) {
                                    Text("你想做个什么？", style = XType.Body, fontWeight = FontWeight.SemiBold)
                                    Text("和小伴一起，把想法变成作品", Modifier.padding(top = 4.dp), style = XType.Caption, color = LocalExtraColors.current.ink3)
                                }
                            }
                        }
                        item { Text("本机作品 · 电脑创作的装扮保存在对应电脑", style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant) }
                        item { CreationChips(creationTypes, category) { category = it } }
                        if (errors.isNotEmpty()) item {
                            Text(errors.joinToString("\n"), style = XType.Caption, color = MaterialTheme.colorScheme.error)
                            LinkText("重新读取", { reload++ })
                        }
                        if (loading && visible.isEmpty()) item { Spinner(Modifier.fillMaxWidth().height(120.dp)) }
                        if (!loading && visible.isEmpty()) item {
                            EmptyState(XIcons.Sparkle, "还没有${if (category == "全部") "" else category}作品", if (category != "全部" && category !in LocalCreationTypes) "桌面装扮在电脑上创作与使用，手机可遥控电脑继续创作。" else "从一个小小的想法开始吧。")
                            AppActionButton("创作第一件作品", { prepare() }, Modifier.fillMaxWidth(), primary = true)
                        }
                        item { CreationSection("本机作品", "刷新", { if (!loading) reload++ }) }
                        items(visible, key = { it.key }) { entry ->
                            Row(Modifier.fillMaxWidth().outlinedCard(16.dp).clickable(role = Role.Button) { detail = entry; panel = "actions" }.padding(16.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                                CreationCover(entry, rememberLocalCover(env.games, entry.game, workRevision), imageLoader, Modifier.size(68.dp))
                                Column(Modifier.weight(1f)) {
                                    Text(entry.name, style = XType.Body, fontWeight = FontWeight.Medium, maxLines = 1, overflow = TextOverflow.Ellipsis)
                                    Text("${entry.kind} · ${entry.device}", Modifier.padding(top = 4.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                    Text("保存在这台手机上", Modifier.padding(top = 4.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                }
                                XIcon(XIcons.More, tint = MaterialTheme.colorScheme.onSurfaceVariant, size = 20.dp)
                            }
                        }
                    }
                }
            }
        }
        if (!active) return@Box
        featuredTransition.AnimatedContent(
            modifier = Modifier.fillMaxSize(),
            transitionSpec = {
                val spec = tween<IntOffset>(PageMillis, easing = FastOutSlowInEasing)
                if (targetState != null) slideInHorizontally(spec) { it } togetherWith ExitTransition.None
                else EnterTransition.None togetherWith slideOutHorizontally(spec) { it }
            },
        ) { page ->
            when (page) {
                FeaturedPage.Tools -> FeaturedToolsPage(env, official.filter { BuiltinMobileGames.isTool(it.id) }, back = { featuredPage = null },
                    featuredShortcuts = featuredShortcuts, featuredTools = featuredTools, retryFeaturedTools = featuredToolsLoader::reload,
                    onDetails = ::showOfficialDetails, onOpen = ::openOfficial, open = open)
                FeaturedPage.Games -> FeaturedGamesPage(official.filterNot { BuiltinMobileGames.isTool(it.id) }, covers, imageLoader, back = { featuredPage = null },
                    onDetails = ::showOfficialDetails, onOpen = ::openOfficial)
                null -> Spacer(Modifier.fillMaxSize())
            }
        }
        detail?.let { entry ->
            val artwork = if (entry.owned) rememberLocalCover(env.games, entry.game, workRevision) else covers[entry.key]
            if (!entry.owned && entry.game != null) OfficialAppSheet(env, entry.game, artwork, imageLoader,
                dismiss = { close() }, remix = { detail = null; remixSource = entry.game })
            else CreationSheet({ if (!busy && panel != "actions") panel = "actions" else close() }) {
                Column(Modifier.fillMaxWidth().heightIn(max = 590.dp).verticalScroll(rememberScrollState())) {
                    if (panel in listOf("actions", "preview")) Row(Modifier.padding(bottom = 21.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                        CreationCover(entry, artwork, imageLoader, Modifier.size(52.dp), cornerRadius = 11)
                        Column(Modifier.weight(1f)) {
                            Text(entry.name, style = XType.Section, fontWeight = FontWeight.SemiBold, maxLines = 2, overflow = TextOverflow.Ellipsis)
                            Text("${entry.kind} · ${if (entry.owned) "我的作品" else if (entry.game != null) "小伴官方" else "创作灵感"}", Modifier.padding(top = 5.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                        IconTap(XIcons.Close, "关闭作品操作", { close() }, Modifier.align(Alignment.Top), tint = MaterialTheme.colorScheme.onSurfaceVariant, size = 17.dp)
                    } else Row(Modifier.padding(bottom = 10.dp), verticalAlignment = Alignment.CenterVertically) {
                        Text(when (panel) { "delete" -> "删除「${entry.name}」？"; "share" -> "分享到其他渠道"; else -> "发送到电脑" }, Modifier.weight(1f), style = XType.Title, fontWeight = FontWeight.SemiBold)
                        IconTap(XIcons.Close, "关闭作品操作", { close() }, tint = MaterialTheme.colorScheme.onSurfaceVariant, size = 17.dp)
                    }
                    when (panel) {
                        "actions" -> {
                            CreationActionGroup {
                                val openLabel = when (entry.game?.kind) { "app" -> "打开小程序"; "game" -> "开始游玩"; else -> "预览灵感" }
                                CreationAction(if (entry.game != null) XIcons.Play else XIcons.Eye, openLabel, if (entry.game != null) entry.device else "电脑装扮", !busy) {
                                    if (entry.game != null) {
                                        if (env.platform?.openGame(entry.id) == true) close() else message = "当前平台无法打开作品"
                                    } else panel = "preview"
                                }
                                CreationAction(XIcons.Pen, if (entry.owned) "继续修改" else "做一个我的版本", enabled = !busy, divider = true) {
                                    if (entry.owned) createLocal("修改《${entry.name}》", "我想修改《${entry.name}》",
                                        "先询问用户想修改什么，再读取 games/${entry.id} 的清单、源码和试玩记录。在草稿中修改，保留原有素材和作品类型。") else prepare(entry)
                                }
                                CreationAction(XIcons.Monitor, if (entry.game != null) "发送到电脑" else "在电脑上创作", enabled = !busy, divider = true) {
                                    if (entry.game == null) createOnComputer("请在电脑上参考以下想法制作小伴桌面${entry.kind}：${entry.description} 先读取规范并给出方案让我确认。") else panel = "computer"
                                }
                            }
                            Spacer(Modifier.height(11.dp))
                            CreationActionGroup {
                                CreationAction(XIcons.Globe, "分享到社区", enabled = !busy) {
                                    close()
                                    if (entry.owned) open(Route.PublishPack(localGameId = entry.id))
                                    else open(Route.CreationPost("分享：${entry.name}", "${entry.name}\n\n${entry.description}\n\n${if (entry.game != null) "来自小伴官方游戏，在小伴手机「应用」中游玩。" else "小伴桌面创作灵感，可连接电脑制作。"}"))
                                }
                                CreationAction(XIcons.Share, "分享到其他渠道", enabled = !busy, divider = true, arrow = true) { panel = "share" }
                            }
                            if (entry.owned) {
                                Spacer(Modifier.height(10.dp))
                                CreationActionGroup {
                                    CreationAction(XIcons.Trash, "删除作品", if (cloud.sending) "创作进行中" else "", enabled = !busy && !cloud.sending, destructive = true) { panel = "delete" }
                                }
                            }
                        }
                        "delete" -> {
                            Text("将删除这台手机上的作品文件、游戏进度和运行记录，无法撤销。已导出、分享或发布到社区的副本不受影响。", Modifier.padding(top = 2.dp, bottom = 18.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            if (entry.owned) CreationActionGroup {
                                CreationAction(XIcons.Trash, if (busy) "正在删除…" else "确认删除", enabled = !busy && !cloud.sending, destructive = true) {
                                    runAction {
                                        val store = env.games ?: error("本机作品不可用")
                                        store.delete(entry.id)
                                        games = games.filterNot { it.id == entry.id }
                                        detail = null
                                        panel = "actions"
                                        env.platform?.forgetGame(entry.id)
                                        message = "已删除《${entry.name}》"
                                    }
                                }
                            }
                        }
                        "preview" -> {
                            CreationCover(entry, artwork, imageLoader, Modifier.fillMaxWidth().height(220.dp))
                            Text(entry.description, Modifier.padding(vertical = 16.dp), style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            Text("这是创作参考图。动画与互动效果需要在电脑上制作并预览。", style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            Spacer(Modifier.height(14.dp))
                            CreationActionGroup { CreationAction(XIcons.Sparkle, "做一个我的版本", enabled = !busy) { prepare(entry) } }
                        }
                        "computer" -> {
                            Text(if (entry.owned) "将作品包通过系统分享发送给电脑，或保存后自行传输。在电脑的小伴中导入即可使用；不会自动覆盖已有作品。" else "这款官方游戏已内置于小伴桌面。可打开电脑对话，让小伴找到它。", style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            Spacer(Modifier.height(16.dp))
                            CreationActionGroup {
                                if (entry.owned) {
                                    CreationAction(XIcons.Share, "分享作品包到电脑", enabled = !busy) { runAction { if (env.platform?.shareGamePackage(entry.id) != true) message = "当前平台不支持分享作品包" } }
                                    CreationAction(XIcons.Download, "保存作品包", enabled = !busy, divider = true) { runAction { if (env.platform?.exportGame(entry.id) != true) message = "当前平台不支持导出" } }
                                } else CreationAction(XIcons.Monitor, "选择电脑", enabled = !busy) { createOnComputer("请帮我在小伴桌面的官方游戏里找到《${entry.name}》（${entry.id}），告诉我怎么开始游玩。") }
                            }
                        }
                        "share" -> {
                            Text("选择分享形式，再选择接收应用", style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            Spacer(Modifier.height(14.dp))
                            CreationActionGroup {
                                CreationAction(XIcons.Image, "分享作品卡片", "图片", !busy) { runAction { if (env.platform?.shareCreationPreview(entry.name, entry.description, artwork) != true) message = "当前平台不支持分享图片" } }
                                if (entry.owned) CreationAction(XIcons.Download, "分享作品包", "可导入使用", !busy, divider = true) { runAction { if (env.platform?.shareGamePackage(entry.id) != true) message = "当前平台不支持分享作品包" } }
                                CreationAction(XIcons.Share, "分享文字介绍", enabled = !busy, divider = true) { env.platform?.share(entry.name, entry.description) }
                            }
                        }
                    }
                    if (busy && panel != "delete") Text("正在准备作品…", Modifier.padding(top = 12.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    Text(if (panel in listOf("actions", "delete")) "取消" else "返回", Modifier.fillMaxWidth().clickable(enabled = !busy) { if (panel == "actions") close() else panel = "actions" }.padding(top = 24.dp, bottom = 13.dp), style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant, textAlign = androidx.compose.ui.text.style.TextAlign.Center)
                }
            }
        }
        if (creating) CreationSheet({ close() }, panelModifier = Modifier.imePadding()) {
            val local = createKind in LocalCreationTypes
            val blocked = local && !chat.canLeaveForCreation()
            Column(Modifier.fillMaxWidth().heightIn(max = 550.dp)) {
                Column(Modifier.weight(1f, fill = false).verticalScroll(rememberScrollState())) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Text("让想法开始发生", Modifier.weight(1f), style = XType.Title, fontWeight = FontWeight.SemiBold)
                        IconTap(XIcons.Close, "关闭创作", { close() }, size = 18.dp)
                    }
                    Text("告诉小伴你想做什么，一起把它变成作品。", Modifier.padding(top = 8.dp, bottom = 16.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    CreationChips(creationTypes.drop(1), createKind) { createKind = it }
                    Box(Modifier.padding(vertical = 14.dp).fillMaxWidth().clip(RoundedCornerShape(14.dp)).background(MaterialTheme.colorScheme.background).padding(14.dp)) {
                        val placeholder = when (createKind) {
                            "游戏" -> "例如：做一个小猫跳跃收集星星的游戏"
                            "小程序" -> "例如：做一个可以设置专注时长的番茄钟"
                            else -> "描述你想要的${createKind}，小伴会在电脑上制作"
                        }
                        if (idea.isEmpty()) Text(placeholder, style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        BasicTextField(idea, { idea = it.take(2000) }, Modifier.fillMaxWidth().semantics { contentDescription = "创作想法" }, minLines = 4, maxLines = 8, textStyle = XType.Body.copy(color = MaterialTheme.colorScheme.onSurface), cursorBrush = Brush.verticalGradient(listOf(MaterialTheme.colorScheme.primary, MaterialTheme.colorScheme.primary)))
                    }
                    val hints = when (createKind) {
                        "游戏" -> listOf("星星跳跃游戏", "像素小冒险", "治愈系消除")
                        "小程序" -> listOf("专注番茄钟", "旅行清单", "性格小测")
                        else -> DesktopIdeas.filter { it.kind == createKind }.map { it.description }
                    }
                    Row(Modifier.horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(7.dp)) {
                        hints.forEach { hint -> Text(hint.take(14), Modifier.clip(RoundedCornerShape(7.dp)).background(MaterialTheme.colorScheme.surfaceVariant).clickable { idea = hint }.padding(9.dp), style = XType.Caption) }
                    }
                    Text("${idea.length}/2000", Modifier.align(Alignment.End).padding(top = 6.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
                val hint = when {
                    blocked -> "当前还有创作任务正在执行。可以先编辑需求。"
                    local && cloud.sending -> "图片继续在原对话生成，可以同时开始新作品。"
                    local -> "先查看并确认方案，再在这台手机上制作。"
                    else -> "选择已连接电脑后进入对话，作品保存在电脑上。"
                }
                Text(hint, Modifier.padding(vertical = 12.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                AppActionButton(if (local) "和小伴一起创作" else "选择电脑继续创作", { startCreation() }, Modifier.fillMaxWidth(), primary = true, enabled = idea.isNotBlank() && !blocked)
            }
        }
        remixSource?.let { source ->
            AppRemixSheet(env, chat, source.name, source.description, { remixSource = null }, open, kind = source.kind)
        }
        remotePrompt?.let { prompt ->
            CreationSheet({ close() }) {
                Text("选择电脑", style = XType.Title, fontWeight = FontWeight.SemiBold)
                Text("将想法带入电脑对话，由你确认发送。", Modifier.padding(vertical = 12.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                Column(Modifier.heightIn(max = 300.dp).verticalScroll(rememberScrollState())) {
                    remote.peers.forEach { peer ->
                        CreationAction(XIcons.Monitor, peer.name, if (peer.online) "在线" else "离线", enabled = peer.online && !remote.sending) {
                            env.remoteChat.selectPeer(peer.id)
                            env.remoteChat.newSession()
                            close()
                            open(Route.RemoteChat(peer.id, prompt))
                        }
                    }
                }
                if (remote.peers.isEmpty()) Text("还没有连接的电脑，请在电脑上登录同一小伴账号并开启手机遥控。", style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant)
                if (remote.sending) Text("电脑正在执行任务，请结束当前任务后再创建。", color = MaterialTheme.colorScheme.onSurfaceVariant, style = XType.Caption)
                Spacer(Modifier.height(12.dp))
                GhostPillButton("管理我的电脑", { close(); open(Route.Devices) }, Modifier.fillMaxWidth())
            }
        }
        Toast(message) { message = null }
    }
}

@Composable
private fun CreationSection(title: String, trailing: String, action: (() -> Unit)? = null) {
    Row(Modifier.fillMaxWidth().padding(top = 4.dp), verticalAlignment = Alignment.CenterVertically) {
        Text(title, Modifier.weight(1f), style = XType.Body, fontWeight = FontWeight.SemiBold)
        Text(trailing, Modifier.then(if (action != null) Modifier.clickable { action() }.padding(6.dp) else Modifier), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}

@Composable
private fun CreationChips(values: List<String>, value: String, select: (String) -> Unit) {
    Row(Modifier.horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
        values.forEach { item ->
            val active = value == item
            Text(item, Modifier.clip(CircleShape).heightIn(min = 34.dp).background(if (active) LocalExtraColors.current.brandSoft else LocalExtraColors.current.fill).semantics { selected = active }.clickable(role = Role.Button) { select(item) }.padding(horizontal = 9.dp, vertical = 5.dp), style = XType.Caption, color = if (active) LocalExtraColors.current.brandInk else MaterialTheme.colorScheme.onSurfaceVariant)
        }
    }
}

/** Creation sheets share the active mobile palette and standard bottom-sheet behavior. */
@OptIn(ExperimentalComposeUiApi::class)
@Composable
private fun CreationSheet(onDismiss: () -> Unit, panelModifier: Modifier = Modifier, content: @Composable ColumnScope.() -> Unit) {
    BottomSheet(onDismiss, panelModifier, cornerRadius = 16.dp) {
        Box(Modifier.align(Alignment.CenterHorizontally).size(32.dp, 4.dp).clip(CircleShape).background(LocalExtraColors.current.line))
        Spacer(Modifier.height(16.dp))
        content()
    }
}

@Composable
private fun CreationActionGroup(content: @Composable ColumnScope.() -> Unit) {
    Column(Modifier.fillMaxWidth().clip(RoundedCornerShape(15.dp)).background(MaterialTheme.colorScheme.surface).padding(horizontal = 14.dp, vertical = 3.dp), content = content)
}

@Composable
private fun CreationAction(icon: ImageVector, title: String, trailing: String = "", enabled: Boolean = true, destructive: Boolean = false, divider: Boolean = false, arrow: Boolean = false, action: () -> Unit) {
    val line = MaterialTheme.colorScheme.outlineVariant
    Row(Modifier.fillMaxWidth().drawBehind { if (divider) drawLine(line, Offset.Zero, Offset(size.width, 0f), 1.dp.toPx()) }.heightIn(min = 51.dp).clickable(enabled = enabled, role = Role.Button, onClick = action).padding(vertical = 12.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
        XIcon(icon, size = 18.dp, tint = (if (destructive) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.onSurfaceVariant).copy(alpha = if (enabled) 1f else .45f))
        Text(title, Modifier.weight(1f), style = XType.Secondary, color = (if (destructive) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.onSurface).copy(alpha = if (enabled) 1f else .45f))
        if (trailing.isNotEmpty()) Text(trailing, style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
        if (arrow) XIcon(XIcons.ArrowRight, size = 18.dp, tint = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}

@Composable
private fun rememberLocalCover(store: MobileGameStore?, game: MobileGame?, revision: Int?): ByteArray? {
    if (store == null || game == null) return null
    var bytes by remember(store, game.id, game.cover, revision) { mutableStateOf<ByteArray?>(null) }
    LaunchedEffect(store, game.id, game.cover, revision) {
        try { bytes = store.readFileBytes(game.id, game.cover) }
        catch (e: CancellationException) { throw e }
        catch (_: Exception) { /* The cover placeholder remains visible. */ }
    }
    return bytes
}

@Composable
private fun CreationCover(item: CreationItem, bytes: ByteArray?, loader: ImageLoader, modifier: Modifier, cornerRadius: Int = 15) {
    var failed by remember(bytes) { mutableStateOf(false) }
    Box(modifier.clip(RoundedCornerShape(cornerRadius.dp)).background(LocalExtraColors.current.fill), contentAlignment = Alignment.Center) {
        if (bytes == null || failed) XIcon(if (item.game != null) XIcons.Gamepad else XIcons.Sparkle, size = 28.dp, tint = MaterialTheme.colorScheme.primary)
        else AsyncImage(model = bytes, imageLoader = loader, contentDescription = null, modifier = Modifier.fillMaxSize().then(if (item.kind == "伙伴") Modifier.padding(10.dp) else Modifier), contentScale = if (item.game != null || item.id == "sunflower") ContentScale.Fit else ContentScale.Crop, onError = { failed = true })
    }
}
