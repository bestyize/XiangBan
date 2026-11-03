package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.layout.*
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import xyz.xiangban.xiaoban.common.community.DiscussController
import xyz.xiangban.xiaoban.common.community.Load

/** Article recommendations and installable works are separate sections of the same native feed. */
@Composable
internal fun EditorialSection(env: CommunityEnv, category: String, title: String, open: (Route) -> Unit, intro: String = "") {
    val loader = rememberLoader(env, "editorial:$category") { env.forum.topics(sort = "new", category = category, home = false) }
    val state by loader.state.collectAsState()
    Column(Modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(10.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text(title, Modifier.weight(1f), style = XType.Section, fontWeight = FontWeight.SemiBold)
            LinkText("查看全部", { open(Route.Articles(category, title)) })
        }
        if (intro.isNotBlank()) Text(intro, style = XType.Caption, color = LocalExtraColors.current.ink3)
        when (val current = state) {
            is Load.Ready -> {
                if (current.value.items.isEmpty()) Text("暂无图文分享", style = XType.Caption, color = LocalExtraColors.current.ink3)
                else current.value.items.take(4).chunked(if (category.startsWith("resource-")) 2 else 1).forEach { row ->
                    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        row.forEach { topic -> Box(Modifier.weight(1f)) {
                            if (category.startsWith("resource-")) VisualTopicCard(env, topic) { open(Route.Topic(topic.id)) }
                            else TopicCard(env, topic, open, {})
                        } }
                        if (category.startsWith("resource-") && row.size == 1) Spacer(Modifier.weight(1f))
                    }
                }
            }
            is Load.Failed -> Banner(current.message, warn = true, actionText = "重试", onAction = loader::reload)
            else -> Spinner(Modifier.fillMaxWidth().height(80.dp))
        }
    }
}

@Composable
internal fun EditorialListScreen(env: CommunityEnv, category: String, title: String, back: () -> Unit, open: (Route) -> Unit) {
    val scope = rememberCoroutineScope()
    val controller = remember(env, category) { DiscussController(env.forum, scope, category = category, home = false) }
    Column(Modifier.fillMaxSize()) {
        ScreenBar(title, back)
        if (category.startsWith("resource-")) CasualWaterfall(env, controller, open)
        else TopicFeed(controller, "暂无图文分享", open, initialSort = "new") { TopicCard(env, it, open, {}) }
    }
}
