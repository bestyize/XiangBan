package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.EnterExitState
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.tween
import androidx.compose.animation.core.updateTransition
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.SideEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.backhandler.BackHandler
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.layout.layout
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import xyz.xiangban.xiaoban.common.community.ChatHistory
import xyz.xiangban.xiaoban.common.community.CloudChatController
import xyz.xiangban.xiaoban.common.community.Load
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive

/**
 * 社区原生界面入口：四个去处（小伴AI、游戏、社区、我的）加一个页面栈，底部是悬浮胶囊导航，中间的“发布”打开弹层。
 * 打开子页面时一级页仍留在下面（只是被盖住），返回后滚动位置和已加载的数据都在。
 */
/** 页面进入、返回的转场时长。 */
internal const val PageMillis = xyz.xiangban.xiaoban.common.ui.theme.XMotion.PagePush

private fun Modifier.layoutVisiblePage(visible: () -> Boolean): Modifier =
    layout { measurable, constraints ->
        val page = if (visible()) measurable.measure(constraints) else null
        layout(constraints.maxWidth, constraints.maxHeight) { page?.placeRelative(0, 0) }
    }

/** Keep visited pages in composition and leave hidden pages out of layout and hit testing. */
@Composable
internal fun RetainedTabPage(
    selected: Boolean,
    modifier: Modifier = Modifier,
    visible: () -> Boolean = { selected },
    content: @Composable () -> Unit,
) {
    var visited by remember { mutableStateOf(false) }
    if (!selected && !visited) return
    if (!visited) SideEffect { visited = true }

    Box(modifier.fillMaxSize().layoutVisiblePage(visible)) {
        content()
    }
}

@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun CommunityNativeApp(env: CommunityEnv) {
    remember(env) { ThemeState.load(env.prefs); SpeechState.load(env.prefs); NotificationState.load(env.prefs) }
    XiaobanTheme {
      CompositionLocalProvider(
          LocalReplyReader provides env.reader,
          LocalCommunityImageLoader provides rememberImageLoader(),
      ) {
        val appScope = rememberCoroutineScope()
        val chat = remember(env) { CloudChatController(env.ai, appScope, agent = env.agent, history = ChatHistory(env.chatFiles, appScope) { env.agent?.awaitSessionIdle(it) }, creationStore = env.games) }
        // 登录、换号后读出这个账号存下的对话。
        val signedIn by env.session.state.collectAsState()
        LaunchedEffect(signedIn.loggedIn, signedIn.accountId, signedIn.generation) {
            chat.useAccount(if (signedIn.loggedIn) signedIn.accountId else null)
            chat.refreshModelCatalogue(force = true)
            while (isActive) { delay(60_000); chat.refreshModelCatalogue() }
        }
        var tab by rememberSaveable { mutableStateOf(Tab.Ai.name) }
        var stackText by rememberSaveable { mutableStateOf("") }
        var publishOpen by rememberSaveable { mutableStateOf(false) }
        var communityPost by remember { mutableStateOf(Route.NewPost()) }
        var gameOverlayOpen by remember { mutableStateOf(false) }
        val stack = remember(stackText) { Route.decodeStack(stackText) }
        val mainAiVisible = tab == Tab.Ai.name && stack.none { it != Route.Login }
        LaunchedEffect(mainAiVisible, signedIn.generation) {
            if (mainAiVisible) chat.openMainConversation()
        }
        // 回调可能在异步操作（如登录）结束后才调用，所以每次都读最新的栈，而不是组合时捕获的那份。
        fun push(route: Route) { stackText = Route.encodeStack(Route.decodeStack(stackText) + route) }
        fun pop() { stackText = Route.encodeStack(Route.decodeStack(stackText).dropLast(1)) }
        fun closeLogin() { if (Route.decodeStack(stackText).lastOrNull() == Route.Login) pop() }

        BackHandler(enabled = stack.isNotEmpty()) { pop() }
        // 回到最外面再按返回：由应用自己结束，用普通的关闭动画，不让系统播放“缩到右下角”的返回桌面动画。放在其它返回处理（弹层、侧栏）前面，优先级最低。
        BackHandler(enabled = env.platform != null && stack.isEmpty()) { env.platform?.exitApp() }

        NotificationSystemObserver(
            env = env,
            chat = chat,
            isAiTabVisible = mainAiVisible || stack.lastOrNull() == Route.AiChat,
            isNotificationsScreenVisible = stack.lastOrNull() == Route.Notifications,
            onOpenAiConversation = { convId ->
                stackText = ""
                tab = Tab.Ai.name
                if (!convId.isNullOrBlank()) chat.openConversation(convId)
            },
            onOpenRemotePeer = { peer ->
                push(Route.RemoteChat(peer))
            },
            onOpenCommunityNotifications = {
                push(if (signedIn.loggedIn) Route.Notifications else Route.Login)
            },
        )

        CommunityLinkConfirmation(env, ::push, Modifier.background(MaterialTheme.colorScheme.background)) {
            val pages = stack.filter { it != Route.Login }
            val pageTransition = updateTransition(pages, label = "页面切换")
            val tabShift by pageTransition.animateFloat(
                transitionSpec = { tween(PageMillis, easing = FastOutSlowInEasing) },
                label = "一级页让位",
            ) { if (it.isEmpty()) 0f else 1f }
            // 一级页跟子页面共用转场，返回时保留滚动位置和已加载的数据。
            Box(
                Modifier.fillMaxSize().graphicsLayer { translationX = -size.width * tabShift }
                    .layoutVisiblePage {
                        pageTransition.currentState.isEmpty() || pageTransition.targetState.isEmpty()
                    },
            ) {
                val selectedTab = Tab.valueOf(tab)
                val gamesSelected = selectedTab == Tab.Games
                RetainedTabPage(selected = gamesSelected) {
                    GamesScreen(
                        env, chat, ::push, active = gamesSelected,
                        onOverlayChanged = { gameOverlayOpen = it },
                    )
                }
                RetainedTabPage(selected = selectedTab == Tab.Community) {
                    CommunityScreen(env, ::push, onPostDestination = { communityPost = it })
                }
                RetainedTabPage(selected = selectedTab == Tab.Ai) {
                    AiChatPage(
                        env, chat, open = ::push, visible = mainAiVisible,
                        onSelectTab = { tab = it.name }, onPublish = { publishOpen = true },
                    )
                }
                RetainedTabPage(selected = selectedTab == Tab.Mine) {
                    MineScreen(env, ::push)
                }
                if (selectedTab != Tab.Ai && (!gameOverlayOpen || !gamesSelected)) {
                    MainBottomNav(selectedTab, onSelect = { tab = it.name }, onPublish = { publishOpen = true },
                        modifier = Modifier.align(Alignment.BottomCenter))
                }
            }
            pageTransition.AnimatedContent(
                modifier = Modifier.fillMaxSize(),
                transitionSpec = {
                    (EnterTransition.None togetherWith ExitTransition.None).using(null)
                },
            ) { visiblePages ->
                visiblePages.lastOrNull()?.let { route ->
                    // 在图层上移动已布局的页面，动画帧只更新横向位移。
                    val pageOffset by transition.animateFloat(
                        transitionSpec = { tween(PageMillis, easing = FastOutSlowInEasing) },
                        label = "子页位移",
                    ) { state ->
                        when (state) {
                            EnterExitState.PreEnter -> if (visiblePages.size > pageTransition.currentState.size) 1f else -1f
                            EnterExitState.Visible -> 0f
                            EnterExitState.PostExit -> if (visiblePages.size < pageTransition.targetState.size) -1f else 1f
                        }
                    }
                    // 子页面盖住一级页，并吃掉点击，免得穿透到下面。
                    val pageInsets = when (route) {
                        Route.Models, Route.MyWorks, Route.Notifications -> Modifier
                        else -> Modifier.navigationBarsPadding()
                    }
                    Box(
                        Modifier.fillMaxSize().graphicsLayer { translationX = size.width * pageOffset }
                            .background(MaterialTheme.colorScheme.background).then(pageInsets)
                            .clickable(indication = null, interactionSource = null) {},
                    ) {
                        RouteScreen(env, chat, route, ::push, ::pop)
                    }
                }
            }
            env.platform?.appUpdates?.let { AppUpdateDialog(it) }
            env.platform?.notifications?.let { NotificationPermissionDialog(it) }
            OverlayTransition(publishOpen) { panel ->
                PublishSheet(
                    onDismiss = { publishOpen = false },
                    onAiCreate = { publishOpen = false; chat.clearConversation(); push(Route.AiChat) },
                    onNewPost = {
                        publishOpen = false
                        push(if (tab == Tab.Community.name) communityPost else Route.NewPost())
                    },
                    panelModifier = panel,
                )
            }
            OverlayTransition(stack.lastOrNull() == Route.Login, direction = "center") {
                LoginDialog(env, done = ::closeLogin, back = ::closeLogin)
            }
        }
      }
    }
}

@Composable
private fun RouteScreen(env: CommunityEnv, chat: CloudChatController, route: Route, push: (Route) -> Unit, pop: () -> Unit) {
    when (route) {
        is Route.Pack -> PackDetailScreen(env, route.id, pop, push, chat)
        is Route.Ability -> AbilityDetailScreen(env, route.id, pop, push)
        is Route.Scheme -> SchemeDetailScreen(env, route.id, pop, push)
        Route.SchemeBackups -> NativeSchemeBackupsScreen(env, pop, push)
        Route.MyWorks -> MyContentScreen(env, chat, pop, push)
        Route.Notifications -> NativeNotificationsScreen(env, pop, push)
        Route.Challenges -> NativeChallengesScreen(env, pop, push)
        is Route.Challenge -> NativeChallengeDetailScreen(env, route.key, pop, push)
        Route.Bookmarks -> NativeBookmarksScreen(env, pop, push)
        Route.Blocks -> NativeBlocksScreen(env, pop, push)
        is Route.User -> NativeProfileScreen(env, route.uid, route.tab, pop, push)
        is Route.PublishPack -> PublishContentScreen(env, route.packId, pop, push, route.localGameId)
        is Route.Topic -> TopicDetailScreen(env, route.id, pop, push, showComments = route.showComments)
        is Route.Weekly -> WeeklyDetailScreen(env, route.week, push, pop)
        is Route.Project -> ProjectDetailScreen(env, route.slug, pop)
        Route.Login -> Unit // 登录作为弹窗显示在当前页面之上。
        Route.Search -> SearchScreen(env, back = pop, open = push)
        is Route.NewPost -> NewPostScreen(env, back = pop, open = push, initialSection = route.section, initialBoardSlug = route.boardSlug)
        is Route.CreationPost -> NewPostScreen(env, back = pop, open = push, initialTitle = route.title, initialBody = route.body)
        Route.NewHelp -> NewPostScreen(env, back = pop, open = push, help = true)
        is Route.Tool -> ToolDetailScreen(env, route.key, pop)
        Route.Daily -> DailyScreen(env, pop, push)
        Route.Frontier -> DailyScreen(env, pop, push, frontier = true)
        is Route.FrontierArticle -> NewsDetailScreen(env, route.key, pop, push, frontier = true)
        is Route.Articles -> EditorialListScreen(env, route.category, route.title, pop, push)
        is Route.News -> NewsDetailScreen(env, route.key, pop, push)
        is Route.Repo -> RepoDetailScreen(env, route.owner, route.repo, pop, push)
        is Route.WebPage -> EmbeddedWebPage(route.url, pop, env.platform)
        Route.MicroGallery -> MicroGalleryScreen(env.config.resourceBase, back = pop, open = push)
        is Route.GalleryExhibit -> MicroGalleryExhibit.byId(route.id)?.let { exhibit ->
            if (exhibit.usesIdeaRuntime) IdeaRuntimeScreen(env, exhibit.id, exhibit.title, back = pop)
            else GalleryExhibitScreen(env.config.resourceBase, exhibit, back = pop, open = push)
        }
        is Route.IdeaWork -> IdeaWorkScreen(env, route.id, back = pop, open = push)
        is Route.IdeaRuntime -> IdeaRuntimeScreen(env, route.id, route.title, back = pop)
        is Route.Software -> SoftwareDetailScreen(env, route.project, pop, push)
        is Route.OpenSource -> RepoDetailScreen(env, route.owner, route.repo, pop, push, curated = true)
        is Route.OpenSourceCollection -> OpenSourceCollectionScreen(env, route.key, pop, push)
        Route.Leaderboard -> LeaderboardScreen(env, pop)
        Route.UsageLeaderboard -> LiveUsageRankingsScreen(env, pop)
        Route.Scatter -> ScatterScreen(env, pop, push)
        Route.Settings -> SettingsScreen(env, back = pop, open = push)
        Route.Models -> ModelsScreen(env, chat, back = pop, open = push)
        Route.Appearance -> AppearanceScreen(back = pop)
        Route.Voice -> VoiceSettingsScreen(env, back = pop)
        Route.NotificationSettings -> NotificationSettingsScreen(env, back = pop)
        Route.ToolEnvironment -> ToolEnvironmentScreen(env.platform?.toolEnvironment, back = pop)
        Route.AiChat -> AiChatPage(env, chat, back = pop, open = push)
        is Route.AiCreate -> if (route.localGameId == null) {
            AiChatPage(env, chat, back = pop, open = push, openCreation = true)
        } else {
            val loader = rememberLoader(env, route) {
                checkNotNull(env.games) { "当前设备无法读取本机作品" }.read(route.localGameId)
            }
            val work by loader.state.collectAsState()
            when (val loaded = work) {
                is Load.Ready -> AiChatPage(env, chat, back = pop, open = push, openCreation = true, workToEdit = loaded.value)
                else -> Column(Modifier.fillMaxSize()) {
                    ScreenBar("继续修改", pop)
                    LoadBox(loaded, loader::reload) {}
                }
            }
        }
        Route.Devices -> DevicesScreen(env, back = pop, open = push)
        is Route.RemoteChat -> RemoteChatScreen(env, route.peer, back = pop, open = push, initialDraft = route.draft)
    }
}

/** 底栏提供 AI 创作和发帖入口；作品可从发帖页添加。 */
@Composable
private fun PublishSheet(onDismiss: () -> Unit, onAiCreate: () -> Unit, onNewPost: () -> Unit, panelModifier: Modifier) {
    BottomSheet(onDismiss, panelModifier, showHandle = true, cancelLabel = "取消") {
        Text("发布", style = XType.Section, fontWeight = FontWeight.SemiBold, modifier = Modifier.padding(bottom = 8.dp))
        SheetAction(XIcons.Sparkle, "用 AI 创作", "一句话生成特效、壁纸、组件或小游戏", onAiCreate)
        SheetAction(XIcons.Pen, "发帖", "发布到创意、讨论或资源板块", onNewPost)
    }
}
