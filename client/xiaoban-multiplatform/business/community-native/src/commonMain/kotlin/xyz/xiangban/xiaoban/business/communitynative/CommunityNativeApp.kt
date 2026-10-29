package xyz.xiangban.xiaoban.business.communitynative

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
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import xyz.xiangban.xiaoban.common.community.CloudChatController

/**
 * 社区原生界面入口：四个去处（小伴AI、游戏、社区、我的）加一个页面栈，底部是悬浮胶囊导航，中间的“发布”打开弹层。
 * 打开子页面时一级页仍留在下面（只是被盖住），返回后滚动位置和已加载的数据都在。
 */
@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun CommunityNativeApp(env: CommunityEnv) {
    XiaobanTheme {
        val appScope = rememberCoroutineScope()
        val chat = remember(env) { CloudChatController(env.ai, appScope) }
        var tab by rememberSaveable { mutableStateOf(Tab.Ai.name) }
        var stackText by rememberSaveable { mutableStateOf("") }
        var publishOpen by rememberSaveable { mutableStateOf(false) }
        var aiDrawerOpen by rememberSaveable { mutableStateOf(false) }
        val stack = Route.decodeStack(stackText)
        // 回调可能在异步操作（如登录）结束后才调用，所以每次都读最新的栈，而不是组合时捕获的那份。
        fun push(route: Route) { stackText = Route.encodeStack(Route.decodeStack(stackText) + route) }
        fun pop() { stackText = Route.encodeStack(Route.decodeStack(stackText).dropLast(1)) }

        BackHandler(enabled = stack.isNotEmpty()) { pop() }

        Box(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.background)) {
            Box(Modifier.fillMaxSize()) {
                when (Tab.valueOf(tab)) {
                    Tab.Ai -> AiHomeScreen(env, chat, ::push, aiDrawerOpen, { aiDrawerOpen = it })
                    Tab.Games -> GamesScreen()
                    Tab.Community -> CommunityScreen(env, ::push)
                    Tab.Mine -> MineScreen(env, ::push)
                }
            }
            if (!aiDrawerOpen) {
                FloatingNav(Tab.valueOf(tab), publishOpen, onSelect = { tab = it.name }, onPublish = { publishOpen = true }, modifier = Modifier.align(Alignment.BottomCenter))
            }
            stack.lastOrNull()?.let { route ->
                // 盖在最上面，并吃掉点击，免得穿透到下面的一级页。
                Box(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.background).navigationBarsPadding().clickable(indication = null, interactionSource = null) {}) {
                    RouteScreen(env, chat, route, ::push, ::pop)
                }
            }
            if (publishOpen) {
                PublishSheet(
                    env,
                    onDismiss = { publishOpen = false },
                    onAiCreate = { publishOpen = false; chat.clearConversation(); push(Route.AiChat) },
                    onNewPost = { publishOpen = false; push(Route.NewPost) },
                )
            }
        }
    }
}

@Composable
private fun RouteScreen(env: CommunityEnv, chat: CloudChatController, route: Route, push: (Route) -> Unit, pop: () -> Unit) {
    when (route) {
        is Route.Pack -> PackDetailScreen(env, route.id, pop)
        is Route.Topic -> TopicDetailScreen(env, route.id, pop, push)
        is Route.Weekly -> WeeklyDetailScreen(env, route.week, push, pop)
        is Route.Project -> ProjectDetailScreen(env, route.slug, pop)
        Route.Login -> LoginScreen(env, done = pop, back = pop)
        Route.Search -> SearchScreen(env, back = pop, open = push)
        Route.NewPost -> NewPostScreen(env, back = pop, open = push)
        Route.Settings -> SettingsScreen(env, back = pop, open = push)
        Route.Models -> ModelsScreen(env, chat, back = pop, open = push)
        Route.AiChat -> AiChatPage(env, chat, back = pop, open = push)
        Route.Devices -> DevicesScreen(env, back = pop, open = push)
        is Route.RemoteList -> RemoteListScreen(env, route.peer, back = pop, open = push)
        is Route.RemoteChat -> RemoteChatScreen(env, route.peer, back = pop, open = push)
    }
}

/** 发布弹层：用 AI 创作走云端小伴；发布已有作品和发帖目前在网页上完成。 */
@Composable
private fun PublishSheet(env: CommunityEnv, onDismiss: () -> Unit, onAiCreate: () -> Unit, onNewPost: () -> Unit) {
    val site = env.config.forumBase.trimEnd('/')
    BottomSheet(onDismiss) {
        Text("发布", fontSize = 18.sp, fontWeight = FontWeight.SemiBold, modifier = Modifier.padding(bottom = 8.dp))
        SheetAction(XIcons.Sparkle, "用 AI 创作", "一句话生成特效、壁纸、组件或小游戏", onAiCreate)
        SheetAction(XIcons.Upload, "发布已有作品", "选择资源包（.zip），在网页上发布到社区") { onDismiss(); env.platform?.openUrl("$site/publish") }
        SheetAction(XIcons.Pen, "发帖", "在讨论或资源里发一个帖子", onNewPost)
        GhostPillButton("取消", onDismiss, Modifier.padding(top = 8.dp).fillMaxWidth())
    }
}
