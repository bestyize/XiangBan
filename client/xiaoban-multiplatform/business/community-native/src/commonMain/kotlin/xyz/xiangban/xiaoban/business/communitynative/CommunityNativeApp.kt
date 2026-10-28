package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.backhandler.BackHandler
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

/**
 * 社区原生界面入口：底部四个 Tab（社区、讨论、小伴、我的）加一个页面栈。
 * 打开详情页时 Tab 页仍留在下面（只是被盖住），返回后滚动位置和已加载的数据都在。
 */
@Composable
fun CommunityNativeApp(env: CommunityEnv) {
    XiaobanTheme {
        var tab by rememberSaveable { mutableStateOf(Tab.Community.name) }
        var stackText by rememberSaveable { mutableStateOf("") }
        val stack = Route.decodeStack(stackText)
        // 回调可能在异步操作（如登录）结束后才调用，所以每次都读最新的栈，而不是组合时捕获的那份。
        fun push(route: Route) { stackText = Route.encodeStack(Route.decodeStack(stackText) + route) }
        fun pop() { stackText = Route.encodeStack(Route.decodeStack(stackText).dropLast(1)) }

        BackHandler(enabled = stack.isNotEmpty()) { pop() }

        Box(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.background)) {
            Column(Modifier.fillMaxSize()) {
                Box(Modifier.weight(1f)) {
                    when (Tab.valueOf(tab)) {
                        Tab.Community -> FeedScreen(env, ::push)
                        Tab.Discuss -> DiscussScreen(env, ::push)
                        Tab.Ai -> AiPlaceholderScreen()
                        Tab.Mine -> MineScreen(env) { push(Route.Login) }
                    }
                }
                BottomBar(Tab.valueOf(tab)) { tab = it.name }
            }
            stack.lastOrNull()?.let { route ->
                // 盖在最上面，并吃掉点击，免得穿透到下面的 Tab 页。
                Box(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.background).navigationBarsPadding().clickable(indication = null, interactionSource = null) {}) {
                    RouteScreen(env, route, ::push, ::pop)
                }
            }
        }
    }
}

@Composable
private fun RouteScreen(env: CommunityEnv, route: Route, push: (Route) -> Unit, pop: () -> Unit) {
    when (route) {
        is Route.Pack -> PackDetailScreen(env, route.id, pop)
        is Route.Topic -> TopicDetailScreen(env, route.id, pop)
        is Route.Weekly -> WeeklyDetailScreen(env, route.week, push, pop)
        is Route.Project -> ProjectDetailScreen(env, route.slug, pop)
        Route.Login -> LoginScreen(env, done = pop, back = pop)
    }
}

@Composable
private fun BottomBar(current: Tab, onSelect: (Tab) -> Unit) {
    Column(Modifier.background(MaterialTheme.colorScheme.surface)) {
        Hairline()
        Row(Modifier.fillMaxWidth().navigationBarsPadding()) {
            Tab.entries.forEach { tab ->
                val selected = tab == current
                Box(Modifier.weight(1f).clickable { onSelect(tab) }.padding(vertical = 14.dp), contentAlignment = Alignment.Center) {
                    Text(
                        tab.label, fontSize = 14.sp,
                        fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Normal,
                        color = if (selected) Brand else LocalExtraColors.current.ink3,
                    )
                }
            }
        }
    }
}
