package xyz.xiangban.xiaoban.business.communitynative

import xyz.xiangban.xiaoban.common.community.cardTitle

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.*

internal data class RefinedTab(val id: String, val label: String, val count: Long? = null)

/** 统一精修吸顶 Tab 条：通栏白底 + 底部细线 + 20dp 圆角主题色指示条。 */
@Composable
internal fun RefinedTabBar(tabs: List<RefinedTab>, selected: String, modifier: Modifier = Modifier, onSelect: (String) -> Unit) {
    val extra = LocalExtraColors.current
    Column(modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface)) {
        Row(
            Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()).padding(horizontal = 10.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(2.dp),
        ) {
            tabs.forEach { tab ->
                val active = tab.id == selected
                Box(
                    Modifier.height(44.dp).clickable { onSelect(tab.id) }.padding(horizontal = 12.dp),
                    contentAlignment = Alignment.Center,
                ) {
                    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(5.dp)) {
                        Text(
                            tab.label,
                            style = XType.Body,
                            fontWeight = if (active) FontWeight.SemiBold else FontWeight.Medium,
                            color = if (active) MaterialTheme.colorScheme.onSurface else MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                        tab.count?.let {
                            Text(
                                compactCount(it.toInt().coerceAtLeast(0)),
                                style = XType.Caption,
                                color = if (active) MaterialTheme.colorScheme.onSurfaceVariant else extra.ink3,
                            )
                        }
                    }
                    if (active) {
                        Box(
                            Modifier.align(Alignment.BottomCenter).width(20.dp).height(2.5.dp)
                                .clip(CircleShape).background(Brand),
                        )
                    }
                }
            }
        }
        Hairline()
    }
}

/** 统一精修空状态：圆角图标块 + 标题 + 副说明 + 可选浅品牌色主操作。 */
@Composable
internal fun RefinedEmptyState(
    icon: ImageVector,
    title: String,
    desc: String = "",
    actionLabel: String? = null,
    onAction: (() -> Unit)? = null,
    modifier: Modifier = Modifier,
) {
    val extra = LocalExtraColors.current
    Column(
        modifier.fillMaxWidth().padding(horizontal = 28.dp, vertical = 68.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        Box(
            Modifier.size(52.dp).clip(RoundedCornerShape(16.dp)).background(extra.fill),
            contentAlignment = Alignment.Center,
        ) {
            XIcon(icon, tint = extra.ink3, size = 23.dp)
        }
        Text(title, style = XType.Body, fontWeight = FontWeight.SemiBold, textAlign = TextAlign.Center)
        if (desc.isNotBlank()) {
            Text(
                desc,
                Modifier.widthIn(max = 240.dp),
                style = XType.Secondary,
                color = extra.ink3,
                textAlign = TextAlign.Center,
            )
        }
        if (actionLabel != null && onAction != null) {
            Box(
                Modifier.padding(top = 4.dp).height(36.dp).clip(CircleShape).background(extra.brandSoft)
                    .clickable(onClick = onAction).padding(horizontal = 18.dp),
                contentAlignment = Alignment.Center,
            ) {
                Text(actionLabel, style = XType.Secondary, fontWeight = FontWeight.SemiBold, color = extra.brandInk)
            }
        }
    }
}

@Composable
private fun SocialPager(page: Int, hasMore: Boolean, select: (Int) -> Unit) {
    Row(Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 14.dp), horizontalArrangement = Arrangement.spacedBy(10.dp), verticalAlignment = Alignment.CenterVertically) {
        GhostPillButton("上一页", { select(page - 1) }, Modifier.weight(1f), enabled = page > 1, height = 40.dp, fontSize = 14, onPage = true)
        Text("第 $page 页", style = XType.Caption, color = LocalExtraColors.current.ink3)
        GhostPillButton("下一页", { select(page + 1) }, Modifier.weight(1f), enabled = hasMore, height = 40.dp, fontSize = 14, onPage = true)
    }
}

private fun notificationTag(kind: String): String = when (kind) {
    "reply", "reply_to" -> "回复"
    "work_like" -> "点赞"
    "mention" -> "提及"
    "accepted" -> "采纳"
    "pack_approved", "pack_rejected", "content_removed" -> "审核通知"
    else -> "通知"
}

private fun trustLevelLabel(level: Int): String? = when {
    level >= 3 -> "核心成员"
    level == 2 -> "活跃成员"
    level == 1 -> "社区成员"
    else -> null
}

@OptIn(ExperimentalMaterial3Api::class)
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
    var toast by remember { mutableStateOf<String?>(null) }
    val extra = LocalExtraColors.current

    if (!session.loggedIn) {
        Column(Modifier.fillMaxSize()) {
            ScreenBar("通知", onBack = back)
            RefinedEmptyState(XIcons.Bell, "登录后查看通知", "帖子的回复、作品点赞与审核结果都会出现在这里", "登录", { open(Route.Login) })
        }
        return
    }

    val loader = rememberLoader(env, "notifications-$page-$unread-${session.accountId}") { env.social.notifications(page, unread) }
    val state by loader.state.collectAsState()
    val entries = (state as? Load.Ready)?.value?.items.orEmpty()
    val unreadIds = entries.filterNot { it.read }.map { it.id }

    fun mark(ids: List<Long>?, reload: Boolean, showDone: Boolean = false) {
        if (busy) return
        busy = true; error = null; retryReadIds = ids; retryReadReload = reload
        scope.launch {
            try {
                env.social.markRead(ids)
                if (reload) loader.reload()
                if (showDone) toast = "已全部标记为已读"
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                error = messageOf(e)
            } finally {
                busy = false
            }
        }
    }
    LaunchedEffect(loader, unreadIds) { if (unreadIds.isNotEmpty()) mark(unreadIds, false) }

    val canMarkAll = !busy && (unreadIds.isNotEmpty() || entries.isNotEmpty())

    Box(Modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize()) {
            ScreenBar(
                "通知",
                onBack = back,
                trailing = {
                    Text(
                        "全部已读",
                        Modifier.clip(CircleShape)
                            .alpha(if (canMarkAll) 1f else 0.38f)
                            .clickable(enabled = canMarkAll) { mark(null, reload = true, showDone = true) }
                            .padding(horizontal = 10.dp, vertical = 6.dp),
                        style = XType.Secondary,
                        fontWeight = FontWeight.Medium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                },
            )
            RefinedTabBar(
                listOf(
                    RefinedTab("all", "全部"),
                    RefinedTab("unread", "未读", unreadIds.size.toLong().takeIf { it > 0 }),
                ),
                if (unread) "unread" else "all",
            ) { unread = it == "unread"; page = 1 }

            error?.let {
                Banner(it, Modifier.padding(horizontal = 12.dp, vertical = 8.dp), warn = true, actionText = "重试已读", onAction = { mark(retryReadIds, retryReadReload) })
            }

            PullToRefreshBox(
                isRefreshing = state is Load.Loading && entries.isNotEmpty(),
                onRefresh = loader::reload,
                modifier = Modifier.weight(1f).fillMaxWidth(),
            ) {
                LoadBox(state, loader::reload) { result ->
                    if (result.items.isEmpty()) {
                        Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState())) {
                            RefinedEmptyState(
                                icon = XIcons.Bell,
                                title = when {
                                    page > 1 -> "这一页没有通知"
                                    unread -> "已全部读完"
                                    else -> "还没有收到通知"
                                },
                                desc = if (unread) "所有消息都已处理，下拉可检查最新动态" else "帖子的回复、作品点赞与审核结果都会出现在这里",
                                actionLabel = if (page > 1) "上一页" else null,
                                onAction = if (page > 1) ({ page-- }) else null,
                            )
                        }
                    } else {
                        LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(top = 8.dp, bottom = 20.dp)) {
                            itemsIndexed(result.items, key = { _, it -> it.id }) { idx, note ->
                                val clickable = note.kind == "content_removed" || note.packId != null || note.topicId != null
                                Column(Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface)) {
                                    if (idx > 0) Hairline(Modifier.padding(start = 68.dp))
                                    Row(
                                        Modifier.fillMaxWidth()
                                            .clickable(enabled = clickable) {
                                                if (!note.read) mark(listOf(note.id), false)
                                                if (note.kind == "content_removed") open(Route.MyWorks)
                                                else note.packId?.let { open(Route.Pack(it)) } ?: note.topicId?.let { open(Route.Topic(it)) }
                                            }
                                            .padding(start = 16.dp, end = 14.dp, top = 14.dp, bottom = 14.dp),
                                        horizontalArrangement = Arrangement.spacedBy(12.dp),
                                        verticalAlignment = Alignment.Top,
                                    ) {
                                        Box(Modifier.padding(top = 1.dp)) {
                                            val actor = note.actor
                                            if (actor != null) {
                                                TopicAuthorAvatar(env, PublicUser(actor.uid, actor.nickname ?: "用户", actor.avatarUrl), 40.dp)
                                            } else {
                                                val approved = note.kind == "pack_approved"
                                                val removed = note.kind == "content_removed" || note.kind == "pack_rejected"
                                                val bg = when {
                                                    approved -> if (ThemeState.dark) Color(0xFF1A3126) else Color(0xFFEAF6F0)
                                                    removed -> DangerSoft
                                                    else -> extra.fill
                                                }
                                                val tint = when {
                                                    approved -> Success
                                                    removed -> Danger
                                                    else -> MaterialTheme.colorScheme.onSurfaceVariant
                                                }
                                                val ico = when {
                                                    approved -> XIcons.Check
                                                    removed -> XIcons.Warning
                                                    else -> XIcons.Bell
                                                }
                                                Box(Modifier.size(40.dp).clip(CircleShape).background(bg), contentAlignment = Alignment.Center) {
                                                    XIcon(ico, tint = tint, size = 19.dp)
                                                }
                                            }
                                            if (!note.read) {
                                                Box(
                                                    Modifier.align(Alignment.TopEnd).offset(x = 1.dp, y = (-1).dp)
                                                        .size(9.dp).clip(CircleShape).background(Danger)
                                                        .border(1.5.dp, MaterialTheme.colorScheme.surface, CircleShape),
                                                )
                                            }
                                        }
                                        Column(Modifier.weight(1f)) {
                                            Text(
                                                notificationHeadline(note),
                                                style = XType.Body,
                                                fontWeight = if (note.read) FontWeight.Normal else FontWeight.SemiBold,
                                                color = if (note.read) MaterialTheme.colorScheme.onSurfaceVariant else MaterialTheme.colorScheme.onSurface,
                                            )
                                            if (note.kind != "content_removed" && note.excerpt.isNotBlank()) {
                                                Text(
                                                    note.excerpt,
                                                    Modifier.padding(top = 6.dp).fillMaxWidth()
                                                        .clip(RoundedCornerShape(8.dp))
                                                        .background(MaterialTheme.colorScheme.background)
                                                        .padding(horizontal = 10.dp, vertical = 6.dp),
                                                    style = XType.Secondary,
                                                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                                                    maxLines = 2,
                                                    overflow = TextOverflow.Ellipsis,
                                                )
                                            }
                                            Text(
                                                "${agoText(note.createdAt)} · ${notificationTag(note.kind)}",
                                                Modifier.padding(top = 6.dp),
                                                style = XType.Caption,
                                                color = extra.ink3,
                                            )
                                        }
                                        if (clickable) {
                                            XIcon(XIcons.ChevronRight, Modifier.padding(top = 11.dp), tint = extra.ink3, size = 15.dp)
                                        }
                                    }
                                }
                            }
                            if (page > 1 || result.hasMore) {
                                item { SocialPager(page, result.hasMore) { page = it } }
                            }
                        }
                    }
                }
            }
        }
        Toast(toast) { toast = null }
    }
}

@Composable
fun NativeBookmarksScreen(env: CommunityEnv, back: () -> Unit, open: (Route) -> Unit) {
    var kind by rememberSaveable { mutableStateOf("topic") }
    val session by env.session.state.collectAsState()
    var page by rememberSaveable { mutableStateOf(1) }
    val scope = rememberCoroutineScope()
    var busy by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    Column(Modifier.fillMaxSize()) {
        ScreenBar("我的收藏", onBack = back)
        TabStrip(listOf("topic" to "帖子", "pack" to "作品", "game" to "游戏", "scheme" to "方案"), kind) { kind = it; page = 1 }
        if (kind == "game") { CommunityGameBookmarks(env, open); return@Column }
        if (!session.loggedIn) { CenterMessage("登录后查看收藏", "登录", { open(Route.Login) }); return@Column }
        if (kind != "topic") { ContentBookmarkFeed(env, kind, open); return@Column }
        val loader = rememberLoader(env, "bookmarks-$page-${session.accountId}") { env.social.bookmarks(page) }
        val state by loader.state.collectAsState()
        error?.let { Banner(it, Modifier.padding(12.dp), warn = true) }
        Box(Modifier.weight(1f)) { LoadBox(state, loader::reload) { result ->
            if (result.items.isEmpty()) CenterMessage(if (page > 1) "这一页没有收藏" else "还没有收藏帖子",
                if (page > 1) "上一页" else "刷新", { if (page > 1) page-- else loader.reload() })
            else LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                item { GhostPillButton("刷新收藏", loader::reload, Modifier.fillMaxWidth()) }
                items(result.items, key = { it.id }) { topic -> WhiteCard(padding = 14.dp) {
                    Text(topic.cardTitle, Modifier.fillMaxWidth().clickable { open(Route.Topic(topic.id)) }, style = XType.Section, fontWeight = FontWeight.SemiBold)
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
    var moreSheet by remember { mutableStateOf(false) }
    var blockConfirmation by remember { mutableStateOf(false) }
    var busy by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    var toast by remember { mutableStateOf<String?>(null) }
    val scope = rememberCoroutineScope()
    val extra = LocalExtraColors.current
    val site = env.config.forumBase.trimEnd('/')
    val avatarFlow = rememberAvatarFlow(env, onToast = { toast = it }, onUpdated = loader::reload)

    fun action(block: suspend () -> Unit) {
        if (!session.loggedIn) { open(Route.Login); return }
        if (busy) return
        busy = true; error = null
        scope.launch {
            try { block() }
            catch (e: CancellationException) { throw e }
            catch (e: Exception) { error = messageOf(e) }
            finally { busy = false }
        }
    }

    fun setFollowing(on: Boolean) = action {
        val result = env.social.follow(uid, on)
        loader.updateReady { profile ->
            profile.copy(
                following = result.following,
                followingCount = result.followingCount,
                followerCount = result.followerCount,
            )
        }
    }

    fun setBlocked(on: Boolean) = action {
        env.social.block(uid, on)
        loader.reload()
    }

    val readyProfile = (state as? Load.Ready)?.value
    val isSelf = uid == session.accountId

    Box(Modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize()) {
            // 极简顶栏：不显示多余的「社区用户」标题
            Row(
                Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface).statusBarsPadding()
                    .height(48.dp).padding(horizontal = 6.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween,
            ) {
                IconTap(XIcons.ChevronLeft, "返回", back, size = 20.dp)
                if (readyProfile != null) {
                    if (isSelf) {
                        IconTap(XIcons.Share, "分享主页", {
                            val url = "$site/u/${readyProfile.uid}"
                            env.platform?.copy(url)
                            toast = "已复制个人主页链接"
                        }, size = 20.dp)
                    } else {
                        IconTap(XIcons.More, "更多操作", { moreSheet = true }, size = 20.dp)
                    }
                } else {
                    Spacer(Modifier.size(44.dp))
                }
            }

            LoadBox(state, loader::reload) { profile ->
                val selfView = profile.uid == session.accountId
                val avatarUrl = (if (selfView) session.user?.avatarUrl ?: profile.avatarUrl else profile.avatarUrl)
                    ?.takeIf { it.isNotBlank() }?.let(env.forum::imageUrl)
                val publicId = if (selfView) session.user?.publicAccountId?.takeIf { it.isNotBlank() } else null

                // 顶部个人名片 + 四列等宽数据条（通栏白底）
                Column(Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface)) {
                    Row(
                        Modifier.fillMaxWidth().padding(start = 16.dp, end = 16.dp, top = 6.dp, bottom = 16.dp),
                        horizontalArrangement = Arrangement.spacedBy(14.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        AccountAvatar(
                            avatarUrl,
                            profile.nickname,
                            68.dp,
                            Modifier.border(1.dp, extra.hairline, CircleShape)
                                .let { if (selfView) it.clickable { avatarFlow.start() } else it },
                        )
                        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                            Text(profile.nickname, style = XType.Headline, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                            if (selfView) {
                                publicId?.let { id ->
                                    Row(
                                        Modifier.clip(RoundedCornerShape(6.dp)).clickable {
                                            env.platform?.copy(id)
                                            toast = "已复制账号 ID $id"
                                        }.padding(vertical = 2.dp),
                                        verticalAlignment = Alignment.CenterVertically,
                                        horizontalArrangement = Arrangement.spacedBy(4.dp),
                                    ) {
                                        Text("ID $id", style = XType.Caption, color = extra.ink3)
                                        XIcon(XIcons.Copy, tint = extra.ink3, size = 12.dp)
                                    }
                                }
                            } else {
                                Row(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
                                    trustLevelLabel(profile.trustLevel)?.let { label ->
                                        Box(
                                            Modifier.height(20.dp).clip(CircleShape).background(extra.fill).padding(horizontal = 8.dp),
                                            contentAlignment = Alignment.Center,
                                        ) {
                                            Text(label, style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                        }
                                    }
                                    if (profile.banned) {
                                        Box(
                                            Modifier.height(20.dp).clip(CircleShape).background(DangerSoft).padding(horizontal = 8.dp),
                                            contentAlignment = Alignment.Center,
                                        ) {
                                            Text("已封禁", style = XType.Caption, color = Danger)
                                        }
                                    }
                                }
                            }
                        }

                        if (selfView) {
                            Box(
                                Modifier.height(32.dp).clip(CircleShape).background(extra.fill)
                                    .clickable { avatarFlow.start() }.padding(horizontal = 12.dp),
                                contentAlignment = Alignment.Center,
                            ) {
                                Text("换头像", style = XType.Caption, fontWeight = FontWeight.Medium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            }
                        } else {
                            val canFollow = !busy && (!profile.banned || profile.following) && !profile.blocked
                            when {
                                profile.blocked -> Box(
                                    Modifier.height(32.dp).clip(CircleShape).background(extra.fill)
                                        .clickable(enabled = !busy) { setBlocked(false) }
                                        .padding(horizontal = 12.dp),
                                    contentAlignment = Alignment.Center,
                                ) {
                                    Text("已屏蔽", style = XType.Caption, fontWeight = FontWeight.Medium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                }
                                profile.following -> Box(
                                    Modifier.height(32.dp).clip(CircleShape).background(extra.fill)
                                        .clickable(enabled = canFollow) { setFollowing(false) }
                                        .padding(horizontal = 12.dp),
                                    contentAlignment = Alignment.Center,
                                ) {
                                    Text("已关注", style = XType.Caption, fontWeight = FontWeight.Medium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                }
                                else -> Row(
                                    Modifier.height(32.dp).alpha(if (canFollow) 1f else 0.45f)
                                        .clip(CircleShape).background(extra.brandSoft)
                                        .clickable(enabled = canFollow) { setFollowing(true) }
                                        .padding(horizontal = 12.dp),
                                    verticalAlignment = Alignment.CenterVertically,
                                    horizontalArrangement = Arrangement.spacedBy(3.dp),
                                ) {
                                    XIcon(XIcons.Plus, tint = extra.brandInk, size = 13.dp)
                                    Text("关注", style = XType.Caption, fontWeight = FontWeight.SemiBold, color = extra.brandInk)
                                }
                            }
                        }
                    }

                    if (!selfView && profile.blocked) {
                        Row(
                            Modifier.padding(start = 16.dp, end = 16.dp, bottom = 12.dp).fillMaxWidth()
                                .clip(RoundedCornerShape(12.dp)).background(DangerSoft)
                                .padding(start = 12.dp, end = 8.dp, top = 8.dp, bottom = 8.dp),
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(10.dp),
                        ) {
                            Text("已屏蔽对方，互相取消关注", Modifier.weight(1f), style = XType.Caption, color = Danger)
                            Box(
                                Modifier.height(28.dp).clip(CircleShape).background(MaterialTheme.colorScheme.surface)
                                    .clickable(enabled = !busy) { setBlocked(false) }
                                    .padding(horizontal = 10.dp),
                                contentAlignment = Alignment.Center,
                            ) {
                                Text("取消屏蔽", style = XType.Caption, fontWeight = FontWeight.SemiBold, color = Danger)
                            }
                        }
                    }

                    error?.let {
                        Text(it, Modifier.padding(start = 16.dp, end = 16.dp, bottom = 10.dp), color = Danger, style = XType.Secondary)
                    }

                    Hairline(Modifier.padding(horizontal = 16.dp))

                    Row(Modifier.fillMaxWidth().padding(horizontal = 8.dp, vertical = 12.dp)) {
                        ProfileStatCell(profile.topicCount, "帖子", Modifier.weight(1f)) { tab = "topics" }
                        ProfileStatCell(profile.postCount, "回复", Modifier.weight(1f), onClick = null)
                        ProfileStatCell(profile.followingCount, "关注", Modifier.weight(1f)) { tab = "following" }
                        ProfileStatCell(profile.followerCount, "粉丝", Modifier.weight(1f)) { tab = "followers" }
                    }
                }

                Spacer(Modifier.height(8.dp))

                RefinedTabBar(
                    listOf(
                        RefinedTab("topics", "帖子", profile.topicCount),
                        RefinedTab("following", "关注", profile.followingCount),
                        RefinedTab("followers", "粉丝", profile.followerCount),
                    ),
                    tab,
                ) { tab = it }

                Box(Modifier.weight(1f).fillMaxWidth()) {
                    if (tab == "topics" && profile.blocked) {
                        RefinedEmptyState(
                            icon = XIcons.Block,
                            title = "你已屏蔽该用户",
                            desc = "对方的帖子与回复已对你隐藏，取消屏蔽后即可恢复查看",
                            actionLabel = "取消屏蔽",
                            onAction = { setBlocked(false) },
                        )
                    } else if (tab == "topics") {
                        ProfileTopics(env, uid, selfView, open)
                    } else {
                        ProfileConnections(env, uid, tab == "followers", profile.following, open)
                    }
                }
            }
        }

        if (moreSheet && readyProfile != null && !isSelf) {
            BottomSheet(onDismiss = { moreSheet = false }, flush = true, cornerRadius = 20.dp) {
                Text(readyProfile.nickname, Modifier.padding(start = 16.dp, end = 16.dp, bottom = 8.dp), style = XType.Secondary, color = extra.ink3)
                SheetMenuRow(XIcons.Share, "分享主页", first = true) {
                    moreSheet = false
                    env.platform?.copy("$site/u/${readyProfile.uid}")
                    toast = "已复制该用户主页链接"
                }
                SheetMenuRow(XIcons.Block, if (readyProfile.blocked) "取消屏蔽该用户" else "屏蔽该用户", danger = !readyProfile.blocked) {
                    moreSheet = false
                    if (!session.loggedIn) open(Route.Login)
                    else if (readyProfile.blocked) setBlocked(false)
                    else blockConfirmation = true
                }
                SheetCancelFooter { moreSheet = false }
            }
        }

        AvatarFlowOverlays(avatarFlow, session.user?.username, session.user?.avatarUrl)
        Toast(toast) { toast = null }
    }

    if (blockConfirmation) AlertDialog(
        onDismissRequest = { if (!busy) blockConfirmation = false },
        title = { Text("屏蔽该用户？") },
        text = { Text("对方的帖子将从你的列表和搜索隐藏，回复会折叠，双方互相取消关注。对方不会收到屏蔽提示。") },
        confirmButton = { PillButton("确认屏蔽", { blockConfirmation = false; setBlocked(true) }, enabled = !busy) },
        dismissButton = { GhostPillButton("取消", { blockConfirmation = false }, enabled = !busy) },
    )
}

@Composable
private fun ProfileStatCell(count: Long, label: String, modifier: Modifier = Modifier, onClick: (() -> Unit)?) {
    Column(
        modifier.clip(RoundedCornerShape(10.dp))
            .let { if (onClick != null) it.clickable(onClick = onClick) else it }
            .padding(vertical = 4.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(2.dp),
    ) {
        Text(compactCount(count.toInt().coerceAtLeast(0)), style = XType.Section, fontWeight = FontWeight.SemiBold)
        Text(label, style = XType.Caption, color = LocalExtraColors.current.ink3)
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun ProfileTopics(env: CommunityEnv, uid: String, isSelf: Boolean, open: (Route) -> Unit) {
    val session by env.session.state.collectAsState()
    var page by rememberSaveable(uid) { mutableStateOf(1) }
    val loader = rememberLoader(env, "profile-topics-$uid-$page-${session.accountId}") { env.social.userTopics(uid, page) }
    val state by loader.state.collectAsState()
    val extra = LocalExtraColors.current
    val items = (state as? Load.Ready)?.value?.items.orEmpty()

    PullToRefreshBox(
        isRefreshing = state is Load.Loading && items.isNotEmpty(),
        onRefresh = loader::reload,
        modifier = Modifier.fillMaxSize(),
    ) {
        LoadBox(state, loader::reload) { result ->
            if (result.items.isEmpty()) {
                Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState())) {
                    RefinedEmptyState(
                        icon = XIcons.File,
                        title = if (page > 1) "这一页没有帖子" else if (isSelf) "还没有发布过帖子" else "暂无可见帖子",
                        desc = if (isSelf && page == 1) "分享你的创作作品、使用技巧或问题讨论，都会展示在这里" else "",
                        actionLabel = if (page > 1) "上一页" else if (isSelf) "去发一篇" else null,
                        onAction = if (page > 1) ({ page-- }) else if (isSelf) ({ open(Route.NewPost()) }) else null,
                    )
                }
            } else {
                LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(bottom = 20.dp)) {
                    itemsIndexed(result.items, key = { _, it -> it.id }) { idx, topic ->
                        val thumb = topic.images.firstOrNull()?.let(env.forum::imageUrl)
                        Column(Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface)) {
                            if (idx > 0) Hairline(Modifier.padding(horizontal = 16.dp))
                            Column(Modifier.fillMaxWidth().clickable { open(Route.Topic(topic.id)) }.padding(horizontal = 16.dp, vertical = 14.dp)) {
                                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.Top) {
                                    Column(Modifier.weight(1f)) {
                                        Text(topic.cardTitle, style = XType.Body, fontWeight = FontWeight.SemiBold, maxLines = 2, overflow = TextOverflow.Ellipsis)
                                        if (topic.excerpt.isNotBlank() && topic.excerpt != topic.cardTitle) {
                                            Text(
                                                topic.excerpt,
                                                Modifier.padding(top = 4.dp),
                                                style = XType.Secondary,
                                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                                                maxLines = 2,
                                                overflow = TextOverflow.Ellipsis,
                                            )
                                        }
                                    }
                                    if (thumb != null) {
                                        AsyncImage(
                                            thumb,
                                            null,
                                            imageLoader = rememberImageLoader(),
                                            contentScale = ContentScale.Crop,
                                            modifier = Modifier.size(84.dp, 62.dp).clip(RoundedCornerShape(8.dp)).background(extra.fill),
                                        )
                                    }
                                }
                                Row(
                                    Modifier.padding(top = 10.dp).fillMaxWidth(),
                                    verticalAlignment = Alignment.CenterVertically,
                                    horizontalArrangement = Arrangement.SpaceBetween,
                                ) {
                                    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                                        if (topic.category.name.isNotBlank()) {
                                            Box(
                                                Modifier.height(20.dp).clip(RoundedCornerShape(6.dp)).background(extra.fill).padding(horizontal = 7.dp),
                                                contentAlignment = Alignment.Center,
                                            ) {
                                                Text(topic.category.name, style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                            }
                                        }
                                        Text(agoText(topic.createdAt), style = XType.Caption, color = extra.ink3)
                                    }
                                    Row(horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
                                        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(3.dp)) {
                                            XIcon(XIcons.Message, tint = extra.ink3, size = 13.dp)
                                            Text(compactCount(topic.replyCount), style = XType.Caption, color = extra.ink3)
                                        }
                                        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(3.dp)) {
                                            XIcon(XIcons.Heart, tint = extra.ink3, size = 13.dp)
                                            Text(compactCount(topic.likeCount), style = XType.Caption, color = extra.ink3)
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if (page > 1 || result.hasMore) {
                        item { SocialPager(page, result.hasMore) { page = it } }
                    }
                }
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun ProfileConnections(env: CommunityEnv, uid: String, followers: Boolean, viewerFollows: Boolean, open: (Route) -> Unit) {
    var page by rememberSaveable(uid, followers) { mutableStateOf(1) }
    val viewerInList = followers && viewerFollows
    val loader = rememberLoader(env, "connections-$uid-$followers-$page-$viewerInList") { env.social.connections(uid, followers, page) }
    val state by loader.state.collectAsState()
    val extra = LocalExtraColors.current
    val items = (state as? Load.Ready)?.value?.items.orEmpty()

    PullToRefreshBox(
        isRefreshing = state is Load.Loading && items.isNotEmpty(),
        onRefresh = loader::reload,
        modifier = Modifier.fillMaxSize(),
    ) {
        LoadBox(state, loader::reload) { result ->
            if (result.items.isEmpty()) {
                Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState())) {
                    RefinedEmptyState(
                        icon = XIcons.Users,
                        title = if (followers) "还没有粉丝" else "还没有关注用户",
                        desc = "去社区发现有趣的创作者和讨论吧",
                        actionLabel = if (page > 1) "上一页" else null,
                        onAction = if (page > 1) ({ page-- }) else null,
                    )
                }
            } else {
                LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(bottom = 20.dp)) {
                    itemsIndexed(result.items, key = { _, it -> it.uid }) { idx, person ->
                        Column(Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface)) {
                            if (idx > 0) Hairline(Modifier.padding(start = 68.dp))
                            Row(
                                Modifier.fillMaxWidth().clickable { open(Route.User(person.uid)) }
                                    .padding(horizontal = 16.dp, vertical = 12.dp),
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(12.dp),
                            ) {
                                TopicAuthorAvatar(env, person, 40.dp)
                                Column(Modifier.weight(1f)) {
                                    Text(person.nickname, style = XType.Body, fontWeight = FontWeight.Medium, maxLines = 1, overflow = TextOverflow.Ellipsis)
                                    trustLevelLabel(person.trustLevel)?.let { label ->
                                        Text(label, Modifier.padding(top = 1.dp), style = XType.Caption, color = extra.ink3)
                                    }
                                }
                                XIcon(XIcons.ChevronRight, tint = extra.ink3, size = 15.dp)
                            }
                        }
                    }
                    if (page > 1 || result.hasMore) {
                        item { SocialPager(page, result.hasMore) { page = it } }
                    }
                }
            }
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
                    Text(person.nickname, Modifier.clickable { open(Route.User(person.uid)) }, style = XType.Body)
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
    Text(if (submitted) "已提交举报" else "举报", Modifier.clickable(enabled = !busy && !submitted) { show = true; error = null }, style = XType.Caption, color = LocalExtraColors.current.ink3)
    if (show) AlertDialog(onDismissRequest = { if (!busy) show = false }, title = { Text(when (targetType) { "post" -> "举报回复"; "pack" -> "举报作品"; "scheme" -> "举报方案"; else -> "举报帖子" }) },
        text = { Column(Modifier.heightIn(max = 420.dp).verticalScroll(rememberScrollState()), verticalArrangement = Arrangement.spacedBy(6.dp)) {
            if (!session.loggedIn) Text("请先登录小伴账号后举报。", style = XType.Body)
            else {
                CommunityReportReasons.forEach { (key, title) -> RadioRow(reason == key, title, desc = null, onClick = { if (!busy) reason = key }) }
                AreaBox(detail, { detail = it }, placeholder = "补充说明（最多 500 字，可选）", minHeight = 70.dp, enabled = !busy)
            }
            error?.let { Text(it, color = Danger, style = XType.Secondary) }
        } },
        confirmButton = {
            if (!session.loggedIn) PillButton("登录", { show = false; open?.invoke(Route.Login) }, enabled = open != null)
            else PillButton(if (busy) "提交中…" else "提交举报", {
            if (busy) return@PillButton
            busy = true; error = null
            scope.launch { try { if (targetType in listOf("pack", "scheme")) env.contentSocial.report(targetType, id, reason, detail) else env.social.report(targetType, id, reason, detail); submitted = true; show = false }
                catch (e: CancellationException) { throw e } catch (e: Exception) { error = messageOf(e) }
                finally { busy = false } }
        }, enabled = session.loggedIn && !busy && reason.isNotEmpty() && detail.length <= 500) },
        dismissButton = { GhostPillButton("取消", { show = false }, enabled = !busy) })
}
