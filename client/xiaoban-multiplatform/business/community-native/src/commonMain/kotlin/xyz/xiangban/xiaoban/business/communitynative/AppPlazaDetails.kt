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
            if (shortcut.valid) Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
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
    val orientation = appOrientation(pack)
    Column(Modifier.fillMaxSize()) {
        val author = pack.author
        DetailAuthorBar(author?.nickname ?: "社区作者", back, onShare = { share = true },
            onAuthor = if (author != null && open != null) ({ open(Route.User(author.uid)) }) else null) {
            author?.let { DetailAuthorFollow(env, it.uid, open) }
        }
        WorkInteractionLayout(env, "pack", pack.id, open, { share = true }, enabled = pack.status == "published",
            linkedTopicId = pack.topicId) { comments ->
            Column(Modifier.fillMaxSize().verticalScroll(scroll)) {
                DetailGallery(listOfNotNull(pack.coverUrl?.takeIf(String::isNotBlank)?.let(env.forum::imageUrl)), pack.title)
                DetailBody {
                    DetailHeading(pack.title)
                    if (pack.summary.isNotBlank()) Text(pack.summary, style = XType.DetailBody)
                    DetailTags(pack.tags)
                    if (pack.status == "published") {
                        GameTrialButton(env, pack.id, open = open, modifier = Modifier.fillMaxWidth(), label = if (pack.kind == "game") "试玩游戏" else "打开应用", appStyle = true)
                        DetailInfoStrip("创作", "做一个我的版本", XIcons.Pen, onClick = { remix = true })
                    } else Text("作品暂不可用", style = XType.Secondary, color = LocalExtraColors.current.ink3)
                    DetailMetadata(listOfNotNull(pack.updatedAt.take(10).takeIf { it.isNotBlank() }, if (pack.kind == "app") "工具" else "游戏",
                        when (orientation) { "landscape" -> "横屏"; "portrait" -> "竖屏"; else -> null }, "${pack.applyCount} 人玩过").joinToString(" · "))
                }
                comments()
            }
        }
    }
    OverlayTransition(remix) { panel -> AppRemixSheet(env, chat, pack.title, pack.summary, { remix = false }, open, panel, kind = pack.kind) }
    OverlayTransition(share) { panel ->
        CommunityShareSheet(
            env = env,
            url = env.forum.packUrl(pack.id),
            heading = "分享作品",
            onDismiss = { share = false },
            panelModifier = panel,
            author = pack.author?.nickname.orEmpty(),
            dateLabel = pack.updatedAt.take(10),
            coverUrl = pack.coverUrl?.takeIf(String::isNotBlank)?.let(env.forum::imageUrl),
        )
    }
}

/** A descriptive reference is carried into the existing local creation chat; no source-copy/provenance claims. */
@Composable
internal fun AppRemixSheet(env: CommunityEnv, chat: CloudChatController, title: String, summary: String,
    dismiss: () -> Unit, open: ((Route) -> Unit)?, panel: Modifier = Modifier, kind: String = "game") {
    var idea by rememberSaveable(title) { mutableStateOf("") }
    var message by remember { mutableStateOf<String?>(null) }
    val session by env.session.state.collectAsState()
    val cloud by chat.state.collectAsState()
    BottomSheet(dismiss, panel, cornerRadius = 16.dp) {
        Column(Modifier.fillMaxWidth().heightIn(max = 540.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            Column(Modifier.weight(1f, fill = false).verticalScroll(rememberScrollState()), verticalArrangement = Arrangement.spacedBy(16.dp)) {
                Text("做一个我的版本", style = XType.Title, fontWeight = FontWeight.SemiBold)
                Text("参考《$title》", style = XType.Secondary, color = LocalExtraColors.current.ink3)
                if (summary.isNotBlank()) Text(summary, style = XType.Secondary, maxLines = 3, overflow = TextOverflow.Ellipsis)
                Box(Modifier.fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(LocalExtraColors.current.fill).padding(12.dp)) {
                    if (idea.isBlank()) Text(if (kind == "app") "想怎么改？比如换一个主题，加一个新功能" else "想怎么改？比如换一个主题，加一个新玩法", style = XType.Body, color = LocalExtraColors.current.ink3)
                    BasicTextField(idea, { idea = it.take(2000) }, Modifier.fillMaxWidth().semantics { contentDescription = "改编想法" },
                        minLines = 4, maxLines = 8, textStyle = XType.Body.copy(color = MaterialTheme.colorScheme.onSurface), cursorBrush = SolidColor(MaterialTheme.colorScheme.primary))
                }
                Row(Modifier.horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    val hints = if (kind == "app") listOf("换一个主题", "加一个新功能", "改得简单一点")
                        else listOf("换一个主题", "加一个新玩法", "改得简单一点", "加一个双人模式")
                    hints.forEach { hint -> AppActionButton(hint, { idea = hint }, height = 34.dp) }
                }
            }
            val blocked = !chat.canLeaveForCreation()
            Text(when {
                blocked -> "当前还有创作任务正在执行。可以先编辑需求。"
                cloud.sending -> "图片继续在原对话生成。确认方案后开始制作。"
                else -> "小伴先给你看方案，确认后再制作。"
            }, style = XType.Caption, color = LocalExtraColors.current.ink3)
            AppActionButton("查看创作方案", {
                when {
                    !session.loggedIn -> open?.invoke(Route.Login)
                    env.agent == null -> message = "当前安装包暂不支持手机本地创作"
                    chat.startCreation("改编《$title》", idea, appRemixPrompt(title, summary, idea, kind)) -> {
                        dismiss()
                        open?.invoke(Route.AiChat)
                    }
                }
            }, Modifier.fillMaxWidth(), primary = true, enabled = idea.isNotBlank() && open != null && !blocked)
            message?.let { Text(it, style = XType.Caption, color = Danger) }
        }
    }
}
