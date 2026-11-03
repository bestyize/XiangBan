package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import coil3.compose.AsyncImage
import xyz.xiangban.xiaoban.common.community.*

/** List covers use real dimensions with bounds; failures retain a neutral surface. */
@Composable
internal fun CommunityCover(url: String, modifier: Modifier = Modifier, natural: Boolean = false, minimum: Float = 2f / 3f, contain: Boolean = false, overlay: @Composable BoxScope.() -> Unit = {}) {
    var ratio by remember(url) { mutableFloatStateOf(16f / 9f) }
    Box(modifier.fillMaxWidth().aspectRatio(ratio).background(LocalExtraColors.current.fill)) {
        AsyncImage(url, null, imageLoader = rememberImageLoader(), contentScale = if (contain) ContentScale.Fit else ContentScale.Crop,
            modifier = Modifier.fillMaxSize(), onSuccess = { if (natural && it.result.image.height > 0) ratio = (it.result.image.width.toFloat() / it.result.image.height).coerceIn(minimum, 16f / 9f) })
        overlay()
    }
}

@Composable
internal fun BoxScope.TopicCoverStats(topic: Topic) {
    Row(Modifier.align(Alignment.BottomStart).fillMaxWidth().background(Brush.verticalGradient(listOf(Color.Transparent, Color.Black.copy(alpha = .72f)))).padding(start = 8.dp, end = 8.dp, top = 16.dp, bottom = 7.dp),
        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) { XIcon(XIcons.Eye, tint = Color.White, size = 12.dp); Text(compactCount(topic.viewCount), style = XType.Caption, color = Color.White) }
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) { XIcon(XIcons.Message, tint = Color.White, size = 12.dp); Text(compactCount(topic.replyCount), style = XType.Caption, color = Color.White) }
    }
}

/** Casual/media are image-led; articles without a cover remain text cards. */
@Composable
internal fun VisualTopicCard(env: CommunityEnv, topic: Topic, onClick: () -> Unit) {
    val image = topic.images.firstOrNull()?.let(env.forum::imageUrl)
    Column(Modifier.fillMaxWidth().outlinedCard(12.dp).clickable(onClick = onClick)) {
        if (image != null) CommunityCover(image, natural = true) { TopicCoverStats(topic) }
        Column(Modifier.padding(10.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            if (topic.pinned || topic.featured) Text(if (topic.pinned) "置顶" else "精选", style = XType.Caption, color = Brand)
            Text(topic.cardTitle, style = XType.Secondary, fontWeight = FontWeight.Normal, maxLines = if (topic.category.slug in setOf("resource-movies", "share-movies")) 3 else 2, overflow = TextOverflow.Ellipsis)
            if (image == null && topic.excerpt.isNotBlank() && topic.excerpt != topic.cardTitle) Text(topic.excerpt, style = XType.Caption, color = LocalExtraColors.current.ink3, maxLines = 2, overflow = TextOverflow.Ellipsis)
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                Avatar(topic.author.nickname, 18.dp, showInitial = true)
                Text(topic.author.nickname, Modifier.weight(1f), style = XType.Caption, maxLines = 1, overflow = TextOverflow.Ellipsis, color = LocalExtraColors.current.ink3)
            }
            Text(agoText(topic.createdAt), style = XType.Caption, color = LocalExtraColors.current.ink3)
            if (image == null) Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) { Stat(XIcons.Eye, compactCount(topic.viewCount)); Stat(XIcons.Message, compactCount(topic.replyCount)) }
            if (topic.likeCount > 0) Stat(XIcons.Heart, compactCount(topic.likeCount))
        }
    }
}
