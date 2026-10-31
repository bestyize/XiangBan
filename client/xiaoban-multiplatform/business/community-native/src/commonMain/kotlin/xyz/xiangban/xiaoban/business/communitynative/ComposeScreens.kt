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
import androidx.compose.material3.Switch
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
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.backhandler.BackHandler
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.launch
import kotlinx.serialization.encodeToString
import xyz.xiangban.xiaoban.common.community.CommunityJson
import xyz.xiangban.xiaoban.common.community.ForumAttachment
import xyz.xiangban.xiaoban.common.community.Category
import xyz.xiangban.xiaoban.common.community.messageOf
import xyz.xiangban.xiaoban.common.community.parsePollOptions
import xyz.xiangban.xiaoban.common.community.pollDraftError

/**
 * 发帖：选一个具体的分类，写标题和正文，发布。发成功后直接打开这个帖子。
 * 图片上传成功后回填站内 Markdown；附件通过既有 attachment_ids 挂到帖子上。
 */
@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun NewPostScreen(env: CommunityEnv, back: () -> Unit, open: (Route) -> Unit, help: Boolean = false) {
    val scope = rememberCoroutineScope()
    val session by env.session.state.collectAsState()
    var categories by remember { mutableStateOf<List<Category>?>(null) }
    var loadError by remember { mutableStateOf<String?>(null) }
    var parentId by rememberSaveable { mutableStateOf(-1) }
    var categoryId by rememberSaveable { mutableStateOf(-1) }
    var title by rememberSaveable { mutableStateOf("") }
    var body by rememberSaveable { mutableStateOf("") }
    var busy by remember { mutableStateOf(false) }
    var uploading by remember { mutableStateOf(false) }
    var leaving by remember { mutableStateOf(false) }
    var attachmentsJson by rememberSaveable { mutableStateOf("[]") }
    val attachments = remember(attachmentsJson) { CommunityJson.decodeFromString<List<ForumAttachment>>(attachmentsJson) }
    fun cancelDraft() {
        if (leaving) return
        leaving = true
        // 只撤销本次草稿上传的 pending ID，服务端再次保证关联到帖子后不可删除。
        val pending = attachments.toList()
        env.scope.launch { pending.forEach { attachment -> runCatching { env.forum.deletePendingAttachment(attachment.id) } } }
        back()
    }
    BackHandler { if (!busy) cancelDraft() }
    var error by remember { mutableStateOf<String?>(null) }
    var attempt by remember { mutableStateOf(0) }
    var bounty by rememberSaveable { mutableStateOf(0) }
    var bountyDays by rememberSaveable { mutableStateOf(7) }
    var pollEnabled by rememberSaveable { mutableStateOf(false) }
    var pollText by rememberSaveable { mutableStateOf("") }
    var pollDays by rememberSaveable { mutableStateOf(7) }

    LaunchedEffect(attempt) {
        loadError = null
        try {
            val all = env.forum.categories()
            categories = all
            val helpBoard = all.firstOrNull { it.slug == "share-help" }
            if (help && helpBoard != null && categoryId < 0) {
                parentId = helpBoard.parentId ?: parentId
                categoryId = helpBoard.id
            } else if (parentId < 0) {
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
    val isHelp = chosen?.slug == "share-help"
    val pollOptions = parsePollOptions(pollText)
    val pollError = if (pollEnabled) pollDraftError(pollOptions, pollDays.takeIf { it != 0 }) else null
    val canPost = session.loggedIn && !busy && !uploading && !leaving && chosen != null && title.trim().length >= 2 && body.trim().length >= 10 && pollError == null

    Column(Modifier.fillMaxSize().imePadding()) {
        ScreenBar(if (help) "发求助" else "发帖", onBack = { if (!busy) cancelDraft() }, trailing = null)
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
                    categories == null -> Text("读取分类中…", Modifier.padding(16.dp), fontSize = 14.sp, color = LocalExtraColors.current.ink3)
                    else -> {
                        ChipRow(parents.map { it.id to it.name }, parentId) { parentId = it; categoryId = -1 }
                        ChipRow(children.map { it.id to it.name }, categoryId) { categoryId = it }
                        chosen?.description?.takeIf { it.isNotBlank() }?.let { Text(it, Modifier.padding(horizontal = 16.dp, vertical = 4.dp), fontSize = 12.sp, lineHeight = 18.sp, color = LocalExtraColors.current.ink3) }
                    }
                }
                TextBox(title, { title = it }, Modifier.padding(start = 16.dp, end = 16.dp, top = 12.dp), placeholder = "标题（至少 2 个字）", enabled = !busy)
                AreaBox(body, { body = it }, Modifier.padding(start = 16.dp, end = 16.dp, top = 12.dp), placeholder = "正文（至少 10 个字）：写清楚用在哪里、大概什么样子、有什么限制。", minHeight = 200.dp, enabled = !busy)
                Box(Modifier.padding(horizontal = 16.dp, vertical = 8.dp)) {
                    ForumUploadControls(env, enabled = !busy && !leaving, onImage = { if (!leaving) body += it }, onBusy = { uploading = it },
                        attachments = attachments, onAttachments = { next ->
                            if (!leaving) attachmentsJson = CommunityJson.encodeToString(next)
                            else env.scope.launch { next.filterNot { file -> attachments.any { it.id == file.id } }.forEach { file ->
                                runCatching { env.forum.deletePendingAttachment(file.id) }
                            } }
                        })
                }
                Row(Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 8.dp), verticalAlignment = Alignment.CenterVertically) {
                    Text("发起投票", Modifier.weight(1f), fontSize = 14.sp)
                    Switch(checked = pollEnabled, onCheckedChange = { pollEnabled = it }, enabled = !busy)
                }
                if (pollEnabled) {
                    AreaBox(pollText, { pollText = it }, Modifier.padding(horizontal = 16.dp), placeholder = "每行一个选项，共 2～6 项，每项最多 40 字", minHeight = 100.dp, enabled = !busy)
                    Text("投票时长", Modifier.padding(start = 16.dp, top = 12.dp), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    ChipRow(listOf(0, 1, 3, 7, 14, 30).map { it to if (it == 0) "一直开放" else "$it 天" }, pollDays) { if (!busy) pollDays = it }
                    Text(pollError ?: "投票发布后不能修改选项。", Modifier.padding(horizontal = 16.dp, vertical = 8.dp), fontSize = 12.sp, color = if (pollError != null) Danger else LocalExtraColors.current.ink3)
                }
                if (isHelp) {
                    Text("悬赏积分", Modifier.padding(start = 16.dp, top = 16.dp), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    ChipRow(listOf(0, 10, 30, 50, 100).map { it to if (it == 0) "不悬赏" else "$it 分" }, bounty) { bounty = it }
                    Text("有效期", Modifier.padding(start = 16.dp, top = 12.dp), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    ChipRow(listOf(7, 14, 30).map { it to "$it 天" }, bountyDays) { bountyDays = it }
                    Text(
                        "悬赏积分发布时先扣下，采纳回答后发给回答者；到期没有采纳或求助关闭，会退还给你。",
                        Modifier.padding(start = 16.dp, end = 16.dp, top = 8.dp), fontSize = 12.sp, lineHeight = 18.sp, color = LocalExtraColors.current.ink3,
                    )
                }
                if (chosen != null && chosen.slug.startsWith("share")) {
                    Text(
                        "资源分享须知：仅限合法来源——正版、开源、免费软件，片单影评乐评歌单，公版或授权影音，公开课程资料，自己的作品。不发破解版和盗版下载，违规内容会被下架。",
                        Modifier.padding(start = 16.dp, end = 16.dp, top = 12.dp), fontSize = 12.sp, lineHeight = 19.sp, color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
                error?.let { Text(it, Modifier.padding(start = 16.dp, end = 16.dp, top = 12.dp), fontSize = 14.sp, color = Danger) }
            }
            Hairline()
            Box(Modifier.fillMaxWidth().height(72.dp).background(MaterialTheme.colorScheme.surface).padding(horizontal = 12.dp), contentAlignment = Alignment.Center) {
                PillButton(
                    if (busy) "发布中…" else if (uploading) "上传中…" else "发布",
                    {
                        if (busy || uploading || leaving) return@PillButton
                        busy = true; error = null
                        scope.launch {
                            try {
                                val id = env.forum.createTopic(categoryId, title.trim(), body.trim(), bounty = if (isHelp) bounty else null, bountyDays = bountyDays,
                                    pollOptions = if (pollEnabled) pollOptions else emptyList(), pollDays = pollDays.takeIf { pollEnabled && it != 0 }, attachments = attachments)
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
                Modifier.height(34.dp).clip(CircleShape).background(if (on) LocalExtraColors.current.brandSoft else MaterialTheme.colorScheme.surface)
                    .clickable { onSelect(id) }.padding(horizontal = 14.dp, vertical = 7.dp),
                fontSize = 14.sp, color = if (on) LocalExtraColors.current.brandInk else MaterialTheme.colorScheme.onSurface, fontWeight = if (on) FontWeight.SemiBold else FontWeight.Normal,
            )
        }
    }
}
