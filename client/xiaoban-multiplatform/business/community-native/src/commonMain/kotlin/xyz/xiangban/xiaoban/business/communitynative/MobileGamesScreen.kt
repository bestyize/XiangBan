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
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.input.pointer.PointerEventPass
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.CustomAccessibilityAction
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.customActions
import androidx.compose.ui.semantics.hideFromAccessibility
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.IntOffset
import coil3.ImageLoader
import coil3.compose.LocalPlatformContext
import coil3.svg.SvgDecoder
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.ApiException
import xyz.xiangban.xiaoban.common.community.AppShortcut
import xyz.xiangban.xiaoban.common.community.recentAppShortcuts
import xyz.xiangban.xiaoban.common.community.BuiltinMobileGames
import xyz.xiangban.xiaoban.common.community.CloudChatController
import xyz.xiangban.xiaoban.common.community.CommunityJson
import xyz.xiangban.xiaoban.common.community.FeaturedToolsCache
import xyz.xiangban.xiaoban.common.community.MobileGame
import kotlin.time.Clock

internal val LocalCreationTypes = listOf("游戏", "小程序")
private val CreationTypes = LocalCreationTypes + listOf("壁纸", "小组件", "特效", "伙伴")
private enum class FeaturedPage { Games, Tools }
private val DesktopCreationHints = mapOf(
    "壁纸" to "制作一张层层山丘随光线缓慢变化的动画壁纸，色调柔和，适合长时间工作。",
    "小组件" to "制作一个简洁的桌面时钟小组件，显示真实时间与日期，配温暖的日光色调。",
    "特效" to "制作一个夜间桌面萤火特效，光点缓慢浮动，不影响点击和正常工作。",
    "伙伴" to "制作一位向日葵桌面伙伴，工作时安静陪伴，点击时轻轻摇摆回应。",
)

@OptIn(androidx.compose.ui.ExperimentalComposeUiApi::class)
@Composable
internal fun GamesScreen(
    env: CommunityEnv,
    chat: CloudChatController,
    open: (Route) -> Unit,
    active: Boolean,
    onOverlayChanged: (Boolean) -> Unit,
) {
    var official by remember { mutableStateOf(emptyList<MobileGame>()) }
    var games by remember { mutableStateOf(emptyList<MobileGame>()) }
    var errors by remember { mutableStateOf(emptyList<String>()) }
    var loading by remember { mutableStateOf(true) }
    var reload by remember { mutableStateOf(0) }
    var detail by remember { mutableStateOf<MobileGame?>(null) }
    var creating by remember { mutableStateOf(false) }
    var remixSource by remember { mutableStateOf<MobileGame?>(null) }
    var featuredPage by rememberSaveable { mutableStateOf<FeaturedPage?>(null) }
    var libraryVisible by remember { mutableStateOf(false) }
    var createKind by rememberSaveable { mutableStateOf("游戏") }
    var idea by rememberSaveable { mutableStateOf("") }
    var remotePrompt by remember { mutableStateOf<String?>(null) }
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
    val workRevision = env.games?.revision?.collectAsState()?.value
    val context = LocalPlatformContext.current
    val imageLoader = remember(context) { ImageLoader.Builder(context).components { add(SvgDecoder.Factory()) }.build() }
    DisposableEffect(imageLoader) { onDispose { imageLoader.shutdown() } }
    val covers = remember { mutableStateMapOf<String, ByteArray>() }
    val overlay = detail != null || creating || remotePrompt != null || remixSource != null
    val featuredTransition = updateTransition(featuredPage, label = "精选列表切换")
    val featuredVisible = featuredTransition.currentState != null || featuredTransition.targetState != null
    val plazaShift by featuredTransition.animateFloat(
        transitionSpec = { tween(PageMillis, easing = FastOutSlowInEasing) },
        label = "广场让位",
    ) { if (it == null) 0f else 1f }
    LaunchedEffect(overlay, featuredVisible, libraryVisible) { onOverlayChanged(overlay || featuredVisible || libraryVisible) }
    DisposableEffect(Unit) { onDispose { onOverlayChanged(false) } }
    fun close() {
        detail = null
        creating = false
        remotePrompt = null
        remixSource = null
    }
    fun openOfficial(game: MobileGame) {
        if (env.platform?.openGame(game.id) != true) message = "当前平台无法打开作品"
    }
    fun showOfficialDetails(game: MobileGame) {
        detail = game
    }
    BackHandler(enabled = active && (overlay || featuredVisible)) {
        if (overlay) close() else featuredPage = null
    }
    fun createLocal(title: String, brief: String, request: String) {
        if (!session.loggedIn && chat.state.value.customModelName == null) { open(Route.Models); return }
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
                try { env.platform?.builtinGameCover(game.id)?.let { covers[game.id] = it } }
                catch (e: CancellationException) { throw e } catch (_: Exception) { }
            }
        } finally { errors = failures; loading = false }
    }
    val appCatalog = remember(official, games) {
        official.map { AppShortcut("builtin", it.id, it.name) } + games.map { AppShortcut("local", it.id, it.name) }
    }
    val recentIds = env.platform?.recentGames?.collectAsState()?.value.orEmpty()
    val communityRecent = env.platform?.recentCommunityGames?.collectAsState()?.value.orEmpty()
    val recent = remember(recentIds, appCatalog, communityRecent) { recentAppShortcuts(recentIds, appCatalog, communityRecent) }
    fun openRecent(item: AppShortcut): Boolean {
        if (item.source == "community") { open(Route.Pack(item.id.toLong())); return true }
        if (env.platform?.openGame(item.id) == true) return true
        message = "当前无法打开这件作品"
        return false
    }
    // Ignore touches while a page is moving or leaving the active tab.
    val blockTouches = if (active && !featuredTransition.isRunning) Modifier else Modifier.pointerInput(Unit) {
        awaitPointerEventScope {
            while (true) awaitPointerEvent(PointerEventPass.Initial).changes.forEach { it.consume() }
        }
    }
    Box(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.background).then(blockTouches)) {
        AppLibraryPullDown(
            enabled = active && !featuredVisible && !overlay,
            onVisibilityChanged = { libraryVisible = it },
            drawer = { dismiss -> AppLibraryDrawer(env, recent, loading, errors, { reload++ }, dismiss) { if (openRecent(it)) dismiss() } },
        ) { showLibrary, dragHandle ->
            val headerDrag = dragHandle.semantics {
                customActions = listOf(CustomAccessibilityAction("打开最近用过") { showLibrary(); true })
            }
            Column(
                Modifier.fillMaxSize()
                    .graphicsLayer { translationX = -size.width * plazaShift }
                    .semantics { if (featuredVisible) hideFromAccessibility() }
                    .background(MaterialTheme.colorScheme.background),
            ) {
                HomeBar("广场", modifier = headerDrag)
                AppPlazaScreen(env, official, loading, errors, { reload++ }, recent, showLibrary, { openRecent(it) },
                    featuredShortcuts = featuredShortcuts, featuredTools = featuredTools, retryFeaturedTools = featuredToolsLoader::reload,
                    onOfficial = ::openOfficial, onOfficialDetails = ::showOfficialDetails,
                    onAllGames = { featuredPage = FeaturedPage.Games },
                    onAllTools = { featuredPage = FeaturedPage.Tools }, open = open)
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
                FeaturedPage.Games -> FeaturedGamesPage(env, official.filterNot { BuiltinMobileGames.isTool(it.id) }, back = { featuredPage = null },
                    onDetails = ::showOfficialDetails, onOpen = ::openOfficial)
                null -> Spacer(Modifier.fillMaxSize())
            }
        }
        detail?.let { game ->
            OfficialAppSheet(env, game, covers[game.id], imageLoader,
                dismiss = ::close, remix = { detail = null; remixSource = game })
        }
        if (creating) MobileCreationSheet(
            env, chat, createKind, idea, onKind = { createKind = it }, onIdea = { idea = it },
            onStart = ::startCreation, onDismiss = ::close,
        )
        remixSource?.let { source ->
            AppRemixSheet(env, chat, source.name, source.description, { remixSource = null }, open, kind = source.kind)
        }
        remotePrompt?.let { prompt -> MobileCreationDeviceSheet(env, prompt, ::close, open) }
        Toast(message) { message = null }
    }
}

@Composable
internal fun MobileCreationSheet(
    env: CommunityEnv, chat: CloudChatController, createKind: String, idea: String,
    onKind: (String) -> Unit, onIdea: (String) -> Unit, onStart: () -> Unit, onDismiss: () -> Unit,
) {
    val cloud by chat.state.collectAsState()
    val creationTypes = if (env.games != null) CreationTypes else CreationTypes.filterNot { it == "小程序" }
    CreationSheet(onDismiss, panelModifier = Modifier.imePadding()) {
        val local = createKind in LocalCreationTypes
        val blocked = local && !chat.canLeaveForCreation()
        Column(Modifier.fillMaxWidth().heightIn(max = 550.dp)) {
            Column(Modifier.weight(1f, fill = false).verticalScroll(rememberScrollState())) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Text("让想法开始发生", Modifier.weight(1f), style = XType.Title, fontWeight = FontWeight.SemiBold)
                    IconTap(XIcons.Close, "关闭创作", onDismiss, size = 18.dp)
                }
                Text("告诉小伴你想做什么，一起把它变成作品。", Modifier.padding(top = 8.dp, bottom = 16.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                CreationChips(creationTypes, createKind, onKind)
                Box(Modifier.padding(vertical = 14.dp).fillMaxWidth().clip(RoundedCornerShape(14.dp)).background(MaterialTheme.colorScheme.background).padding(14.dp)) {
                    val placeholder = when (createKind) {
                        "游戏" -> "例如：做一个小猫跳跃收集星星的游戏"
                        "小程序" -> "例如：做一个可以设置专注时长的番茄钟"
                        else -> "描述你想要的${createKind}，小伴会在电脑上制作"
                    }
                    if (idea.isEmpty()) Text(placeholder, style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    BasicTextField(idea, { onIdea(it.take(2000)) }, Modifier.fillMaxWidth().semantics { contentDescription = "创作想法" }, minLines = 4, maxLines = 8, textStyle = XType.Body.copy(color = MaterialTheme.colorScheme.onSurface), cursorBrush = Brush.verticalGradient(listOf(MaterialTheme.colorScheme.primary, MaterialTheme.colorScheme.primary)))
                }
                val hints = when (createKind) {
                    "游戏" -> listOf("星星跳跃游戏", "像素小冒险", "治愈系消除")
                    "小程序" -> listOf("专注番茄钟", "旅行清单", "性格小测")
                    else -> listOfNotNull(DesktopCreationHints[createKind])
                }
                Row(Modifier.horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(7.dp)) {
                    hints.forEach { hint -> Text(hint.take(14), Modifier.clip(RoundedCornerShape(7.dp)).background(MaterialTheme.colorScheme.surfaceVariant).clickable { onIdea(hint) }.padding(9.dp), style = XType.Caption) }
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
            AppActionButton(if (local) "和小伴一起创作" else "选择电脑继续创作", onStart, Modifier.fillMaxWidth(), primary = true, enabled = idea.isNotBlank() && !blocked)
        }
    }
}

@Composable
internal fun MobileCreationDeviceSheet(env: CommunityEnv, prompt: String, onDismiss: () -> Unit, open: (Route) -> Unit) {
    val remote by env.remoteChat.state.collectAsState()
    CreationSheet(onDismiss) {
        Text("选择电脑", style = XType.Title, fontWeight = FontWeight.SemiBold)
        Text("将想法带入电脑对话，由你确认发送。", Modifier.padding(vertical = 12.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
        Column(Modifier.heightIn(max = 300.dp).verticalScroll(rememberScrollState())) {
            remote.peers.forEach { peer ->
                val enabled = peer.online && !remote.sending
                val alpha = if (enabled) 1f else .45f
                val secondary = MaterialTheme.colorScheme.onSurfaceVariant
                Row(
                    Modifier.fillMaxWidth().heightIn(min = 51.dp).clickable(enabled = enabled, role = Role.Button) {
                        env.remoteChat.selectPeer(peer.id)
                        env.remoteChat.newSession()
                        onDismiss()
                        open(Route.RemoteChat(peer.id, prompt))
                    }.padding(vertical = 12.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(12.dp),
                ) {
                    XIcon(XIcons.Monitor, size = 18.dp, tint = secondary.copy(alpha = alpha))
                    Text(peer.name, Modifier.weight(1f), style = XType.Secondary,
                        color = MaterialTheme.colorScheme.onSurface.copy(alpha = alpha))
                    Text(if (peer.online) "在线" else "离线", style = XType.Caption, color = secondary)
                }
            }
        }
        if (remote.peers.isEmpty()) Text("还没有连接的电脑，请在电脑上登录同一小伴账号并开启手机遥控。", style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant)
        if (remote.sending) Text("电脑正在执行任务，请结束当前任务后再创建。", color = MaterialTheme.colorScheme.onSurfaceVariant, style = XType.Caption)
        Spacer(Modifier.height(12.dp))
        GhostPillButton("管理我的电脑", { onDismiss(); open(Route.Devices) }, Modifier.fillMaxWidth())
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
