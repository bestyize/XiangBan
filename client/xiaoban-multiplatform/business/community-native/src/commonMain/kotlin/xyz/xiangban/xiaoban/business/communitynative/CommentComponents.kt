package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import xyz.xiangban.xiaoban.common.community.BodyBlock
import xyz.xiangban.xiaoban.common.community.NewsComment
import xyz.xiangban.xiaoban.common.community.PublicUser
import xyz.xiangban.xiaoban.common.community.agoText
import xyz.xiangban.xiaoban.common.community.forumImageId
import xyz.xiangban.xiaoban.common.community.compactCount

@Composable
internal fun CommentBar(
    label: String,
    onWrite: () -> Unit,
    enabled: Boolean = true,
    actions: @Composable RowScope.() -> Unit = {},
) {
    val detail = LocalCommunityDetail.current
    Column(Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface).navigationBarsPadding()) {
        Hairline()
        Row(
            Modifier.fillMaxWidth().heightIn(min = if (detail) 51.dp else 0.dp)
                .padding(horizontal = 12.dp, vertical = if (detail) 0.dp else 8.dp),
            horizontalArrangement = Arrangement.spacedBy(if (detail) 12.dp else 4.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Row(
                Modifier.weight(1f).heightIn(min = if (detail) 36.dp else 44.dp).clip(CircleShape).background(LocalExtraColors.current.fill)
                    .clickable(enabled = enabled, role = Role.Button, onClick = onWrite).padding(horizontal = 12.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                val tint = if (detail) LocalExtraColors.current.ink3 else MaterialTheme.colorScheme.onSurfaceVariant
                XIcon(if (detail) XIcons.Pen else XIcons.Message, tint = tint, size = if (detail) 14.dp else 16.dp)
                Text(label, style = XType.Secondary, color = tint, maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
            actions()
        }
    }
}

@Composable
internal fun CommentCountAction(count: Int?, onClick: () -> Unit) {
    if (LocalCommunityDetail.current) {
        DetailDockAction(XIcons.DetailComment, count?.let(::compactCount), count?.let { "查看评论，共 $it 条" } ?: "查看评论", onClick)
        return
    }
    Row(
        Modifier.heightIn(min = 44.dp).widthIn(min = 44.dp)
            .semantics { contentDescription = count?.let { "查看评论，共 $it 条" } ?: "查看评论" }
            .clickable(role = Role.Button, onClick = onClick).padding(horizontal = 4.dp),
        horizontalArrangement = Arrangement.spacedBy(4.dp, Alignment.CenterHorizontally),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        XIcon(XIcons.Message, size = 20.dp)
        count?.let { Text(compactCount(it), style = XType.Caption) }
    }
}

@Composable
internal fun DetailDockAction(
    icon: ImageVector,
    text: String?,
    label: String,
    onClick: () -> Unit,
    tint: Color = MaterialTheme.colorScheme.onSurface,
    enabled: Boolean = true,
) {
    Row(
        Modifier.heightIn(min = 44.dp).semantics { contentDescription = label }
            .clickable(enabled = enabled, role = Role.Button, onClick = onClick).padding(horizontal = 2.dp),
        horizontalArrangement = Arrangement.spacedBy(4.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        XIcon(icon, tint = tint, size = 19.dp)
        text?.let { Text(it, style = XType.Secondary, color = tint, fontWeight = FontWeight.Medium) }
    }
}

@Composable
internal fun DetailCommentPrompt(env: CommunityEnv, onWrite: () -> Unit) {
    val user by env.session.state.collectAsState()
    val muted = LocalExtraColors.current.ink3
    Row(
        Modifier.fillMaxWidth().padding(top = 10.dp, bottom = 16.dp),
        horizontalArrangement = Arrangement.spacedBy(10.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        val account = user.user
        if (account != null) TopicAuthorAvatar(env, PublicUser(account.uid, account.username, account.avatarUrl), 30.dp)
        else DetailInitial("伴", background = Color(0xFFD93A44))
        Row(
            Modifier.weight(1f).heightIn(min = 36.dp).clip(CircleShape).background(LocalExtraColors.current.fill)
                .clickable(role = Role.Button, onClick = onWrite).padding(horizontal = 12.dp),
            horizontalArrangement = Arrangement.spacedBy(8.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text("留下你的想法吧", Modifier.weight(1f), style = XType.Secondary, color = muted, maxLines = 1, overflow = TextOverflow.Ellipsis)
            XIcon(XIcons.Image, tint = muted, size = 15.dp)
            XIcon(XIcons.Smile, tint = muted, size = 15.dp)
        }
    }
}

@Composable
internal fun CommentSectionTitle(
    count: Int?,
    modifier: Modifier = Modifier,
    label: String = "评论",
    newestFirst: Boolean = true,
    onSort: (() -> Unit)? = null,
) {
    if (LocalCommunityDetail.current) {
        Row(modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
            Text("共 ${count ?: 0} 条$label", style = XType.DetailComment, fontWeight = FontWeight.SemiBold, color = MaterialTheme.colorScheme.onSurfaceVariant)
            onSort?.let {
                Box(Modifier.size(28.dp).clickable(role = Role.Button, onClick = it), contentAlignment = Alignment.Center) {
                    XIcon(XIcons.DetailSort, tint = LocalExtraColors.current.ink3, size = 14.dp,
                        description = if (newestFirst) "切换到最早优先" else "切换到最新优先")
                }
            }
        }
        return
    }
    Row(modifier.fillMaxWidth().heightIn(min = 40.dp), verticalAlignment = Alignment.CenterVertically) {
        Text(label, style = XType.Section, fontWeight = FontWeight.SemiBold)
        count?.let {
            Text(it.toString(), Modifier.padding(start = 8.dp), style = XType.Secondary,
                color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
    }
}

@Composable
internal fun CommentListFooter(
    loading: Boolean,
    error: String?,
    onRetry: () -> Unit,
    onMore: (() -> Unit)?,
    modifier: Modifier = Modifier,
) {
    Column(modifier.fillMaxWidth()) {
        when {
            loading -> Spinner(Modifier.fillMaxWidth().height(48.dp))
            error != null -> Banner(error, warn = true, actionText = "重试", onAction = onRetry)
            onMore != null -> CommentTextAction("加载更多评论", onMore)
        }
    }
}

@Composable
internal fun CommentRow(
    env: CommunityEnv,
    author: PublicUser,
    metadata: String,
    parent: String?,
    modifier: Modifier = Modifier,
    onAuthor: (() -> Unit)? = null,
    onReply: (() -> Unit)? = null,
    like: (@Composable () -> Unit)? = null,
    actions: @Composable RowScope.() -> Unit = {},
    content: @Composable ColumnScope.() -> Unit,
) {
    val detail = LocalCommunityDetail.current
    val ink = if (detail) LocalExtraColors.current.ink3 else MaterialTheme.colorScheme.onSurfaceVariant
    val authorClick = if (onAuthor == null) Modifier else Modifier.clickable(role = Role.Button, onClick = onAuthor)
    Column(modifier.fillMaxWidth().padding(bottom = if (detail) 14.dp else 0.dp)) {
        Row(Modifier.fillMaxWidth().padding(top = if (detail) 0.dp else 12.dp), horizontalArrangement = Arrangement.spacedBy(if (detail) 10.dp else 12.dp)) {
            Box(authorClick.clip(CircleShape)) { TopicAuthorAvatar(env, author, 32.dp) }
            Column(Modifier.weight(1f)) {
                Text(
                    author.nickname, authorClick, style = XType.Secondary,
                    color = if (detail) ink else MaterialTheme.colorScheme.onSurface,
                    fontWeight = if (detail) FontWeight.Normal else FontWeight.SemiBold,
                    maxLines = 1, overflow = TextOverflow.Ellipsis,
                )
                parent?.let {
                    Text(it, Modifier.padding(top = 4.dp).clip(RoundedCornerShape(8.dp))
                        .background(LocalExtraColors.current.fill).padding(horizontal = 8.dp, vertical = 4.dp),
                        style = XType.Caption, color = ink, maxLines = 1, overflow = TextOverflow.Ellipsis)
                }
                Spacer(Modifier.height(if (detail) 3.dp else 4.dp))
                content()
                Row(
                    Modifier.fillMaxWidth().padding(top = if (detail) 5.dp else 0.dp).heightIn(min = if (detail) 26.dp else 36.dp),
                    horizontalArrangement = Arrangement.spacedBy(if (detail) 10.dp else 12.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text(metadata, Modifier.weight(1f, fill = false), style = XType.Caption, color = ink,
                        maxLines = 1, overflow = TextOverflow.Ellipsis)
                    onReply?.let { CommentTextAction("回复", it) }
                    actions()
                    if (detail && like != null) {
                        Spacer(Modifier.weight(1f))
                        like()
                    }
                }
            }
            if (!detail) like?.invoke()
        }
        if (!detail) Hairline(Modifier.padding(start = 44.dp))
    }
}

@Composable
internal fun CommentTextAction(label: String, onClick: () -> Unit) {
    Box(
        Modifier.heightIn(min = if (LocalCommunityDetail.current) 26.dp else 36.dp).clickable(role = Role.Button, onClick = onClick),
        contentAlignment = Alignment.Center,
    ) {
        Text(label, style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}

@Composable
internal fun CommentLikeAction(liked: Boolean, count: Int, onLike: () -> Unit, enabled: Boolean = true) {
    if (LocalCommunityDetail.current) {
        val tint = if (liked) Color(0xFFE11D48) else LocalExtraColors.current.ink3
        Row(
            Modifier.heightIn(min = 26.dp).semantics { contentDescription = if (liked) "取消赞" else "点赞评论" }
                .clickable(enabled = enabled, role = Role.Button, onClick = onLike),
            horizontalArrangement = Arrangement.spacedBy(3.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            XIcon(XIcons.Heart, size = 14.dp, tint = tint)
            if (count > 0) Text(compactCount(count), style = XType.Caption, color = tint)
        }
        return
    }
    val tint = if (liked) LocalExtraColors.current.brandInk else MaterialTheme.colorScheme.onSurfaceVariant
    Column(
        Modifier.size(width = 40.dp, height = 44.dp)
            .semantics { contentDescription = if (liked) "取消赞" else "点赞评论" }
            .clickable(enabled = enabled, role = Role.Button, onClick = onLike),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        XIcon(XIcons.Heart, size = 20.dp, tint = tint)
        if (count > 0) Text(compactCount(count), style = XType.Caption, color = tint)
    }
}

@Composable
internal fun PlainCommentRow(
    env: CommunityEnv,
    comment: NewsComment,
    parentName: String?,
    onAuthor: () -> Unit,
    onLike: () -> Unit,
    onReply: () -> Unit,
    onDelete: () -> Unit,
    modifier: Modifier = Modifier,
) {
    CommentRow(
        env, PublicUser(comment.author.uid, comment.author.nickname, comment.author.avatarUrl),
        metadata = agoText(comment.createdAt),
        parent = comment.replyTo?.let { "回复 ${parentName ?: "评论 #$it"}" },
        modifier = modifier,
        onAuthor = onAuthor,
        onReply = onReply,
        like = { CommentLikeAction(comment.liked, comment.likeCount, onLike) },
        actions = { if (comment.mine) CommentTextAction("删除", onDelete) },
    ) {
        var zoom by remember { mutableStateOf<String?>(null) }
        if (comment.body.isNotEmpty()) Text(comment.body, style = if (LocalCommunityDetail.current) XType.DetailComment else XType.Body)
        comment.images.forEach { path ->
            if (forumImageId(path) != null) {
                Spacer(Modifier.height(8.dp))
                val src = env.forum.imageUrl(path)
                BodyImage(BodyBlock.Image(src, "评论图片")) { zoom = src }
            }
        }
        zoom?.let { ImageViewer(it) { zoom = null } }
    }
}
