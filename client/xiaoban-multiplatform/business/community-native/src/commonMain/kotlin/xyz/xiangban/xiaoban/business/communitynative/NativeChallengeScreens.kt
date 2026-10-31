package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.*

@Composable
private fun ChallengePager(page: Int, more: Boolean, select: (Int) -> Unit) {
    Row(Modifier.fillMaxWidth().padding(vertical = 10.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        GhostPillButton("上一页", { select(page - 1) }, Modifier.weight(1f), enabled = page > 1)
        Text("第 $page 页", Modifier.padding(top = 10.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
        GhostPillButton("下一页", { select(page + 1) }, Modifier.weight(1f), enabled = more)
    }
}

@Composable
fun NativeChallengesScreen(env: CommunityEnv, back: () -> Unit, open: (Route) -> Unit) {
    var page by rememberSaveable { mutableStateOf(1) }
    val loader = rememberLoader(env, "challenges-$page") { env.challenges.list(page) }
    val state by loader.state.collectAsState()
    Column(Modifier.fillMaxSize()) {
        ScreenBar("官方挑战", onBack = back)
        Box(Modifier.weight(1f)) { LoadBox(state, loader::reload) { result ->
            if (result.items.isEmpty()) CenterMessage("这里还没有已发布的官方挑战", if (page > 1) "上一页" else "刷新", { if (page > 1) page-- else loader.reload() })
            else LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                item { GhostPillButton("刷新挑战", loader::reload, Modifier.fillMaxWidth()) }
                items(result.items, key = { it.key }) { challenge -> WhiteCard(Modifier.clickable { open(Route.Challenge(challenge.key)) }, padding = 14.dp) {
                    Text(challenge.title, fontSize = 17.sp, fontWeight = FontWeight.SemiBold)
                    Text("${challengePhase(challenge)} · ${challenge.entryCount} 件投稿", Modifier.padding(top = 6.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                    if (challenge.description.isNotBlank()) Text(challenge.description, Modifier.padding(top = 8.dp), fontSize = 14.sp, lineHeight = 22.sp, maxLines = 3)
                    Text("截止 ${challenge.endsAt}", Modifier.padding(top = 8.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                } }
                item { ChallengePager(page, result.hasMore) { page = it } }
            }
        } }
    }
}

@Composable
fun NativeChallengeDetailScreen(env: CommunityEnv, key: String, back: () -> Unit, open: (Route) -> Unit) {
    val session by env.session.state.collectAsState()
    var page by rememberSaveable(key) { mutableStateOf(1) }
    val loader = rememberLoader(env, "challenge-$key-$page") { env.challenges.detail(key, page) }
    val state by loader.state.collectAsState()
    var choose by remember { mutableStateOf(false) }
    var notice by remember { mutableStateOf<String?>(null) }
    var submitted by remember(key, session.accountId) { mutableStateOf<Set<Long>>(emptySet()) }
    Column(Modifier.fillMaxSize()) {
        ScreenBar("官方挑战", onBack = back)
        Box(Modifier.weight(1f)) { LoadBox(state, loader::reload) { detail ->
            LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                item { WhiteCard(padding = 16.dp) {
                    Text(detail.challenge.title, fontSize = 22.sp, fontWeight = FontWeight.SemiBold)
                    Text("${challengePhase(detail.challenge)} · ${detail.challenge.entryCount} 件投稿", Modifier.padding(top = 6.dp), fontSize = 13.sp, color = LocalExtraColors.current.ink3)
                    Text("开始 ${detail.challenge.startsAt}\n截止 ${detail.challenge.endsAt}", Modifier.padding(top = 10.dp), fontSize = 12.sp, lineHeight = 20.sp, color = LocalExtraColors.current.ink3)
                    Text(detail.challenge.description, Modifier.padding(top = 12.dp), fontSize = 14.sp, lineHeight = 23.sp)
                    Text("投稿标签 #${detail.challenge.submissionTag}", Modifier.padding(top = 8.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                    Row(Modifier.padding(top = 12.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        GhostPillButton("刷新状态", loader::reload, Modifier.weight(1f))
                        PillButton(if (!challengeCanSubmit(detail.challenge)) challengePhase(detail.challenge) else if (!session.loggedIn) "登录后投稿" else "选择我的作品投稿", {
                            if (!session.loggedIn) open(Route.Login) else choose = true
                        }, Modifier.weight(1f), enabled = challengeCanSubmit(detail.challenge))
                    }
                    if (!challengeCanSubmit(detail.challenge)) Text("当前不接受新投稿，以挑战最新状态为准。", Modifier.padding(top = 8.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                    detail.challenge.topicId?.let { id -> GhostPillButton("查看挑战讨论", { open(Route.Topic(id)) }, Modifier.padding(top = 8.dp)) }
                } }
                notice?.let { message -> item { Banner(message) } }
                item { Text("投稿作品", Modifier.padding(4.dp), fontSize = 16.sp, fontWeight = FontWeight.SemiBold) }
                if (detail.entries.items.isEmpty()) item { CenterMessage("当前没有可见投稿", modifier = Modifier.fillMaxWidth().height(120.dp)) }
                items(detail.entries.items, key = { it.packId }) { entry -> WhiteCard(padding = 14.dp) {
                    if (entry.featured) Text("官方精选", fontSize = 12.sp, color = LocalExtraColors.current.brandInk)
                    Text(entry.pack.title, Modifier.fillMaxWidth().padding(top = 4.dp).clickable { open(if (entry.pack.kind in CapabilityPackKinds) Route.Ability(entry.packId) else Route.Pack(entry.packId)) }, fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                    Text("${kindLabel(entry.pack.kind)} · ${entry.pack.author?.nickname ?: "作者"} · v${entry.pack.version}", Modifier.padding(top = 6.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                    if (entry.pack.summary.isNotBlank()) Text(entry.pack.summary, Modifier.padding(top = 8.dp), fontSize = 14.sp, lineHeight = 22.sp)
                    ChallengeEntryLikes(env, entry, open)
                } }
                item { ChallengePager(page, detail.entries.hasMore) { page = it } }
                item { FootNote("投稿仅限已上架、公开可读的本人作品。作品下架或权限改变后，不再出现在公开投稿列表。作品赞数来自它的公开讨论，不能用于换算报名人数。") }
            }
            if (choose) ChallengeSubmissionDialog(env, detail.challenge, submitted + detail.entries.items.map { it.packId },
                onDismiss = { choose = false }, onRefresh = { choose = false; loader.reload() }, onSubmitted = { id ->
                    choose = false; submitted = submitted + id; notice = "作品已提交。同一作品重复操作不会增加投稿记录。"; loader.reload()
                })
        } }
    }
}

@Composable
private fun ChallengeSubmissionDialog(env: CommunityEnv, challenge: CommunityChallenge, submitted: Set<Long>, onDismiss: () -> Unit, onRefresh: () -> Unit, onSubmitted: (Long) -> Unit) {
    val session by env.session.state.collectAsState()
    val loader = rememberLoader(env, "challenge-my-packs-${session.accountId}") { env.content.myPacks() }
    val state by loader.state.collectAsState()
    val owned = (state as? Load.Ready)?.value?.items?.let { ownedChallengePacks(it, session.accountId) }.orEmpty()
    var selected by rememberSaveable(challenge.key, session.accountId) { mutableStateOf<Long?>(null) }
    var busy by remember { mutableStateOf(false) }
    var stale by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    val scope = rememberCoroutineScope()
    AlertDialog(onDismissRequest = { if (!busy) onDismiss() }, title = { Text("选择已上架的本人作品") },
        text = { Column(Modifier.heightIn(max = 420.dp)) {
            Text("每件作品只记录一次投稿；受限阅读、已下架或非本人作品不能投稿。", fontSize = 12.sp, lineHeight = 18.sp, color = LocalExtraColors.current.ink3)
            when (val result = state) {
                Load.Loading -> Spinner(Modifier.fillMaxWidth().height(100.dp))
                is Load.Failed -> Banner(result.message, warn = true, actionText = "重试", onAction = loader::reload)
                is Load.Ready -> if (owned.isEmpty()) Text("没有已上架的本人作品，请先从“我的作品”发布。", Modifier.padding(vertical = 16.dp), fontSize = 14.sp)
                else LazyColumn(Modifier.weight(1f, fill = false).padding(top = 8.dp)) {
                    items(owned, key = { it.id }) { pack ->
                        RadioRow(selected == pack.id, pack.title, desc = if (pack.id in submitted) "已提交当前挑战" else "${kindLabel(pack.kind)} · v${pack.version}", onClick = {
                            if (!busy && pack.id !in submitted) { selected = pack.id; error = null }
                        })
                    }
                }
            }
            error?.let { Text(it, Modifier.padding(top = 8.dp), color = Danger, fontSize = 13.sp) }
        } },
        confirmButton = { PillButton(if (busy) "投稿中…" else "提交作品", {
            if (busy) return@PillButton
            val pack = owned.firstOrNull { it.id == selected } ?: return@PillButton
            busy = true; error = null
            scope.launch {
                try { env.challenges.submit(challenge.key, pack.id); onSubmitted(pack.id) }
                catch (e: CancellationException) { throw e }
                catch (e: Exception) { error = messageOf(e); if ((e as? ApiException)?.status == 409) stale = true }
                finally { busy = false }
            }
        }, enabled = !busy && !stale && session.loggedIn && challengeCanSubmit(challenge) && selected != null && selected !in submitted && owned.any { it.id == selected }) },
        dismissButton = { GhostPillButton(if (stale) "关闭并刷新状态" else "取消", { if (stale) onRefresh() else onDismiss() }, enabled = !busy) })
}

/** 点赞仅走作品真实帖子；每次操作后重新读计数和当前用户状态。 */
@Composable
private fun ChallengeEntryLikes(env: CommunityEnv, entry: CommunityChallengeEntry, open: (Route) -> Unit) {
    val topicId = challengeVoteTopic(entry)
    if (topicId == null) { Text("此作品暂无可点赞的公开讨论", Modifier.padding(top = 10.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3); return }
    val session by env.session.state.collectAsState()
    val scope = rememberCoroutineScope()
    var topic by remember(topicId, session.accountId) { mutableStateOf<Topic?>(null) }
    var busy by remember(topicId, session.accountId) { mutableStateOf(false) }
    var error by remember(topicId, session.accountId) { mutableStateOf<String?>(null) }
    var retry by remember { mutableStateOf(0) }
    LaunchedEffect(topicId, session.accountId, retry) {
        if (session.loggedIn) {
            busy = true; error = null
            try { topic = env.forum.topic(topicId) }
            catch (e: CancellationException) { throw e } catch (e: Exception) { error = messageOf(e) }
            finally { busy = false }
        }
    }
    Row(Modifier.padding(top = 10.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        Text((topic?.likeCount?.toLong() ?: entry.likes)?.let { "$it 赞" } ?: "暂无点赞数据", Modifier.weight(1f).padding(top = 10.dp), fontSize = 13.sp, color = LocalExtraColors.current.ink3)
        GhostPillButton(when { busy -> "处理中…"; !session.loggedIn -> "登录后点赞"; topic == null -> "重试读取"; topic?.viewer?.liked == true -> "取消点赞"; else -> "点赞" }, {
            if (!session.loggedIn) { open(Route.Login); return@GhostPillButton }
            if (busy) return@GhostPillButton
            val current = topic
            if (current == null) { retry++; return@GhostPillButton }
            busy = true; error = null
            scope.launch {
                try { topic = env.forum.setChallengeEntryLiked(entry, current.viewer?.liked != true) }
                catch (e: CancellationException) { throw e } catch (e: Exception) { error = messageOf(e) }
                finally { busy = false }
            }
        }, enabled = !busy)
    }
    error?.let { Text(it, Modifier.padding(top = 4.dp), color = Danger, fontSize = 12.sp) }
}
