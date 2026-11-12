package xyz.xiangban.xiaoban.business.communitynative

import kotlinx.coroutines.launch
import kotlinx.coroutines.CancellationException
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.draw.clip
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.luminance
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import coil3.compose.AsyncImage
import xyz.xiangban.xiaoban.common.community.*

/** List covers use real dimensions with bounds; failures retain a neutral surface. */
@Composable
internal fun CommunityCover(url: String, modifier: Modifier = Modifier, natural: Boolean = false, minimum: Float = 2f / 3f, contain: Boolean = false, onError: () -> Unit = {}, overlay: @Composable BoxScope.() -> Unit = {}) {
    var ratio by remember(url) { mutableFloatStateOf(16f / 9f) }
    Box(modifier.fillMaxWidth().aspectRatio(ratio).background(LocalExtraColors.current.fill)) {
        AsyncImage(url, null, imageLoader = rememberImageLoader(), contentScale = if (contain) ContentScale.Fit else ContentScale.Crop,
            modifier = Modifier.fillMaxSize(), onError = { onError() }, onSuccess = { if (natural && it.result.image.height > 0) ratio = (it.result.image.width.toFloat() / it.result.image.height).coerceIn(minimum, 16f / 9f) })
        overlay()
    }
}

@Composable
internal fun BoxScope.TopicCoverStats(topic: Topic) {
    CoverStats {
        CoverStat(XIcons.Eye, compactCount(topic.viewCount), "浏览量")
        CoverStat(XIcons.Message, compactCount(topic.replyCount), "评论数")
    }
}

@Composable
internal fun BoxScope.CoverStats(content: @Composable RowScope.() -> Unit) {
    Row(
        Modifier.align(Alignment.BottomStart).fillMaxWidth()
            .background(Brush.verticalGradient(listOf(Color.Transparent, Color.Black.copy(alpha = .72f))))
            .padding(start = 8.dp, end = 8.dp, top = 16.dp, bottom = 7.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(10.dp),
        content = content,
    )
}

@Composable
internal fun CoverStat(icon: ImageVector, count: String, label: String, modifier: Modifier = Modifier) {
    Row(modifier.clearAndSetSemantics { contentDescription = "$label $count" },
        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
        XIcon(icon, tint = Color.White, size = 12.dp)
        Text(count, style = XType.Caption, color = Color.White, maxLines = 1, overflow = TextOverflow.Ellipsis)
    }
}

/** Casual/media are image-led; articles without a cover remain text cards. */
@Composable
internal fun VisualTopicCard(env: CommunityEnv, topic: Topic, noteCover: Boolean = false, onCategory: (() -> Unit)? = null, onLogin: () -> Unit = {}, onClick: () -> Unit) {
    var failed by remember(topic.id, topic.images) { mutableIntStateOf(0) }
    val image = topic.images.distinct().getOrNull(failed)?.let(env.forum::imageUrl)
    val scope = rememberCoroutineScope()
    var liked by rememberSaveable(topic.id, topic.liked) { mutableStateOf(topic.liked) }
    var likeCount by rememberSaveable(topic.id, topic.likeCount) { mutableStateOf(topic.likeCount) }
    var liking by remember(topic.id) { mutableStateOf(false) }
    var likeError by remember(topic.id) { mutableStateOf(false) }
    Column(Modifier.fillMaxWidth().outlinedCard(12.dp).clickable(onClick = onClick)) {
        if (image != null) CommunityCover(image, natural = true, onError = { failed++ }) { TopicCoverStats(topic) }
        else if (noteCover) {
            val dark = MaterialTheme.colorScheme.background.luminance() < .5f
            val tint = when (topic.id % 3) {
                1L -> Color(0xFFB26768)
                2L -> Color(0xFF7D8E66)
                else -> Color(0xFFB78E67)
            }
            Column(
                Modifier.fillMaxWidth().background(tint.copy(alpha = if (dark) .24f else .20f)).heightIn(min = 112.dp).padding(12.dp),
                verticalArrangement = Arrangement.Center,
            ) {
                Text(topic.category.name, style = XType.Caption, color = Brand)
                Text(topic.cardTitle, style = XType.Section, maxLines = 3, overflow = TextOverflow.Ellipsis)
            }
        }
        Column(Modifier.padding(10.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {

            if (topic.pinned || topic.featured) Text(if (topic.pinned) "置顶" else "精选", style = XType.Caption, color = Brand)
            Text(topic.cardTitle, style = XType.Secondary, fontWeight = FontWeight.Normal, maxLines = if (topic.category.slug in setOf("resource-movies", "share-movies")) 3 else 2, overflow = TextOverflow.Ellipsis)
            if (image == null && topic.excerpt.isNotBlank() && topic.excerpt != topic.cardTitle) Text(topic.excerpt, style = XType.Caption, color = LocalExtraColors.current.ink3, maxLines = 2, overflow = TextOverflow.Ellipsis)
            if (onCategory != null) LinkText(topic.category.name, onCategory)
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                TopicAuthorAvatar(env, topic.author, 18.dp)
                Text(topic.author.nickname, Modifier.weight(1f), style = XType.Caption, maxLines = 1, overflow = TextOverflow.Ellipsis, color = LocalExtraColors.current.ink3)
                if (noteCover) Row(
                    Modifier.clickable(enabled = !liking) {
                        if (!env.session.state.value.loggedIn) onLogin()
                        else {
                            liking = true; likeError = false
                            scope.launch {
                                try {
                                    val result = env.forum.setTopicLiked(topic.id, !liked)
                                    liked = result.liked; likeCount = result.likeCount
                                } catch (e: CancellationException) { throw e }
                                catch (e: Exception) { likeError = true }
                                finally { liking = false }
                            }
                        }
                    }.padding(vertical = 6.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    XIcon(XIcons.Heart, description = if (liked) "取消点赞" else "点赞", size = 14.dp, tint = if (liked) Brand else LocalExtraColors.current.ink3)
                    Text(likeCount.toString(), style = XType.Caption)
                }
            }
            if (!noteCover) Text(agoText(topic.createdAt), style = XType.Caption, color = LocalExtraColors.current.ink3)
            if (!noteCover && image == null) Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) { Stat(XIcons.Eye, compactCount(topic.viewCount)); Stat(XIcons.Message, compactCount(topic.replyCount)) }
            if (!noteCover && topic.likeCount > 0) Stat(XIcons.Heart, compactCount(topic.likeCount))
            if (likeError) Text("点赞失败，请重试", color = Danger, style = XType.Caption)
        }
    }
}

@Composable
internal fun DiscussionPostCard(
    env: CommunityEnv,
    topic: Topic,
    liked: Boolean,
    likeCount: Int,
    liking: Boolean,
    onOpen: () -> Unit,
    onComments: () -> Unit,
    onLike: () -> Unit,
    onCategory: () -> Unit,
    onTag: (String) -> Unit,
) {
    val colors = MaterialTheme.colorScheme
    val extra = LocalExtraColors.current
    val title = topic.cardTitle
    val excerpt = topic.excerpt.trim().takeIf { topic.title.isNotBlank() && it.isNotEmpty() && it != title }
    val images = remember(topic.images) { topic.images.filter { it.isNotBlank() }.distinct() }
    var imageIndex by remember(topic.id, images) { mutableIntStateOf(0) }
    val image = images.getOrNull(imageIndex)?.let(env.forum::imageUrl)
    val tag = topic.tags.firstOrNull { it.isNotBlank() && it != topic.category.name }

    Column(Modifier.fillMaxWidth().outlinedCard(16.dp)) {
        Column(
            Modifier.fillMaxWidth().clickable(role = Role.Button, onClickLabel = "打开帖子", onClick = onOpen)
                .padding(start = 12.dp, top = 12.dp, end = 12.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                TopicAuthorAvatar(env, topic.author, 26.dp)
                Text(
                    topic.author.nickname,
                    Modifier.weight(1f),
                    style = XType.Caption,
                    fontWeight = FontWeight.SemiBold,
                    color = colors.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                if (topic.createdAt.isNotBlank()) Text(
                    agoText(topic.createdAt),
                    Modifier.widthIn(max = 128.dp),
                    style = XType.Caption,
                    color = extra.ink3,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
            Row(horizontalArrangement = Arrangement.spacedBy(10.dp), verticalAlignment = Alignment.Top) {
                Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(3.dp)) {
                    if (topic.pinned || topic.featured || topic.poll != null) Row(
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        if (topic.pinned || topic.featured) Text(
                            if (topic.pinned) "置顶" else "精选",
                            style = XType.Caption,
                            color = extra.brandInk,
                            fontWeight = FontWeight.SemiBold,
                        )
                        if (topic.poll != null) Text("投票", style = XType.Caption, color = extra.brandInk)
                    }
                    Text(title, style = XType.Section, fontWeight = FontWeight.SemiBold, maxLines = 2, overflow = TextOverflow.Ellipsis)
                    excerpt?.let {
                        Text(it, style = XType.Secondary, color = colors.onSurfaceVariant, maxLines = 2, overflow = TextOverflow.Ellipsis)
                    }
                }
                if (image != null) AsyncImage(
                    model = image,
                    contentDescription = null,
                    imageLoader = rememberImageLoader(),
                    contentScale = ContentScale.Crop,
                    modifier = Modifier.size(width = 76.dp, height = 68.dp)
                        .clip(RoundedCornerShape(10.dp)).background(extra.fill),
                    onError = { imageIndex++ },
                )
            }
        }
        Row(
            Modifier.fillMaxWidth().padding(start = 12.dp, end = 6.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Row(
                Modifier.weight(1f).padding(end = 4.dp),
                horizontalArrangement = Arrangement.spacedBy(6.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                if (topic.category.name.isNotBlank()) DiscussionCardTag(
                    topic.category.name, Modifier.weight(1f, fill = false), onCategory,
                )
                if (tag != null) DiscussionCardTag(tag, Modifier.weight(1f, fill = false)) { onTag(tag) }
            }
            DiscussionCountButton(XIcons.Message, "查看评论", topic.replyCount, onClick = onComments)
            DiscussionCountButton(
                XIcons.Heart, if (liked) "取消点赞" else "点赞", likeCount,
                onClick = onLike, active = liked, enabled = !liking,
            )
        }
    }
}

@Composable
private fun DiscussionCardTag(label: String, modifier: Modifier, onClick: () -> Unit) {
    Box(
        modifier.heightIn(min = 44.dp).clip(RoundedCornerShape(8.dp))
            .clickable(role = Role.Button, onClick = onClick),
        contentAlignment = Alignment.CenterStart,
    ) {
        Text("# $label", style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis)
    }
}

@Composable
private fun DiscussionCountButton(
    icon: ImageVector,
    label: String,
    count: Int,
    onClick: () -> Unit,
    active: Boolean = false,
    enabled: Boolean = true,
) {
    val extra = LocalExtraColors.current
    val color = if (active) extra.brandInk else extra.ink3
    Row(
        Modifier.sizeIn(minWidth = 44.dp, minHeight = 44.dp).clip(CircleShape)
            .clickable(enabled = enabled, role = Role.Button, onClick = onClick)
            .padding(horizontal = 6.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(4.dp, Alignment.CenterHorizontally),
    ) {
        XIcon(icon, tint = color, size = 16.dp, description = label)
        Text(compactCount(count), style = XType.Caption, color = color, maxLines = 1)
    }
}


@Composable
internal fun TopicAuthorAvatar(env: CommunityEnv, author: PublicUser, size: Dp) {
    Box(Modifier.size(size).clip(CircleShape)) {
        Avatar(author.nickname, size, showInitial = true)
        author.avatarUrl?.takeIf { it.isNotBlank() }?.let { url ->
            AsyncImage(env.forum.imageUrl(url), null, imageLoader = rememberImageLoader(),
                modifier = Modifier.fillMaxSize(), contentScale = ContentScale.Crop)
        }
    }
}
