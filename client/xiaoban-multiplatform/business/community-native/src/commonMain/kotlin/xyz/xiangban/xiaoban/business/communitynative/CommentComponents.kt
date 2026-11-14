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
    Column(Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface).navigationBarsPadding()) {
        Hairline()
        Row(
            Modifier.fillMaxWidth().padding(horizontal = 12.dp, vertical = 8.dp),
            horizontalArrangement = Arrangement.spacedBy(4.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Row(
                Modifier.weight(1f).heightIn(min = 44.dp).clip(CircleShape).background(LocalExtraColors.current.fill)
                    .clickable(enabled = enabled, role = Role.Button, onClick = onWrite).padding(horizontal = 12.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                val tint = MaterialTheme.colorScheme.onSurfaceVariant
                XIcon(XIcons.Message, tint = tint, size = 16.dp)
                Text(label, style = XType.Secondary, color = tint, maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
            actions()
        }
    }
}

@Composable
internal fun CommentCountAction(count: Int?, onClick: () -> Unit) {
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
internal fun CommentSectionTitle(
    count: Int?,
    modifier: Modifier = Modifier,
    label: String = "评论",
) {
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
    val ink = MaterialTheme.colorScheme.onSurfaceVariant
    val authorClick = if (onAuthor == null) Modifier else Modifier.clickable(role = Role.Button, onClick = onAuthor)
    Column(modifier.fillMaxWidth()) {
        Row(Modifier.fillMaxWidth().padding(top = 12.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            Box(authorClick.clip(CircleShape)) { TopicAuthorAvatar(env, author, 32.dp) }
            Column(Modifier.weight(1f)) {
                Text(
                    author.nickname, authorClick, style = XType.Secondary, fontWeight = FontWeight.SemiBold,
                    maxLines = 1, overflow = TextOverflow.Ellipsis,
                )
                parent?.let {
                    Text(it, Modifier.padding(top = 4.dp).clip(RoundedCornerShape(8.dp))
                        .background(LocalExtraColors.current.fill).padding(horizontal = 8.dp, vertical = 4.dp),
                        style = XType.Caption, color = ink, maxLines = 1, overflow = TextOverflow.Ellipsis)
                }
                Spacer(Modifier.height(4.dp))
                content()
                Row(
                    Modifier.fillMaxWidth().heightIn(min = 36.dp),
                    horizontalArrangement = Arrangement.spacedBy(12.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text(metadata, Modifier.weight(1f, fill = false), style = XType.Caption, color = ink,
                        maxLines = 1, overflow = TextOverflow.Ellipsis)
                    onReply?.let { CommentTextAction("回复", it) }
                    actions()
                }
            }
            like?.invoke()
        }
        Hairline(Modifier.padding(start = 44.dp))
    }
}

@Composable
internal fun CommentTextAction(label: String, onClick: () -> Unit) {
    Box(
        Modifier.heightIn(min = 36.dp).clickable(role = Role.Button, onClick = onClick),
        contentAlignment = Alignment.Center,
    ) {
        Text(label, style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}

@Composable
internal fun CommentLikeAction(liked: Boolean, count: Int, onLike: () -> Unit, enabled: Boolean = true) {
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
        if (comment.body.isNotEmpty()) Text(comment.body, style = XType.Body)
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
