package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
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
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.backhandler.BackHandler
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import xyz.xiangban.xiaoban.common.community.ChatHistory
import xyz.xiangban.xiaoban.common.community.CloudChatController

/**
 * 社区原生界面入口：四个去处（小伴AI、游戏、社区、我的）加一个页面栈，底部是悬浮胶囊导航，中间的“发布”打开弹层。
 * 打开子页面时一级页仍留在下面（只是被盖住），返回后滚动位置和已加载的数据都在。
 */
/** 页面进入、返回的转场时长。 */
private const val PageMillis = 260

@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun CommunityNativeApp(env: CommunityEnv) {
    remember(env) { ThemeState.load(env.prefs); SpeechState.load(env.prefs) }
    XiaobanTheme {
      CompositionLocalProvider(LocalReplyReader provides env.reader) {
        val appScope = rememberCoroutineScope()
        val chat = remember(env) { CloudChatController(env.ai, appScope, agent = env.agent, history = ChatHistory(env.chatFiles, appScope)) }
        // 登录、换号后读出这个账号存下的对话。
        val signedIn by env.session.state.collectAsState()
        LaunchedEffect(signedIn.loggedIn, signedIn.accountId) { if (signedIn.loggedIn) chat.useAccount(signedIn.accountId) }
        var tab by rememberSaveable { mutableStateOf(Tab.Ai.name) }
        var stackText by rememberSaveable { mutableStateOf("") }
        var publishOpen by rememberSaveable { mutableStateOf(false) }
        var aiDrawerOpen by rememberSaveable { mutableStateOf(false) }
        val stack = Route.decodeStack(stackText)
        // 回调可能在异步操作（如登录）结束后才调用，所以每次都读最新的栈，而不是组合时捕获的那份。
        fun push(route: Route) { stackText = Route.encodeStack(Route.decodeStack(stackText) + route) }
        fun pop() { stackText = Route.encodeStack(Route.decodeStack(stackText).dropLast(1)) }
        fun closeLogin() { if (Route.decodeStack(stackText).lastOrNull() == Route.Login) pop() }

        BackHandler(enabled = stack.isNotEmpty()) { pop() }
        // 回到最外面再按返回：由应用自己结束，用普通的关闭动画，不让系统播放“缩到右下角”的返回桌面动画。放在其它返回处理（弹层、侧栏）前面，优先级最低。
        BackHandler(enabled = env.platform != null && stack.isEmpty()) { env.platform?.exitApp() }

        Box(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.background)) {
            // 一级页（含底部导航和对话列表侧栏）：打开子页面时整层往左推出屏幕，和子页面同步移动，过程中不会露出首页；
            // 返回时再原样推回来，滚动位置和已加载的数据都在。
            val tabShift by animateFloatAsState(if (stack.any { it != Route.Login }) 1f else 0f, tween<Float>(PageMillis, easing = FastOutSlowInEasing), label = "一级页让位")
            Box(Modifier.fillMaxSize().graphicsLayer { translationX = -size.width * tabShift }) {
                AnimatedContent(targetState = Tab.valueOf(tab), modifier = Modifier.fillMaxSize(), transitionSpec = { fadeIn(tween(180)) togetherWith fadeOut(tween(180)) }, label = "主导航切换") { currentTab ->
                    when (currentTab) {
                        Tab.Ai -> AiHomeScreen(env, chat, ::push, { aiDrawerOpen = true })
                        Tab.Games -> GamesScreen()
                        Tab.Community -> CommunityScreen(env, ::push)
                        Tab.Mine -> MineScreen(env, ::push)
                    }
                }
                FloatingNav(Tab.valueOf(tab), publishOpen, onSelect = { tab = it.name }, onPublish = { publishOpen = true }, modifier = Modifier.align(Alignment.BottomCenter))
                AiDrawerOverlay(chat, aiDrawerOpen, { aiDrawerOpen = false }, ::push)
            }
            val pages = stack.filter { it != Route.Login }
            AnimatedContent(
                targetState = pages,
                modifier = Modifier.fillMaxSize(),
                // 页面一直是实心的，不叠淡入淡出；新旧两页（包括下面的一级页）整屏一起滑动，任何时候都铺满屏幕，不会露出别的页：
                // 进入时新页从右边推进来、旧页往左推出；返回时反过来。
                transitionSpec = {
                    val push = targetState.size > initialState.size
                    val spec = tween<IntOffset>(PageMillis, easing = FastOutSlowInEasing)
                    if (push) slideInHorizontally(spec) { it } togetherWith slideOutHorizontally(spec) { -it }
                    else slideInHorizontally(spec) { -it } togetherWith slideOutHorizontally(spec) { it }
                },
                label = "页面切换",
            ) { visiblePages ->
                visiblePages.lastOrNull()?.let { route ->
                    // 子页面盖住一级页，并吃掉点击，免得穿透到下面。
                    Box(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.background).navigationBarsPadding().clickable(indication = null, interactionSource = null) {}) {
                        RouteScreen(env, chat, route, ::push, ::pop)
                    }
                }
            }
            OverlayTransition(publishOpen) { panel ->
                PublishSheet(
                    env,
                    onDismiss = { publishOpen = false },
                    onAiCreate = { publishOpen = false; chat.clearConversation(); push(Route.AiChat) },
                    onNewPost = { publishOpen = false; push(Route.NewPost) },
                    onPublishPack = { publishOpen = false; push(Route.PublishPack()) },
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
        is Route.Pack -> PackDetailScreen(env, route.id, pop, push)
        is Route.Ability -> AbilityDetailScreen(env, route.id, pop, push)
        is Route.Scheme -> SchemeDetailScreen(env, route.id, pop, push)
        Route.MyWorks -> MyContentScreen(env, pop, push)
        Route.Notifications -> NativeNotificationsScreen(env, pop, push)
        Route.Challenges -> NativeChallengesScreen(env, pop, push)
        is Route.Challenge -> NativeChallengeDetailScreen(env, route.key, pop, push)
        Route.Bookmarks -> NativeBookmarksScreen(env, pop, push)
        Route.Blocks -> NativeBlocksScreen(env, pop, push)
        is Route.User -> NativeProfileScreen(env, route.uid, route.tab, pop, push)
        is Route.PublishPack -> PublishContentScreen(env, route.packId, pop, push)
        is Route.Topic -> TopicDetailScreen(env, route.id, pop, push)
        is Route.Weekly -> WeeklyDetailScreen(env, route.week, push, pop)
        is Route.Project -> ProjectDetailScreen(env, route.slug, pop)
        Route.Login -> Unit // 登录作为弹窗显示在当前页面之上。
        Route.Search -> SearchScreen(env, back = pop, open = push)
        Route.NewPost -> NewPostScreen(env, back = pop, open = push)
        Route.NewHelp -> NewPostScreen(env, back = pop, open = push, help = true)
        is Route.Tool -> ToolDetailScreen(env, route.key, pop)
        Route.Daily -> DailyScreen(env, pop)
        Route.Leaderboard -> LeaderboardScreen(env, pop)
        Route.Scatter -> ScatterScreen(env, pop, push)
        Route.Settings -> SettingsScreen(env, back = pop, open = push)
        Route.Models -> ModelsScreen(env, chat, back = pop, open = push)
        Route.Appearance -> AppearanceScreen(back = pop, open = push)
        Route.AccentColor -> AccentScreen(back = pop)
        Route.Voice -> VoiceSettingsScreen(env, back = pop)
        Route.AiChat -> AiChatPage(env, chat, back = pop, open = push)
        Route.Devices -> DevicesScreen(env, back = pop, open = push)
        is Route.RemoteChat -> RemoteChatScreen(env, route.peer, back = pop, open = push)
    }
}

/** 发布弹层：用 AI 创作走云端小伴；发布已有作品和发帖目前在网页上完成。 */
@Composable
private fun PublishSheet(env: CommunityEnv, onDismiss: () -> Unit, onAiCreate: () -> Unit, onNewPost: () -> Unit, onPublishPack: () -> Unit, panelModifier: Modifier) {
    val site = env.config.forumBase.trimEnd('/')
    BottomSheet(onDismiss, panelModifier) {
        Text("发布", fontSize = 18.sp, fontWeight = FontWeight.SemiBold, modifier = Modifier.padding(bottom = 8.dp))
        SheetAction(XIcons.Sparkle, "用 AI 创作", "一句话生成特效、壁纸、组件或小游戏", onAiCreate)
        SheetAction(XIcons.Upload, "发布已有作品", "选择资源包（.zip），发布到社区", onPublishPack)
        SheetAction(XIcons.Pen, "发帖", "在讨论或资源里发一个帖子", onNewPost)
        GhostPillButton("取消", onDismiss, Modifier.padding(top = 8.dp).fillMaxWidth())
    }
}
