package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.Pack
import xyz.xiangban.xiaoban.common.community.Topic
import xyz.xiangban.xiaoban.common.community.messageOf

private const val HistoryKey = "search_history"
private const val HistoryMax = 8

/** 搜索历史存在本机偏好里，一行一个，最新的在前。 */
internal fun readHistory(env: CommunityEnv): List<String> = env.prefs.get(HistoryKey).orEmpty().split('\n').filter { it.isNotBlank() }

internal fun pushHistory(env: CommunityEnv, query: String): List<String> {
    val next = (listOf(query) + readHistory(env).filter { it != query }).take(HistoryMax)
    env.prefs.put(HistoryKey, next.joinToString("\n"))
    return next
}

/**
 * 搜索：顶部是红边的圆角输入框和「取消」；没输入时下面是搜索历史（可清空）和热门搜索榜（前三名红色名次，词右边可带「热」「新」）；
 * 搜索后分「作品」「帖子」两组显示结果。
 */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun SearchScreen(env: CommunityEnv, back: () -> Unit, open: (Route) -> Unit) {
    val scope = rememberCoroutineScope()
    var query by rememberSaveable { mutableStateOf("") }
    var history by remember { mutableStateOf(readHistory(env)) }
    var packs by remember { mutableStateOf<List<Pack>>(emptyList()) }
    var topics by remember { mutableStateOf<List<Topic>>(emptyList()) }
    var busy by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    var searched by remember { mutableStateOf(false) }
    val hot = rememberLoader(env, "hot-search") { env.forum.hotSearch() }
    val hotState by hot.state.collectAsState()

    fun search(text: String) {
        val q = text.trim()
        if (q.isEmpty() || busy) return
        query = q
        history = pushHistory(env, q)
        busy = true; error = null; searched = true
        scope.launch {
            try {
                val p = runCatching { env.forum.packs(q = q).items }
                val t = runCatching { env.forum.searchTopics(q).items }
                packs = p.getOrDefault(emptyList())
                topics = t.getOrDefault(emptyList())
                if (p.isFailure && t.isFailure) error = messageOf(p.exceptionOrNull() ?: t.exceptionOrNull() ?: RuntimeException())
            } finally {
                busy = false
            }
        }
    }

    Column(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface)) {
        Row(Modifier.fillMaxWidth().statusBarsPadding().padding(start = 16.dp, end = 8.dp, top = 8.dp, bottom = 8.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Row(
                Modifier.weight(1f).height(40.dp).clip(RoundedCornerShape(20.dp)).background(LocalExtraColors.current.fill).border(1.dp, Brand, RoundedCornerShape(20.dp)).padding(horizontal = 14.dp),
                verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                BasicTextField(
                    query, { query = it; if (it.isEmpty()) searched = false }, Modifier.weight(1f), singleLine = true,
                    textStyle = TextStyle(fontSize = 14.sp, color = MaterialTheme.colorScheme.onSurface), cursorBrush = SolidColor(Brand),
                    keyboardOptions = KeyboardOptions(imeAction = ImeAction.Search), keyboardActions = KeyboardActions(onSearch = { search(query) }),
                    decorationBox = { inner ->
                        Box(contentAlignment = Alignment.CenterStart) {
                            if (query.isEmpty()) Text("搜索作品、项目、帖子", fontSize = 14.sp, color = LocalExtraColors.current.ink3)
                            inner()
                        }
                    },
                )
                XIcon(XIcons.Search, size = 20.dp, description = "搜索", modifier = Modifier.clickable { search(query) })
            }
            Text("取消", Modifier.clickable(onClick = back).padding(horizontal = 8.dp, vertical = 12.dp), fontSize = 15.sp)
        }
        when {
            busy -> Spinner()
            error != null -> CenterMessage(error.orEmpty(), "重试", onAction = { search(query) })
            !searched -> Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(horizontal = 16.dp)) {
                if (history.isNotEmpty()) {
                    Row(Modifier.fillMaxWidth().padding(top = 16.dp, bottom = 10.dp), verticalAlignment = Alignment.CenterVertically) {
                        Text("搜索历史", Modifier.weight(1f), fontSize = 15.sp, fontWeight = FontWeight.SemiBold)
                        Text("清空", Modifier.clickable { env.prefs.put(HistoryKey, ""); history = emptyList() }.padding(horizontal = 4.dp, vertical = 6.dp), fontSize = 14.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                    FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        history.forEach { h ->
                            Text(h, Modifier.height(32.dp).clip(RoundedCornerShape(16.dp)).background(LocalExtraColors.current.fill).clickable { search(h) }.padding(horizontal = 14.dp, vertical = 6.dp), fontSize = 14.sp, maxLines = 1)
                        }
                    }
                }
                val words = (hotState as? xyz.xiangban.xiaoban.common.community.Load.Ready<xyz.xiangban.xiaoban.common.community.HotWords>)?.value?.items.orEmpty()
                if (words.isNotEmpty()) {
                    Text("热门搜索", Modifier.padding(top = 20.dp, bottom = 6.dp), fontSize = 15.sp, fontWeight = FontWeight.SemiBold)
                    words.forEachIndexed { i, w ->
                        if (i > 0) Hairline()
                        Row(Modifier.fillMaxWidth().clickable { search(w.text) }.padding(vertical = 12.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                            Text("${i + 1}", Modifier.width(20.dp), fontSize = 15.sp, fontWeight = FontWeight.SemiBold, color = if (i < 3) Brand else MaterialTheme.colorScheme.onSurfaceVariant)
                            Text(w.text, Modifier.weight(1f), fontSize = 15.sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
                            if (w.tag.isNotEmpty()) Text(w.tag, fontSize = 12.sp, fontWeight = FontWeight.SemiBold, color = Brand)
                        }
                    }
                } else if (history.isEmpty()) {
                    Text("输入关键词，搜索社区里的作品和帖子。", Modifier.padding(top = 20.dp), fontSize = 14.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
            packs.isEmpty() && topics.isEmpty() -> CenterMessage("没有找到和“$query”相关的内容")
            else -> Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState())) {
                if (packs.isNotEmpty()) {
                    GroupLabel("作品")
                    packs.forEachIndexed { i, pack ->
                        if (i > 0) Hairline(Modifier.padding(start = 16.dp))
                        Column(Modifier.fillMaxWidth().clickable { open(Route.Pack(pack.id)) }.padding(16.dp)) {
                            Text(pack.title, fontSize = 15.sp, fontWeight = FontWeight.Medium, maxLines = 1, overflow = TextOverflow.Ellipsis)
                            Text(packMeta(pack), Modifier.padding(top = 4.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
                        }
                    }
                }
                if (topics.isNotEmpty()) {
                    GroupLabel("帖子")
                    topics.forEachIndexed { i, topic ->
                        if (i > 0) Hairline(Modifier.padding(start = 16.dp))
                        TopicRow(topic) { open(Route.Topic(topic.id)) }
                    }
                }
            }
        }
    }
}
