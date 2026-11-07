package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import coil3.ImageLoader
import coil3.compose.AsyncImage
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.*

/** The catalog has one cover, no screenshots API. Show that image once without invented slides. */
@Composable
internal fun OfficialAppSheet(env: CommunityEnv, game: MobileGame, cover: ByteArray?, loader: ImageLoader,
    dismiss: () -> Unit, remix: () -> Unit) {
    var message by remember { mutableStateOf<String?>(null) }
    val shortcut = AppShortcut("builtin", game.id, game.name)
    val favorite = env.appLibrary.favorites.any { it.key == shortcut.key }
    BottomSheet(dismiss, cornerRadius = 16.dp) {
        Column(Modifier.fillMaxWidth().heightIn(max = 620.dp).verticalScroll(rememberScrollState()), verticalArrangement = Arrangement.spacedBy(16.dp)) {
            Box(Modifier.align(Alignment.CenterHorizontally).size(32.dp, 4.dp).clip(CircleShape).background(LocalExtraColors.current.line))
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                AppShortcutIcon(env, shortcut, Modifier.size(64.dp))
                Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                    Text(game.name, style = XType.Title, fontWeight = FontWeight.SemiBold)
                    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        Text("官方", Modifier.clip(CircleShape).background(LocalExtraColors.current.brandSoft).padding(horizontal = 8.dp, vertical = 4.dp), style = XType.Caption, color = LocalExtraColors.current.brandInk)
                        Text("${BuiltinMobileGames.genre(game.id)} · ${if (game.orientation == "landscape") "横屏" else "竖屏"}", style = XType.Caption, color = LocalExtraColors.current.ink3)
                    }
                }
                IconTap(XIcons.Close, "关闭官方应用详情", dismiss)
            }
            if (cover != null) Row(Modifier.horizontalScroll(rememberScrollState())) {
                AsyncImage(cover, "${game.name}封面", imageLoader = loader, contentScale = ContentScale.Fit,
                    modifier = Modifier.width(280.dp).aspectRatio(if (game.orientation == "landscape") 16f / 9f else 3f / 4f).clip(RoundedCornerShape(16.dp)).background(LocalExtraColors.current.fill))
            }
            AppActionButton(if (game.orientation == "landscape") "横屏打开" else "打开", {
                if (env.platform?.openGame(game.id) == true) dismiss() else message = "当前平台无法打开游戏"
            }, Modifier.fillMaxWidth(), primary = true, icon = XIcons.Play)
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                AppActionButton(if (favorite) "已收藏" else "收藏", { env.appLibrary.toggle(shortcut) }, Modifier.weight(1f), icon = XIcons.Bookmark)
                AppActionButton("做一个我的版本", remix, Modifier.weight(1f))
            }
            Text(game.description, style = XType.Body, color = MaterialTheme.colorScheme.onSurfaceVariant)
            message?.let { Text(it, style = XType.Caption, color = Danger) }
        }
    }
}

/** Posts reuse the existing real reply/like/bookmark services. */
@Composable
internal fun AppPostContent(env: CommunityEnv, chat: CloudChatController, pack: Pack, back: () -> Unit, open: ((Route) -> Unit)?) {
    var remix by rememberSaveable(pack.id) { mutableStateOf(false) }
    var share by rememberSaveable(pack.id) { mutableStateOf(false) }
    val scroll = rememberScrollState()
    val scope = rememberCoroutineScope()
    val orientation = appOrientation(pack)
    Column(Modifier.fillMaxSize()) {
        Row(Modifier.fillMaxWidth().statusBarsPadding().heightIn(min = 56.dp).padding(horizontal = 8.dp), verticalAlignment = Alignment.CenterVertically) {
            IconTap(XIcons.ChevronLeft, "返回", back)
            val author = pack.author
            Avatar(author?.nickname ?: "作者", 36.dp, showInitial = true)
            Text(author?.nickname ?: "社区作者", Modifier.weight(1f).padding(horizontal = 8.dp).clickable(enabled = author != null && open != null) { author?.let { open?.invoke(Route.User(it.uid)) } },
                style = XType.Secondary, maxLines = 1, overflow = TextOverflow.Ellipsis)
            if (author != null) AppAuthorFollow(env, author.uid, open)
            IconTap(XIcons.Share, "分享作品", { share = true })
        }
        WorkInteractionLayout(env, "pack", pack.id, open, { share = true }, enabled = pack.status == "published",
            onComments = { scope.launch { scroll.animateScrollTo(scroll.maxValue) } }) { comments ->
            Column(Modifier.fillMaxSize().verticalScroll(scroll)) {
                // Pack provides a cover, not a screenshot gallery. Preserve horizontal gallery structure for future images.
                pack.coverUrl?.let { url -> Row(Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()).padding(horizontal = 16.dp)) {
                    AsyncImage(env.forum.imageUrl(url), "${pack.title}封面", imageLoader = rememberAppImageLoader(), contentScale = ContentScale.Fit,
                        modifier = Modifier.width(340.dp).aspectRatio(if (orientation == "landscape") 16f / 9f else 1f).clip(RoundedCornerShape(16.dp)).background(LocalExtraColors.current.fill))
                } }
                Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text(pack.title, style = XType.Headline, fontWeight = FontWeight.SemiBold)
                    if (pack.summary.isNotBlank()) Text(pack.summary, style = XType.Body)
                    if (pack.tags.isNotEmpty()) Text(pack.tags.joinToString("  ") { "#$it" }, style = XType.Secondary, color = LocalExtraColors.current.brandInk)
                    Text(listOfNotNull(pack.updatedAt.take(10).takeIf { it.isNotBlank() }, "游戏",
                        when (orientation) { "landscape" -> "横屏"; "portrait" -> "竖屏"; else -> null }, "${pack.applyCount} 人玩过").joinToString(" · "), style = XType.Caption, color = LocalExtraColors.current.ink3)
                    if (pack.status == "published") {
                        GameTrialButton(env, pack.id, open = open, modifier = Modifier.fillMaxWidth(), label = if (orientation == "landscape") "横屏进入" else "进入", appStyle = true)
                        AppActionButton("做一个我的版本", { remix = true }, Modifier.fillMaxWidth(), icon = XIcons.Pen)
                    } else Text("作品暂不可用", style = XType.Secondary, color = LocalExtraColors.current.ink3)
                    Text("评论 · ${pack.replyCount}", Modifier.fillMaxWidth().clickable { scope.launch { scroll.animateScrollTo(scroll.maxValue) } }.padding(vertical = 8.dp), style = XType.Section, fontWeight = FontWeight.SemiBold)
                }
                comments()
            }
        }
    }
    OverlayTransition(remix) { panel -> AppRemixSheet(env, chat, pack.title, pack.summary, { remix = false }, open, panel) }
    OverlayTransition(share) { panel -> CommunityShareSheet(env, env.forum.packUrl(pack.id), "分享作品", null, { share = false }, {}, panel) }
}

@Composable
private fun AppAuthorFollow(env: CommunityEnv, uid: String, open: ((Route) -> Unit)?) {
    val session by env.session.state.collectAsState()
    if (session.accountId == uid) return
    val loader = rememberLoader(env, "app-author-$uid-${session.accountId}") { env.social.profile(uid) }
    val state by loader.state.collectAsState()
    val profile = (state as? Load.Ready)?.value ?: return // unavailable follow state stays hidden
    val scope = rememberCoroutineScope()
    var busy by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    Column {
        AppActionButton(if (profile.following) "已关注" else "关注", {
            if (!session.loggedIn) { open?.invoke(Route.Login); return@AppActionButton }
            busy = true
            scope.launch {
                try { env.social.follow(uid, !profile.following); loader.reload(); error = null }
                catch (e: CancellationException) { throw e }
                catch (e: Exception) { error = messageOf(e) }
                finally { busy = false }
            }
        }, primary = !profile.following, enabled = !busy && !profile.blocked && (!profile.banned || profile.following), height = 34.dp)
        error?.let { Text(it, style = XType.Caption, color = Danger) }
    }
}

/** A descriptive reference is carried into the existing local creation chat; no source-copy/provenance claims. */
@Composable
internal fun AppRemixSheet(env: CommunityEnv, chat: CloudChatController, title: String, summary: String,
    dismiss: () -> Unit, open: ((Route) -> Unit)?, panel: Modifier = Modifier) {
    var idea by rememberSaveable(title) { mutableStateOf("") }
    var message by remember { mutableStateOf<String?>(null) }
    val session by env.session.state.collectAsState()
    val cloud by chat.state.collectAsState()
    BottomSheet(dismiss, panel, cornerRadius = 16.dp) {
        Column(Modifier.fillMaxWidth().heightIn(max = 540.dp).verticalScroll(rememberScrollState()), verticalArrangement = Arrangement.spacedBy(16.dp)) {
            Text("做一个我的版本", style = XType.Title, fontWeight = FontWeight.SemiBold)
            Text("参考《$title》", style = XType.Secondary, color = LocalExtraColors.current.ink3)
            if (summary.isNotBlank()) Text(summary, style = XType.Secondary, maxLines = 3, overflow = TextOverflow.Ellipsis)
            Box(Modifier.fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(LocalExtraColors.current.fill).padding(12.dp)) {
                if (idea.isBlank()) Text("想怎么改？比如换一个主题，加一个新玩法", style = XType.Body, color = LocalExtraColors.current.ink3)
                BasicTextField(idea, { idea = it.take(2000) }, Modifier.fillMaxWidth().heightIn(min = 104.dp).semantics { contentDescription = "改编想法" },
                    textStyle = XType.Body.copy(color = MaterialTheme.colorScheme.onSurface), cursorBrush = SolidColor(MaterialTheme.colorScheme.primary))
            }
            Row(Modifier.horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                listOf("换一个主题", "加一个新玩法", "改得简单一点", "加一个双人模式").forEach { hint -> AppActionButton(hint, { idea = hint }, height = 34.dp) }
            }
            Text("小伴先给你看方案，确认后再制作。", style = XType.Caption, color = LocalExtraColors.current.ink3)
            AppActionButton("开始", {
                when {
                    !session.loggedIn -> open?.invoke(Route.Login)
                    env.agent == null -> message = "当前安装包暂不支持手机本地创作"
                    cloud.sending -> message = "先结束当前任务，再开始新作品"
                    else -> {
                        chat.clearConversation()
                        dismiss()
                        open?.invoke(Route.AiChat)
                        chat.send(appRemixPrompt(title, summary, idea))
                    }
                }
            }, Modifier.fillMaxWidth(), primary = true, enabled = idea.isNotBlank() && open != null)
            message?.let { Text(it, style = XType.Caption, color = Danger) }
        }
    }
}
