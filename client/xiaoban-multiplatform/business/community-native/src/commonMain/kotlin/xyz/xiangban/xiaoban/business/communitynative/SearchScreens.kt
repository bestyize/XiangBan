package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
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
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.HotWords
import xyz.xiangban.xiaoban.common.community.Load
import xyz.xiangban.xiaoban.common.community.Pack
import xyz.xiangban.xiaoban.common.community.Topic
import xyz.xiangban.xiaoban.common.community.agoText
import xyz.xiangban.xiaoban.common.community.compactCount
import xyz.xiangban.xiaoban.common.community.kindLabel
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

/** 把 [text] 里和 [query] 相同的片段（不分大小写）用 [color] 加粗标出来。 */
internal fun highlight(text: String, query: String, color: Color): AnnotatedString = buildAnnotatedString {
    if (query.isBlank()) { append(text); return@buildAnnotatedString }
    var i = 0
    while (i < text.length) {
        val at = text.indexOf(query, i, ignoreCase = true)
        if (at < 0) { append(text.substring(i)); break }
        append(text.substring(i, at))
        withStyle(SpanStyle(color = color, fontWeight = FontWeight.SemiBold)) { append(text.substring(at, at + query.length)) }
        i = at + query.length
    }
}

private enum class ResultTab(val label: String) { All("全部"), Packs("作品"), Topics("帖子") }

/**
 * 搜索：灰色页底加白色卡片，输入框是浅灰圆角、没有描边。没输入时是「搜索历史」和「热门搜索」两张卡（前三名名次是红色方块，词右边可带「热」「新」）；
 * 搜索后显示结果数、「全部 / 作品 / 帖子」筛选，作品和帖子各一张卡，匹配的关键词用主题色标出。
 */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun SearchScreen(env: CommunityEnv, back: () -> Unit, open: (Route) -> Unit) {
    val scope = rememberCoroutineScope()
    var query by rememberSaveable { mutableStateOf("") }
    var shown by rememberSaveable { mutableStateOf("") }
    var history by remember { mutableStateOf(readHistory(env)) }
    var packs by remember { mutableStateOf<List<Pack>>(emptyList()) }
    var topics by remember { mutableStateOf<List<Topic>>(emptyList()) }
    var busy by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    var searched by remember { mutableStateOf(false) }
    var tabName by rememberSaveable { mutableStateOf(ResultTab.All.name) }
    val hot = rememberLoader(env, "hot-search") { env.forum.hotSearch() }
    val hotState by hot.state.collectAsState()

    fun search(text: String) {
        val q = text.trim()
        if (q.isEmpty() || busy) return
        query = q; shown = q
        history = pushHistory(env, q)
        busy = true; error = null; searched = true; tabName = ResultTab.All.name
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

    Column(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.background)) {
        // 顶部：白底，浅灰圆角输入框和「取消」。
        Row(
            Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface).statusBarsPadding().padding(start = 16.dp, end = 8.dp, top = 8.dp, bottom = 8.dp),
            verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Row(
                Modifier.weight(1f).height(40.dp).clip(RoundedCornerShape(20.dp)).background(LocalExtraColors.current.fill).padding(horizontal = 14.dp),
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
                if (query.isNotEmpty()) XIcon(XIcons.Close, tint = LocalExtraColors.current.ink3, size = 18.dp, description = "清空输入", modifier = Modifier.clip(RoundedCornerShape(9.dp)).clickable { query = ""; searched = false })
                XIcon(XIcons.Search, size = 20.dp, description = "搜索", modifier = Modifier.clickable { search(query) })
            }
            Text("取消", Modifier.clickable(onClick = back).padding(horizontal = 8.dp, vertical = 12.dp), fontSize = 15.sp)
        }
        when {
            busy -> Spinner()
            error != null -> CenterMessage(error.orEmpty(), "重试", onAction = { search(query) })
            !searched -> Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(start = 12.dp, end = 12.dp, top = 8.dp, bottom = 24.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                val words = (hotState as? Load.Ready<HotWords>)?.value?.items.orEmpty()
                if (history.isNotEmpty()) WhiteCard(padding = 14.dp) {
                    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                        XIcon(XIcons.Clock, size = 18.dp)
                        Text("搜索历史", Modifier.weight(1f), fontSize = 15.sp, fontWeight = FontWeight.SemiBold)
                        Text("清空", Modifier.clip(RoundedCornerShape(8.dp)).clickable { env.prefs.put(HistoryKey, ""); history = emptyList() }.padding(horizontal = 6.dp, vertical = 4.dp), fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                    FlowRow(Modifier.padding(top = 12.dp), horizontalArrangement = Arrangement.spacedBy(8.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        history.forEach { h ->
                            Text(h, Modifier.height(32.dp).clip(RoundedCornerShape(16.dp)).background(LocalExtraColors.current.fill).clickable { search(h) }.padding(horizontal = 14.dp, vertical = 6.dp), fontSize = 14.sp, maxLines = 1)
                        }
                    }
                }
                if (words.isNotEmpty()) WhiteCard(padding = 0.dp) {
                    Row(Modifier.padding(start = 14.dp, end = 14.dp, top = 14.dp, bottom = 4.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                        XIcon(XIcons.Bolt, tint = Brand, size = 18.dp)
                        Text("热门搜索", fontSize = 15.sp, fontWeight = FontWeight.SemiBold)
                    }
                    words.forEachIndexed { i, w ->
                        if (i > 0) Hairline(Modifier.padding(start = 50.dp))
                        Row(Modifier.fillMaxWidth().clickable { search(w.text) }.padding(horizontal = 14.dp, vertical = 11.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                            // 前三名是红色方块白字，其余灰字。
                            Box(Modifier.size(22.dp).clip(RoundedCornerShape(6.dp)).background(if (i < 3) Brand else Color.Transparent), contentAlignment = Alignment.Center) {
                                Text("${i + 1}", fontSize = 13.sp, fontWeight = FontWeight.Bold, color = if (i < 3) Color.White else LocalExtraColors.current.ink3)
                            }
                            Text(w.text, Modifier.weight(1f), fontSize = 15.sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
                            when (w.tag) {
                                "" -> {}
                                "新" -> BlueTag(w.tag)
                                else -> Text(w.tag, Modifier.clip(RoundedCornerShape(6.dp)).background(LocalExtraColors.current.brandSoft).padding(horizontal = 8.dp, vertical = 2.dp), fontSize = 12.sp, fontWeight = FontWeight.SemiBold, color = LocalExtraColors.current.brandInk)
                            }
                        }
                    }
                    Box(Modifier.height(6.dp))
                }
                if (history.isEmpty() && words.isEmpty()) EmptyState(XIcons.Search, "搜点什么", "输入关键词，搜索社区里的作品、项目和帖子。", Modifier.padding(top = 64.dp))
            }
            packs.isEmpty() && topics.isEmpty() -> Box(Modifier.fillMaxSize().padding(top = 64.dp), contentAlignment = Alignment.TopCenter) {
                EmptyState(XIcons.Search, "没有找到相关内容", "没有和“$shown”相关的作品或帖子，换个关键词试试。")
            }
            else -> {
                val tab = ResultTab.valueOf(tabName)
                Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(start = 12.dp, end = 12.dp, top = 8.dp, bottom = 24.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    Row(Modifier.padding(horizontal = 2.dp), verticalAlignment = Alignment.CenterVertically) {
                        Text("“$shown”", fontSize = 13.sp, fontWeight = FontWeight.SemiBold, color = Brand)
                        Text(" 共 ${packs.size + topics.size} 个结果", fontSize = 13.sp, color = LocalExtraColors.current.ink3)
                    }
                    PageChipRow(
                        ResultTab.entries.map { it.name to (if (it == ResultTab.All) it.label else "${it.label} ${if (it == ResultTab.Packs) packs.size else topics.size}") },
                        tabName, { tabName = it },
                    )
                    if (tab != ResultTab.Topics && packs.isNotEmpty()) WhiteCard(padding = 0.dp) {
                        ResultHeader("作品", packs.size, XIcons.Sparkle)
                        packs.forEachIndexed { i, pack ->
                            if (i > 0) Hairline(Modifier.padding(start = 14.dp))
                            PackResult(pack, shown) { open(Route.Pack(pack.id)) }
                        }
                    }
                    if (tab != ResultTab.Packs && topics.isNotEmpty()) WhiteCard(padding = 0.dp) {
                        ResultHeader("帖子", topics.size, XIcons.Message)
                        topics.forEachIndexed { i, topic ->
                            if (i > 0) Hairline(Modifier.padding(start = 14.dp))
                            TopicResult(env, topic, shown) { open(Route.Topic(topic.id)) }
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun ResultHeader(title: String, count: Int, icon: ImageVector) {
    Row(Modifier.fillMaxWidth().padding(start = 14.dp, end = 14.dp, top = 14.dp, bottom = 6.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
        XIcon(icon, size = 18.dp)
        Text(title, fontSize = 15.sp, fontWeight = FontWeight.SemiBold)
        Text("$count", fontSize = 12.sp, color = LocalExtraColors.current.ink3)
    }
}

/** 作品结果：左边 64dp 封面，右边标题（关键词高亮）、简介、类型 · 作者 · 应用数。 */
@Composable
private fun PackResult(pack: Pack, query: String, onClick: () -> Unit) {
    Row(Modifier.fillMaxWidth().clickable(onClick = onClick).padding(horizontal = 14.dp, vertical = 12.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
        Cover(pack.coverUrl, pack.id, Modifier.size(64.dp))
        Column(Modifier.weight(1f)) {
            Text(highlight(pack.title, query, Brand), fontSize = 15.sp, fontWeight = FontWeight.Medium, lineHeight = 21.sp, maxLines = 2, overflow = TextOverflow.Ellipsis)
            if (pack.summary.isNotBlank()) Text(highlight(pack.summary, query, Brand), Modifier.padding(top = 2.dp), fontSize = 13.sp, lineHeight = 19.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 2, overflow = TextOverflow.Ellipsis)
            Row(Modifier.padding(top = 6.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                GrayTag(kindLabel(pack.kind))
                Text(pack.author?.nickname ?: "小伴官方", Modifier.weight(1f, fill = false), fontSize = 12.sp, color = LocalExtraColors.current.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
                if (pack.applyCount > 0) Text("${pack.applyCount} 次应用", fontSize = 12.sp, color = LocalExtraColors.current.ink3, maxLines = 1)
            }
        }
    }
}

/** 帖子结果：标题（关键词高亮）、摘要，有配图时右边 64dp 缩略图；底下是板块 · 作者 · 时间 · 回复。 */
@Composable
private fun TopicResult(env: CommunityEnv, topic: Topic, query: String, onClick: () -> Unit) {
    val thumb = topic.images.firstOrNull()?.let { env.forum.imageUrl(it) }
    Row(Modifier.fillMaxWidth().clickable(onClick = onClick).padding(horizontal = 14.dp, vertical = 12.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
        Column(Modifier.weight(1f)) {
            Text(highlight(topic.title, query, Brand), fontSize = 15.sp, fontWeight = FontWeight.Medium, lineHeight = 21.sp, maxLines = 2, overflow = TextOverflow.Ellipsis)
            if (topic.excerpt.isNotBlank()) Text(highlight(topic.excerpt, query, Brand), Modifier.padding(top = 2.dp), fontSize = 13.sp, lineHeight = 19.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 2, overflow = TextOverflow.Ellipsis)
            Row(Modifier.padding(top = 6.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                GrayTag(topic.category.name)
                Text(topic.author.nickname, Modifier.weight(1f, fill = false), fontSize = 12.sp, color = LocalExtraColors.current.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
                Text(agoText(topic.createdAt), fontSize = 12.sp, color = LocalExtraColors.current.ink3, maxLines = 1)
                Stat(XIcons.Message, compactCount(topic.replyCount))
            }
        }
        if (thumb != null) Cover(thumb, topic.id, Modifier.size(64.dp))
    }
}
