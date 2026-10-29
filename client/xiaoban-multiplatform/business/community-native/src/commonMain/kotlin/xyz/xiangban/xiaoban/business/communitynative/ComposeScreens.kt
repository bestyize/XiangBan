package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
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
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.Category
import xyz.xiangban.xiaoban.common.community.messageOf

/**
 * 发帖：选一个具体的分类，写标题和正文，发布。发成功后直接打开这个帖子。
 * 图片、附件、悬赏求助这些还在网页端发帖时才有。
 */
@Composable
fun NewPostScreen(env: CommunityEnv, back: () -> Unit, open: (Route) -> Unit) {
    val scope = rememberCoroutineScope()
    val session by env.session.state.collectAsState()
    var categories by remember { mutableStateOf<List<Category>?>(null) }
    var loadError by remember { mutableStateOf<String?>(null) }
    var parentId by rememberSaveable { mutableStateOf(-1) }
    var categoryId by rememberSaveable { mutableStateOf(-1) }
    var title by rememberSaveable { mutableStateOf("") }
    var body by rememberSaveable { mutableStateOf("") }
    var busy by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    var attempt by remember { mutableStateOf(0) }

    LaunchedEffect(attempt) {
        loadError = null
        try {
            val all = env.forum.categories()
            categories = all
            if (parentId < 0) {
                val firstParent = all.firstOrNull { it.parentId == null && !it.adminOnly && all.any { c -> c.parentId == it.id } }
                if (firstParent != null) parentId = firstParent.id
            }
        } catch (e: kotlinx.coroutines.CancellationException) {
            throw e
        } catch (e: Exception) {
            loadError = messageOf(e)
        }
    }

    val all = categories.orEmpty()
    val parents = all.filter { it.parentId == null && !it.adminOnly && all.any { c -> c.parentId == it.id } }
    val children = all.filter { it.parentId == parentId && !it.adminOnly }
    val chosen = all.firstOrNull { it.id == categoryId }
    val canPost = session.loggedIn && !busy && chosen != null && title.trim().length >= 2 && body.trim().length >= 10

    Column(Modifier.fillMaxSize().imePadding()) {
        ScreenBar("发帖", onBack = back, trailing = null)
        if (!session.loggedIn) {
            Box(Modifier.weight(1f).fillMaxWidth(), contentAlignment = Alignment.Center) {
                Column(Modifier.padding(horizontal = 24.dp), horizontalAlignment = Alignment.CenterHorizontally) {
                    EmptyState(XIcons.Pen, "登录后才能发帖", "用小伴账号登录，在讨论或资源里发一个帖子。")
                    PillButton("登录小伴账号", { open(Route.Login) }, Modifier.padding(top = 20.dp).fillMaxWidth())
                }
            }
        } else {
            Column(Modifier.weight(1f).fillMaxWidth().verticalScroll(rememberScrollState()).background(MaterialTheme.colorScheme.surface).padding(top = 12.dp, bottom = 16.dp)) {
                Text("发到哪里", Modifier.padding(horizontal = 16.dp), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                when {
                    loadError != null -> Banner(loadError.orEmpty(), Modifier.padding(horizontal = 16.dp, vertical = 8.dp), warn = true, actionText = "重试", onAction = { attempt += 1 })
                    categories == null -> Text("读取分类中…", Modifier.padding(16.dp), fontSize = 13.sp, color = LocalExtraColors.current.ink3)
                    else -> {
                        ChipRow(parents.map { it.id to it.name }, parentId) { parentId = it; categoryId = -1 }
                        ChipRow(children.map { it.id to it.name }, categoryId) { categoryId = it }
                        chosen?.description?.takeIf { it.isNotBlank() }?.let { Text(it, Modifier.padding(horizontal = 16.dp, vertical = 4.dp), fontSize = 12.sp, lineHeight = 18.sp, color = LocalExtraColors.current.ink3) }
                    }
                }
                TextBox(title, { title = it }, Modifier.padding(start = 16.dp, end = 16.dp, top = 12.dp), placeholder = "标题（至少 2 个字）", enabled = !busy)
                AreaBox(body, { body = it }, Modifier.padding(start = 16.dp, end = 16.dp, top = 12.dp), placeholder = "正文（至少 10 个字）：写清楚用在哪里、大概什么样子、有什么限制。", minHeight = 200.dp, enabled = !busy)
                if (chosen != null && chosen.slug.startsWith("share")) {
                    Text(
                        "资源分享须知：仅限合法来源——正版、开源、免费软件，片单影评乐评歌单，公版或授权影音，公开课程资料，自己的作品。不发破解版和盗版下载，违规内容会被下架。",
                        Modifier.padding(start = 16.dp, end = 16.dp, top = 12.dp), fontSize = 12.sp, lineHeight = 19.sp, color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
                error?.let { Text(it, Modifier.padding(start = 16.dp, end = 16.dp, top = 12.dp), fontSize = 13.sp, color = Brand) }
                Text("需要贴图片、传附件或发悬赏求助，请到网页端发帖。", Modifier.padding(start = 16.dp, end = 16.dp, top = 12.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
            }
            Hairline()
            Box(Modifier.fillMaxWidth().height(72.dp).background(MaterialTheme.colorScheme.surface).padding(horizontal = 12.dp), contentAlignment = Alignment.Center) {
                PillButton(
                    if (busy) "发布中…" else "发布",
                    {
                        busy = true; error = null
                        scope.launch {
                            try {
                                val id = env.forum.createTopic(categoryId, title.trim(), body.trim())
                                back()
                                open(Route.Topic(id))
                            } catch (e: kotlinx.coroutines.CancellationException) {
                                throw e
                            } catch (e: Exception) {
                                error = messageOf(e)
                            } finally {
                                busy = false
                            }
                        }
                    },
                    Modifier.fillMaxWidth(), enabled = canPost,
                )
            }
        }
    }
}

@Composable
private fun ChipRow(items: List<Pair<Int, String>>, selected: Int, onSelect: (Int) -> Unit) {
    LazyRow(Modifier.fillMaxWidth().padding(top = 8.dp), contentPadding = PaddingValues(horizontal = 16.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        items(items, key = { it.first }) { (id, name) ->
            val on = id == selected
            Text(
                name,
                Modifier.height(32.dp).clip(CircleShape).background(if (on) MaterialTheme.colorScheme.onSurface else MaterialTheme.colorScheme.surface)
                    .border(1.dp, if (on) MaterialTheme.colorScheme.onSurface else LocalExtraColors.current.line, CircleShape).clickable { onSelect(id) }.padding(horizontal = 14.dp, vertical = 6.dp),
                fontSize = 13.sp, color = if (on) MaterialTheme.colorScheme.surface else MaterialTheme.colorScheme.onSurface, fontWeight = if (on) FontWeight.SemiBold else FontWeight.Normal,
            )
        }
    }
}
