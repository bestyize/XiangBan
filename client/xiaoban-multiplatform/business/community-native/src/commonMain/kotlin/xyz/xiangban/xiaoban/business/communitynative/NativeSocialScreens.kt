package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.clickable
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
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
private fun SocialPager(page: Int, hasMore: Boolean, select: (Int) -> Unit) {
    Row(Modifier.fillMaxWidth().padding(vertical = 10.dp), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
        GhostPillButton("上一页", { select(page - 1) }, Modifier.weight(1f), enabled = page > 1)
        Text("第 $page 页", Modifier.padding(top = 10.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
        GhostPillButton("下一页", { select(page + 1) }, Modifier.weight(1f), enabled = hasMore)
    }
}

@Composable
fun NativeNotificationsScreen(env: CommunityEnv, back: () -> Unit, open: (Route) -> Unit) {
    val session by env.session.state.collectAsState()
    var unread by rememberSaveable { mutableStateOf(false) }
    var page by rememberSaveable { mutableStateOf(1) }
    val scope = rememberCoroutineScope()
    var busy by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    var retryReadIds by remember { mutableStateOf<List<Long>?>(emptyList()) }
    var retryReadReload by remember { mutableStateOf(false) }
    Column(Modifier.fillMaxSize()) {
        ScreenBar("通知", onBack = back)
        if (!session.loggedIn) { CenterMessage("登录后查看通知", "登录", { open(Route.Login) }); return@Column }
        TabStrip(listOf("all" to "全部", "unread" to "未读"), if (unread) "unread" else "all") { unread = it == "unread"; page = 1 }
        val loader = rememberLoader(env, "notifications-$page-$unread-${session.accountId}") { env.social.notifications(page, unread) }
        val state by loader.state.collectAsState()
        val entries = (state as? Load.Ready)?.value?.items.orEmpty()
        val unreadIds = entries.filterNot { it.read }.map { it.id }
        fun mark(ids: List<Long>?, reload: Boolean) {
            if (busy) return
            busy = true; error = null; retryReadIds = ids; retryReadReload = reload
            scope.launch {
                try { env.social.markRead(ids); if (reload) loader.reload() }
                catch (e: CancellationException) { throw e } catch (e: Exception) { error = messageOf(e) }
                finally { busy = false }
            }
        }
        LaunchedEffect(loader, unreadIds) { if (unreadIds.isNotEmpty()) mark(unreadIds, false) }
        Row(Modifier.padding(horizontal = 12.dp, vertical = 6.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            GhostPillButton("刷新", loader::reload, Modifier.weight(1f))
            GhostPillButton("全部已读", { mark(null, true) }, Modifier.weight(1f), enabled = !busy)
        }
        error?.let { Banner(it, Modifier.padding(horizontal = 12.dp), warn = true, actionText = "重试已读", onAction = { mark(retryReadIds, retryReadReload) }) }
        Box(Modifier.weight(1f)) {
            LoadBox(state, loader::reload) { result ->
                if (result.items.isEmpty()) CenterMessage(if (page > 1) "这一页没有通知" else if (unread) "没有未读通知" else "还没有通知",
                    if (page > 1) "上一页" else "刷新", { if (page > 1) page-- else loader.reload() })
                else LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    items(result.items, key = { it.id }) { note -> WhiteCard(Modifier.clickable(enabled = note.packId != null || note.topicId != null) {
                        if (!note.read) mark(listOf(note.id), false)
                        note.packId?.let { open(Route.Pack(it)) } ?: note.topicId?.let { open(Route.Topic(it)) }
                    }, padding = 14.dp) {
                        Text((if (!note.read) "● " else "") + notificationHeadline(note), fontSize = 14.sp, lineHeight = 22.sp, fontWeight = if (note.read) FontWeight.Normal else FontWeight.SemiBold)
                        if (note.excerpt.isNotBlank()) Text(note.excerpt, Modifier.padding(top = 6.dp), fontSize = 13.sp, color = LocalExtraColors.current.ink3)
                        Text(agoText(note.createdAt), Modifier.padding(top = 6.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                    } }
                    item { SocialPager(page, result.hasMore) { page = it } }
                }
            }
        }
    }
}

@Composable
fun NativeBookmarksScreen(env: CommunityEnv, back: () -> Unit, open: (Route) -> Unit) {
    val session by env.session.state.collectAsState()
    var page by rememberSaveable { mutableStateOf(1) }
    val scope = rememberCoroutineScope()
    var busy by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    Column(Modifier.fillMaxSize()) {
        ScreenBar("帖子收藏", onBack = back)
        if (!session.loggedIn) { CenterMessage("登录后查看收藏", "登录", { open(Route.Login) }); return@Column }
        val loader = rememberLoader(env, "bookmarks-$page-${session.accountId}") { env.social.bookmarks(page) }
        val state by loader.state.collectAsState()
        error?.let { Banner(it, Modifier.padding(12.dp), warn = true) }
        Box(Modifier.weight(1f)) { LoadBox(state, loader::reload) { result ->
            if (result.items.isEmpty()) CenterMessage(if (page > 1) "这一页没有收藏" else "还没有收藏帖子",
                if (page > 1) "上一页" else "刷新", { if (page > 1) page-- else loader.reload() })
            else LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                item { GhostPillButton("刷新收藏", loader::reload, Modifier.fillMaxWidth()) }
                items(result.items, key = { it.id }) { topic -> WhiteCard(padding = 14.dp) {
                    Text(topic.title, Modifier.fillMaxWidth().clickable { open(Route.Topic(topic.id)) }, fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                    if (topic.excerpt.isNotBlank()) Text(topic.excerpt, Modifier.padding(top = 6.dp), fontSize = 13.sp, color = LocalExtraColors.current.ink3)
                    GhostPillButton("取消收藏", {
                        if (busy) return@GhostPillButton
                        busy = true; error = null
                        scope.launch { try { env.forum.setTopicBookmarked(topic.id, false); loader.reload() }
                            catch (e: CancellationException) { throw e } catch (e: Exception) { error = messageOf(e) }
                            finally { busy = false } }
                    }, Modifier.padding(top = 8.dp), enabled = !busy)
                } }
                item { SocialPager(page, result.hasMore) { page = it } }
            }
        } }
    }
}

@Composable
fun NativeProfileScreen(env: CommunityEnv, uid: String, initialTab: String, back: () -> Unit, open: (Route) -> Unit) {
    val session by env.session.state.collectAsState()
    val loader = rememberLoader(env, "profile-$uid-${session.accountId}") { env.social.profile(uid) }
    val state by loader.state.collectAsState()
    var tab by rememberSaveable(uid) { mutableStateOf(initialTab.takeIf { it in listOf("topics", "following", "followers") } ?: "topics") }
    var blockConfirmation by remember { mutableStateOf(false) }
    var busy by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    val scope = rememberCoroutineScope()
    fun action(block: suspend () -> Unit) {
        if (!session.loggedIn) { open(Route.Login); return }
        if (busy) return
        busy = true; error = null
        scope.launch { try { block(); loader.reload() }
            catch (e: CancellationException) { throw e } catch (e: Exception) { error = messageOf(e) }
            finally { busy = false } }
    }
    Column(Modifier.fillMaxSize()) {
        ScreenBar("社区用户", onBack = back)
        LoadBox(state, loader::reload) { profile ->
            WhiteCard(Modifier.padding(12.dp), padding = 14.dp) {
                Text(profile.nickname, fontSize = 20.sp, fontWeight = FontWeight.SemiBold)
                Text("${profile.topicCount} 帖子 · ${profile.postCount} 回复 · ${profile.followingCount} 关注 · ${profile.followerCount} 粉丝", Modifier.padding(top = 6.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                if (profile.banned) Text("该用户已被封禁", Modifier.padding(top = 6.dp), color = Danger, fontSize = 13.sp)
                if (profile.uid != session.accountId) Row(Modifier.padding(top = 10.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    GhostPillButton(if (profile.following) "取消关注" else "关注", { action { env.social.follow(uid, !profile.following) } }, Modifier.weight(1f), enabled = !busy && (!profile.banned || profile.following) && !profile.blocked)
                    GhostPillButton(if (profile.blocked) "取消屏蔽" else "屏蔽", {
                        if (!session.loggedIn) open(Route.Login) else if (profile.blocked) action { env.social.block(uid, false) } else blockConfirmation = true
                    }, Modifier.weight(1f), enabled = !busy)
                }
                error?.let { Text(it, Modifier.padding(top = 8.dp), color = Danger, fontSize = 13.sp) }
            }
            TabStrip(listOf("topics" to "帖子", "following" to "关注", "followers" to "粉丝"), tab) { tab = it }
            Box(Modifier.weight(1f)) {
                if (tab == "topics" && profile.blocked) CenterMessage("你已屏蔽该用户")
                else if (tab == "topics") ProfileTopics(env, uid, open) else ProfileConnections(env, uid, tab == "followers", open)
            }
        }
    }
    if (blockConfirmation) AlertDialog(onDismissRequest = { if (!busy) blockConfirmation = false },
        title = { Text("屏蔽该用户？") },
        text = { Text("对方的帖子将从你的列表和搜索隐藏，回复会折叠，双方互相取消关注。对方不会收到屏蔽提示。") },
        confirmButton = { PillButton("确认屏蔽", { blockConfirmation = false; action { env.social.block(uid, true) } }, enabled = !busy) },
        dismissButton = { GhostPillButton("取消", { blockConfirmation = false }, enabled = !busy) })
}

@Composable
private fun ProfileTopics(env: CommunityEnv, uid: String, open: (Route) -> Unit) {
    val session by env.session.state.collectAsState()
    var page by rememberSaveable(uid) { mutableStateOf(1) }
    val loader = rememberLoader(env, "profile-topics-$uid-$page-${session.accountId}") { env.social.userTopics(uid, page) }
    val state by loader.state.collectAsState()
    LoadBox(state, loader::reload) { result ->
        if (result.items.isEmpty()) CenterMessage("暂无可见帖子", if (page > 1) "上一页" else "刷新", { if (page > 1) page-- else loader.reload() })
        else LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            items(result.items, key = { it.id }) { topic -> WhiteCard(Modifier.clickable { open(Route.Topic(topic.id)) }, padding = 14.dp) {
                Text(topic.title, fontSize = 15.sp, fontWeight = FontWeight.Medium)
                Text(topic.excerpt, Modifier.padding(top = 6.dp), fontSize = 13.sp, color = LocalExtraColors.current.ink3)
            } }
            item { SocialPager(page, result.hasMore) { page = it } }
        }
    }
}

@Composable
private fun ProfileConnections(env: CommunityEnv, uid: String, followers: Boolean, open: (Route) -> Unit) {
    var page by rememberSaveable(uid, followers) { mutableStateOf(1) }
    val loader = rememberLoader(env, "connections-$uid-$followers-$page") { env.social.connections(uid, followers, page) }
    val state by loader.state.collectAsState()
    LoadBox(state, loader::reload) { result ->
        if (result.items.isEmpty()) CenterMessage(if (followers) "还没有粉丝" else "还没有关注用户",
            if (page > 1) "上一页" else "刷新", { if (page > 1) page-- else loader.reload() })
        else LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            items(result.items, key = { it.uid }) { person -> WhiteCard(Modifier.clickable { open(Route.User(person.uid)) }, padding = 14.dp) { Text(person.nickname, fontSize = 15.sp) } }
            item { SocialPager(page, result.hasMore) { page = it } }
        }
    }
}

@Composable
fun NativeBlocksScreen(env: CommunityEnv, back: () -> Unit, open: (Route) -> Unit) {
    val session by env.session.state.collectAsState()
    val scope = rememberCoroutineScope()
    var busy by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    Column(Modifier.fillMaxSize()) {
        ScreenBar("屏蔽的用户", onBack = back)
        if (!session.loggedIn) { CenterMessage("登录后管理屏蔽用户", "登录", { open(Route.Login) }); return@Column }
        val loader = rememberLoader(env, "blocks-${session.accountId}") { env.social.blocks() }
        val state by loader.state.collectAsState()
        error?.let { Banner(it, Modifier.padding(12.dp), warn = true) }
        LoadBox(state, loader::reload) { result ->
            if (result.items.isEmpty()) CenterMessage("没有屏蔽用户", "刷新", loader::reload)
            else LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                items(result.items, key = { it.uid }) { person -> WhiteCard(padding = 14.dp) {
                    Text(person.nickname, Modifier.clickable { open(Route.User(person.uid)) }, fontSize = 15.sp)
                    GhostPillButton("取消屏蔽", {
                        if (busy) return@GhostPillButton
                        busy = true; error = null
                        scope.launch { try { env.social.block(person.uid, false); loader.reload() }
                            catch (e: CancellationException) { throw e } catch (e: Exception) { error = messageOf(e) }
                            finally { busy = false } }
                    }, Modifier.padding(top = 8.dp), enabled = !busy)
                } }
            }
        }
    }
}

/** 普通帖子/回复都使用后端支持的举报对象，不伪造资源包举报类型。 */
@Composable
internal fun NativeReportAction(env: CommunityEnv, targetType: String, id: Long, open: ((Route) -> Unit)? = null) {
    val session by env.session.state.collectAsState()
    var show by remember { mutableStateOf(false) }
    var reason by remember { mutableStateOf("") }
    var detail by remember { mutableStateOf("") }
    var busy by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    var submitted by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()
    Text(if (submitted) "已提交举报" else "举报", Modifier.clickable(enabled = !busy && !submitted) { show = true; error = null }, fontSize = 12.sp, color = LocalExtraColors.current.ink3)
    if (show) AlertDialog(onDismissRequest = { if (!busy) show = false }, title = { Text(if (targetType == "post") "举报回复" else "举报帖子") },
        text = { Column(Modifier.heightIn(max = 420.dp).verticalScroll(rememberScrollState()), verticalArrangement = Arrangement.spacedBy(6.dp)) {
            if (!session.loggedIn) Text("请先登录小伴账号后举报。", fontSize = 14.sp)
            else {
                CommunityReportReasons.forEach { (key, title) -> RadioRow(reason == key, title, desc = null, onClick = { if (!busy) reason = key }) }
                AreaBox(detail, { detail = it }, placeholder = "补充说明（最多 500 字，可选）", minHeight = 70.dp, enabled = !busy)
            }
            error?.let { Text(it, color = Danger, fontSize = 13.sp) }
        } },
        confirmButton = {
            if (!session.loggedIn) PillButton("登录", { show = false; open?.invoke(Route.Login) }, enabled = open != null)
            else PillButton(if (busy) "提交中…" else "提交举报", {
            if (busy) return@PillButton
            busy = true; error = null
            scope.launch { try { env.social.report(targetType, id, reason, detail); submitted = true; show = false }
                catch (e: CancellationException) { throw e } catch (e: Exception) { error = messageOf(e) }
                finally { busy = false } }
        }, enabled = session.loggedIn && !busy && reason.isNotEmpty() && detail.length <= 500) },
        dismissButton = { GhostPillButton("取消", { show = false }, enabled = !busy) })
}
